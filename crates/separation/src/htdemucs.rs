//! Meta's HTDemucs v4 (MIT weights), through the vendored `demucs-core`
//! (third_party/demucs-core). Weights are the safetensors conversion
//! `htdemucs.safetensors` (84 MB).

use burn::prelude::Backend;
use demucs_core::{Demucs, ModelOptions};

pub use demucs_core::model::metadata::{download_url, HTDEMUCS};

/// The stems in the order [`HtDemucs::separate`] returns them.
pub const STEMS: [&str; 4] = ["drums", "bass", "other", "vocals"];

pub struct HtDemucs<B: Backend> {
    model: Demucs<B>,
}

impl<B: Backend> HtDemucs<B> {
    pub fn load(bytes: &[u8], device: &B::Device) -> Result<Self, String> {
        Demucs::from_bytes(ModelOptions::FourStem, bytes, device.clone())
            .map(|model| HtDemucs { model })
            .map_err(|e| e.to_string())
    }

    /// Compiles the GPU kernels for the model's shapes, which otherwise
    /// happens during the first separation.
    pub fn warm_up(&self) {
        pollster::block_on(self.model.warmup());
    }

    /// Separates a stereo track at 44.1 kHz into [`STEMS`]:
    /// `[stem][channel][sample]`.
    pub fn separate(&self, channels: [&[f32]; 2]) -> Result<Vec<[Vec<f32>; 2]>, String> {
        let stems = pollster::block_on(self.model.separate(
            channels[0],
            channels[1],
            crate::SAMPLE_RATE,
        ))
        .map_err(|e| e.to_string())?;
        Ok(stems.into_iter().map(|s| [s.left, s.right]).collect())
    }

    /// [`HtDemucs::separate`], calling `progress` with 0..1 as each chunk
    /// of the track is done.
    pub fn separate_with_progress(
        &self,
        channels: [&[f32]; 2],
        progress: impl FnMut(f32),
    ) -> Result<Vec<[Vec<f32>; 2]>, String> {
        use demucs_core::listener::{ForwardEvent, ForwardListener};
        struct Chunks<F: FnMut(f32)>(F);
        impl<F: FnMut(f32)> ForwardListener for Chunks<F> {
            fn on_event(&mut self, event: ForwardEvent) {
                if let ForwardEvent::ChunkDone { index, total } = event {
                    (self.0)((index + 1) as f32 / total.max(1) as f32);
                }
            }
        }
        let stems = pollster::block_on(self.model.separate_with_listener(
            channels[0],
            channels[1],
            crate::SAMPLE_RATE,
            &mut Chunks(progress),
        ))
        .map_err(|e| e.to_string())?;
        Ok(stems.into_iter().map(|s| [s.left, s.right]).collect())
    }
}
