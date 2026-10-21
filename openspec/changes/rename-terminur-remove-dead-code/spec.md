# Requirements

- The package and binary SHALL be named `terminur`; the UI SHALL display `TermiNur`.
- The application SHALL prefer `terminur/hosts.json` and SHALL read `rustterm/hosts.json` only when the new file does not exist. It MUST NOT modify either file during loading.
- Given both files exist, when loading hosts, then the new file wins. Given only the legacy file exists, then legacy hosts remain available. Given neither exists or the selected file is invalid, then existing localhost fallback behavior remains.
- Unused saving/resizing helpers and dead-code lint suppressions SHALL be removed. The PTY master SHALL remain owned until the session is dropped; background reader and GPUI notification lifecycle behavior SHALL remain unchanged.
