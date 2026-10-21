# Clarify SSH connection state

The bundled localhost entry implies that an SSH server is running on the user's Mac; the screenshot shows a refused connection. Remove the implicit localhost host when no valid config exists and avoid calling a freshly spawned `ssh` process "Connected" before authentication succeeds. Preserve explicitly saved localhost entries and legacy config lookup. A refused connection must remain visible as OpenSSH output, with the tab status updated after process exit. No host editor or password storage in this fix.
