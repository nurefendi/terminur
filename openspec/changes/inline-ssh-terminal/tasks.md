# Tasks

- [x] Replace the hand-written emulator with the `vt100` parser and styled text runs.
- [x] Wire SSH PTY output into the inline terminal view and remove the external Terminal.app path.
- [x] Add PTY/parser resize from the window grid.
- [x] Encode keyboard, paste, and wheel input.
- [x] Update README for the inline workflow.
- [x] Run formatting, checks, tests, Clippy, and release build (local macOS; remote CI unavailable).
- [x] Keep the scope to SSH: no local-shell entry point in the product UI.
- [ ] Manual verification with a real host: authentication, `vim`, resize, scrollback.
- [ ] Validate Windows and Linux PTY support in a separate change.