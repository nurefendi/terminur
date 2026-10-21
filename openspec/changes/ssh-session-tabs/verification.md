# Verification

Local macOS commands after formatting (2026-10-22):

- `cargo fmt --all -- --check` — passed.
- `cargo check --all-targets` — passed.
- `cargo test --all-targets` — passed (13 tests, including two tab selection tests).
- `cargo clippy --all-targets -- -D warnings` — passed.
- `cargo build --release` — passed.

Not verified: two live authenticated SSH sessions, UI interactions and resize against real hosts. GitHub Actions status was not checked: `gh` is unavailable in this environment; no workflow link available.
