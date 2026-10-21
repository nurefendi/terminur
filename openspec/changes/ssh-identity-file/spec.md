# Requirements

- Given a new host, the Add host form SHALL accept an optional SSH private key file path, either absolute or `~/...`.
- Given an empty key field, the saved host SHALL omit the field and SSH SHALL retain its default identity selection.
- Given a path that is missing, not a readable regular file, or is relative, Save SHALL show a validation error and leave both the host list and saved JSON unchanged.
- Given a valid path, Save SHALL store only the path; when the host is clicked, SSH SHALL receive the expanded path as one `-i` argument and `IdentitiesOnly=yes` to prioritize that chosen identity.
- Given a legacy host with no identity field, loading and connection SHALL continue to work as before.
- Given an invalid path supplied by hand-edited JSON after saving, connecting SHALL fail visibly in that tab rather than starting SSH without the requested identity.
- Given other open tabs, using a key for one new session SHALL NOT affect other SSH sessions or tab lifecycles. Key contents/passphrases MUST NOT be stored or displayed by TermiNur.
