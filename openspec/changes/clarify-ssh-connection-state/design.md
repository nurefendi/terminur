# Design

`src/hosts.rs` loads an empty list for missing or invalid config, retaining current-over-legacy precedence. `src/main.rs` renders a short empty-state prompt and reports process startup rather than connection success. The existing per-tab GPUI polling loop checks `Session::try_wait` and updates only its tab, leaving PTY output to the reader thread. No parsing of SSH output as authentication evidence. `README.md` explains that localhost requires Remote Login/sshd and documents configuring a real host.
