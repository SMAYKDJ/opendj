## Purpose

A vertical livestream of a set, for TikTok and the like, with phone cameras over openDJ's own screen and the shot picked from what is heard.

## ADDED Requirements

### Requirement: Phone cameras
LIVE SHALL show a QR code for each of two camera slots. A phone that opens it on the same network SHALL be able to send its camera, over HTTPS, without an app. Only phones with the session's key SHALL get in; a phone joining a slot SHALL take it over.

#### Scenario: Joining
- **WHEN** a phone scans camera 1's code and allows the camera
- **THEN** the panel shows camera 1 as live and the preview shows its picture

### Requirement: The screen stays as it is
openDJ's window SHALL look the same while live; only the stream SHALL carry the vertical picture.

#### Scenario: Going live
- **WHEN** the DJ goes live
- **THEN** the decks, library and mixer on the laptop are unchanged

### Requirement: Shot from what is heard
In B2B the shot SHALL follow the decks heard in the master (meter, channel fader and crossfader together), not the crossfader alone. The deck heard with the least time left SHALL count as the one playing; the other DJ is next. One DJ heard SHALL show that DJ; the playing track within 60 s of its end, or both DJs heard, SHALL show both with the next DJ named; a shot SHALL last at least 4 bars, and change after 16 bars without a change. A shot picked by hand SHALL hold until Automatic is picked.

#### Scenario: Mixing on the channel faders
- **WHEN** the crossfader is in the middle, DJ A's deck plays with its fader up and DJ B's deck plays with its fader down
- **THEN** only DJ A is shown

### Requirement: Sending
The stream SHALL go to an RTMP server with a stream key, to a recording, or both, as H.264 and AAC, 720 × 1280 at 30 pictures a second, with picture and sound in step. The stream key SHALL be kept in the system keychain.

#### Scenario: No ffmpeg
- **WHEN** ffmpeg is not installed
- **THEN** the panel says how to install it and going live does nothing else
