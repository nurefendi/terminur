# Specification

- Header MUST contain a visible control to hide/show sidebar.
- Sidebar MUST be absent from layout when hidden and restored when shown.
- Terminal panel MUST occupy available content width with no redundant outer padding.
- Terminal inner padding MUST be small and explicitly included in grid calculations.
- Toggling sidebar MUST resize terminal screen and PTY without closing sessions.
- Terminal font zoom MUST remain functional in both modes.

## Scenarios
- Toggle while connected: session remains open and PTY columns increase when sidebar hides.
- Toggle again: sidebar returns and PTY columns decrease.
- Font zoom after toggle: rows and columns recalculate correctly.
