# TermiNur

**An open-source-minded SSH desktop client for macOS, built with Rust and GPUI.**

TermiNur is an early-stage community project exploring a lightweight native-feeling SSH experience. Contributions, testing reports, accessibility feedback, and design discussions are welcome.

> **Status: prototype / MVP 1.2 in development.** The terminal screen implementation is experimental. Do not rely on this app as your only way to administer production systems.

## What works today

- macOS application UI built with **Rust + GPUI** (no Electron, Tauri, Flutter, or SwiftUI).
- SSH sessions via the system OpenSSH executable and a pseudo-terminal (`portable-pty`).
- Host definitions loaded from local JSON configuration.
- Keyboard input for basic typing, Enter, Backspace, Tab, arrow keys, and common Ctrl combinations.
- MVP 1.2 branch: bounded terminal screen with basic cursor movement, erase controls, streaming UTF-8, and ANSI sequence handling.
- Automated macOS CI: formatting, type checking, unit tests, Clippy, release build, and binary artifact upload.

## What is not ready yet

This is **not a fully compatible xterm terminal emulator**. ANSI colors and styles, alternate-screen mode, mouse protocols, Unicode character-width handling, paste/IME, automatic PTY resizing, tabs, SFTP, and a graphical host editor are not implemented. Full-screen programs such as `vim`, `nano`, and `htop` may render incorrectly.

Host-key verification is performed by OpenSSH. Check server fingerprints carefully. TermiNur does not store SSH passwords.

## Requirements

- macOS
- Xcode Command Line Tools: `xcode-select --install`
- Stable Rust toolchain: https://rustup.rs/
- macOS OpenSSH executable at `/usr/bin/ssh`

## Quick start

```bash
git clone https://github.com/nurefendi/terminur.git
cd terminur

# MVP 1.2 is currently in a feature branch / draft PR.
git fetch origin
git switch feat/mvp-1-2-terminal-emulator

cargo test
cargo run
```

For the current stable branch, use `git switch main` instead. The app name in Cargo is currently `rustterm`; the project is branded **TermiNur**.

## Configure SSH hosts

On macOS, edit:

```text
~/Library/Application Support/rustterm/hosts.json
```

Create the parent directory if necessary. Example:

```json
[
  {
    "name": "Development Server",
    "hostname": "192.0.2.10",
    "username": "developer",
    "port": 22
  }
]
```

Replace the documentation-only address `192.0.2.10` with a server you are authorized to access. Restart the app after changing the file; host editing in the UI is not available yet. If the file is missing or invalid, the app falls back to a localhost entry.

The exact configuration directory is resolved by Rust's `dirs::config_dir()` on the host system.

## Help test MVP 1.2

Connect to an SSH host, click the terminal panel to focus it, and try:

```bash
whoami
pwd && ls -la
printf 'hello\rworld\n'
printf 'abc\033[1;1HZ\n'
seq 1 50
```

Please report whether output, cursor overwrites, keyboard input, and scrolling behave correctly. You can also test `vim`, `nano`, or `htop`, but full-screen compatibility is **not yet expected**.

When reporting an issue, include your macOS version, TermiNur branch/commit, steps to reproduce, expected vs. actual behavior, and sanitized screenshots or logs. **Never include private keys, passwords, tokens, public IP addresses you want kept private, or sensitive terminal output.**

## Community and contributions

We welcome:

- **Bug reports:** https://github.com/nurefendi/terminur/issues
- **Feature ideas and discussions:** open an issue describing the problem and use case.
- **Code and documentation contributions:** fork the repository, create a focused feature branch, and open a pull request.
- **Testing:** macOS/SSH compatibility feedback and reproducible terminal-output cases.
- **Translations, accessibility, and UX feedback.**

Before submitting a PR:

1. Read `openspec/README.md` and add `proposal.md`, `spec.md`, `design.md`, and `tasks.md` under `openspec/changes/<change-id>/` for substantive changes.
2. Keep changes scoped and include tests where possible.
3. Run `cargo fmt --all -- --check`, `cargo check`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, and `cargo build --release`.
4. Open a **draft PR** until CI and applicable manual macOS checks pass. Do not push directly to `main`.

Maintainers review proposals and PRs as time allows; please keep discussion respectful and constructive.

## Project roadmap

- **MVP 1.2:** stateful terminal screen and baseline regression tests — in draft PR.
- **Next:** ANSI styling, alternate screen, Unicode cell widths, dynamic PTY resize, and interactive-program compatibility.
- **Later:** graphical host management, multiple SSH sessions/tabs, packaging, and additional SSH workflows.

See the OpenSpec change documents for implementation scope and acceptance criteria.

## Security and privacy

SSH authentication and host-key prompts are handled by the system OpenSSH client. Do not disable host-key checking to work around a connection warning.

Do not commit real `hosts.json` files, credentials, private SSH keys, access tokens, or `.env` files. Review `git status` and diffs before pushing. Use RFC 5737 example IP addresses in documentation.

**Security concerns:** avoid posting secrets or exploit details publicly. Contact a repository maintainer privately through an appropriate channel if one is available.

## License

A license file has **not yet been added**. Until a license is selected and committed, the repository should not be assumed to grant open-source redistribution or modification rights. Contributors should discuss licensing with the maintainer before substantial contributions.

---

If TermiNur interests you, consider **starring the repository**, testing an early build, sharing constructive feedback, or contributing a small improvement. Community participation helps shape the project's direction.
