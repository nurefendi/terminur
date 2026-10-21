---
name: openspec
description: Plan and implement TermiNur changes using versioned proposals, behavioral specs, designs, tasks, and verification.
---

# OpenSpec — TermiNur

Use this skill when a request introduces a feature, changes behavior, alters architecture, or requires a multi-file refactor.

## Project constraints

- Target macOS first; Rust + GPUI only for the desktop UI.
- Keep SSH/PTy operations separate from GPUI rendering.
- Never claim ANSI terminal emulation is complete unless cursor movement, alternate screen, input, and resizing are tested.
- Keep changes small and compileable. CI passing is not the same as manual runtime verification.

## Workflow

1. **Explore**: read existing Rust code, dependencies, README, and latest GitHub Actions status.
2. **Propose**: create `openspec/changes/<change-id>/proposal.md` documenting motivation, scope, exclusions, risks, and expected behavior.
3. **Specify**: write `openspec/changes/<change-id>/spec.md` with requirements using SHALL/MUST and Given/When/Then acceptance scenarios. Include negative cases and concurrency/lifecycle behavior.
4. **Design**: write `openspec/changes/<change-id>/design.md` with affected files, GPUI threading rules, state ownership, tradeoffs, and testing strategy.
5. **Plan**: write `openspec/changes/<change-id>/tasks.md` with ordered Markdown checkboxes, each independently verifiable.
6. **Implement**: modify code only against an approved or explicitly requested plan. Update tasks as work completes.
7. **Verify**: run or inspect `cargo fmt --all -- --check`, `cargo check --all-targets`, `cargo test --all-targets`, `cargo clippy --all-targets -- -D warnings`, and `cargo build --release` on macOS CI. Record results in `verification.md` with workflow links.
8. **Archive**: after requirements are met and accepted, move change docs to `openspec/archive/<change-id>/` and update canonical specs in `openspec/specs/`.

## Rules

- Never invent test results, GitHub Actions outcomes, or SonarQube findings.
- If CI fails, read the exact failing job logs and fix the first root cause before declaring success.
- Keep secrets and private SSH credentials out of Git.
- Do not silently widen scope. Update the proposal when requirements change.
- When a user requests a quick bug fix, a lightweight proposal is sufficient, but still document verification.

## Suggested commands

```bash
cargo fmt --all -- --check
cargo check --all-targets
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
cargo build --release
```
