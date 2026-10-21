# Verification

Local macOS verification:

- `cargo fmt --all -- --check`: passed.
- `cargo check --all-targets`: passed.
- `cargo test --all-targets`: passed, 7 tests including configuration path precedence.
- `cargo clippy --all-targets -- -D warnings`: passed.
- `git diff --check`: passed.
- `cargo build --release`: passed, optimized `terminur` binary built in 1m 26s.

Formatting, all 7 tests, and strict Clippy also passed after the titlebar naming adjustment. GitHub CI status and workflow links are unavailable (`gh` is not installed). No interactive UI or real SSH runtime verification was performed. Cargo.lock was already untracked before this task; Cargo updated its root package name to match the requested rename.
