//! Terminal screen backed by the `vt100` parser.
//!
//! The parser handles ANSI escape sequences (SGR colors and attributes, cursor
//! addressing, erase commands, scroll regions, and the alternate screen). This
//! module only converts the parsed grid into GPUI text runs.

use gpui::{font, rgb, Font, FontWeight, Hsla, SharedString, TextRun, UnderlineStyle};
use std::num::NonZeroU16;
use vt100::Color;

pub const FONT_SIZE: f32 = 13.;
pub const LINE_HEIGHT: f32 = 17.;
const SCROLLBACK_ROWS: usize = 10_000;

pub const DEFAULT_FG: u32 = 0xe6edf3;
pub const DEFAULT_BG: u32 = 0x0d1117;

pub fn terminal_font() -> Font {
    font("Menlo")
}

/// A visible row converted to styled text for GPUI.
pub struct Row {
    pub text: SharedString,
    pub runs: Vec<TextRun>,
}

pub struct TerminalBuffer {
    parser: vt100::Parser,
    rows: u16,
    cols: u16,
}

impl TerminalBuffer {
    pub fn new(rows: u16, cols: u16) -> Self {
        let rows = rows.max(1);
        let cols = cols.max(1);
        Self {
            parser: vt100::Parser::new(non_zero(rows), non_zero(cols), SCROLLBACK_ROWS),
            rows,
            cols,
        }
    }

    /// Returns true when the grid size actually changed.
    pub fn resize(&mut self, rows: u16, cols: u16) -> bool {
        let rows = rows.max(1);
        let cols = cols.max(1);
        if (rows, cols) == (self.rows, self.cols) {
            return false;
        }
        self.rows = rows;
        self.cols = cols;
        self.parser.set_size(non_zero(rows), non_zero(cols));
        true
    }

    pub fn push(&mut self, bytes: &[u8]) {
        self.parser.process(bytes);
    }

    #[cfg(test)]
    pub fn alternate_screen(&self) -> bool {
        self.parser.screen().alternate_screen()
    }

    pub fn application_cursor(&self) -> bool {
        self.parser.screen().application_cursor()
    }

    pub fn bracketed_paste(&self) -> bool {
        self.parser.screen().bracketed_paste()
    }

    pub fn scrollback_len(&self) -> usize {
        self.parser.screen().scrollback_len()
    }

    pub fn set_scrollback(&mut self, rows: usize) {
        self.parser.screen_mut().set_scrollback(rows);
    }

    /// Converts every visible grid row into a styled text row.
    pub fn visible_rows(&self) -> Vec<Row> {
        let screen = self.parser.screen();
        let cursor = screen.cursor_position();
        let show_cursor = !screen.hide_cursor() && screen.scrollback() == 0;
        let (rows, cols) = (self.rows, self.cols);
        let mut out = Vec::with_capacity(rows as usize);
        for row in 0..rows {
            let mut text = String::new();
            let mut runs: Vec<TextRun> = Vec::new();
            let mut pending: Option<CellStyle> = None;
            let mut pending_len = 0usize;
            for col in 0..cols {
                let Some(cell) = screen.cell(row, col) else {
                    continue;
                };
                if cell.is_wide_continuation() {
                    continue;
                }
                let piece = match cell.contents() {
                    "" => " ",
                    contents => contents,
                };
                let mut style = CellStyle::new(cell);
                if show_cursor && cursor == (row, col) {
                    style.invert();
                }
                text.push_str(piece);
                match &mut pending {
                    Some(current) if current.same_as(&style) => pending_len += piece.len(),
                    Some(current) => {
                        runs.push(current.into_run(pending_len));
                        pending_len = piece.len();
                        *current = style;
                    }
                    None => {
                        pending = Some(style);
                        pending_len = piece.len();
                    }
                }
            }
            if let Some(style) = pending {
                runs.push(style.into_run(pending_len));
            }
            out.push(Row {
                text: text.into(),
                runs,
            });
        }
        out
    }

    /// Plain text of a single visible row; used by tests and diagnostics.
    #[cfg(test)]
    pub fn row_text(&self, row: u16) -> String {
        let screen = self.parser.screen();
        let mut text = String::new();
        for col in 0..self.cols {
            if let Some(cell) = screen.cell(row, col) {
                if cell.is_wide_continuation() {
                    continue;
                }
                text.push_str(cell.contents());
            }
        }
        text.trim_end().to_string()
    }
}

fn non_zero(value: u16) -> NonZeroU16 {
    NonZeroU16::new(value).expect("terminal grid dimensions are clamped to at least one")
}

#[derive(Clone, Copy)]
struct CellStyle {
    fg: Hsla,
    bg: Hsla,
    bold: bool,
    italic: bool,
    underline: bool,
}

impl CellStyle {
    fn new(cell: &vt100::Cell) -> Self {
        let mut style = Self {
            fg: foreground(cell.fgcolor()),
            bg: background(cell.bgcolor()),
            bold: cell.bold(),
            italic: cell.italic(),
            underline: cell.underline(),
        };
        if cell.inverse() {
            style.invert();
        }
        style
    }

    fn invert(&mut self) {
        std::mem::swap(&mut self.fg, &mut self.bg);
    }

    fn same_as(&self, other: &Self) -> bool {
        self.fg == other.fg
            && self.bg == other.bg
            && self.bold == other.bold
            && self.italic == other.italic
            && self.underline == other.underline
    }

    fn into_run(self, len: usize) -> TextRun {
        let mut font = terminal_font();
        if self.bold {
            font.weight = FontWeight::BOLD;
        }
        if self.italic {
            font.style = gpui::FontStyle::Italic;
        }
        TextRun {
            len,
            font,
            color: self.fg,
            background_color: (self.bg != default_background()).then_some(self.bg),
            underline: self.underline.then_some(UnderlineStyle {
                thickness: px(1.),
                color: Some(self.fg),
                wavy: false,
            }),
            strikethrough: None,
        }
    }
}

fn default_foreground() -> Hsla {
    rgb(DEFAULT_FG).into()
}

fn default_background() -> Hsla {
    rgb(DEFAULT_BG).into()
}

fn foreground(color: Color) -> Hsla {
    match color {
        Color::Default => default_foreground(),
        Color::Idx(index) => indexed(index),
        Color::Rgb(r, g, b) => rgb(u32::from(r) << 16 | u32::from(g) << 8 | u32::from(b)).into(),
    }
}

fn background(color: Color) -> Hsla {
    match color {
        Color::Default => default_background(),
        Color::Idx(index) => indexed(index),
        Color::Rgb(r, g, b) => rgb(u32::from(r) << 16 | u32::from(g) << 8 | u32::from(b)).into(),
    }
}

/// Standard 256-color xterm palette.
fn indexed(index: u8) -> Hsla {
    const BASE: [u32; 16] = [
        0x000000, 0xcd0000, 0x00cd00, 0xcdcd00, 0x0000ee, 0xcd00cd, 0x00cdcd, 0xe5e5e5, 0x7f7f7f,
        0xff0000, 0x00ff00, 0xffff00, 0x5c5cff, 0xff00ff, 0x00ffff, 0xffffff,
    ];
    let hex = if index < 16 {
        BASE[usize::from(index)]
    } else if index < 232 {
        let cube = u32::from(index) - 16;
        let channel = |value: u32| if value == 0 { 0 } else { 55 + value * 40 };
        let r = channel(cube / 36);
        let g = channel(cube % 36 / 6);
        let b = channel(cube % 6);
        r << 16 | g << 8 | b
    } else {
        let level = 8 + (u32::from(index) - 232) * 10;
        level << 16 | level << 8 | level
    };
    rgb(hex).into()
}

/// Encodes a keystroke as terminal input bytes.
pub fn encode_key(
    key: &str,
    characters: Option<&str>,
    ctrl: bool,
    alt: bool,
    shift: bool,
    application_cursor: bool,
) -> Option<Vec<u8>> {
    let arrows = |suffix: &str| {
        let prefix = if application_cursor { "\x1bO" } else { "\x1b[" };
        Some(format!("{prefix}{suffix}").into_bytes())
    };
    match key {
        "enter" => Some(vec![b'\r']),
        "backspace" => Some(vec![0x7f]),
        "tab" if shift => Some(b"\x1b[Z".to_vec()),
        "tab" => Some(vec![b'\t']),
        "escape" => Some(vec![0x1b]),
        "up" => arrows("A"),
        "down" => arrows("B"),
        "right" => arrows("C"),
        "left" => arrows("D"),
        "home" => arrows("H"),
        "end" => arrows("F"),
        "insert" => Some(b"\x1b[2~".to_vec()),
        "delete" => Some(b"\x1b[3~".to_vec()),
        "pageup" => Some(b"\x1b[5~".to_vec()),
        "pagedown" => Some(b"\x1b[6~".to_vec()),
        "space" if ctrl => Some(vec![0x00]),
        _ if ctrl && key.len() == 1 => {
            let byte = key.as_bytes()[0].to_ascii_lowercase();
            byte.is_ascii_lowercase().then(|| vec![byte - b'a' + 1])
        }
        _ => {
            let characters = characters?;
            let mut bytes = Vec::with_capacity(characters.len() + 1);
            if alt {
                bytes.push(0x1b);
            }
            bytes.extend_from_slice(characters.as_bytes());
            Some(bytes)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn buffer() -> TerminalBuffer {
        TerminalBuffer::new(4, 10)
    }

    #[test]
    fn carriage_return_overwrites() {
        let mut t = buffer();
        t.push(b"hello\rworld");
        assert_eq!(t.row_text(0), "world");
    }

    #[test]
    fn cursor_addressing_and_erase() {
        let mut t = buffer();
        t.push(b"abc\x1b[1;1HZ");
        assert_eq!(t.row_text(0), "Zbc");
        t.push(b"\x1b[2K");
        assert_eq!(t.row_text(0), "");
    }

    #[test]
    fn split_utf8() {
        let mut t = buffer();
        t.push(&[0xe2, 0x82]);
        t.push(&[0xac]);
        assert_eq!(t.row_text(0), "€");
    }

    #[test]
    fn scrolls_with_bounded_rows() {
        let mut t = TerminalBuffer::new(2, 5);
        t.push(b"one\r\ntwo\r\nthree");
        assert_eq!(t.row_text(0), "two");
        assert_eq!(t.row_text(1), "three");
    }

    #[test]
    fn strips_osc() {
        let mut t = buffer();
        t.push(b"a\x1b]0;title\x07b");
        assert_eq!(t.row_text(0), "ab");
    }

    #[test]
    fn renders_colors_and_attributes() {
        let mut t = buffer();
        t.push(b"\x1b[1;31mred\x1b[0m plain");
        let rows = t.visible_rows();
        let first = rows.first().expect("row");
        assert_eq!(first.text.trim_end(), "red plain");
        // The cursor cell is inverted, so it becomes a separate trailing run.
        assert!(first.runs.len() >= 2);
        assert_eq!(first.runs[0].len, 3);
        assert_eq!(first.runs[0].color, indexed(1));
        assert_eq!(first.runs[0].font.weight, FontWeight::BOLD);
        assert_eq!(first.runs[1].color, default_foreground());
    }

    #[test]
    fn alternate_screen_switches_content() {
        let mut t = buffer();
        t.push(b"normal");
        t.push(b"\x1b[?1049h\x1b[Halt");
        assert!(t.alternate_screen());
        assert_eq!(t.row_text(0), "alt");
        t.push(b"\x1b[?1049l");
        assert!(!t.alternate_screen());
        assert_eq!(t.row_text(0), "normal");
    }

    #[test]
    fn run_lengths_match_text_bytes() {
        let mut t = buffer();
        t.push("\x1b[1;31mbold\x1b[0m plain \x1b[4;44mund[0m €".as_bytes());
        for row in t.visible_rows() {
            let total: usize = row.runs.iter().map(|run| run.len).sum();
            assert_eq!(total, row.text.len());
            assert!(row.text.is_char_boundary(total));
        }
    }

    #[test]
    fn resize_keeps_grid_consistent() {
        let mut t = buffer();
        assert!(t.resize(6, 20));
        assert!(!t.resize(6, 20));
        assert_eq!(t.visible_rows().len(), 6);
    }
}
