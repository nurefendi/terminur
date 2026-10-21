# TermiNur

**An early-stage SSH client for macOS with an inline terminal, built with Rust and GPUI.**

TermiNur is an early-stage community project exploring a lightweight native-feeling SSH experience. Contributions, testing reports, accessibility feedback, and design discussions are welcome.

> **Status: prototype.** The terminal is inline in the TermiNur window. Do not rely on this app as your only way to administer production systems.

## What works today

- macOS application UI built with **Rust + GPUI** (no Electron, Tauri, Flutter, or SwiftUI).
- Inline SSH sessions in tabs: system OpenSSH runs in separate pseudo-terminals and output is rendered inside the TermiNur window. Clicking a host again opens another independent session; close its tab to terminate that session.
- Terminal emulation via the `vt100` parser: SGR colors, bold/italic/underline, cursor addressing, erase, scroll regions, alternate screen, and scrolling history.
- Keyboard input including Enter, Backspace, Tab, arrows, Home/End, Page Up/Down, Delete, Escape, Ctrl combinations, Alt prefixes, bracketed paste (Cmd+V), and mouse-wheel scrollback.
- The pseudo-terminal is resized to the window grid so full-screen programs such as `vim`, `nano`, and `htop` receive the correct size.
- Add SSH hosts in the app (+ New host) with name, hostname/IP, username, port, and optional SSH private key path; saved to local JSON configuration. Existing hosts can also be configured manually.
- A compact Hosts workspace inspired by the supplied desktop reference: sidebar, host cards, SSH tabs in the macOS titlebar area and an address-first Create host flow. TermiNur retains its own branding; unfinished sections are marked “Later”. The native window controls are retained.
- Automated macOS CI: formatting, type checking, unit tests, Clippy, release build, and binary artifact upload.

## What is not ready yet

Windows and Linux have not been implemented or tested yet. Android and iOS are later milestones. Host editing/deleting, credentials storage, SFTP, mouse reporting, image protocols, IME composition, and text selection/copy are not implemented. Keychain, Port Forwarding, Snippets, Known Hosts, Logs and SFTP are only **Later** labels, not working screens. The Add host form supports basic keyboard editing and paste, not full IME/selection. Scrollback is view-only; the terminal does not yet expose search or copy of history. Multi-tab SSH and this layout still need manual GUI testing against a real host.

Host-key verification is performed by OpenSSH. Check server fingerprints carefully. TermiNur does not store SSH passwords or private key contents/passphrases; only the selected key's file path is saved.

## Requirements

- macOS
- Full Xcode with Metal Toolchain (required to compile GPUI). Command Line Tools alone may not suffice.
- Stable Rust toolchain: https://rustup.rs/
- macOS OpenSSH executable at `/usr/bin/ssh`

## Quick start

```bash
git clone https://github.com/nurefendi/terminur.git
cd terminur
git switch feat/mvp-1-2-terminal-emulator

cargo test
cargo run
```

The Cargo package and binary are named `terminur`; the application is branded **TermiNur**. On the Hosts screen, enter an IP/hostname and click **Continue** (or use **+ New host**), complete the host details and choose **Save host**. Click a saved host card to open an inline SSH session. The sidebar **Hosts** entry or top **Workspace** tab returns to the workspace without closing SSH tabs; select a session tab to resume it. Click the terminal panel to type. Host-key prompts and password prompts appear in the terminal panel.

## Configure SSH hosts

The **+ New host** button saves host details here on macOS (you can also edit the file manually):

```text
~/Library/Application Support/terminur/hosts.json
```

If editing manually, create the parent directory if necessary. Example:

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

Replace the documentation-only address `192.0.2.10` with a server you are authorized to access. Hosts added in the app appear immediately; restart the app after changing JSON manually. If the file is missing or invalid, the host list is empty. The app refuses to save over malformed existing JSON; repair that file manually first. An explicitly saved localhost entry remains available.

The exact configuration directory is resolved by Rust's `dirs::config_dir()` on the host system. If the new file does not exist, TermiNur still reads `~/Library/Application Support/rustterm/hosts.json` for compatibility. Saving an additional host in that case updates the existing legacy file without moving it. If the new file exists but is invalid, the host list is empty rather than loading the legacy file.

**Using an SSH key:** In the optional **SSH private key path** field, enter an existing readable key file such as `~/.ssh/id_ed25519` (or an absolute path). This saves only the path under `identity_file` in `hosts.json` and passes the expanded path to system OpenSSH using `-i` and `IdentitiesOnly=yes`. The server must have the matching public key authorized; OpenSSH handles passphrase prompts. Leave the field blank to keep OpenSSH's default `~/.ssh/config` / ssh-agent behavior. Key files are never copied into the app or the host list. A manually edited entry pointing to a missing key will show a connection-start error, not silently fall back to another identity.

If you explicitly connect to `localhost` and see `Connection refused` on port 22, macOS is not accepting SSH there. For a remote server, configure its actual hostname/IP and SSH port instead. To SSH into your own Mac, enable **Remote Login** in macOS System Settings → General → Sharing (only if you intend to run an SSH server). Starting the `ssh` program is not proof that authentication succeeded; check its terminal output and exit status.

## Help test inline SSH

Click a configured host, click the terminal panel to focus it, then run:

```bash
whoami
pwd && ls -la
printf 'hello\rworld\n'
printf 'abc\033[1;1HZ\n'
seq 1 50
```

Please report whether host verification, authentication, typing, colors, cursor placement, resizing, and full-screen programs behave correctly. `vim`, `nano`, and `htop` are expected to work now, but color fidelity and Unicode cell widths are still approximate.

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

- **Current milestone:** inline terminal SSH on macOS.
- **Next:** complete host management (edit/delete), credentials, SFTP, and manual validation of tabs and Add host with real SSH hosts.
- **Later:** Windows/Linux support (validate PTY and platform behavior per OS), text selection/copy, packaging, and mobile platform designs.

See the OpenSpec change documents for implementation scope and acceptance criteria.

## Security and privacy

SSH authentication and host-key prompts are handled by the system OpenSSH client. Do not disable host-key checking to work around a connection warning.

Do not commit real `hosts.json` files, credentials, private SSH keys, access tokens, or `.env` files. Review `git status` and diffs before pushing. Use RFC 5737 example IP addresses in documentation.

**Security concerns:** avoid posting secrets or exploit details publicly. Contact a repository maintainer privately through an appropriate channel if one is available.

## License

A license file has **not yet been added**. Until a license is selected and committed, the repository should not be assumed to grant open-source redistribution or modification rights. Contributors should discuss licensing with the maintainer before substantial contributions.

---

If TermiNur interests you, consider **starring the repository**, testing an early build, sharing constructive feedback, or contributing a small improvement. Community participation helps shape the project's direction.
