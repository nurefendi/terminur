# Design

`src/main.rs`: configure `TitlebarOptions.appears_transparent` for macOS only; the native title text is hidden by GPUI and the full-size GPUI content view occupies the titlebar region. Move the existing 46px tab/New host row from the workspace into the root top row, aligned after a fixed traffic-light safe zone. Keep sidebar and workspace below. `fit_grid` still subtracts the one root header. Avoid custom window controls and SSH/process changes. Tests/checks cover compilation and state tests; drag/traffic-light clicks/fullscreen and alignment require hands-on macOS validation.
