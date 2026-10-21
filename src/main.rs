mod hosts;
mod ssh;
mod terminal;

use gpui::{
    div, prelude::*, px, rgb, size, App, Application, AsyncApp, Bounds, Context, FocusHandle,
    Focusable, KeyDownEvent, Window, WindowBounds, WindowOptions,
};
use hosts::Host;
use ssh::{Session, SharedSession};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};
use terminal::OutputDecoder;

struct RustTerm {
    hosts: Vec<Host>,
    active: Option<usize>,
    session: Option<SharedSession>,
    output: Arc<Mutex<String>>,
    output_dirty: Arc<AtomicBool>,
    connection_generation: u64,
    focus: FocusHandle,
}

impl Focusable for RustTerm {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl RustTerm {
    fn connect(&mut self, index: usize, cx: &mut Context<Self>) {
        let Some(host) = self.hosts.get(index).cloned() else {
            return;
        };
        self.active = Some(index);
        self.session = None;
        self.connection_generation += 1;
        let generation = self.connection_generation;
        self.output_dirty.store(false, Ordering::Release);
        *self.output.lock().unwrap() =
            format!("Connecting to {}@{}...\n", host.username, host.hostname);
        let output = self.output.clone();
        let dirty = self.output_dirty.clone();
        let mut decoder = OutputDecoder::default();
        match Session::connect(&host, move |bytes| {
            let text = decoder.push(&bytes);
            let mut out = output.lock().unwrap();
            out.push_str(&text);
            if out.len() > 150_000 {
                let cut = out.len() - 100_000;
                let boundary = (cut..out.len())
                    .find(|&i| out.is_char_boundary(i))
                    .unwrap_or(out.len());
                out.drain(..boundary);
            }
            drop(out);
            dirty.store(true, Ordering::Release);
        }) {
            Ok(session) => self.session = Some(Arc::new(Mutex::new(session))),
            Err(e) => self
                .output
                .lock()
                .unwrap()
                .push_str(&format!("Connection error: {e}\n")),
        }
        cx.notify();
        if self.session.is_some() {
            // PTY callbacks run on a native thread. Notify GPUI only from its
            // own async executor, and stop the task when this session changes.
            cx.spawn(
                move |this: gpui::WeakEntity<RustTerm>, cx: &mut AsyncApp| async move {
                    loop {
                        cx.background_executor()
                            .timer(Duration::from_millis(33))
                            .await;
                        let keep_running = this.update(cx, |this, cx| {
                            if this.connection_generation != generation {
                                return false;
                            }
                            if this.output_dirty.swap(false, Ordering::AcqRel) {
                                cx.notify();
                            }
                            true
                        });
                        if !matches!(keep_running, Ok(true)) {
                            break;
                        }
                    }
                },
            )
            .detach();
        }
    }
}

impl Render for RustTerm {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let output = self.output.lock().unwrap().clone();
        let mut sidebar = div()
            .w(px(250.))
            .h_full()
            .p_4()
            .bg(rgb(0x161b22))
            .flex()
            .flex_col()
            .gap_3()
            .child(div().text_xl().child("RustTerm"))
            .child(div().text_sm().text_color(rgb(0x8b949e)).child("HOSTS"));
        for (i, host) in self.hosts.iter().enumerate() {
            let title = format!("{}  ·  {}", host.name, host.hostname);
            sidebar = sidebar.child(
                div()
                    .id(("host", i))
                    .p_2()
                    .rounded_md()
                    .bg(if self.active == Some(i) {
                        rgb(0x303c4a)
                    } else {
                        rgb(0x21262d)
                    })
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _, _, cx| this.connect(i, cx)))
                    .child(title),
            );
        }
        div()
            .size_full()
            .flex()
            .flex_row()
            .bg(rgb(0x0d1117))
            .text_color(rgb(0xe6edf3))
            .child(sidebar)
            .child(
                div()
                    .flex_1()
                    .p_4()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(div().text_lg().child("Terminal · click here, then type"))
                    .child(
                        div()
                            .id("terminal")
                            .flex_1()
                            .p_3()
                            .rounded_md()
                            .bg(rgb(0x010409))
                            .text_color(rgb(0x5cda89))
                            .track_focus(&self.focus)
                            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
                                if let Some(session) = &this.session {
                                    let key = &event.keystroke;
                                    let data: Option<Vec<u8>> = match key.key.as_str() {
                                        "enter" => Some(vec![b'\r']),
                                        "backspace" => Some(vec![0x7f]),
                                        "tab" => Some(vec![b'\t']),
                                        "up" => Some(b"\x1b[A".to_vec()),
                                        "down" => Some(b"\x1b[B".to_vec()),
                                        "left" => Some(b"\x1b[D".to_vec()),
                                        "right" => Some(b"\x1b[C".to_vec()),
                                        _ if key.modifiers.control && key.key.len() == 1 => {
                                            let b = key.key.as_bytes()[0].to_ascii_lowercase();
                                            if b.is_ascii_lowercase() {
                                                Some(vec![b - b'a' + 1])
                                            } else {
                                                None
                                            }
                                        }
                                        _ => key.key_char.as_ref().map(|s| s.as_bytes().to_vec()),
                                    };
                                    if let Some(data) = data {
                                        let _ = session.lock().unwrap().send(&data);
                                    }
                                }
                                cx.notify();
                            }))
                            .child(div().text_sm().child(output)),
                    ),
            )
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(1100.), px(720.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|cx| RustTerm {
                    hosts: hosts::load(),
                    active: None,
                    session: None,
                    output: Arc::new(Mutex::new("Select a host to connect.\n".into())),
                    output_dirty: Arc::new(AtomicBool::new(false)),
                    connection_generation: 0,
                    focus: cx.focus_handle(),
                })
            },
        )
        .expect("Unable to open RustTerm window");
    });
}
