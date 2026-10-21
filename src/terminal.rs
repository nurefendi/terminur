/// Stateful byte-stream decoder for the initial terminal output pipeline.
///
/// This is intentionally NOT a full terminal emulator. It handles split UTF-8
/// sequences and common ANSI control sequences without leaking escape codes
/// into the GPUI text view. Cursor positioning and screen buffers come next.
#[derive(Default)]
pub struct OutputDecoder {
    pending_utf8: Vec<u8>,
    escape: Escape,
}

#[derive(Default)]
enum Escape {
    #[default]
    None,
    Start,
    Csi,
    Osc,
    OscEscape,
}

impl OutputDecoder {
    pub fn push(&mut self, bytes: &[u8]) -> String {
        let mut visible = Vec::with_capacity(bytes.len());
        for &byte in bytes {
            match self.escape {
                Escape::None => {
                    if byte == 0x1b {
                        self.escape = Escape::Start;
                    } else if byte == b'\r' {
                        // CRLF becomes a single line break; bare CR is omitted.
                    } else if byte == b'\n' || byte == b'\t' || byte >= 0x20 {
                        visible.push(byte);
                    }
                }
                Escape::Start => {
                    self.escape = match byte {
                        b'[' => Escape::Csi,
                        b']' => Escape::Osc,
                        _ => Escape::None,
                    };
                }
                Escape::Csi => {
                    if (0x40..=0x7e).contains(&byte) {
                        self.escape = Escape::None;
                    }
                }
                Escape::Osc => {
                    if byte == 0x07 {
                        self.escape = Escape::None;
                    } else if byte == 0x1b {
                        self.escape = Escape::OscEscape;
                    }
                }
                Escape::OscEscape => {
                    self.escape = if byte == b'\\' {
                        Escape::None
                    } else {
                        Escape::Osc
                    };
                }
            }
        }
        self.pending_utf8.extend(visible);
        let mut result = String::new();
        loop {
            match std::str::from_utf8(&self.pending_utf8) {
                Ok(s) => {
                    result.push_str(s);
                    self.pending_utf8.clear();
                    break;
                }
                Err(error) => {
                    let valid = error.valid_up_to();
                    if valid > 0 {
                        result.push_str(
                            std::str::from_utf8(&self.pending_utf8[..valid])
                                .expect("validated UTF-8 prefix"),
                        );
                        self.pending_utf8.drain(..valid);
                    }
                    if let Some(len) = error.error_len() {
                        result.push('�');
                        self.pending_utf8.drain(..len);
                    } else {
                        break;
                    }
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::OutputDecoder;

    #[test]
    fn filters_split_ansi_sequences() {
        let mut decoder = OutputDecoder::default();
        assert_eq!(decoder.push(b"hello\x1b[31"), "hello");
        assert_eq!(decoder.push(b"mred\x1b[0m!"), "red!");
    }

    #[test]
    fn preserves_split_utf8() {
        let mut decoder = OutputDecoder::default();
        assert_eq!(decoder.push(&[0xe2, 0x82]), "");
        assert_eq!(decoder.push(&[0xac]), "€");
    }

    #[test]
    fn strips_osc_titles() {
        let mut decoder = OutputDecoder::default();
        assert_eq!(decoder.push(b"a\x1b]0;secret\x07b"), "ab");
    }

    #[test]
    fn preserves_line_breaks() {
        let mut decoder = OutputDecoder::default();
        assert_eq!(decoder.push(b"one\r\ntwo\n"), "one\ntwo\n");
    }
}
