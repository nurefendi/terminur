# Proposal: MVP 1.2 Terminal Emulator

## Problem
The current SSH output is append-only text with ANSI escapes stripped. Cursor-addressed applications overwrite incorrectly and terminal state is lost.

## Goal
Introduce a bounded, stateful terminal screen model, connect it to PTY output and GPUI rendering, and establish deterministic regression tests.

## Scope
- Fixed-size screen with cursor movement, carriage return, line feed, backspace, erase and core CSI controls.
- Streaming UTF-8 and split ANSI control handling.
- Preserve output across chunks, bounded scrolling and test coverage.
- Maintain the existing GPUI and portable-pty architecture.

## Non-goals
- Full xterm compatibility, alternate screen, mouse protocols, selection/copy, Unicode grapheme width, complete SGR styling and dynamic PTY geometry are follow-up milestones.
- No changes to SSH host storage or credentials.

## Dependencies
Stacked on PR #1 (validated GPUI async callback), not on the currently failing main.
