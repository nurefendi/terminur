# Verification

Local macOS verification (2026-10-22): `cargo fmt --all -- --check`, `cargo check --all-targets`, `cargo test --all-targets` (20 passed), `cargo clippy --all-targets -- -D warnings`, and `cargo build --release` passed.

Tests cover legacy JSON with no identity field, path-only serialization/roundtrip, missing/relative/directory path rejection, `~/` expansion, valid form selection, SSH argv containing one `-i` path argument plus `IdentitiesOnly=yes`, and default SSH argv without these options. Test fixtures contain no real private keys. No real SSH authentication, passphrase entry, or GUI interaction has been verified.

Latest public macOS CI run visible during exploration: [macOS Rust CI #25](https://github.com/nurefendi/terminur/actions/runs/39661380412), success on commit `7c3666d` (2026-10-21). It does **not** cover these uncommitted changes; no CI result is claimed for them.
