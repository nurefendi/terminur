# Termius-inspired Hosts layout

## Motivation
The existing terminal-first screen does not resemble the host-management workspace in the user's screenshot. Keep TermiNur's own brand while aligning the desktop information hierarchy: top workspace strip, Hosts navigation, host toolbar, central Create host empty state and existing SSH session tabs.

## Scope
- Dark navy/purple two-column desktop layout, Hosts selected in sidebar, New host toolbar, host cards in workspace, SSH sessions in top tabs.
- Initial empty Hosts view with hostname input and Continue; then existing detail form (name, user, port, optional key). Keep persisted host and SSH behavior.
- Other areas visible as clearly marked "Later" non-interactive labels, not fake working controls. No account/login screen.

## Exclusions
No exact copying of proprietary icons/assets or branding, no new auth, SFTP, keychain, port forwarding, snippets or quick-connect implementation. No redesign of PTY or persistence.

## Risks
GPUI input is basic keyboard handling, not a full IME text widget. Resizing and focus must preserve existing tabs and prevent editor keystrokes reaching the PTY. GUI appearance needs manual visual review.
