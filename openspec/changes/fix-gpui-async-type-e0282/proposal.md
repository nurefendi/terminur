# Proposal: Fix GPUI async type inference

## Problem
macOS CI fails cargo check with E0282 at src/main.rs:76 because the async task closure parameter `this` has no inferable type.

## Scope
Explicitly type the GPUI weak entity handle in the refresh task without changing runtime behavior. No unrelated changes.

## Acceptance
Formatting, type check, unit tests, Clippy and release build must pass on macOS CI. No merge before successful verification.
