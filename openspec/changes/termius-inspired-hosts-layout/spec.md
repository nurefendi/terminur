# Requirements

- Given no hosts, the application SHALL open on the Hosts workspace showing centered Create host messaging, a hostname field and Continue; it SHALL not connect automatically.
- Given a valid hostname, Continue SHALL prefill host details and open the existing full form; a blank/invalid hostname SHALL show an error without leaving the step.
- Given Save, the host SHALL be persisted using existing validation and appear in Hosts workspace immediately; clicking its card SHALL open a separate SSH tab.
- Given multiple SSH tabs, clicking Hosts SHALL display the host workspace without disconnecting them; clicking an existing tab SHALL restore its terminal, and closing a tab SHALL terminate only that session.
- Form keyboard events SHALL not reach an SSH tab. Cancel or form save SHALL NOT kill existing sessions.
- Unimplemented areas SHALL be visibly labeled unavailable and SHALL NOT look like functioning buttons. TermiNur SHALL not claim Termius account/sync or SFTP functionality.
