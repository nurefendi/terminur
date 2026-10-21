# Specification

- Terminal toolbar MUST provide decrement, increment and reset controls for font size.
- Font size MUST be clamped to 10–24 px, with default 13 px.
- Changing font size MUST update row heights, text size and terminal cell calculations together.
- PTY dimensions MUST be updated through existing resize integration after zoom changes.
- Terminal toolbar MUST remain visible above the terminal without stealing keyboard focus permanently.
- Terminal state, tabs and SSH sessions MUST survive zoom changes.
- The dark UI MUST distinguish active tab, content background, terminal and connection status.

## Scenarios
- At default 13 px, A+ yields 14 px; A− returns to 13 px.
- Repeated zoom out stops at 10 px; repeated zoom in stops at 24 px.
- Reset returns any valid size to 13 px.
- Zooming while connected retains screen content and resizes the PTY grid.
