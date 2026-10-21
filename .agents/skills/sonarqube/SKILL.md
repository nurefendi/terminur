---
name: sonarqube
description: Review TermiNur Rust code quality, security, maintainability, coverage, and SonarQube or SonarCloud quality gates.
---

# SonarQube — TermiNur

Use this skill for code-quality reviews, SonarQube setup, CI quality gates, issue triage, and security or maintainability checks.

## First determine availability

- Check whether a SonarQube/SonarCloud project, scanner, token secret, and quality gate actually exist.
- If not configured, report that SonarQube analysis is **not running**. Do not fabricate a dashboard, issue list, coverage percentage, or quality-gate status.
- Prefer the official SonarQube documentation for scanner and Rust-language support applicable to the installed server edition/version before changing CI.
- Never commit tokens or print secrets.

## Local Rust quality baseline

```bash
cargo fmt --all -- --check
cargo check --all-targets
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
```

Optionally use `cargo audit` when installed; do not treat it as equivalent to SonarQube. Gather test coverage using a supported coverage tool when available, and report the tool and coverage scope.

## Review priorities

1. **Reliability**: deadlocks, panics, poisoned mutexes, thread lifecycle, PTY termination, stale SSH session callbacks, and GPUI UI-thread ownership.
2. **Security**: SSH host-key verification, credential leakage, command argument safety, secrets in logs, and dependency advisories.
3. **Maintainability**: oversized functions, duplicated logic, unclear ownership, excessive suppression attributes, and error handling.
4. **Testing**: streaming UTF-8 boundaries, ANSI escape fragmentation, terminal state transitions, resize, and connection failure cases.
5. **Performance**: unbounded buffers, excessive UI notifications, locking contention, and background task leaks.

## SonarQube setup procedure

1. Ask for or discover the SonarQube/SonarCloud URL, project key, edition/version, and desired quality-gate policy.
2. Configure the scanner only after confirming compatibility with the Rust project and chosen service.
3. Add `SONAR_TOKEN` as a GitHub Actions secret; never place its value in repository files.
4. Add CI integration with minimal permissions and without leaking token values to untrusted pull requests.
5. Configure exclusions narrowly; never exclude `src/**` just to make the gate pass.
6. Run the scanner and inspect the server's actual analysis status and quality gate.
7. Report confirmed issues with file, line, severity, rule, impact, and suggested fix. Distinguish findings from manual hypotheses.

## Output format

Summarize: **Configured?**, **CI checks**, **Quality gate**, **Confirmed findings**, **Next actions**. Use `not configured` or `not verified` when evidence is missing.
