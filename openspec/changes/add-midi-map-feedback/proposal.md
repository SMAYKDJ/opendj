# Proposal

Planned for: **v0.7 · Your library, everywhere**

## Why

When a DJ uses Map mode to assign a hardware control, they have to watch the screen to know which control is waiting for a press. Controllers with RGB or bi-colour pads and buttons — such as the Akai APC Key 25 — can light up to confirm that openDJ is listening. Without feedback, the DJ glances away from their deck to find the right pad; with it, the hardware tells them. A user request (2026-11-21): illuminate the pad or button being armed in Map mode and keep the LED on while the control is mapped, so the DJ has tactile confirmation without looking at the screen.

## What Changes

- When Map mode arms a control (the DJ clicked a MapMark and openDJ is waiting for a hardware press), openDJ sends a **Note On** message to the connected controller on the channel and note it learned during mapping, with a configurable velocity (default 127, full brightness).
- When the hardware press is received and the mapping is confirmed, the LED **stays on** (the same Note On velocity) to show that the control is mapped.
- When Map mode ends — either the DJ clicks Done or presses Escape — openDJ sends **Note Off** (velocity 0) for every control that received feedback during the session, clearing all lit pads in one pass.
- If the controller's output port is unavailable (the port was opened for input but no matching output exists), openDJ skips feedback silently; Map mode works as before.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `controllers`: MIDI output feedback during Map mode

## Out of Scope

- LED editing (colours, brightness per control, blinking) — this is covered by task 2.1b in `add-controller-mapper`.
- Feedback during Quick map / Mapping wizard.
- Feedback on HID controllers.
- Sending Note On for currently-mapped controls when the controller connects (that is the existing 1.5-second LED refresh in the spec).

## Impact

crates/controllers (new `send_feedback` path in the output channel), crates/app (wire-up in map_mode.rs when armed state changes and when Map mode ends).
