//! Battle Live: a vertical (9:16) livestream of an openDJ set for TikTok
//! and the like, with phone cameras, without OBS.
//!
//! - [`phones`]: phones join by QR code and send their camera.
//! - [`compose`]: the vertical picture: cameras on top, openDJ's screen
//!   below, the space the platform covers left clear.
//! - [`director`]: which cameras to show, from what is heard.
//! - [`output`]: ffmpeg encodes the picture and the master and sends them
//!   to an RTMP server, a file, or both.

pub mod compose;
pub mod director;
pub mod output;
pub mod phones;
pub mod session;

/// A picture: RGBA, 8 bits a channel, rows top to bottom with no padding.
#[derive(Clone, Debug, PartialEq)]
pub struct Frame {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

impl Frame {
    pub fn new(width: u32, height: u32, rgba: Vec<u8>) -> Option<Self> {
        (rgba.len() == width as usize * height as usize * 4 && width > 0 && height > 0).then_some(
            Self {
                width,
                height,
                rgba,
            },
        )
    }
}
