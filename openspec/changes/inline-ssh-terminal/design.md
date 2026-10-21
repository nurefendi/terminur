# Design

## Files

- `src/terminal.rs`: `TerminalBuffer` wraps `vt100::Parser` with a 10k-row scrollback. `visible_rows()` walks the visible grid, merges adjacent cells with identical style into `gpui::TextRun`s, and inverts the cursor cell. `encode_key` maps GPUI keystrokes to terminal bytes and honors DEC application cursor mode.
- `src/session.rs`: `Session` owns the PTY master, writer, and child process. `connect_host` runs system OpenSSH; `resize` calls `MasterPty::resize`; `Drop` kills the child so switching hosts does not leave SSH processes behind. A test-only `connect_shell` exercises the same PTY path with `/bin/sh`.
- `src/main.rs`: the `TermiNur` view owns the session, shared `TerminalBuffer` behind `Arc<Mutex<_>>`, and a dirty flag.

## Threading

The PTY reader thread only locks the screen mutex to feed the parser and sets an `AtomicBool`. GPUI is never touched from that thread; a `cx.spawn` loop polls the flag every 16 ms and calls `cx.notify()`, stopping when the connection generation changes.

## Sizing

`fit_grid` runs at the start of `render`, converts the viewport into a cell count using `ch_advance` of the monospace font at 13 px with a 17 px line height, then resizes both the parser and the PTY only when the count changes. Layout heights are constants defined next to the render code so the arithmetic matches the actual layout.

## Rendering

Each visible row becomes a `StyledText` with explicit runs. Default background is omitted so the panel background shows through; non-default backgrounds are emitted per run. The font is `Menlo`, narrowed to the terminal grid assumption.

## Tradeoffs

- `vt100` supports the escape sequences most interactive programs need but is not a full xterm implementation: mouse reporting, images, and some Unicode width edge cases are missing.
- Scrollback is view-only; there is no selection or copy yet.
- Fixed layout constants are simpler than a two-pass measurement but must be updated if the chrome changes.

## Testing

Unit tests cover carriage return, cursor addressing and erase, split UTF-8, bounded scrolling, OSC stripping, color/attribute runs, alternate screen switching, and resize bookkeeping. Manual verification is required for real SSH authentication, PTY resize behavior in `vim`, and scrollback.