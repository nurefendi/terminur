# Design

`src/main.rs` owns a list of `Tab` values. Each owns its own optional SSH session, shared terminal buffer, dirty flag, scroll offset and status. The application holds the active tab ID and current grid. Clicking a host allocates a new tab; closing removes it, dropping its child. PTY callbacks update only that tab's buffer from a native thread; one GPUI async polling loop per successfully spawned tab checks its dirty flag and exits when its ID is removed. Resizing all tabs avoids stale PTY dimensions on switching. The UI renders only the active buffer; a tab strip sits above the terminal header.

`src/session.rs` continues to own the SSH/PTY lifecycle; no SSH operations are added to rendering. Repeated hosts are distinguished by unique tab IDs, not host indices. Tests cover tab selection/closure independent of GPUI, plus existing PTY and parser tests. macOS build checks are necessary but real-host behavior still needs manual verification.
