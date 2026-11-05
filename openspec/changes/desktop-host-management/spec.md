# Requirements

- Editing SHALL prefill all five existing host fields and replace the selected entry, not append it.
- Given a valid saved host, when editing is cancelled, then disk and the current host list MUST remain unchanged.
- Deletion MUST require a separate explicit confirmation. Enter in the confirmation screen SHALL NOT confirm; Escape or Keep host cancels.
- Given malformed existing JSON or a write failure, when saving/deleting, then the operation SHALL display an error and retain the current list and editor.
- Given an active SSH session, when its saved host is changed/deleted, then its tab, title and running connection SHALL remain unchanged; future connections use the saved configuration.
- Host mutations SHALL execute on the GPUI UI thread. New/edit forms SHALL reset pending deletion state. Existing OpenSSH host-key checks and secret-storage policy MUST remain unchanged.
