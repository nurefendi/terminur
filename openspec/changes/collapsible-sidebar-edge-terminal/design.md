# Design

Store `sidebar_visible: bool` on the GPUI root view. Add a header toggle button; render sidebar conditionally. Derive sidebar width and terminal horizontal insets from visibility and actual padding constants in `fit_grid`. Remove terminal content outer padding and reduce terminal inner padding. Keep header toolbar and status compact. Use the existing `Session::resize` and `TerminalBuffer::resize` paths. Avoid new dependencies.
