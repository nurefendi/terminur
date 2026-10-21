# Verification

Local macOS run:

- `cargo fmt --all -- --check`: passed.
- `cargo check --all-targets`: passed.
- `cargo test --all-targets`: passed, 10 tests.
- `cargo clippy --all-targets -- -D warnings`: passed.
- `cargo build --release`: passed.
- `git diff --check`: passed.
- Release binary smoke launch: the process stayed alive for 4 seconds and logged no errors, then was terminated.

Not verified: an authenticated SSH session, host-key prompt rendering, `vim`/`htop` behavior, PTY resize against a real remote shell, scrollback with real output, and Windows/Linux builds. `gh` is unavailable, so no GitHub Actions run could be inspected. Remote CI is unknown. Manual testing with an authorized host is required before claiming the terminal is usable end to end.