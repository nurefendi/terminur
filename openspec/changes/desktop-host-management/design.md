# Design

`HostDraft::from_host` clones metadata into the current editor. `TermiNur` owns the selected optional host index and pending deletion flag. Edit skips the address step. Card connect and edit are sibling click targets to avoid accidental SSH starts.

`hosts::replace_or_remove` returns a candidate list without mutating its source. The UI persists the candidate using the existing atomic JSON save path and only commits memory after success. Confirmation is a distinct editor view; it never calls SSH or changes session ownership. Indexing is sufficient while mutations remain serialized in one UI; multi-process concurrent writers need a future revision/conflict strategy.

Tests use real temporary JSON files and draft values, without mocks. Native macOS CI performs fmt/check/test/clippy/release. Manual GUI checks must cover cancel, save errors, deleting the last host, and preserving multiple running sessions. Windows/Linux/mobile acceptance remains outstanding.
