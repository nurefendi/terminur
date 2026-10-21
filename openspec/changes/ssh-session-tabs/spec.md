# Requirements

- The application SHALL create a separate SSH PTY and terminal buffer for each host click, including repeated clicks on the same host.
- The application SHALL show a selectable, closable tab for each session; keyboard, paste and wheel events MUST affect only the active tab.
- Given two tabs with different output, when switching between them, then each tab SHALL retain its own display and scrollback.
- Given a tab is closed, when other tabs remain, then only that tab's child SHALL be terminated and another tab SHALL become active.
- Given the last tab is closed, then the terminal SHALL show an empty welcome state and input SHALL not be sent to any process.
- Given the window changes size, then each open PTY and parser SHALL receive the new grid size; a tab opened later SHALL start with the current grid.
- Reader threads MUST NOT call GPUI directly, and a closed tab's UI polling loop MUST stop.
- A spawn failure SHALL show the error in its tab without terminating existing tabs.
