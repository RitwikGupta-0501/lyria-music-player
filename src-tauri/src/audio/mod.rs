pub mod opus_decoder;
use rodio::{OutputStream, OutputStreamHandle, Sink, Source};
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


fn recreate_sink(
    audio_output: &mut Option<(OutputStream, OutputStreamHandle)>,
    app_handle: &AppHandle,
) -> Option<Sink> {
    if let Some((_, handle)) = audio_output.as_ref() {
        if let Ok(new_sink) = Sink::try_new(handle) {
            return Some(new_sink);
        }
    }

    match OutputStream::try_default() {
        Ok((new_stream, new_handle)) => {
            match Sink::try_new(&new_handle) {
                Ok(new_sink) => {
                    *audio_output = Some((new_stream, new_handle));
                    Some(new_sink)
                }
                Err(e) => {
                    tracing::error!("Failed to create audio sink on new output device: {}", e);
                    *audio_output = None;
                    let _ = app_handle.emit("device-disconnected", ());
                    None
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to acquire default audio output device: {}", e);
            *audio_output = None;
            let _ = app_handle.emit("device-disconnected", ());
            None
        }
    }
}

pub fn start_audio_thread(
    rx: Receiver<AudioCommand>,
    app_handle: AppHandle,
    reqwest_client: reqwest::Client,
    runtime_handle: tauri::async_runtime::RuntimeHandle,
    db_tx: std::sync::mpsc::Sender<DbRequest>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut audio_output: Option<(OutputStream, OutputStreamHandle)> = OutputStream::try_default().ok();
        let mut sink: Option<Sink> = audio_output.as_ref().and_then(|(_, handle)| Sink::try_new(handle).ok());
        if sink.is_none() {
            tracing::warn!("No default audio output device available on startup. Entering recovery mode.");
            let _ = app_handle.emit("device-disconnected", ());
        }
        
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
        let mut sink_pos_baseline: f64 = 0.0;
        let mut is_seeking = false;
        let mut seek_generation: u64 = 0;
        let (seek_tx, seek_rx) = std::sync::mpsc::channel::<(u64, f64, Result<SymphoniaSource, String>, bool)>();
        let (advance_tx, advance_rx) = std::sync::mpsc::channel::<()>();

        struct QueuedTrackMeta {
            source: TrackSource,
            title: String,
            artist: Option<String>,
            album: Option<String>,
            duration_sec: f64,
            track_path: String,
        }
        let mut queued_tracks: std::collections::VecDeque<QueuedTrackMeta> = std::collections::VecDeque::new();

        enum LoadRequest {
            Load {
                gen: u64,
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
        }

        enum LoadResponse {
            Load {
                gen: u64,
                source: TrackSource,
                title: String,
                artist: Option<String>,
                album: Option<String>,
                duration_hint: Option<u64>,
                res: Result<SymphoniaSource, String>,
            },
            QueueNext {
                source: TrackSource,
                title: String,
                artist: Option<String>,
                album: Option<String>,
                duration_hint: Option<u64>,
                res: Result<SymphoniaSource, String>,
            },
        }

        let (load_req_tx, load_req_rx) = std::sync::mpsc::channel::<LoadRequest>();
        let (load_res_tx, load_res_rx) = std::sync::mpsc::channel::<LoadResponse>();

        let reqwest_client_clone = reqwest_client.clone();
        let runtime_handle_clone = runtime_handle.clone();
        let audio_config_clone = audio_config.clone();

        thread::spawn(move || {
            while let Ok(req) = load_req_rx.recv() {
                match req {
                    LoadRequest::Load { gen, source, title, artist, album, duration_hint } => {
                        let res = match &source {
                            TrackSource::Local(path) => {
                                SymphoniaSource::from_path(path, audio_config_clone.clone()).map_err(|e| e.to_string())
                            },
                            TrackSource::Remote(url, headers) => {
                                if url.path().ends_with(".m3u8") {
                                    SymphoniaSource::from_hls(url.clone(), reqwest_client_clone.clone(), runtime_handle_clone.clone(), audio_config_clone.clone()).map_err(|e| e.to_string())
                                } else {
                                    SymphoniaSource::from_url(url.clone(), reqwest_client_clone.clone(), runtime_handle_clone.clone(), headers.clone(), audio_config_clone.clone()).map_err(|e| e.to_string())
                                }
                            }
                        };
                        let _ = load_res_tx.send(LoadResponse::Load { gen, source, title, artist, album, duration_hint, res });
                    }
                    LoadRequest::QueueNext { source, title, artist, album, duration_hint } => {
                        let res = match &source {
                            TrackSource::Local(path) => {
                                SymphoniaSource::from_path(path, audio_config_clone.clone()).map_err(|e| e.to_string())
                            },
                            TrackSource::Remote(url, headers) => {
                                if url.path().ends_with(".m3u8") {
                                    SymphoniaSource::from_hls(url.clone(), reqwest_client_clone.clone(), runtime_handle_clone.clone(), audio_config_clone.clone()).map_err(|e| e.to_string())
                                } else {
                                    SymphoniaSource::from_url(url.clone(), reqwest_client_clone.clone(), runtime_handle_clone.clone(), headers.clone(), audio_config_clone.clone()).map_err(|e| e.to_string())
                                }
                            }
                        };
                        let _ = load_res_tx.send(LoadResponse::QueueNext { source, title, artist, album, duration_hint, res });
                    }
                }
            }
        });

        let media_controls = OSMediaControls::new(&app_handle);

        let get_active_pos = |sink: &Option<Sink>, baseline: f64, offset: f64| -> f64 {
            let raw = sink.as_ref().map_or(0.0, |s| s.get_pos().as_secs_f64());
            (raw - baseline + offset).max(0.0)
        };

        let emit_sync =
            |handle: &AppHandle, state: &str, pos: f64, track: &str, duration: f64| {
                let _ = handle.emit(
                    "player-sync",
                    PlayerSync {
                        state: state.to_string(),
                        position: pos,
                        duration,
                        track: track.to_string(),
                    },
                );
            };

        loop {
            // Drain completed async load operations
            while let Ok(res) = load_res_rx.try_recv() {
                match res {
                    LoadResponse::Load { gen, source, title, artist, album, duration_hint, res } => {
                        if gen == seek_generation {
                            match res {
                                Ok(src) => {
                                    let hint_sec = duration_hint.map(|ms| ms as f64 / 1000.0).filter(|&d| d > 0.0);
                                    let probed_sec = src.total_duration().map(|d| d.as_secs_f64()).filter(|&d| d > 0.0);
                                    
                                    current_duration = match &source {
                                        TrackSource::Remote(..) => {
                                            hint_sec.or(probed_sec).unwrap_or(0.0)
                                        }
                                        TrackSource::Local(..) => {
                                            probed_sec.or(hint_sec).unwrap_or(0.0)
                                        }
                                    };
                                    
                                    let adv_tx = advance_tx.clone();
                                    let tracked = TrackedSource::new(src, move || {
                                        let _ = adv_tx.send(());
                                    });
                                    if let Some(s) = &sink {
                                        s.append(tracked);
                                        s.play();
                                    } else {
                                        media_controls.set_playback_status(false);
                                    }
                                    
                                    media_controls.update_metadata(
                                        &title,
                                        artist.as_deref().unwrap_or("Unknown Artist"),
                                        album.as_deref().unwrap_or("Unknown Album"),
                                        Some(Duration::from_secs_f64(current_duration))
                                    );
                                    media_controls.set_playback_status(true);

                                    emit_sync(
                                        &app_handle,
                                        "Playing",
                                        0.0,
                                        &current_track_path,
                                        current_duration,
                                    );
                                }
                                Err(e) => {
                                    eprintln!("Audio load failed: {e}");
                                    let _ = app_handle.emit("audio-error", e);
                                    current_track_path.clear();
                                    current_track_source = None;
                                    emit_sync(&app_handle, "Stopped", 0.0, "", 0.0);
                                }
                            }
                        }
                    }
                    LoadResponse::QueueNext { source, title, artist, album, duration_hint, res } => {
                        let seg_path = match &source {
                            TrackSource::Local(path) => path.to_string_lossy().to_string(),
                            TrackSource::Remote(url, _) => url.to_string(),
                        };
                        match res {
                            Ok(src) => {
                                let hint_sec = duration_hint.map(|ms| ms as f64 / 1000.0).filter(|&d| d > 0.0);
                                let probed_sec = src.total_duration().map(|d| d.as_secs_f64()).filter(|&d| d > 0.0);
                                let seg_duration = match &source {
                                    TrackSource::Remote(..) => hint_sec.or(probed_sec).unwrap_or(0.0),
                                    TrackSource::Local(..) => probed_sec.or(hint_sec).unwrap_or(0.0),
                                };

                                queued_tracks.push_back(QueuedTrackMeta {
                                    source: source.clone(),
                                    title,
                                    artist,
                                    album,
                                    duration_sec: seg_duration,
                                    track_path: seg_path,
                                });

                                let adv_tx = advance_tx.clone();
                                let tracked = TrackedSource::new(src, move || {
                                    let _ = adv_tx.send(());
                                });
                                if let Some(s) = &sink { s.append(tracked); }
                            }
                            Err(e) => {
                                eprintln!("Audio queue_next failed: {e}");
                                let _ = app_handle.emit("audio-error", e);
                            }
                        }
                    }
                }
            }

            // Drain completed async seek operations
            while let Ok((gen, _seek_pos, res, was_paused)) = seek_rx.try_recv() {
                if gen == seek_generation {
                    is_seeking = false;
                    match res {
                        Ok(src) => {
                            if let Some(s) = &sink {
                                s.stop();
                            }
                            sink = recreate_sink(&mut audio_output, &app_handle);
                            if let Some(s) = &sink {
                                if is_muted {
                                    s.set_volume(0.0);
                                } else {
                                    s.set_volume(current_volume);
                                }
                            }
                            sink_pos_baseline = 0.0;

                            let adv_tx = advance_tx.clone();
                            let tracked = TrackedSource::new(src, move || {
                                let _ = adv_tx.send(());
                            });
                            if let Some(s) = &sink {
                                s.append(tracked);
                                if !was_paused {
                                    s.play();
                                    media_controls.set_playback_status(true);
                                }
                            } else {
                                media_controls.set_playback_status(false);
                            }

                            let is_p = sink.as_ref().is_none_or(|s| s.is_paused());
                            let state_str = if is_p {
                                "Paused"
                            } else {
                                "Playing"
                            };
                            let pos = get_active_pos(&sink, sink_pos_baseline, seek_offset);
                            emit_sync(
                                &app_handle,
                                state_str,
                                pos,
                                &current_track_path,
                                current_duration,
                            );
                        }
                        Err(e) => eprintln!("Audio seek failed: {e}"),
                    }
                }
            }

            // Drain gapless track advancement signals from TrackedSource
            while let Ok(()) = advance_rx.try_recv() {
                let _ = app_handle.emit("track-advanced", ());
                if let Some(next_track) = queued_tracks.pop_front() {
                    sink_pos_baseline = sink.as_ref().map_or(0.0, |s| s.get_pos().as_secs_f64());
                    seek_offset = 0.0;
                    current_track_source = Some(next_track.source);
                    current_track_path = next_track.track_path;
                    current_duration = next_track.duration_sec;

                    media_controls.update_metadata(
                        &next_track.title,
                        next_track.artist.as_deref().unwrap_or("Unknown Artist"),
                        next_track.album.as_deref().unwrap_or("Unknown Album"),
                        Some(Duration::from_secs_f64(current_duration))
                    );
                    media_controls.set_playback_status(true);

                    emit_sync(
                        &app_handle,
                        "Playing",
                        0.0,
                        &current_track_path,
                        current_duration,
                    );
                }
            }

            // Track End Detection (Queue completely drained and playback finished)
            let sink_is_empty = sink.as_ref().is_none_or(|s| s.empty());
            if sink_is_empty && !is_seeking && !current_track_path.is_empty() && queued_tracks.is_empty() {
                let _ = app_handle.emit("track-ended", ());
                current_track_path.clear();
                current_track_source = None;
                current_duration = 0.0;
                seek_offset = 0.0;
                sink_pos_baseline = 0.0;
                media_controls.set_playback_status(false);
                // BUG-06: Emit Stopped so the frontend player bar resets correctly.
                // Without this, playbackState stays "Playing" at 100% forever after queue exhaustion.
                emit_sync(&app_handle, "Stopped", 0.0, "", 0.0);
            }

            let wait_time = if !current_track_path.is_empty() && sink.as_ref().is_some_and(|s| !s.is_paused()) {
                Duration::from_millis(50)
            } else {
                Duration::from_millis(500)
            };

            match rx.recv_timeout(wait_time) {
                Ok(cmd) => match cmd {
                    AudioCommand::Load { source, title, artist, album, duration_hint } => {
                        seek_generation += 1;
                        is_seeking = false;
                        queued_tracks.clear();
                        if let Some(s) = &sink {
                            s.stop();
                        }
                        sink = recreate_sink(&mut audio_output, &app_handle);
                        if let Some(s) = &sink {
                            if is_muted {
                                s.set_volume(0.0);
                            } else {
                                s.set_volume(current_volume);
                            }
                        }
                        seek_offset = 0.0;
                        sink_pos_baseline = 0.0;
                        
                        current_track_source = Some(source.clone());
                        current_track_path = match &source {
                            TrackSource::Local(path) => path.to_string_lossy().to_string(),
                            TrackSource::Remote(url, _) => url.to_string(),
                        };
                        
                        let _ = load_req_tx.send(LoadRequest::Load {
                            gen: seek_generation,
                            source,
                            title,
                            artist,
                            album,
                            duration_hint,
                        });
                    }
                    AudioCommand::QueueNext { source, title, artist, album, duration_hint } => {
                        let _ = load_req_tx.send(LoadRequest::QueueNext {
                            source,
                            title,
                            artist,
                            album,
                            duration_hint,
                        });
                    }
                    AudioCommand::Play => {
                        if let Some(s) = &sink {
                            s.play();
                            media_controls.set_playback_status(true);
                        } else {
                            sink = recreate_sink(&mut audio_output, &app_handle);
                            if let Some(s) = &sink {
                                s.play();
                                media_controls.set_playback_status(true);
                            } else {
                                media_controls.set_playback_status(false);
                            }
                        }
                        let pos = get_active_pos(&sink, sink_pos_baseline, seek_offset);
                        emit_sync(
                            &app_handle,
                            "Playing",
                            pos,
                            &current_track_path,
                            current_duration,
                        );
                    }
                    AudioCommand::Pause => {
                        if let Some(s) = &sink {
                            s.pause();
                        }
                        media_controls.set_playback_status(false);
                        let pos = get_active_pos(&sink, sink_pos_baseline, seek_offset);
                        emit_sync(
                            &app_handle,
                            "Paused",
                            pos,
                            &current_track_path,
                            current_duration,
                        );
                    }
                    AudioCommand::Stop => {
                        is_seeking = false;
                        queued_tracks.clear();
                        if let Some(s) = &sink {
                            s.stop();
                        }
                        current_track_path.clear();
                        current_track_source = None;
                        current_duration = 0.0;
                        seek_offset = 0.0;
                        sink_pos_baseline = 0.0;
                        media_controls.set_playback_status(false);
                        emit_sync(&app_handle, "Stopped", 0.0, "", 0.0);
                    }
                    AudioCommand::SyncState => {
                        let state_str = if current_track_path.is_empty() {
                            "Stopped"
                        } else if sink.as_ref().is_none_or(|s| s.is_paused()) {
                            "Paused"
                        } else {
                            "Playing"
                        };
                        let pos = get_active_pos(&sink, sink_pos_baseline, seek_offset);
                        emit_sync(
                            &app_handle,
                            state_str,
                            pos,
                            &current_track_path,
                            current_duration,
                        );
                    }
                    AudioCommand::Seek(pos) => {
                        if current_track_path.is_empty() {
                            continue;
                        }
                        queued_tracks.clear();
                        seek_generation += 1;
                        is_seeking = true;
                        let gen = seek_generation;
                        let was_paused = sink.as_ref().is_none_or(|s| s.is_paused());
                        if let Some(s) = &sink {
                            s.stop();
                        }
                        seek_offset = pos;
                        sink_pos_baseline = 0.0;

                        emit_sync(
                            &app_handle,
                            if was_paused { "Paused" } else { "Playing" },
                            pos,
                            &current_track_path,
                            current_duration,
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
                            if let Some(s) = &sink {
                                s.set_volume(current_volume);
                            }
                        }
                    }
                    AudioCommand::SetMute(muted) => {
                        is_muted = muted;
                        if let Some(s) = &sink {
                            if is_muted {
                                s.set_volume(0.0);
                            } else {
                                s.set_volume(current_volume);
                            }
                        }
                    }
                    AudioCommand::Quit => {
                        break;
                    }
                },
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    if !sink.as_ref().is_none_or(|s| s.empty()) {
                        let pos = get_active_pos(&sink, sink_pos_baseline, seek_offset);
                        if pos > current_duration && current_duration > 0.0 {
                            current_duration = pos;
                        }
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    break;
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// BUG-06: Guards the PlayerSync wire format — every state string the backend
    /// can emit must serialise correctly over the IPC bridge.
    #[test]
    fn player_sync_serializes_stopped_state() {
        let sync = PlayerSync {
            state: "Stopped".to_string(),
            position: 0.0,
            duration: 0.0,
            track: String::new(),
        };
        let json = serde_json::to_string(&sync).unwrap();
        assert!(json.contains("\"state\":\"Stopped\""), "state field missing or wrong");
        assert!(json.contains("\"position\":0.0"), "position field wrong");
        assert!(json.contains("\"track\":\"\""), "track field wrong");
    }

    #[test]
    fn player_sync_serializes_playing_state() {
        let sync = PlayerSync {
            state: "Playing".to_string(),
            position: 42.5,
            duration: 180.0,
            track: "file:///music/test.flac".to_string(),
        };
        let json = serde_json::to_string(&sync).unwrap();
        assert!(json.contains("\"state\":\"Playing\""));
        assert!(json.contains("\"position\":42.5"));
        assert!(json.contains("\"duration\":180.0"));
        assert!(json.contains("\"track\":\"file:///music/test.flac\""));
    }

    #[test]
    fn player_sync_serializes_paused_state() {
        let sync = PlayerSync {
            state: "Paused".to_string(),
            position: 15.0,
            duration: 240.0,
            track: "https://cdn.example.com/stream.opus".to_string(),
        };
        let json = serde_json::to_string(&sync).unwrap();
        assert!(json.contains("\"state\":\"Paused\""));
        assert!(json.contains("\"position\":15.0"));
    }
}
