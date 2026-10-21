# SSH session tabs

## Motivation
TermiNur currently replaces the active SSH session when another host is clicked. A Termius-like client needs independent sessions that can be switched without disconnecting.

## Scope
- Clicking a saved host opens a new inline SSH tab (including repeat connections to the same host).
- Switching tabs retains each session's PTY, rendered screen and scrollback.
- Closing a tab terminates only that tab's SSH child; resizing updates every open session.

## Exclusions
Host editing, credentials, SFTP, split panes and Windows/Linux support are separate milestones. No local-shell UI.

## Risks
Each tab holds a child process and reader thread. Closing must release the child and stop its UI polling loop. SSH authentication remains unverified against a real host.
