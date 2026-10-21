# Specification: Terminal screen MVP

## Requirements
1. Incoming PTY bytes MUST update a persistent terminal screen, rather than append plain output.
2. CR MUST return cursor to column zero; LF MUST advance a row; backspace MUST move left without erasing.
3. Printable characters MUST overwrite the cell under the cursor, with automatic wrapping at the right edge.
4. CSI A/B/C/D/H/f/G/J/K MUST move the cursor or erase cells within bounds.
5. UTF-8 sequences and CSI sequences split across callbacks MUST be handled correctly.
6. Screen memory MUST remain bounded by configured rows and columns.
7. The GPUI view MUST render the screen snapshot and notify on PTY output without notifying from the native reader thread.

## Scenarios
- Progress: `hello\rworld` shows `world`, not `helloworld`.
- Cursor addressing: `abc\x1b[1;1HZ` shows `Zbc`.
- Erase: `abc\x1b[2K` clears the current line.
- Split controls: `\x1b[2` followed by `D` moves cursor left twice.
- Unicode: split UTF-8 bytes produce the intended scalar.
- Scrolling: writing past the last row discards the oldest row.
