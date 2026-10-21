# Design

Remove duplicated branding and `INTRO_BAR_HEIGHT`/`WORKSPACE_BAR_HEIGHT` in `src/main.rs`. Render the sidebar and workspace side-by-side directly below the macOS titlebar. The workspace has one short header containing existing tab strip and New host button, and the existing host/editor/terminal content beneath. Reduce sidebar padding and recompute `fit_grid` using one header, not three; do not change PTY ownership, navigation callbacks, or persistence. Check formatting/tests/Clippy/release build and manually inspect UI/layout later.
