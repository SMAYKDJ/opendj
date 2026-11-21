# Proposal

Planned for: **v0.7 · Your library, everywhere**

## Why

A DJ using the mouse cannot easily tell how to delete a hot cue from a pad. Shift+click works (and is correct for controllers) but is not discoverable on screen. A user request (2026-11-21): add a small × in the corner of each set hot cue pad so the DJ can remove it with one click, without needing to know the keyboard shortcut.

## What Changes

- When a hot cue pad is set **and** the mouse is hovering over it, a small **×** appears in the top-right corner of the pad.
- Clicking the × deletes the hot cue (same as Shift+click on the pad body).
- The × is small enough not to crowd the time and play-triangle already shown, and uses the pad's tint color at reduced opacity so it blends in until hovered.
- Shift+click on the pad body continues to work as before (controller parity).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `controls` / hot cue pads in `deck.slint`

## Out of Scope

- Right-click context menu for hot cues.
- Renaming or recoloring a hot cue from the pad (that is the Cues tab).
- Deleting memory cues or saved loops from their pads (same treatment can follow).

## Impact

`crates/app/ui/deck.slint` — `HotcueCell` component only.
