mod host_editor;
mod hosts;
mod session;
mod terminal;

use gpui::prelude::*;
use gpui::{
    div, px, rgb, size, App, Application, AsyncApp, Bounds, Context, FocusHandle, Focusable,
    KeyDownEvent, ScrollDelta, ScrollWheelEvent, StyledText, TitlebarOptions, Window, WindowBounds,
    WindowOptions,
};
use host_editor::{HostDraft, LABELS};
use hosts::Host;
use session::{Session, SharedSession};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};
use terminal::{TerminalBuffer, DEFAULT_BG, DEFAULT_FG, FONT_SIZE, LINE_HEIGHT};

const SIDEBAR_WIDTH: f32 = 212.;
const PANEL_PADDING: f32 = 8.;
const TERMINAL_PADDING: f32 = 4.;
const GAP: f32 = 6.;
const HEADER_HEIGHT: f32 = 46.;
const HINT_HEIGHT: f32 = 32.;
const MIN_TERMINAL_FONT: f32 = 10.;
const MAX_TERMINAL_FONT: f32 = 24.;

fn clamp_font_size(size: f32) -> f32 {
    size.clamp(MIN_TERMINAL_FONT, MAX_TERMINAL_FONT)
}

fn terminal_line_height(size: f32) -> f32 {
    size + (LINE_HEIGHT - FONT_SIZE)
}
const STATUS_HEIGHT: f32 = 36.;

#[derive(Clone, Copy, PartialEq, Eq)]
enum EditorStage {
    Address,
    Details,
}

struct TermiNur {
    hosts: Vec<Host>,
    tabs: Vec<Tab>,
    active: Option<u64>,
    next_tab_id: u64,
    focus: FocusHandle,
    editor_focus: FocusHandle,
    editor: Option<HostDraft>,
    editor_stage: EditorStage,
    grid: (u16, u16),
    font_size: f32,
    sidebar_visible: bool,
}

struct Tab {
    id: u64,
    title: String,
    session: Option<SharedSession>,
    screen: Arc<Mutex<TerminalBuffer>>,
    screen_dirty: Arc<AtomicBool>,
    scroll_offset: usize,
    status: String,
}

fn active_after_close(ids: &[u64], active: Option<u64>, closed: u64) -> Option<u64> {
    let index = ids.iter().position(|id| *id == closed)?;
    if active != Some(closed) {
        return active;
    }
    ids.get(index + 1)
        .or_else(|| index.checked_sub(1).and_then(|i| ids.get(i)))
        .copied()
}

impl Focusable for TermiNur {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl TermiNur {
    fn open_editor(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let mut draft = HostDraft::default();
        draft.select(1);
        self.editor = Some(draft);
        self.editor_stage = EditorStage::Address;
        self.active = None;
        window.focus(&self.editor_focus);
        cx.notify();
    }

    fn show_hosts(&mut self, cx: &mut Context<Self>) {
        self.editor = None;
        self.active = None;
        cx.notify();
    }

    fn continue_editor(&mut self, cx: &mut Context<Self>) {
        if let Some(editor) = &mut self.editor {
            if editor.continue_from_address().is_ok() {
                self.editor_stage = EditorStage::Details;
            }
            cx.notify();
        }
    }

    fn editor_key(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) {
        let key = &event.keystroke;
        if key.key == "escape" {
            self.editor = None;
        } else if key.key == "enter" {
            if self.editor_stage == EditorStage::Address {
                self.continue_editor(cx);
            } else {
                self.save_host(cx);
            }
        } else if let Some(editor) = &mut self.editor {
            if key.modifiers.platform {
                if key.key == "v" {
                    if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
                        editor.insert(&text);
                    }
                }
            } else if !key.modifiers.control && !key.modifiers.alt {
                match key.key.as_str() {
                    "tab" if self.editor_stage == EditorStage::Details => {
                        editor.next(key.modifiers.shift)
                    }
                    "backspace" => editor.backspace(),
                    "delete" => editor.delete(),
                    "left" => editor.move_cursor(false),
                    "right" => editor.move_cursor(true),
                    "home" => editor.cursor = 0,
                    "end" => editor.cursor = editor.values[editor.selected].len(),
                    _ => {
                        if let Some(text) = key.key_char.as_deref() {
                            editor.insert(text);
                        }
                    }
                }
            }
        }
        cx.stop_propagation();
        cx.notify();
    }

    fn save_host(&mut self, cx: &mut Context<Self>) {
        let Some(editor) = self.editor.as_mut() else {
            return;
        };
        let host = match editor.validate() {
            Ok(host) => host,
            Err(error) => {
                editor.error = Some(error);
                cx.notify();
                return;
            }
        };
        let mut updated = self.hosts.clone();
        updated.push(host);
        match hosts::save(&updated) {
            Ok(()) => {
                self.hosts = updated;
                self.editor = None;
            }
            Err(error) => editor.error = Some(format!("Could not save host: {error}")),
        }
        cx.notify();
    }

    fn active_tab(&self) -> Option<&Tab> {
        self.tabs.iter().find(|tab| Some(tab.id) == self.active)
    }

    fn active_tab_mut(&mut self) -> Option<&mut Tab> {
        self.tabs.iter_mut().find(|tab| Some(tab.id) == self.active)
    }

    fn close_tab(&mut self, id: u64, cx: &mut Context<Self>) {
        let ids: Vec<_> = self.tabs.iter().map(|tab| tab.id).collect();
        if !ids.contains(&id) {
            return;
        }
        self.active = active_after_close(&ids, self.active, id);
        self.tabs.retain(|tab| tab.id != id);
        cx.notify();
    }

    fn connect(&mut self, index: usize, cx: &mut Context<Self>) {
        let Some(host) = self.hosts.get(index).cloned() else {
            return;
        };
        let id = self.next_tab_id;
        self.next_tab_id += 1;
        self.active = Some(id);
        let (rows, cols) = self.grid;
        let screen = Arc::new(Mutex::new(TerminalBuffer::new(rows, cols)));
        let dirty = Arc::new(AtomicBool::new(false));
        let output = screen.clone();
        let output_dirty = dirty.clone();
        let on_data = move |bytes: Vec<u8>| {
            output.lock().unwrap().push(&bytes);
            output_dirty.store(true, Ordering::Release);
        };
        let (session, status) = match Session::connect_host(&host, rows, cols, on_data) {
            Ok(session) => (
                Some(Arc::new(Mutex::new(session))),
                format!(
                    "SSH started · {}@{}:{} · awaiting authentication",
                    host.username, host.hostname, host.port
                ),
            ),
            Err(error) => {
                screen
                    .lock()
                    .unwrap()
                    .push(format!("Connection error: {error}\r\n").as_bytes());
                (None, format!("Failed to start: {error}"))
            }
        };
        let polling = session.is_some();
        self.tabs.push(Tab {
            id,
            title: host.name,
            session,
            screen,
            screen_dirty: dirty,
            scroll_offset: 0,
            status,
        });
        cx.notify();
        if polling {
            // PTY callbacks run on a native thread. Notify GPUI only from its
            // own async executor, and stop the task when this tab is closed.
            cx.spawn(
                async move |this: gpui::WeakEntity<TermiNur>, cx: &mut AsyncApp| loop {
                    cx.background_executor()
                        .timer(Duration::from_millis(16))
                        .await;
                    let keep_running = this.update(cx, |this, cx| {
                        let Some(tab) = this.tabs.iter_mut().find(|tab| tab.id == id) else {
                            return false;
                        };
                        let mut changed = tab.screen_dirty.swap(false, Ordering::AcqRel);
                        let exit = tab.session.as_ref().map(|session| {
                            let mut session = session.lock().unwrap();
                            session.try_wait()
                        });
                        match exit {
                            Some(Ok(Some(status))) => {
                                tab.status = format!(
                                    "SSH exited (code {}) · see terminal output",
                                    status.exit_code()
                                );
                                tab.session = None;
                                changed = true;
                            }
                            Some(Err(error)) => {
                                tab.status = format!("Cannot check SSH process: {error}");
                                tab.session = None;
                                changed = true;
                            }
                            Some(Ok(None)) | None => {}
                        }
                        if changed && this.active == Some(id) {
                            cx.notify();
                        }
                        true
                    });
                    if !matches!(keep_running, Ok(true)) {
                        break;
                    }
                },
            )
            .detach();
        }
    }

    fn toggle_sidebar(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.sidebar_visible = !self.sidebar_visible;
        self.fit_grid(window, cx);
        cx.notify();
    }

    fn zoom_terminal(&mut self, delta: f32, window: &mut Window, cx: &mut Context<Self>) {
        self.font_size = clamp_font_size(self.font_size + delta);
        self.fit_grid(window, cx);
        cx.notify();
    }

    fn reset_terminal_zoom(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.font_size = FONT_SIZE;
        self.fit_grid(window, cx);
        cx.notify();
    }

    fn paste(&mut self, text: &str) {
        let Some(tab) = self.active_tab() else {
            return;
        };
        let Some(session) = &tab.session else {
            return;
        };
        let bracketed = tab.screen.lock().unwrap().bracketed_paste();
        let payload = if bracketed {
            format!("\x1b[200~{text}\x1b[201~")
        } else {
            text.to_owned()
        };
        let _ = session.lock().unwrap().send(payload.as_bytes());
    }

    fn scroll(&mut self, lines: f32) {
        if lines == 0. {
            return;
        }
        let Some(tab) = self.active_tab_mut() else {
            return;
        };
        let limit = tab.screen.lock().unwrap().scrollback_len();
        let step = lines.abs().ceil() as usize;
        let next = if lines > 0. {
            (tab.scroll_offset + step).min(limit)
        } else {
            tab.scroll_offset.saturating_sub(step)
        };
        if next != tab.scroll_offset {
            tab.scroll_offset = next;
            tab.screen.lock().unwrap().set_scrollback(next);
        }
    }

    /// Fits the terminal grid into the space the layout actually reserves.
    fn fit_grid(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let viewport = window.viewport_size();
        let cell_width = cx
            .text_system()
            .ch_advance(
                cx.text_system().resolve_font(&terminal::terminal_font()),
                px(self.font_size),
            )
            .map(f32::from)
            .unwrap_or(8.);
        let usable_width = f32::from(viewport.width)
            - if self.sidebar_visible {
                SIDEBAR_WIDTH
            } else {
                0.
            }
            - PANEL_PADDING * 2.
            - TERMINAL_PADDING * 2.
            - 4.;
        let usable_height = f32::from(viewport.height)
            - PANEL_PADDING * 2.
            - HEADER_HEIGHT
            - HINT_HEIGHT
            - STATUS_HEIGHT
            - GAP * 2.
            - TERMINAL_PADDING * 2.
            - 4.;
        let cols = (usable_width / cell_width).floor().clamp(20., 400.) as u16;
        let rows = (usable_height / terminal_line_height(self.font_size))
            .floor()
            .clamp(4., 200.) as u16;
        if (rows, cols) == self.grid {
            return;
        }
        self.grid = (rows, cols);
        for tab in &self.tabs {
            tab.screen.lock().unwrap().resize(rows, cols);
            if let Some(session) = &tab.session {
                let _ = session.lock().unwrap().resize(rows, cols);
            }
        }
    }
}

impl Render for TermiNur {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.fit_grid(window, cx);
        let line_height = terminal_line_height(self.font_size);
        let font_size = self.font_size;
        let rows = self
            .active_tab()
            .map(|tab| tab.screen.lock().unwrap().visible_rows())
            .unwrap_or_default();

        let entry_style = |selected: bool| {
            if selected {
                rgb(0x303c4a)
            } else {
                rgb(0x21262d)
            }
        };

        let planned_section = |label: &'static str| {
            div()
                .p_2()
                .flex()
                .flex_row()
                .justify_between()
                .text_sm()
                .text_color(rgb(0x989bb1))
                .child(label)
                .child(div().text_color(rgb(0x777a91)).child("Later"))
        };

        let sidebar = div()
            .w(px(SIDEBAR_WIDTH))
            .h_full()
            .p(px(8.))
            .bg(rgb(0x202738))
            .flex()
            .flex_col()
            .gap(px(6.))
            .child(
                div()
                    .id("hosts-navigation")
                    .p_2()
                    .rounded_md()
                    .bg(rgb(0x33445e))
                    .cursor_pointer()
                    .on_click(cx.listener(|this, _, _, cx| this.show_hosts(cx)))
                    .child("▦   Hosts"),
            )
            .child(planned_section("Keychain"))
            .child(planned_section("Port Forwarding"))
            .child(planned_section("Snippets"))
            .child(planned_section("Known Hosts"))
            .child(planned_section("SFTP"))
            .child(planned_section("Logs"));

        let form = self.editor.as_ref().map(|editor| {
            if self.editor_stage == EditorStage::Address {
                return div()
                    .id("host-address-form")
                    .size_full()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap(px(18.))
                    .track_focus(&self.editor_focus)
                    .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
                        this.editor_key(event, cx)
                    }))
                    .child(
                        div()
                            .p(px(20.))
                            .rounded_lg()
                            .bg(rgb(0x34374e))
                            .text_xl()
                            .child("▦"),
                    )
                    .child(div().text_xl().child("Create host"))
                    .child(
                        div()
                            .text_color(rgb(0x989bb1))
                            .child("Save a connection to reach your server in one click."),
                    )
                    .child(
                        div()
                            .id("address-input")
                            .w(px(460.))
                            .p(px(12.))
                            .rounded_md()
                            .bg(rgb(0x292b3e))
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, window, cx| {
                                if let Some(editor) = &mut this.editor {
                                    editor.select(1);
                                }
                                window.focus(&this.editor_focus);
                                cx.notify();
                            }))
                            .child(if editor.values[1].is_empty() {
                                "Type IP or Hostname  │".into()
                            } else {
                                editor.display(1)
                            }),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0xff918a))
                            .child(editor.error.clone().unwrap_or_default()),
                    )
                    .child(
                        div()
                            .id("continue-host")
                            .w(px(460.))
                            .p(px(12.))
                            .rounded_md()
                            .bg(rgb(0x478ceb))
                            .cursor_pointer()
                            .text_center()
                            .on_click(cx.listener(|this, _, _, cx| this.continue_editor(cx)))
                            .child("Continue"),
                    )
                    .into_any_element();
            }
            let mut fields = div().flex().flex_col().gap(px(10.));
            for (index, label) in LABELS.iter().enumerate() {
                let selected = editor.selected == index;
                let value = editor.display(index);
                fields = fields.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(3.))
                        .child(div().text_sm().text_color(rgb(0x8b949e)).child(*label))
                        .child(
                            div()
                                .id(("host-field", index))
                                .p_2()
                                .rounded_md()
                                .bg(entry_style(selected))
                                .cursor_pointer()
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    if let Some(editor) = &mut this.editor {
                                        editor.select(index);
                                    }
                                    window.focus(&this.editor_focus);
                                    cx.notify();
                                }))
                                .child(if value.is_empty() { " ".into() } else { value }),
                        ),
                );
            }
            div()
                .id("host-form")
                .size_full()
                .flex()
                .flex_col()
                .items_center()
                .overflow_y_scroll()
                .p_2()
                .gap(px(10.))
                .track_focus(&self.editor_focus)
                .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
                    this.editor_key(event, cx)
                }))
                .child(div().text_lg().child("Host details"))
                .child(div().text_sm().text_color(rgb(0x8b949e)).child("Key path optional: ~/.ssh/id_ed25519 or an absolute path. Only the path is saved; no key bytes or passphrase."))
                .child(fields.w(px(480.)))
                .child(
                    div().text_sm().text_color(rgb(0xff918a))
                        .child(editor.error.clone().unwrap_or_default()),
                )
                .child(
                    div().flex().flex_row().gap(px(10.))
                        .child(div().id("save-host").p_2().rounded_md().bg(rgb(0x303c4a))
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, _, cx| this.save_host(cx)))
                            .child("Save host"))
                        .child(div().id("cancel-host").p_2().rounded_md().bg(rgb(0x21262d))
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.editor = None;
                                cx.notify();
                            }))
                            .child("Cancel")),
                )
                .into_any_element()
        });

        let mut terminal_lines = div().flex().flex_col().w_full();
        for row in rows {
            let line = if row.runs.is_empty() {
                StyledText::new(row.text)
            } else {
                StyledText::new(row.text).with_runs(row.runs)
            };
            terminal_lines = terminal_lines.child(div().h(px(line_height)).child(line));
        }

        let mut tab_strip = div()
            .id("session-tabs")
            .flex_1()
            .flex()
            .flex_row()
            .overflow_x_scroll()
            .gap(px(4.));
        tab_strip = tab_strip.child(
            div()
                .id("hosts-tab")
                .p_2()
                .rounded_md()
                .bg(entry_style(self.active.is_none()))
                .cursor_pointer()
                .on_click(cx.listener(|this, _, _, cx| this.show_hosts(cx)))
                .child("Workspace"),
        );
        for tab in &self.tabs {
            let id = tab.id;
            tab_strip = tab_strip.child(
                div()
                    .flex()
                    .flex_row()
                    .gap(px(8.))
                    .rounded_md()
                    .bg(entry_style(self.active == Some(id)))
                    .child(
                        div()
                            .id(("tab", id))
                            .p_1()
                            .cursor_pointer()
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.active = Some(id);
                                this.editor = None;
                                cx.notify();
                            }))
                            .child(tab.title.clone()),
                    )
                    .child(
                        div()
                            .id(("close-tab", id))
                            .p_1()
                            .cursor_pointer()
                            .on_click(cx.listener(move |this, _, _, cx| this.close_tab(id, cx)))
                            .child("×"),
                    ),
            );
        }

        let mut host_cards = div()
            .id("host-cards")
            .size_full()
            .p(px(PANEL_PADDING))
            .flex()
            .flex_col()
            .overflow_y_scroll()
            .gap(px(10.));
        if self.hosts.is_empty() {
            host_cards = host_cards.child(
                div()
                    .size_full()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap(px(12.))
                    .child(div().text_xl().child("Create host"))
                    .child(
                        div()
                            .text_color(rgb(0x989bb1))
                            .child("No SSH hosts saved yet."),
                    )
                    .child(
                        div()
                            .id("create-first-host")
                            .p_2()
                            .rounded_md()
                            .bg(rgb(0x478ceb))
                            .cursor_pointer()
                            .on_click(
                                cx.listener(|this, _, window, cx| this.open_editor(window, cx)),
                            )
                            .child("+ New host"),
                    ),
            );
        } else {
            host_cards = host_cards.child(div().text_lg().child("Saved hosts"));
            for (index, host) in self.hosts.iter().enumerate() {
                host_cards =
                    host_cards.child(
                        div()
                            .id(("host-card", index))
                            .p(px(16.))
                            .rounded_md()
                            .bg(rgb(0x292b3e))
                            .cursor_pointer()
                            .on_click(cx.listener(move |this, _, _, cx| this.connect(index, cx)))
                            .child(format!("▦  {}", host.name))
                            .child(div().text_sm().text_color(rgb(0x989bb1)).child(format!(
                                "{}@{}:{}",
                                host.username, host.hostname, host.port
                            ))),
                    );
            }
        }

        let content = if let Some(form) = form {
            div()
                .size_full()
                .p(px(PANEL_PADDING))
                .child(form)
                .into_any_element()
        } else if self.active_tab().is_some() {
            div()
                .size_full()
                .px(px(PANEL_PADDING))
                .pt(px(4.))
                .pb(px(4.))
                .flex()
                .flex_col()
                .gap(px(GAP))
                .child(
                    div()
                        .h(px(HINT_HEIGHT))
                        .flex()
                        .flex_row()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(0x9caac0))
                                .child("TERMINAL  /  Click to type · Scroll for history"),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap(px(6.))
                                .child(
                                    div()
                                        .id("terminal-font-decrease")
                                        .px(px(10.))
                                        .py(px(5.))
                                        .rounded_md()
                                        .bg(rgb(0x303a50))
                                        .cursor_pointer()
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.zoom_terminal(-1., window, cx)
                                        }))
                                        .child("A−"),
                                )
                                .child(
                                    div()
                                        .id("terminal-font-reset")
                                        .px(px(10.))
                                        .py(px(5.))
                                        .rounded_md()
                                        .bg(rgb(0x303a50))
                                        .cursor_pointer()
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.reset_terminal_zoom(window, cx)
                                        }))
                                        .child(format!("{font_size:.0} px")),
                                )
                                .child(
                                    div()
                                        .id("terminal-font-increase")
                                        .px(px(10.))
                                        .py(px(5.))
                                        .rounded_md()
                                        .bg(rgb(0x303a50))
                                        .cursor_pointer()
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.zoom_terminal(1., window, cx)
                                        }))
                                        .child("A+"),
                                ),
                        ),
                )
                .child(
                    div()
                        .id("terminal")
                        .flex_1()
                        .p(px(TERMINAL_PADDING))
                        .rounded_md()
                        .bg(rgb(DEFAULT_BG))
                        .font(terminal::terminal_font())
                        .text_size(px(font_size))
                        .line_height(px(line_height))
                        .whitespace_nowrap()
                        .overflow_hidden()
                        .track_focus(&self.focus)
                        .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
                            let key = &event.keystroke;
                            if key.modifiers.platform {
                                if key.key == "v" {
                                    if let Some(text) =
                                        cx.read_from_clipboard().and_then(|item| item.text())
                                    {
                                        this.paste(&text);
                                    }
                                }
                                cx.stop_propagation();
                                return;
                            }
                            let Some(tab) = this.active_tab() else {
                                return;
                            };
                            let application_cursor =
                                tab.screen.lock().unwrap().application_cursor();
                            let bytes = terminal::encode_key(
                                &key.key,
                                key.key_char.as_deref(),
                                key.modifiers.control,
                                key.modifiers.alt,
                                key.modifiers.shift,
                                application_cursor,
                            );
                            if let Some(bytes) = bytes {
                                if let Some(session) = &tab.session {
                                    let _ = session.lock().unwrap().send(&bytes);
                                }
                            }
                            cx.stop_propagation();
                            cx.notify();
                        }))
                        .on_scroll_wheel(cx.listener(|this, event: &ScrollWheelEvent, _, cx| {
                            let lines = match event.delta {
                                ScrollDelta::Lines(delta) => delta.y,
                                ScrollDelta::Pixels(delta) => {
                                    f32::from(delta.y) / terminal_line_height(this.font_size)
                                }
                            };
                            this.scroll(lines);
                            cx.notify();
                        }))
                        .child(terminal_lines),
                )
                .child(
                    div()
                        .h(px(STATUS_HEIGHT))
                        .p_2()
                        .rounded_md()
                        .bg(rgb(0x202d40))
                        .text_sm()
                        .text_color(rgb(0x989bb1))
                        .child(
                            self.active_tab()
                                .map(|tab| tab.status.clone())
                                .unwrap_or_default(),
                        ),
                )
                .into_any_element()
        } else {
            host_cards.into_any_element()
        };

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(0x151c2b))
            .text_color(rgb(DEFAULT_FG))
            .child(
                div()
                    .h(px(HEADER_HEIGHT))
                    .p_1()
                    .flex()
                    .flex_row()
                    .gap(px(8.))
                    .bg(rgb(0x1c2638))
                    // GPUI draws into the transparent macOS titlebar. Keep
                    // this region free for the native traffic light buttons.
                    .child(div().w(px(104.)).flex_shrink_0())
                    .child(
                        div()
                            .id("toggle-sidebar")
                            .p_2()
                            .rounded_md()
                            .bg(rgb(0x303a50))
                            .cursor_pointer()
                            .on_click(
                                cx.listener(|this, _, window, cx| this.toggle_sidebar(window, cx)),
                            )
                            .child(if self.sidebar_visible {
                                "☰ Hide menu"
                            } else {
                                "☰ Show menu"
                            }),
                    )
                    .child(tab_strip)
                    .child(
                        div()
                            .id("add-host")
                            .flex_shrink_0()
                            .p_2()
                            .rounded_md()
                            .bg(rgb(0x355b88))
                            .cursor_pointer()
                            .on_click(
                                cx.listener(|this, _, window, cx| this.open_editor(window, cx)),
                            )
                            .child("+ New host"),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_row()
                    .children(self.sidebar_visible.then_some(sidebar))
                    .child(div().flex_1().child(content)),
            )
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(1100.), px(720.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("TermiNur".into()),
                    appears_transparent: cfg!(target_os = "macos"),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |window, cx| {
                let hosts = hosts::load();
                let editor = if hosts.is_empty() {
                    let mut draft = HostDraft::default();
                    draft.select(1);
                    Some(draft)
                } else {
                    None
                };
                let view = cx.new(|cx| TermiNur {
                    hosts,
                    tabs: Vec::new(),
                    active: None,
                    next_tab_id: 0,
                    focus: cx.focus_handle(),
                    editor_focus: cx.focus_handle(),
                    editor,
                    editor_stage: EditorStage::Address,
                    grid: (30, 100),
                    font_size: FONT_SIZE,
                    sidebar_visible: true,
                });
                if view.read(cx).editor.is_some() {
                    window.focus(&view.read(cx).editor_focus);
                }
                view
            },
        )
        .expect("Unable to open TermiNur window");
    });
}

#[cfg(test)]
mod tests {
    use super::{active_after_close, clamp_font_size, terminal_line_height, FONT_SIZE};

    #[test]
    fn font_zoom_is_bounded_and_line_height_tracks_font() {
        assert_eq!(clamp_font_size(9.), 10.);
        assert_eq!(clamp_font_size(25.), 24.);
        assert_eq!(clamp_font_size(FONT_SIZE + 1.), 14.);
        assert_eq!(terminal_line_height(14.), 18.);
    }

    #[test]
    fn closing_active_tab_selects_neighbor() {
        let ids = [10, 11, 12];
        assert_eq!(active_after_close(&ids, Some(11), 11), Some(12));
        assert_eq!(active_after_close(&ids, Some(12), 12), Some(11));
        assert_eq!(active_after_close(&[10], Some(10), 10), None);
    }

    #[test]
    fn closing_background_tab_keeps_selection() {
        assert_eq!(active_after_close(&[10, 11, 12], Some(11), 10), Some(11));
        assert_eq!(active_after_close(&[10, 11], None, 10), None);
        assert_eq!(active_after_close(&[10, 11], Some(11), 99), None);
    }
}
