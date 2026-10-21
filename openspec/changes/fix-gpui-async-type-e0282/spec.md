# Specification

## Requirement: type-safe GPUI refresh task
The SSH output refresh task SHALL compile with GPUI 0.2.2 and SHALL retain its generation check to stop refreshing when a new connection supersedes it.

### Scenario: compile
Given the existing GPUI async refresh task, when cargo check runs on macOS, then the closure's entity handle type is resolved without E0282.

### Scenario: superseded connection
Given an old refresh task, when a new session increments the generation, then the old task stops notifying the UI.
