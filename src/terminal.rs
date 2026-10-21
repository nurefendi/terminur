/// A bounded, stateful screen for PTY output.
///
/// This MVP supports cursor addressing and erase commands; it is not a complete
/// xterm emulator (no alternate screen, SGR colors, or wide-character cells).
pub struct TerminalScreen {
    cells: Vec<Vec<char>>,
    row: usize,
    col: usize,
    rows: usize,
    cols: usize,
    escape: Escape,
    utf8: Vec<u8>,
}

#[derive(Default)]
enum Escape {
    #[default]
    None,
    Start,
    Csi(Vec<u8>),
    Osc,
    OscEnd,
}

impl TerminalScreen {
    pub fn new(rows: usize, cols: usize) -> Self {
        let rows = rows.max(1);
        let cols = cols.max(1);
        Self {
            cells: vec![vec![' '; cols]; rows],
            row: 0,
            col: 0,
            rows,
            cols,
            escape: Escape::None,
            utf8: Vec::new(),
        }
    }

    pub fn push(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            match &mut self.escape {
                Escape::None => match byte {
                    0x1b => {
                        self.flush_incomplete_utf8();
                        self.escape = Escape::Start;
                    }
                    b'\r' => self.col = 0,
                    b'\n' => self.line_feed(),
                    0x08 => self.col = self.col.saturating_sub(1),
                    b'\t' => {
                        let next = (self.col / 8 + 1) * 8;
                        self.col = next.min(self.cols - 1);
                    }
                    0x00..=0x1f | 0x7f => {}
                    _ => self.print_byte(byte),
                },
                Escape::Start => {
                    self.escape = match byte {
                        b'[' => Escape::Csi(Vec::new()),
                        b']' => Escape::Osc,
                        _ => Escape::None,
                    };
                }
                Escape::Csi(buf) => {
                    if (0x40..=0x7e).contains(&byte) {
                        let params = std::mem::take(buf);
                        self.escape = Escape::None;
                        self.apply_csi(&params, byte);
                    } else if buf.len() < 64 {
                        buf.push(byte);
                    } else {
                        self.escape = Escape::None;
                    }
                }
                Escape::Osc => {
                    if byte == 0x07 {
                        self.escape = Escape::None;
                    } else if byte == 0x1b {
                        self.escape = Escape::OscEnd;
                    }
                }
                Escape::OscEnd => {
                    self.escape = if byte == b'\\' {
                        Escape::None
                    } else {
                        Escape::Osc
                    };
                }
            }
        }
    }

    fn flush_incomplete_utf8(&mut self) {
        if !self.utf8.is_empty() {
            self.utf8.clear();
            self.put('�');
        }
    }

    fn print_byte(&mut self, byte: u8) {
        self.utf8.push(byte);
        loop {
            match std::str::from_utf8(&self.utf8) {
                Ok(s) => {
                    let chars: Vec<char> = s.chars().collect();
                    self.utf8.clear();
                    for ch in chars {
                        self.put(ch);
                    }
                    break;
                }
                Err(err) => {
                    let valid = err.valid_up_to();
                    if valid > 0 {
                        let chars: Vec<char> =
                            String::from_utf8_lossy(&self.utf8[..valid]).chars().collect();
                        self.utf8.drain(..valid);
                        for ch in chars {
                            self.put(ch);
                        }
                    }
                    if let Some(len) = err.error_len() {
                        self.utf8.drain(..len);
                        self.put('�');
                    } else {
                        break;
                    }
                }
            }
        }
    }

    fn put(&mut self, ch: char) {
        if self.col >= self.cols {
            self.col = 0;
            self.line_feed();
        }
        self.cells[self.row][self.col] = ch;
        self.col += 1;
    }

    fn line_feed(&mut self) {
        if self.row + 1 == self.rows {
            self.cells.rotate_left(1);
            self.cells[self.rows - 1].fill(' ');
        } else {
            self.row += 1;
        }
    }

    fn apply_csi(&mut self, raw: &[u8], command: u8) {
        let text = String::from_utf8_lossy(raw);
        if text.starts_with('?') {
            return;
        }
        let args: Vec<usize> = text
            .split(';')
            .take(8)
            .map(|part| part.parse::<usize>().unwrap_or(0).min(10_000))
            .collect();
        let first = args.first().copied().unwrap_or(0);
        let count = first.max(1);
        match command {
            b'A' => self.row = self.row.saturating_sub(count),
            b'B' => self.row = self.row.saturating_add(count).min(self.rows - 1),
            b'C' => self.col = self.col.saturating_add(count).min(self.cols - 1),
            b'D' => self.col = self.col.saturating_sub(count),
            b'G' => self.col = count.saturating_sub(1).min(self.cols - 1),
            b'H' | b'f' => {
                self.row = count.saturating_sub(1).min(self.rows - 1);
                self.col = args.get(1).copied().unwrap_or(1).max(1).saturating_sub(1).min(self.cols - 1);
            }
            b'J' => match first {
                2 | 3 => {
                    for row in &mut self.cells {
                        row.fill(' ');
                    }
                }
                0 => {
                    self.cells[self.row][self.col.min(self.cols - 1)..].fill(' ');
                    for row in &mut self.cells[self.row + 1..] {
                        row.fill(' ');
                    }
                }
                1 => {
                    for row in &mut self.cells[..self.row] {
                        row.fill(' ');
                    }
                    self.cells[self.row][..=self.col.min(self.cols - 1)].fill(' ');
                }
                _ => {}
            },
            b'K' => match first {
                0 => self.cells[self.row][self.col.min(self.cols - 1)..].fill(' '),
                1 => self.cells[self.row][..=self.col.min(self.cols - 1)].fill(' '),
                2 => self.cells[self.row].fill(' '),
                _ => {}
            },
            _ => {}
        }
    }

    pub fn snapshot(&self) -> String {
        self.cells
            .iter()
            .map(|row| row.iter().collect::<String>().trim_end().to_owned())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl Default for TerminalScreen {
    fn default() -> Self {
        Self::new(30, 100)
    }
}

#[cfg(test)]
mod tests {
    use super::TerminalScreen;

    #[test]
    fn carriage_return_overwrites() {
        let mut t = TerminalScreen::new(2, 10);
        t.push(b"hello\rworld");
        assert_eq!(t.snapshot().lines().next(), Some("world"));
    }

    #[test]
    fn cursor_addressing_and_erase() {
        let mut t = TerminalScreen::new(2, 10);
        t.push(b"abc\x1b[1;1HZ");
        assert_eq!(t.snapshot().lines().next(), Some("Zbc"));
        t.push(b"\x1b[2K");
        assert_eq!(t.snapshot().lines().next(), Some(""));
    }

    #[test]
    fn split_csi() {
        let mut t = TerminalScreen::new(2, 10);
        t.push(b"abcd\x1b[2");
        t.push(b"DZ");
        assert_eq!(t.snapshot().lines().next(), Some("abZd"));
    }

    #[test]
    fn split_utf8() {
        let mut t = TerminalScreen::new(2, 10);
        t.push(&[0xe2, 0x82]);
        t.push(&[0xac]);
        assert_eq!(t.snapshot().lines().next(), Some("€"));
    }

    #[test]
    fn scrolls_with_bounded_rows() {
        let mut t = TerminalScreen::new(2, 5);
        t.push(b"one\r\ntwo\r\nthree");
        assert_eq!(t.snapshot(), "two\nthree");
    }

    #[test]
    fn strips_osc() {
        let mut t = TerminalScreen::new(2, 10);
        t.push(b"a\x1b]0;title\x07b");
        assert_eq!(t.snapshot().lines().next(), Some("ab"));
    }
}
