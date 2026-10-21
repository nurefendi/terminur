# Requirements

- Given a configured host, when the user clicks it, TermiNur SHALL start `/usr/bin/ssh` in a pseudo-terminal and render its output inside the application window.
- The terminal SHALL interpret ANSI/VT sequences via the `vt100` parser, including SGR foreground/background colors, bold, italic, underline, inverse, cursor addressing, erase, scroll regions, and the alternate screen.
- The terminal SHALL display a block cursor at the parser cursor position by inverting the cell, and SHALL hide it when the application hides the cursor or while the view is scrolled back.
- The PTY size and the parser grid SHALL be derived from the terminal panel size and kept in sync via `TIOCSWINSZ`, so full-screen programs receive correct dimensions.
- Given a keystroke, when the terminal has focus, TermiNur SHALL send the corresponding bytes. Arrow/Home/End SHALL honor DEC application cursor mode; Tab, Shift+Tab, Enter, Backspace, Escape, Delete, Insert, Page Up/Down, Ctrl combinations, and Alt prefixes SHALL be encoded. Cmd+V SHALL paste, wrapped in bracketed-paste markers when the remote enabled that mode.
- Given mouse-wheel input, when the pointer is over the terminal, TermiNur SHALL move through the parser scrollback buffer and MUST NOT send the wheel to the remote host.
- Given a new host selection, when a session already exists, TermiNur SHALL terminate the previous SSH child process before starting the next one.
- The application MUST NOT claim Windows, Linux, Android, or iOS support in this change.