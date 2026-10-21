# RustTerm (macOS prototype)

Rust + GPUI desktop SSH client prototype. No Flutter, Electron, Tauri or SwiftUI.

## Prerequisites
- macOS with Xcode Command Line Tools: `xcode-select --install`
- Rust stable via rustup
- OpenSSH client (`/usr/bin/ssh`, provided by macOS)

## Run
```sh
cargo check
cargo run
```

## Hosts
Edit `~/Library/Application Support/rustterm/hosts.json` (path may vary with dirs crate), or use the example `hosts.example.json`. The app currently reads the file at startup; it does not yet include a host-editing form.

## Important limitations
- **Unverified build:** generated in an environment without Rust/Cargo and without network access. GPUI version/API compatibility must be checked on macOS.
- The terminal panel is **not yet a real ANSI terminal emulator**. Raw escape sequences may appear, and interactive full-screen programs like vim/htop will not render correctly.
- Output is stored by a PTY reader thread but **does not automatically repaint** until a GPUI event (e.g. key press).
- No automatic terminal resize, paste/IME, mouse handling, host-editing UI, tabs, or SFTP yet.
- Host-key prompts are delegated to OpenSSH. Review fingerprints carefully before trusting them. Passwords are not saved.
- This prototype is macOS desktop only. GPUI Android/iOS backends are not provided.

## Next steps
1. Validate and pin GPUI version and fix any API mismatches on macOS.
2. Add a GPUI foreground event channel to notify on PTY output.
3. Add a proper terminal state machine/ANSI parser and grid renderer.
4. Implement a GPUI add/edit/remove host dialog with `hosts::save`.
5. Implement resize and full key mapping, plus session lifecycle.

## Repository security
Only commit example host definitions with documentation-only IP addresses (RFC 5737).
Never commit real host configurations, passwords, SSH private keys, tokens, or `.env` files.
The `.gitignore` excludes common sensitive files, but review `git status` before each push.
