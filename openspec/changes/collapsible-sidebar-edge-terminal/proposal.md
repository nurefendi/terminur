# Proposal: Collapsible navigation and space-efficient terminal

## Problem
The sidebar occupies permanent width and the terminal is inset by multiple nested paddings, wasting usable terminal columns and rows.

## Goal
Allow users to hide/show navigation, reduce terminal padding, and keep PTY geometry aligned with the available viewport.

## Scope
- Toggle sidebar from header with an accessible textual control.
- Remove redundant outer terminal margins and minimize internal terminal padding.
- Recalculate terminal grid and resize active PTYs when navigation visibility changes.
- Preserve tabs, connections and scrollback.

## Non-goals
Persist sidebar preference across app restarts, draggable sidebar widths, full-screen mode.
