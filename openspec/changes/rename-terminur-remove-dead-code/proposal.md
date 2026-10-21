# Rename TermiNur and remove unused code

User-requested scope: replace RustTerm/rustterm branding in active code, package, CI, and documentation; remove unused host-saving and PTY-resizing helpers rather than suppressing dead-code warnings. Keep the PTY owner alive for the SSH session. Prefer the new configuration directory while retaining read-only compatibility with existing rustterm hosts. Termius-like features and a host editor are out of scope. Risks: external scripts must use the new binary name; stale binaries are not removed.
