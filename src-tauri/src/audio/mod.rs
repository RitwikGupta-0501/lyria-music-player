pub mod opus_decoder;
use rodio::{OutputStream, Sink, Source};
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

pub mod commands;
pub mod media_controls;
pub mod symphonia_source;
pub mod hls;

use symphonia_source::{SymphoniaSource, TrackedSource};
use media_controls::OSMediaControls;
use crate::db::DbRequest;
use rubato::SincInterpolationType;

pub struct AudioEngineConfig {
    pub output_rate: u32,
    pub interpolation: SincInterpolationType,
    pub sinc_len: usize,
}

impl Clone for AudioEngineConfig {
    fn clone(&self) -> Self {
        Self {
            output_rate: self.output_rate,
            interpolation: match self.interpolation {
                SincInterpolationType::Linear => SincInterpolationType::Linear,
                SincInterpolationType::Quadratic => SincInterpolationType::Quadratic,
                SincInterpolationType::Cubic => SincInterpolationType::Cubic,
                SincInterpolationType::Nearest => SincInterpolationType::Nearest,
            },
            sinc_len: self.sinc_len,
        }
    }
}

#[derive(serde::Serialize, Clone)]
pub struct PlayerSync {
    pub state: String,
    pub position: f64,
    pub duration: f64,
    pub track: String,
}

#[derive(Clone)]
pub enum TrackSource {
    Local(std::path::PathBuf),
    Remote(url::Url, Option<std::collections::HashMap<String, String>>),
}

pub enum AudioCommand {
    Load {
        source: TrackSource,
        title: String,
        artist: Option<String>,
        album: Option<String>,
        duration_hint: Option<u64>,
    },
    QueueNext {
        source: TrackSource,
        title: String,
        artist: Option<String>,
        album: Option<String>,
        duration_hint: Option<u64>,
    },
    Play,
    Pause,
    Stop,
    Seek(f64),
    SetVolume(f32),
    SetMute(bool),
    SyncState,
    Quit,
}

pub fn start_audio_thread(
    rx: Receiver<AudioCommand>,
    app_handle: AppHandle,
    reqwest_client: reqwest::Client,
    runtime_handle: tauri::async_runtime::RuntimeHandle,
    db_tx: std::sync::mpsc::Sender<DbRequest>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let (_stream, stream_handle) =
            OutputStream::try_default().expect("Failed to get audio output");
        let mut sink = Sink::try_new(&stream_handle).expect("Failed to create audio sink");
        
        let output_rate = runtime_handle.block_on(async {
            let (tx, rx) = tokio::sync::oneshot::channel();
            if db_tx.send(DbRequest::GetSetting { key: "output_sample_rate".to_string(), resp: tx }).is_ok() {
                if let Ok(Ok(Some(val))) = rx.await {
                    return val.parse().unwrap_or(48000u32);
                }
            }
            48000u32
        });
        
        let interpolation = runtime_handle.block_on(async {
            let (tx, rx) = tokio::sync::oneshot::channel();
            if db_tx.send(DbRequest::GetSetting { key: "resampling_quality".to_string(), resp: tx }).is_ok() {
                if let Ok(Ok(Some(val))) = rx.await {
                    return match val.as_str() {
                        "linear" => SincInterpolationType::Linear,
                        _ => SincInterpolationType::Cubic,
                    };
                }
            }
            SincInterpolationType::Cubic
        });
        
        let sinc_len = runtime_handle.block_on(async {
            let (tx, rx) = tokio::sync::oneshot::channel();
            if db_tx.send(DbRequest::GetSetting { key: "sinc_len".to_string(), resp: tx }).is_ok() {
                if let Ok(Ok(Some(val))) = rx.await {
                    return val.parse().unwrap_or(128usize);
                }
            }
            128usize
        });
        
        let audio_config = AudioEngineConfig { output_rate, interpolation, sinc_len };

        let mut current_track_source: Option<TrackSource> = None;
        let mut current_track_path = String::new();
        let mut current_duration: f64 = 0.0;
        let mut current_volume: f32 = 1.0;
        let mut is_muted: bool = false;
        let mut seek_offset: f64 = 0.0;
        let mut is_seeking = false;
        let mut seek_generation: u64 = 0;
        let (seek_tx, seek_rx) = std::sync::mpsc::channel::<(u64, f64, Result<SymphoniaSource, String>, bool)>();

        let media_controls = OSMediaControls::new(&app_handle);

        let emit_sync =
            |handle: &AppHandle, state: &str, sink: &Sink, track: &str, duration: f64, offset: f64| {
                let _ = handle.emit(
                    "player-sync",
                    PlayerSync {
                        state: state.to_string(),
                        position: sink.get_pos().as_secs_f64() + offset,
                        duration,
                        track: track.to_string(),
                    },
                );
            };

        loop {
            // Drain completed async seek operations
            while let Ok((gen, _seek_pos, res, was_paused)) = seek_rx.try_recv() {
                if gen == seek_generation {
                    is_seeking = false;
                    match res {
                        Ok(src) => {
                            sink.stop();
                            sink = Sink::try_new(&stream_handle).unwrap_or_else(|_| Sink::try_new(&stream_handle).unwrap());
                            if is_muted {
                                sink.set_volume(0.0);
                            } else {
                                sink.set_volume(current_volume);
                            }

                            let app_clone = app_handle.clone();
                            let tracked = TrackedSource::new(src, move || {
                                let _ = app_clone.emit("track-advanced", ());
                            });
                            sink.append(tracked);
                            if !was_paused {
                                sink.play();
                                media_controls.set_playback_status(true);
                            }

                            let state_str = if sink.is_paused() {
                                "Paused"
                            } else {
                                "Playing"
                            };
                            emit_sync(
                                &app_handle,
                                state_str,
                                &sink,
                                &current_track_path,
                                current_duration,
                                seek_offset,
                            );
                        }
                        Err(e) => eprintln!("Audio seek failed: {e}"),
                    }
                }
            }

            // Track End Detection
            if sink.empty() && !is_seeking && !current_track_path.is_empty() {
                let _ = app_handle.emit("track-ended", ());
                current_track_path.clear();
                current_track_source = None;
                current_duration = 0.0;
                seek_offset = 0.0;
                media_controls.set_playback_status(false);
            }

            match rx.try_recv() {
                Ok(cmd) => match cmd {
                    AudioCommand::Load { source, title, artist, album, duration_hint } => {
                        is_seeking = false;
                        sink.stop();
                        sink = Sink::try_new(&stream_handle).unwrap_or_else(|_| Sink::try_new(&stream_handle).unwrap());
                        if is_muted {
                            sink.set_volume(0.0);
                        } else {
                            sink.set_volume(current_volume);
                        }
                        seek_offset = 0.0;
                        
                        current_track_source = Some(source.clone());
                        let src_result = match &source {
                            TrackSource::Local(path) => {
                                current_track_path = path.to_string_lossy().to_string();
                                SymphoniaSource::from_path(path, audio_config.clone())
                            },
                            TrackSource::Remote(url, headers) => {
                                current_track_path = url.to_string();
                                if url.path().ends_with(".m3u8") {
                                    SymphoniaSource::from_hls(url.clone(), reqwest_client.clone(), runtime_handle.clone(), audio_config.clone())
                                } else {
                                    SymphoniaSource::from_url(url.clone(), reqwest_client.clone(), runtime_handle.clone(), headers.clone(), audio_config.clone())
                                }
                            }
                        };

                        match src_result {
                            Ok(src) => {
                                let hint_sec = duration_hint.map(|ms| ms as f64 / 1000.0).filter(|&d| d > 0.0);
                                let probed_sec = src.total_duration().map(|d| d.as_secs_f64()).filter(|&d| d > 0.0);
                                
                                current_duration = match &source {
                                    TrackSource::Remote(..) => {
                                        // For remote streams, metadata duration_hint from provider/WASM is authoritative if present
                                        hint_sec.or(probed_sec).unwrap_or(0.0)
                                    }
                                    TrackSource::Local(..) => {
                                        // For local files, container-probed duration (headers on disk) is authoritative
                                        probed_sec.or(hint_sec).unwrap_or(0.0)
                                    }
                                };
                                
                                let app_clone = app_handle.clone();
                                let tracked = TrackedSource::new(src, move || {
                                    let _ = app_clone.emit("track-advanced", ());
                                });
                                sink.append(tracked);
                                sink.play();
                                
                                media_controls.update_metadata(
                                    &title,
                                    artist.as_deref().unwrap_or("Unknown Artist"),
                                    album.as_deref().unwrap_or("Unknown Album"),
                                    Some(Duration::from_secs_f64(current_duration))
                                );
                                media_controls.set_playback_status(true);

                                let _ = app_handle.emit(
                                    "player-sync",
                                    PlayerSync {
                                        state: "Playing".to_string(),
                                        position: 0.0,
                                        duration: current_duration,
                                        track: current_track_path.clone(),
                                    },
                                );
                            }
                            Err(e) => eprintln!("Audio load failed: {e}"),
                        }
                    }
                    AudioCommand::QueueNext { source, title: _, artist: _, album: _, duration_hint: _ } => {
                        let src_result = match &source {
                            TrackSource::Local(path) => {
                                SymphoniaSource::from_path(path, audio_config.clone())
                            },
                            TrackSource::Remote(url, headers) => {
                                if url.path().ends_with(".m3u8") {
                                    SymphoniaSource::from_hls(url.clone(), reqwest_client.clone(), runtime_handle.clone(), audio_config.clone())
                                } else {
                                    SymphoniaSource::from_url(url.clone(), reqwest_client.clone(), runtime_handle.clone(), headers.clone(), audio_config.clone())
                                }
                            }
                        };

                        match src_result {
                            Ok(src) => {
                                let app_clone = app_handle.clone();
                                let tracked = TrackedSource::new(src, move || {
                                    let _ = app_clone.emit("track-advanced", ());
                                });
                                sink.append(tracked);
                            }
                            Err(e) => eprintln!("Audio queue_next failed: {e}"),
                        }
                    }
                    AudioCommand::Play => {
                        sink.play();
                        media_controls.set_playback_status(true);
                        emit_sync(
                            &app_handle,
                            "Playing",
                            &sink,
                            &current_track_path,
                            current_duration,
                            seek_offset,
                        );
                    }
                    AudioCommand::Pause => {
                        sink.pause();
                        media_controls.set_playback_status(false);
                        emit_sync(
                            &app_handle,
                            "Paused",
                            &sink,
                            &current_track_path,
                            current_duration,
                            seek_offset,
                        );
                    }
                    AudioCommand::Stop => {
                        is_seeking = false;
                        sink.stop();
                        current_track_path.clear();
                        current_track_source = None;
                        current_duration = 0.0;
                        seek_offset = 0.0;
                        media_controls.set_playback_status(false);
                        emit_sync(&app_handle, "Stopped", &sink, "", 0.0, 0.0);
                    }
                    AudioCommand::SyncState => {
                        let state_str = if current_track_path.is_empty() {
                            "Stopped"
                        } else if sink.is_paused() {
                            "Paused"
                        } else {
                            "Playing"
                        };
                        emit_sync(
                            &app_handle,
                            state_str,
                            &sink,
                            &current_track_path,
                            current_duration,
                            seek_offset,
                        );
                    }
                    AudioCommand::Seek(pos) => {
                        if current_track_path.is_empty() {
                            continue;
                        }
                        seek_generation += 1;
                        is_seeking = true;
                        let gen = seek_generation;
                        let was_paused = sink.is_paused();
                        sink.stop();
                        seek_offset = pos;

                        emit_sync(
                            &app_handle,
                            if was_paused { "Paused" } else { "Playing" },
                            &sink,
                            &current_track_path,
                            current_duration,
                            seek_offset,
                        );

                        let seek_tx_clone = seek_tx.clone();
                        let source_clone = current_track_source.clone();
                        let client_clone = reqwest_client.clone();
                        let runtime_clone = runtime_handle.clone();
                        let config_clone = audio_config.clone();
                        let seek_pos = Duration::from_secs_f64(pos);

                        thread::spawn(move || {
                            let res = match &source_clone {
                                Some(TrackSource::Local(path)) => {
                                    SymphoniaSource::from_path_seeked(path, seek_pos, config_clone).map_err(|e| e.to_string())
                                }
                                Some(TrackSource::Remote(url, headers)) => {
                                    if url.path().ends_with(".m3u8") {
                                        SymphoniaSource::from_hls_seeked(url.clone(), client_clone, runtime_clone, seek_pos, config_clone).map_err(|e| e.to_string())
                                    } else {
                                        SymphoniaSource::from_url_seeked(url.clone(), client_clone, runtime_clone, seek_pos, headers.clone(), config_clone).map_err(|e| e.to_string())
                                    }
                                }
                                None => Err("No track source".to_string()),
                            };
                            let _ = seek_tx_clone.send((gen, pos, res, was_paused));
                        });
                    }
                    AudioCommand::SetVolume(vol) => {
                        current_volume = vol;
                        if !is_muted {
                            sink.set_volume(current_volume);
                        }
                    }
                    AudioCommand::SetMute(muted) => {
                        is_muted = muted;
                        if is_muted {
                            sink.set_volume(0.0);
                        } else {
                            sink.set_volume(current_volume);
                        }
                    }
                    AudioCommand::Quit => {
                        break;
                    }
                },
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    if !sink.empty() {
                        let pos = sink.get_pos().as_secs_f64() + seek_offset;
                        if pos > current_duration && current_duration > 0.0 {
                            current_duration = pos;
                        }
                    }
                    thread::sleep(Duration::from_millis(50));
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    break;
                }
            }
        }
    })
}
