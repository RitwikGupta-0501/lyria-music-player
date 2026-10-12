use symphonia::core::audio::{AudioBuffer, AudioBufferRef, Signal, SignalSpec};
use symphonia::core::codecs::{CodecParameters, Decoder, DecoderOptions, CODEC_TYPE_OPUS};
use symphonia::core::errors::{Error, Result};
use symphonia::core::formats::Packet;
use opus::Decoder as OpusDecoder;

pub struct OpusSymphoniaDecoder {
    decoder: OpusDecoder,
    buf: AudioBuffer<f32>,
    scratch: Vec<f32>,
    codec_params: CodecParameters,
    channels: usize,
    sample_rate: u32,
}

impl Decoder for OpusSymphoniaDecoder {
    fn try_new(codec_params: &CodecParameters, _options: &DecoderOptions) -> Result<Self> {
        if codec_params.codec != CODEC_TYPE_OPUS {
            return Err(Error::Unsupported("Codec is not Opus"));
        }

        let sample_rate = codec_params.sample_rate.unwrap_or(48000);
        let input_channels = codec_params.channels.map(|c| c.count()).unwrap_or(2);
        // Multichannel Opus (e.g. 5.1 or 7.1) must be clamped to stereo (2 channels).
        // Since libopus decodes as Stereo when channels > 1, setting channels = 2 ensures
        // scratch buffer sizing, SignalSpec, and the de-interleaving loop match libopus output,
        // preventing out-of-bounds reads and audio corruption.
        let channels = if input_channels == 1 { 1 } else { 2 };
        
        let decoder_channels = if channels == 1 { opus::Channels::Mono } else { opus::Channels::Stereo };
        let decoder = OpusDecoder::new(sample_rate, decoder_channels)
            .map_err(|_| Error::DecodeError("opus decode error"))?;

        // Opus frames max out at 120ms (5760 samples per channel at 48kHz)
        let max_frames = (sample_rate * 120) / 1000;
        let spec = SignalSpec::new(
            sample_rate, 
            if channels == 1 {
                symphonia::core::audio::Channels::FRONT_LEFT
            } else {
                symphonia::core::audio::Channels::FRONT_LEFT | symphonia::core::audio::Channels::FRONT_RIGHT
            }
        );
        
        let buf = AudioBuffer::new(max_frames as u64, spec);
        let scratch = vec![0.0; (max_frames as usize) * channels];

        Ok(Self {
            decoder,
            buf,
            scratch,
            codec_params: codec_params.clone(),
            channels,
            sample_rate,
        })
    }

    fn supported_codecs() -> &'static [symphonia::core::codecs::CodecDescriptor] {
        &[
            symphonia::core::codecs::CodecDescriptor {
                codec: CODEC_TYPE_OPUS,
                short_name: "opus",
                long_name: "Opus (via libopus)",
                inst_func: instantiate_opus_decoder,
            }
        ]
    }
    fn decode(&mut self, packet: &Packet) -> Result<AudioBufferRef<'_>> {
        let frames_decoded = self.decoder.decode_float(&packet.data, &mut self.scratch, false)
            .map_err(|_| Error::DecodeError("opus decode error"))?;

        self.buf.clear();
        self.buf.render_reserved(Some(frames_decoded));

        // Opus decode_float interleaves samples: L R L R
        // Symphonia AudioBuffer stores samples in planar format: L L L L, R R R R
        for ch in 0..self.channels {
            let chan_buf = self.buf.chan_mut(ch);
            for (i, sample) in chan_buf.iter_mut().enumerate().take(frames_decoded) {
                *sample = self.scratch[i * self.channels + ch];
            }
        }

        Ok(AudioBufferRef::F32(std::borrow::Cow::Borrowed(&self.buf)))
    }

    fn finalize(&mut self) -> symphonia::core::codecs::FinalizeResult {
        Default::default()
    }

    fn last_decoded(&self) -> AudioBufferRef<'_> {
        AudioBufferRef::F32(std::borrow::Cow::Borrowed(&self.buf))
    }

    fn reset(&mut self) {
        // Re-instantiate the decoder to clear state for seek boundaries
        let decoder_channels = if self.channels == 1 { opus::Channels::Mono } else { opus::Channels::Stereo };
        if let Ok(new_dec) = OpusDecoder::new(self.sample_rate, decoder_channels) {
            self.decoder = new_dec;
        }
        self.buf.clear();
    }

    fn codec_params(&self) -> &CodecParameters {
        &self.codec_params
    }
}

pub fn register_opus_decoder(registry: &mut symphonia::core::codecs::CodecRegistry) {
    registry.register_all::<OpusSymphoniaDecoder>();
}

fn instantiate_opus_decoder(
    params: &CodecParameters,
    options: &DecoderOptions,
) -> Result<Box<dyn Decoder>> {
    let dec = OpusSymphoniaDecoder::try_new(params, options)?;
    Ok(Box::new(dec))
}

// opus::Decoder wraps a raw C pointer and is not marked Sync by default.
// However, Symphonia requires Decoders to be Send + Sync. 
// This is safe because `decode` and `reset` require `&mut self` (exclusive access), 
// and `opus` decoding has no thread-local state, meaning `Send` is safe.
// The only `&self` methods (concurrent access) only read Rust data (`buf`, `codec_params`).
unsafe impl Send for OpusSymphoniaDecoder {}
unsafe impl Sync for OpusSymphoniaDecoder {}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opus_decoder_stereo_initialization() {
        let mut params = CodecParameters::new();
        params.for_codec(CODEC_TYPE_OPUS);
        params.with_sample_rate(48000);
        params.with_channels(symphonia::core::audio::Channels::FRONT_LEFT | symphonia::core::audio::Channels::FRONT_RIGHT);

        let options = DecoderOptions::default();
        let decoder = OpusSymphoniaDecoder::try_new(&params, &options).expect("Decoder init failed");
        assert_eq!(decoder.channels, 2);
    }

    #[test]
    fn test_opus_decoder_multichannel_5_1_clamps_to_stereo() {
        let mut params = CodecParameters::new();
        params.for_codec(CODEC_TYPE_OPUS);
        params.with_sample_rate(48000);
        // 5.1 surround = 6 channels
        params.with_channels(
            symphonia::core::audio::Channels::FRONT_LEFT
                | symphonia::core::audio::Channels::FRONT_RIGHT
                | symphonia::core::audio::Channels::FRONT_CENTRE
                | symphonia::core::audio::Channels::LFE1
                | symphonia::core::audio::Channels::REAR_LEFT
                | symphonia::core::audio::Channels::REAR_RIGHT,
        );

        let options = DecoderOptions::default();
        let decoder = OpusSymphoniaDecoder::try_new(&params, &options).expect("Decoder init failed");
        assert_eq!(decoder.channels, 2, "Multichannel 5.1 must clamp to stereo (2 channels)");
        assert_eq!(decoder.scratch.len(), ((48000 * 120) / 1000) * 2);
    }

    #[test]
    fn test_opus_decoder_mono_initialization() {
        let mut params = CodecParameters::new();
        params.for_codec(CODEC_TYPE_OPUS);
        params.with_sample_rate(48000);
        params.with_channels(symphonia::core::audio::Channels::FRONT_LEFT);

        let options = DecoderOptions::default();
        let decoder = OpusSymphoniaDecoder::try_new(&params, &options).expect("Decoder init failed");
        assert_eq!(decoder.channels, 1, "Mono input must preserve 1 channel");
    }
}
