# Verification

Local macOS checks (2026-10-22): `cargo fmt --all -- --check`, `cargo check --all-targets`, `cargo test --all-targets` (13 passed), `cargo clippy --all-targets -- -D warnings`, and `cargo build --release` all passed. Host-loading test covers missing, explicit legacy localhost, and invalid current JSON. PTY test exercises `try_wait` on exit. A real SSH server, the GUI empty state, and SSH exit display have not been manually validated. No CI status was checked in this change.
