# SSH tabs in macOS titlebar

## Motivation
The user wants SSH tabs in the bar with the macOS traffic lights, not a second row below the native titlebar.

## Scope
On macOS use GPUI's transparent titlebar/full-size content view, hide the redundant native title text, render the existing Workspace/SSH tabs and New host action in one top row, and reserve a non-interactive area for native traffic lights. Keep the native close/minimize/zoom controls. Preserve host/SSH state and layout sizing.

## Exclusions
No custom traffic light clones, native macOS tabbing, new platform support, or titlebar drag gestures outside normal macOS behavior. Other platforms retain a standard titlebar until specifically supported.

## Risk
Traffic light placement and window drag/hit-testing require manual macOS GUI checks; automated Rust tests cannot validate the compositor's titlebar layout. The transparent option is enabled only on macOS.
