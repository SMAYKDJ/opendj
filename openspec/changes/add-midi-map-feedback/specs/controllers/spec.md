# controllers — delta spec for add-midi-map-feedback

Adds to `openspec/specs/controllers/spec.md`. Merge after the change ships.

---

### Requirement: Map mode LED feedback

When Map mode arms a control (the DJ clicked a MapMark on screen and openDJ is waiting for a hardware press), openDJ SHALL send a Note On message on the controller's output port for the channel and note that the control was last seen on, at velocity 127.

When the hardware press is received and the mapping is confirmed, the LED SHALL remain on (the Note On is not retracted).

When Map mode ends — the DJ clicks Done or presses Escape — openDJ SHALL send Note Off (velocity 0) for every control that received a Note On during the session, in a single pass, clearing all feedback LEDs.

If the controller's output port cannot be opened or is unavailable, openDJ SHALL skip feedback silently; Map mode SHALL continue to work as before.

#### Scenario: Arm a control

- **GIVEN** Map mode is active and a controller with an output port is connected
- **WHEN** the DJ clicks a MapMark (arming that control)
- **THEN** the controller receives Note On on the channel and note last seen for that control, at velocity 127, and the pad or button lights up

#### Scenario: Confirm a mapping

- **GIVEN** Map mode is active and a control is armed
- **WHEN** the DJ presses a hardware control (mapping confirmed)
- **THEN** the LED stays on; no Note Off is sent

#### Scenario: End Map mode

- **GIVEN** Map mode is active and one or more controls received feedback LEDs
- **WHEN** Map mode ends (Done or Escape)
- **THEN** openDJ sends Note Off for each control that was lit, the pads go dark, and no LEDs remain from the Map mode session

#### Scenario: No output port

- **GIVEN** Map mode is active and the controller has no output port
- **WHEN** the DJ arms a control
- **THEN** Map mode works normally; no error is shown; the hardware is simply not lit
