# Add SSH host form

## Motivation
Users currently have to hand-edit `hosts.json` before TermiNur can connect to a real SSH server. Add a simple in-app "Add host" form to create a saved SSH target.

## Scope
- Name, hostname/IP, username, and SSH port fields; validate before saving.
- Save to the existing host configuration path, retaining legacy-file compatibility; reflect saved hosts in the sidebar immediately.
- Provide clear validation and write-error feedback without discarding form contents or existing hosts.

## Exclusions
Editing/deleting existing hosts, credentials, SFTP, shell-local mode, automatic connection, and IME/full text-selection behavior are separate work. No real host address or secret will be invented or stored.

## Risk
The existing loader treats invalid JSON as an empty list. Saving must refuse to overwrite such a file. Test-only builds do not prove GUI focus/typing behavior.
