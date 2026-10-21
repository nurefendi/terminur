# Design

Add a `font_size: f32` field to `TermiNur` and pure `clamp_font_size`/`terminal_line_height` helpers. `fit_grid` uses the active font size for font metrics and row heights. On each zoom action, call `fit_grid(window, cx)` and `cx.notify()` so the terminal buffer and PTY are resized together. Replace fixed hint text with a toolbar containing compact text zoom buttons, current font-size indicator, and focus hint. Use consistent dark navy/slate surfaces and restrained accent colors. Keep the implementation in existing GPUI render code and avoid new dependencies.
