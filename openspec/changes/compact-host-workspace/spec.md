# Requirements

- Given the app opens, the native titlebar SHALL be the sole TermiNur brand title; the workspace SHALL have one compact row rather than stacked empty top bars.
- The sidebar SHALL start with Hosts; the compact row SHALL retain Hosts/tab switching, tab closing, and New host creation.
- When switching between Hosts and SSH sessions, PTYs MUST remain alive; keyboard input MUST continue to route only to the active terminal or form.
- Window resizing SHALL update PTYs to dimensions calculated from the actual reduced header layout.
- Given a narrow window, users SHALL still be able to access tabs through horizontal scrolling; the host form SHALL remain available.
