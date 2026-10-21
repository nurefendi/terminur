# SSH identity file per host

## Motivation
Users need to choose a private key for individual SSH hosts in the Add host form instead of editing `~/.ssh/config` manually.

## Scope
Add an optional path to an existing, readable private key file. Store only its path in hosts.json. On connection, pass the resolved path to system OpenSSH as a separate `-i` argument, using that selected identity rather than guessing among agent keys. Hosts without a configured identity retain default OpenSSH/agent behavior.

## Exclusions
Importing/storing key contents, creating keys, keychain/passphrase storage, editing existing hosts, and non-macOS verification are out of scope.

## Risks
OpenSSH still needs host authorization and a corresponding server-side public key. An invalid path in a manually edited configuration must fail visibly instead of silently falling back to another key. Private key material must never be written to the configuration or logged.
