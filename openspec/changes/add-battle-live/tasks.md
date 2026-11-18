# Tasks

## 1. Phone cameras

- [x] 1.1 HTTPS camera server with a kept self-signed certificate, a random key per session and two slots; QR code per slot
- [x] 1.2 Camera page: getUserMedia, front/back switch, JPEG pictures over a WebSocket with back-pressure, wake lock, reconnect
- [ ] 1.3 Tried on an iPhone and an Android phone on the same Wi-Fi

## 2. openDJ's screen

- [x] 2.1 The window's picture with `take_snapshot`, shrunk to the size it is drawn at
- [ ] 2.2 Measured on a MacBook Air: snapshot cost at 10 pictures a second

## 3. The vertical picture

- [x] 3.1 Layout as the design: camera row, VS, next notice, vote bar, screen, brand, TikTok's bands left clear
- [x] 3.2 Solo with the hands camera as a picture in picture

## 4. Scene director

- [x] 4.1 Heard decks from the meters, faders and crossfader; least time left goes out
- [x] 4.2 Rules: alone, both, next, rotation in bars, minimum hold, by hand
- [x] 4.3 Tests

## 5. Encoding and sending

- [x] 5.1 Second master tap in the engine
- [x] 5.2 ffmpeg process: raw picture on stdin, master over a local socket, each on its own thread; VideoToolbox on macOS, x264 elsewhere
- [x] 5.3 RTMP, fragmented MP4 recording, or both (tee)
- [x] 5.4 Picture and sound in step (tested: 3.02 s of picture, 2.99 s of sound)
- [ ] 5.5 Tried live on TikTok with a stream key

## 6. The app

- [x] 6.1 LIVE in the top bar and the Battle Live panel: mode, names, QR codes, camera state, rotation, shot by hand, stream key in the keychain, recording, preview
- [ ] 6.2 User guide page
