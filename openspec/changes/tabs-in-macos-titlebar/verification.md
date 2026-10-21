# Verification

Local macOS checks (2026-10-22): `cargo fmt --all -- --check`, `cargo check --all-targets`, `cargo test --all-targets` (21 passed), `cargo clippy --all-targets -- -D warnings`, and `cargo build --release` passed. GPUI 0.2.2's `TitlebarOptions.appears_transparent` enables `NSFullSizeContentViewWindowMask` on macOS and hides the native title text while retaining window controls. The top 46px GPUI row now contains the tabs/New host and leaves a non-shrinking 104px safe zone for native traffic lights. `fit_grid` still reserves exactly that row.

Not verified manually: appearance, traffic-light click targets, window dragging, fullscreen transition, and terminal behavior in a live SSH tab. The latest public macOS CI observed is [run #25](https://github.com/nurefendi/terminur/actions/runs/39661380412), success on commit `7c3666d`, not these uncommitted changes.
