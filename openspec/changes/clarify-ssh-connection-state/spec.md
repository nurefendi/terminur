# Requirements

- Given no hosts file or malformed JSON, the host list SHALL be empty rather than inventing a localhost SSH server.
- Given an explicit localhost entry in a valid current or legacy file, the entry SHALL remain available.
- Given a spawned OpenSSH process, the UI MUST NOT claim authentication has succeeded; when it exits, the tab SHALL report its exit status while preserving the terminal output.
- Given multiple tabs, exit state for one SSH process SHALL NOT update another tab.
