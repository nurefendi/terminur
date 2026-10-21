# Design: Screen-buffer integration

## Data flow
portable-pty reader -> byte callback -> `TerminalScreen::push` under Arc<Mutex<_>> -> AtomicBool dirty -> GPUI async timer -> cx.notify -> render snapshot.

## Model
`TerminalScreen` owns a fixed rows x cols matrix of chars, cursor row/column, a bounded parser state and pending UTF-8 bytes. The parser recognizes ESC, CSI and OSC (OSC content discarded). ANSI attributes are not yet rendered; SGR sequences are consumed.

## UI
GPUI renders the screen as a monospaced text snapshot. This phase favors deterministic terminal semantics over advanced cell-by-cell styling. The SSH PTY remains at 100 x 30 until resize handling is wired to window layout.

## Safety and performance
Bounded cells and CSI parameter buffer; no GPUI API calls from PTY reader threads. No credentials are logged. Snapshot cloning occurs only on UI redraw.

## Validation
cargo fmt --check; cargo check; cargo test; cargo clippy --all-targets -- -D warnings; cargo build --release; manual SSH smoke test on macOS.
