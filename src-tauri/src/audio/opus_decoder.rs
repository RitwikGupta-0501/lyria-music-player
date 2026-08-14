use symphonia::core::audio::{AudioBuffer, AudioBufferRef, Signal, SignalSpec};
use symphonia::core::codecs::{CodecParameters, Decoder, DecoderOptions, CODEC_TYPE_OPUS};
use symphonia::core::errors::{Error, Result};
use symphonia::core::formats::Packet;
use opus_decoder::OpusDecoder;

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
        let channels = codec_params.channels.map(|c| c.count()).unwrap_or(2);
        
        let decoder = OpusDecoder::new(sample_rate, channels as usize)
            .map_err(|_| Error::DecodeError("opus decode error"))?;

        // Opus frames max out at 120ms (5760 samples per channel at 48kHz)
        let max_frames = (sample_rate * 120) / 1000;
        let spec = SignalSpec::new(
            sample_rate, 
            codec_params.channels.unwrap_or(symphonia::core::audio::Channels::FRONT_LEFT | symphonia::core::audio::Channels::FRONT_RIGHT)
        );
        
        let buf = AudioBuffer::new(max_frames as u64, spec);
        let scratch = vec![0.0; (max_frames as usize) * (channels as usize)];

        Ok(Self {
            decoder,
            buf,
            scratch,
            codec_params: codec_params.clone(),
            channels: channels as usize,
            sample_rate,
        })
    }

    fn supported_codecs() -> &'static [symphonia::core::codecs::CodecDescriptor] {
        &[
            symphonia::core::codecs::CodecDescriptor {
                codec: CODEC_TYPE_OPUS,
                short_name: "opus",
                long_name: "Opus (via opus-decoder)",
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
            for i in 0..frames_decoded {
                chan_buf[i] = self.scratch[i * self.channels + ch];
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
        if let Ok(new_dec) = OpusDecoder::new(self.sample_rate, self.channels) {
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
