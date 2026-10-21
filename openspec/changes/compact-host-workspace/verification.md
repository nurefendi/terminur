# Verification

Local macOS checks (2026-10-22): `cargo fmt --all -- --check`, `cargo check --all-targets`, `cargo test --all-targets` (21 passed), `cargo clippy --all-targets -- -D warnings`, and `cargo build --release` passed. The titlebar is left native; duplicate in-app brand/intro rows were removed and three stacked header bars reduced to one 46px row. PTY grid calculation uses the same 46px header instead of the old 58+64+66px reserved area. This is a code/layout calculation, not a claim of pixel-perfect screenshot fidelity.

Real GUI/window-resize and SSH-host testing is still needed; the latest visible CI run is [macOS Rust CI #25](https://github.com/nurefendi/terminur/actions/runs/39661380412), successful on older commit `7c3666d`, not on these uncommitted changes.
