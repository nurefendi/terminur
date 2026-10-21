# Requirements

- Given a user clicks Add host, the app SHALL show fields for name, hostname, username, and port with Save/Cancel controls and keyboard entry, paste, Tab navigation, and Backspace.
- Given valid fields, Save SHALL persist the host and update the sidebar immediately without starting SSH. A repeat launch SHALL load the saved entry.
- Given blank, invalid, or out-of-range fields, Save SHALL show a validation error and leave the form contents and host list unchanged.
- Given a write error or malformed existing hosts file, Save MUST NOT claim success or overwrite a malformed existing file. The form SHALL remain available for correction or cancellation.
- Given a valid legacy host file and no new host file, Save SHALL retain existing hosts rather than discarding them.
- Given other SSH tabs exist, opening/canceling/saving the form SHALL NOT close or restart them; terminal keystrokes SHALL NOT be sent while typing in the form.
