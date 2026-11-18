# Proposal

Planned for: **v0.8 · From the booth to the studio**, beside step 3 of add-live-broadcast (both send RTMP). Added 2026-11-18 from a user request.

## Why

DJs go live on TikTok more and more, often two at a time (back to back, "B2B battles"). TikTok LIVE is vertical (9:16), and the usual way to get a DJ set on it is OBS with a phone mount, several scenes and a plugin to switch them, set up by hand for every venue. The audience wants to see the DJs' faces and hands, and who is playing, more than the software.

openDJ already pairs phones by QR code (the phone remote), knows which deck is heard in the master, how long each track has left and the tempo, and copies the master for broadcasting. It can do the rest itself: take the phones as cameras, put them over its own screen in a vertical picture, pick the shot from what is heard, and send it.

## What Changes

1. **Phone cameras.** LIVE in the top bar opens Battle Live: a QR code per phone (two). The phone's browser opens a page served over HTTPS on the local network (browsers only lend a page the camera over HTTPS, so openDJ makes a certificate for its own address and keeps it) and sends its camera as JPEG pictures over a WebSocket. A phone that scans a slot's code takes the slot over.
2. **openDJ's screen.** The window's own picture (Slint's `take_snapshot`), a few times a second. The screen on the laptop never changes; only the stream gets the vertical picture.
3. **The vertical picture.** 720 × 1280: the band TikTok covers at the top left clear, the camera row (both DJs with VS, or one), "PRÓXIMO: DJ B · faixa atual acaba em 0:42" near the end of a track, an optional #A/#B vote bar, openDJ's screen, and the bottom third left to TikTok's comments.
4. **Scene director.** Modes Solo (one DJ: face camera, optional hands camera) and B2B (two DJs, a camera each). It goes by what is heard, not the crossfader: a deck is heard when its channel meter, with the channel fader and the crossfader applied, is above −30 dBFS. The deck heard with the least time left is on its way out, so its DJ is playing and the other is next. Rules: one DJ heard → that DJ; near the end (60 s) or both heard → both; the old track stops → the next DJ alone; nothing changes for 16 bars → the shot moves on; at least 4 bars a shot; a shot picked by hand pauses it all.
5. **Encoding and sending.** ffmpeg (found where Homebrew puts it, not bundled) gets the picture as raw RGBA and a second copy of the master (a new engine tap, so it can run beside the radio broadcast) over a local socket, and sends H.264 (VideoToolbox on a Mac) and AAC to the RTMP server and stream key from TikTok LIVE Studio, records a fragmented MP4, or both. Pictures are repeated when drawing falls behind so picture and sound stay in step.

## Capabilities

### New Capabilities

- `battle-live`: a vertical livestream with phone cameras, an automatic scene director and RTMP output

## Out of Scope

- Reading the TikTok chat for the #A/#B votes: TikTok has no public chat API for third-party apps. The vote bar is drawn when votes are given; counting them waits for a supported way.
- Logging in to TikTok and starting the live from openDJ: there is no public API for it; the DJ starts it in TikTok LIVE Studio or with a stream key.
- Sound from the phones (the stream's sound is the master).
- More than two cameras, transitions other than a cut, and drop detection for Solo.
- Bundling ffmpeg.

## Impact

- New crate `crates/live` (`opendj-live`): `phones`, `compose`, `director`, `output`, `session`.
- `crates/engine`: a second master tap, `AudioEngine::take_live_feed`, which also turns on the true-peak air ceiling while live.
- `crates/app`: `src/live.rs`, `ui/live.slint`, LIVE in the top bar.
- New dependency: `rcgen` (the phones' certificate).
