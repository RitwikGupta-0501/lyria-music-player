use std::{fs::File, path::Path, time::Duration, io::{Read, Seek, SeekFrom}};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

// ── StreamMediaSource Wrapper ───────────────────────────────────────────────

pub struct StreamMediaSource<T> {
    inner: T,
    content_length: Option<u64>,
}

impl<T> StreamMediaSource<T> {
    pub fn new(inner: T, content_length: Option<u64>) -> Self {
        Self { inner, content_length }
    }
}

impl<T: Read> Read for StreamMediaSource<T> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.inner.read(buf)
    }
}

impl<T: Seek> Seek for StreamMediaSource<T> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.inner.seek(pos)
    }
}

impl<T: Read + Seek + Send + Sync> symphonia::core::io::MediaSource for StreamMediaSource<T> {
    fn is_seekable(&self) -> bool {
        true
    }

    fn byte_len(&self) -> Option<u64> {
        self.content_length
    }
}

use ringbuf::{Consumer, HeapRb};
use rodio::Source;
use symphonia::core::{
    codecs::{DecoderOptions, CODEC_TYPE_NULL},
    errors::Error as SymphoniaError,
    formats::{FormatOptions, SeekMode, SeekTo},
    io::MediaSourceStream,
    meta::MetadataOptions,
    probe::Hint,
    units::Time,
};
use stream_download::{StreamDownload, Settings, http::HttpStream};

pub struct SymphoniaSource {
    consumer: Consumer<i16, Arc<HeapRb<i16>>>,
    channels: u16,
    sample_rate: u32,
    total_duration: Option<Duration>,
    abort_flag: Arc<AtomicBool>,
    eof_flag: Arc<AtomicBool>,
}

impl SymphoniaSource {
    pub fn from_path(path: &Path, audio_config: crate::audio::AudioEngineConfig) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Self::open(Box::new(File::open(path)?), path.extension().and_then(|e| e.to_str()), None, audio_config)
    }

    pub fn from_path_seeked(
        path: &Path,
        pos: Duration,
        audio_config: crate::audio::AudioEngineConfig
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Self::open(Box::new(File::open(path)?), path.extension().and_then(|e| e.to_str()), Some(pos), audio_config)
    }

    pub fn from_url(
        url: url::Url,
        client: reqwest::Client,
        runtime_handle: tauri::async_runtime::RuntimeHandle,
        headers: Option<std::collections::HashMap<String, String>>,
        audio_config: crate::audio::AudioEngineConfig
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Self::from_url_internal(url, client, runtime_handle, None, headers, audio_config)
    }

    pub fn from_url_seeked(
        url: url::Url,
        client: reqwest::Client,
        runtime_handle: tauri::async_runtime::RuntimeHandle,
        seek_to: Duration,
        headers: Option<std::collections::HashMap<String, String>>,
        audio_config: crate::audio::AudioEngineConfig
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Self::from_url_internal(url, client, runtime_handle, Some(seek_to), headers, audio_config)
    }

    pub fn from_hls(
        url: url::Url,
        client: reqwest::Client,
        runtime_handle: tauri::async_runtime::RuntimeHandle,
        audio_config: crate::audio::AudioEngineConfig
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let stream = crate::audio::hls::HlsStream::new(url, client, runtime_handle, None)?;
        Self::open(Box::new(stream), None, None, audio_config)
    }

    pub fn from_hls_seeked(
        url: url::Url,
        client: reqwest::Client,
        runtime_handle: tauri::async_runtime::RuntimeHandle,
        seek_to: Duration,
        audio_config: crate::audio::AudioEngineConfig
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let stream = crate::audio::hls::HlsStream::new(url, client, runtime_handle, Some(seek_to))?;
        Self::open(Box::new(stream), None, None, audio_config)
    }

    fn from_url_internal(
        url: url::Url,
        client: reqwest::Client,
        runtime_handle: tauri::async_runtime::RuntimeHandle,
        seek_to: Option<Duration>,
        headers: Option<std::collections::HashMap<String, String>>,
        audio_config: crate::audio::AudioEngineConfig
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let (download, content_length) = runtime_handle.block_on(async {
            use stream_download::source::SourceStream;
            
            let mut clean_client_builder = reqwest::Client::builder();
            if let Some(h) = headers {
                let mut header_map = reqwest::header::HeaderMap::new();
                for (k, v) in h {
                    if let Ok(header_name) = reqwest::header::HeaderName::from_bytes(k.as_bytes()) {
                        if let Ok(header_value) = reqwest::header::HeaderValue::from_str(&v) {
                            header_map.insert(header_name, header_value);
                        }
                    }
                }
                clean_client_builder = clean_client_builder.default_headers(header_map);
            }
            let clean_client = clean_client_builder.build().unwrap_or(client.clone());

            let stream = HttpStream::new(clean_client, url.clone()).await?;
            let content_length = stream.content_length();
            let settings = Settings::default().prefetch_bytes(256 * 1024); // 256KB buffer for fast startup
            
            // Use TempStorageProvider instead of BoundedStorageProvider to avoid subtraction overflow
            // panics when the MP4 demuxer seeks backward from the end of the file.
            let storage = stream_download::storage::temp::TempStorageProvider::new();
            
            let download = StreamDownload::from_stream(stream, storage, settings).await?;
            Ok::<_, Box<dyn std::error::Error + Send + Sync>>((download, content_length))
        })?;
        let wrapper = StreamMediaSource::new(download, content_length); // Pass the fetched content length
        Self::open(Box::new(wrapper), None, seek_to, audio_config)
    }

    fn open(
        media_source: Box<dyn symphonia::core::io::MediaSource>,
        extension: Option<&str>,
        seek_to: Option<Duration>,
        audio_config: crate::audio::AudioEngineConfig,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mss = MediaSourceStream::new(media_source, Default::default());

        let mut hint = Hint::new();
        if let Some(ext) = extension {
            hint.with_extension(ext);
        }

        let probed = symphonia::default::get_probe().format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )?;

        let mut format = probed.format;

        let track = format
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .ok_or("no supported audio track")?;

        let track_id = track.id;
        let codec_params = track.codec_params.clone();

        let sample_rate = if codec_params.codec == symphonia::core::codecs::CODEC_TYPE_OPUS {
            48000
        } else {
            codec_params.sample_rate.unwrap_or(44100)
        };
        let channels = codec_params.channels.map(|c| c.count() as u16).unwrap_or(2);

        let total_duration = codec_params
            .n_frames
            .zip(codec_params.sample_rate)
            .map(|(frames, rate)| Duration::from_secs_f64(frames as f64 / rate as f64));

        let mut codecs = symphonia::core::codecs::CodecRegistry::new();
        symphonia::default::register_enabled_codecs(&mut codecs);
        crate::audio::opus_decoder::register_opus_decoder(&mut codecs);

        let mut decoder = codecs.make(&codec_params, &DecoderOptions::default())?;

        if let Some(pos) = seek_to {
            let secs = pos.as_secs_f64();
            // SeekMode::Coarse works even for FLAC files without a SEEKTABLE —
            // Symphonia falls back to a bisection search over the byte stream.
            let _ = format.seek(
                SeekMode::Coarse,
                SeekTo::Time {
                    time: Time { seconds: secs as u64, frac: secs.fract() },
                    track_id: None,
                },
            );
            decoder.reset();
        }

        // Create exactly ~4 seconds of buffer based on sample rate and channels
        let buffer_capacity = sample_rate as usize * channels as usize * 4;
        let rb = HeapRb::<i16>::new(buffer_capacity);
        let (mut producer, consumer) = rb.split();

        let abort_flag = Arc::new(AtomicBool::new(false));
        let eof_flag = Arc::new(AtomicBool::new(false));

        let abort_clone = abort_flag.clone();
        let eof_clone = eof_flag.clone();

        let output_rate = audio_config.output_rate;

        // Spawn background decoder thread
        thread::spawn(move || {
            use rubato::{Resampler, SincFixedIn, SincInterpolationParameters, WindowFunction};
            
            let f_ratio = output_rate as f64 / sample_rate as f64;
            
            // Set up rubato resampler if rates don't match
            let mut resampler: Option<SincFixedIn<f32>> = if (f_ratio - 1.0).abs() > 0.0001 {
                let params = SincInterpolationParameters {
                    sinc_len: audio_config.sinc_len,
                    f_cutoff: 0.95,
                    interpolation: audio_config.interpolation,
                    oversampling_factor: 256,
                    window: WindowFunction::BlackmanHarris2,
                };
                match SincFixedIn::<f32>::new(f_ratio, 2.0, params, 1024, channels as usize) {
                    Ok(r) => Some(r),
                    Err(e) => {
                        eprintln!("Failed to initialize resampler: {}", e);
                        None
                    }
                }
            } else {
                None
            };
            
            let chunk_size = 1024;
            let mut input_buffers: Vec<Vec<f32>> = vec![Vec::new(); channels as usize];

            let push_to_rb = |producer: &mut ringbuf::Producer<i16, Arc<HeapRb<i16>>>, samples: &[i16]| {
                let mut pushed = 0;
                while pushed < samples.len() {
                    if abort_clone.load(Ordering::Acquire) {
                        return false;
                    }
                    let pushed_now = producer.push_slice(&samples[pushed..]);
                    pushed += pushed_now;
                    if pushed < samples.len() {
                        thread::sleep(Duration::from_millis(5));
                    }
                }
                true
            };

            loop {
                if abort_clone.load(Ordering::Acquire) {
                    break;
                }

                if producer.is_full() {
                    thread::sleep(Duration::from_millis(10));
                    continue;
                }

                let packet = match format.next_packet() {
                    Ok(p) => p,
                    Err(_) => {
                        // Flush the remaining buffer if resampling
                        if let Some(resamp) = resampler.as_mut() {
                            let in_len = input_buffers[0].len();
                            if in_len > 0 {
                                // pad with zeroes to chunk_size
                                for ch in &mut input_buffers {
                                    ch.resize(chunk_size, 0.0);
                                }
                                if let Ok(resampled) = resamp.process(&input_buffers, None) {
                                    let expected_out = (in_len as f64 * f_ratio) as usize;
                                    let mut interleaved = Vec::with_capacity(expected_out * channels as usize);
                                    for i in 0..expected_out {
                                        for ch in 0..channels as usize {
                                            if i < resampled[ch].len() {
                                                let sample = (resampled[ch][i] * 32767.0).clamp(-32768.0, 32767.0) as i16;
                                                interleaved.push(sample);
                                            }
                                        }
                                    }
                                    let _ = push_to_rb(&mut producer, &interleaved);
                                }
                            }
                        }
                        eof_clone.store(true, Ordering::Release);
                        break;
                    }
                };

                if packet.track_id() != track_id {
                    continue;
                }

                match decoder.decode(&packet) {
                    Ok(decoded) => {
                        let mut sbuf = symphonia::core::audio::SampleBuffer::<f32>::new(
                            decoded.capacity() as u64,
                            *decoded.spec(),
                        );
                        sbuf.copy_interleaved_ref(decoded);
                        
                        let samples = sbuf.samples();
                        
                        if let Some(resamp) = resampler.as_mut() {
                            // Deinterleave and push to input_buffers
                            for (i, &sample) in samples.iter().enumerate() {
                                let ch = i % (channels as usize);
                                input_buffers[ch].push(sample);
                            }
                            
                            // Process in chunks
                            while input_buffers[0].len() >= chunk_size {
                                let mut process_buf = vec![vec![0.0; chunk_size]; channels as usize];
                                for ch in 0..channels as usize {
                                    process_buf[ch].copy_from_slice(&input_buffers[ch][..chunk_size]);
                                    input_buffers[ch].drain(..chunk_size);
                                }
                                
                                if let Ok(resampled) = resamp.process(&process_buf, None) {
                                    let out_len = resampled[0].len();
                                    let mut interleaved = Vec::with_capacity(out_len * channels as usize);
                                    for i in 0..out_len {
                                        for ch in 0..channels as usize {
                                            let sample = (resampled[ch][i] * 32767.0).clamp(-32768.0, 32767.0) as i16;
                                            interleaved.push(sample);
                                        }
                                    }
                                    if !push_to_rb(&mut producer, &interleaved) {
                                        return;
                                    }
                                }
                            }
                        } else {
                            // Direct path (no resampling)
                            let mut interleaved = Vec::with_capacity(samples.len());
                            for &sample in samples {
                                interleaved.push((sample * 32767.0).clamp(-32768.0, 32767.0) as i16);
                            }
                            if !push_to_rb(&mut producer, &interleaved) {
                                return;
                            }
                        }
                    }
                    Err(SymphoniaError::DecodeError(_)) => continue,
                    Err(_) => {
                        eof_clone.store(true, Ordering::Release);
                        break;
                    }
                }
            }
        });

        Ok(Self {
            consumer,
            channels,
            sample_rate: output_rate,
            total_duration,
            abort_flag,
            eof_flag,
        })
    }
}

impl Iterator for SymphoniaSource {
    type Item = i16;

    fn next(&mut self) -> Option<i16> {
        match self.consumer.pop() {
            Some(sample) => Some(sample),
            None => {
                if self.eof_flag.load(Ordering::Acquire) {
                    None // Track actually ended
                } else {
                    Some(0) // Underrun masked with silence; never blocks!
                }
            }
        }
    }
}

impl Source for SymphoniaSource {
    fn current_frame_len(&self) -> Option<usize> { None }
    fn channels(&self) -> u16 { self.channels }
    fn sample_rate(&self) -> u32 { self.sample_rate }
    fn total_duration(&self) -> Option<Duration> { self.total_duration }
}

impl Drop for SymphoniaSource {
    fn drop(&mut self) {
        self.abort_flag.store(true, Ordering::Release);
    }
}

pub struct TrackedSource<S> {
    inner: S,
    on_end: Option<Box<dyn FnOnce() + Send>>,
}

impl<S> TrackedSource<S> {
    pub fn new(inner: S, on_end: impl FnOnce() + Send + 'static) -> Self {
        Self {
            inner,
            on_end: Some(Box::new(on_end)),
        }
    }
}

impl<S: Iterator> Iterator for TrackedSource<S> 
where
    S::Item: rodio::Sample,
{
    type Item = S::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.inner.next();
        if item.is_none() {
            if let Some(cb) = self.on_end.take() {
                cb();
            }
        }
        item
    }
}

impl<S: Source> Source for TrackedSource<S> 
where
    S::Item: rodio::Sample,
{
    fn current_frame_len(&self) -> Option<usize> { self.inner.current_frame_len() }
    fn channels(&self) -> u16 { self.inner.channels() }
    fn sample_rate(&self) -> u32 { self.inner.sample_rate() }
    fn total_duration(&self) -> Option<Duration> { self.inner.total_duration() }
}
