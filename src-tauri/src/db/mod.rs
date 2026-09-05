pub mod canonical;
use rusqlite::Connection;
use std::sync::mpsc::Receiver;
use tokio::sync::oneshot;

pub mod schema;
pub mod queries;

use crate::{Album, LocalTrack, Playlist};

pub struct TrackData {
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub track_number: Option<i64>,
    pub file_path: String,
}

pub enum DbRequest {
    RecordPlaybackEvent {
        title: String,
        artist: String,
        album: Option<String>,
        cover_art_url: Option<String>,
        provider_id: String,
        source_id: String,
        duration_ms: Option<u64>,
        resp: oneshot::Sender<Result<(), String>>,
    },
    RecordResolutionResult {
        provider_id: String,
        success: bool,
        resp: oneshot::Sender<Result<(), String>>,
    },
    GetCanonicalQuickPicks {
        mood: Option<String>,
        limit: usize,
        resp: oneshot::Sender<Result<Vec<queries::CanonicalSong>, String>>,
    },
    GetCanonicalKeepListening {
        mood: Option<String>,
        limit: usize,
        resp: oneshot::Sender<Result<Vec<queries::CanonicalSong>, String>>,
    },
    GetCanonicalForgottenFavorites {
        mood: Option<String>,
        limit: usize,
        resp: oneshot::Sender<Result<Vec<queries::CanonicalSong>, String>>,
    },
    GetCanonicalDiscoverSeeds {
        mood: Option<String>,
        limit: usize,
        resp: oneshot::Sender<Result<Vec<queries::CanonicalSong>, String>>,
    },
    ToggleCanonicalLike {
        canonical_key: String,
        title: Option<String>,
        artist: Option<String>,
        album: Option<String>,
        cover_art_url: Option<String>,
        provider_id: Option<String>,
        source_id: Option<String>,
        local_track_id: Option<i64>,
        duration_ms: Option<u64>,
        resp: oneshot::Sender<Result<bool, String>>,
    },
    GetCanonicalLikedSongs {
        limit: usize,
        resp: oneshot::Sender<Result<Vec<queries::CanonicalSong>, String>>,
    },
    GetExtensionMetrics {
        resp: oneshot::Sender<Result<Vec<queries::ExtensionMetric>, String>>,
    },
    GetHeavyRotation7d {
        limit: usize,
        resp: oneshot::Sender<Result<queries::HeavyRotationShelf, String>>,
    },
    GetIncompletePlaybackSessions {
        limit: usize,
        resp: oneshot::Sender<Result<Vec<queries::IncompleteSessionItem>, String>>,
    },
    GetColdStartLocalArtists {
        limit: usize,
        resp: oneshot::Sender<Result<Vec<queries::ColdStartSeedItem>, String>>,
    },
    GetLocalTracks { limit: u32, offset: u32, resp: oneshot::Sender<Result<Vec<LocalTrack>, String>> },
    GetAlbums { limit: u32, offset: u32, resp: oneshot::Sender<Result<Vec<Album>, String>> },
    GetRecentAlbums { limit: u32, resp: oneshot::Sender<Result<Vec<Album>, String>> },
    GetAlbumTracks { album_id: i64, limit: u32, offset: u32, resp: oneshot::Sender<Result<Vec<LocalTrack>, String>> },
    GetPlaylists { limit: u32, offset: u32, resp: oneshot::Sender<Result<Vec<Playlist>, String>> },
    CreatePlaylist { name: String, resp: oneshot::Sender<Result<i64, String>> },
    SaveQueueAsPlaylist { name: String, tracks: Vec<crate::queue::QueueTrack>, resp: oneshot::Sender<Result<i64, String>> },
    AddToPlaylist { playlist_id: i64, track_id: i64, resp: oneshot::Sender<Result<(), String>> },
    GetPlaylistTracks { playlist_id: i64, limit: u32, offset: u32, resp: oneshot::Sender<Result<Vec<LocalTrack>, String>> },
    RemoveFromPlaylist { playlist_id: i64, track_id: i64, resp: oneshot::Sender<Result<(), String>> },
    DeletePlaylist { playlist_id: i64, resp: oneshot::Sender<Result<(), String>> },
    RenamePlaylist { playlist_id: i64, new_name: String, resp: oneshot::Sender<Result<(), String>> },
    ReorderPlaylistTrack { playlist_id: i64, from_pos: i64, to_pos: i64, resp: oneshot::Sender<Result<(), String>> },
    ClearLocalLibrary { resp: oneshot::Sender<Result<(), String>> },
    GetSetting { key: String, resp: oneshot::Sender<Result<Option<String>, String>> },
    GetAllSettings { resp: oneshot::Sender<Result<std::collections::HashMap<String, String>, String>> },
    SetSetting { key: String, value: String, resp: oneshot::Sender<Result<(), String>> },
    FactoryReset { resp: oneshot::Sender<Result<(), String>> },
    InsertTracks { tracks: Vec<TrackData>, resp: oneshot::Sender<Result<usize, String>> },
    LoadAudioCache { path: String, resp: oneshot::Sender<Result<(), String>> },
    RemoveTrackByPath { path: String, resp: oneshot::Sender<Result<(), String>> },
    SearchLibrary { query: String, limit: u32, resp: oneshot::Sender<Result<Vec<LocalTrack>, String>> },
    SyncProviders { providers: Vec<crate::ProviderInfo>, resp: oneshot::Sender<Result<(), String>> },
    GetProviders { resp: oneshot::Sender<Result<Vec<crate::ProviderInfo>, String>> },
    DeleteProvider {
        provider_id: String,
        resp: oneshot::Sender<Result<Option<String>, String>>,
    },
    ToggleProvider { provider_id: String, enabled: bool, resp: oneshot::Sender<Result<(), String>> },
    SaveProviderSettings { provider_id: String, settings_json: String, resp: oneshot::Sender<Result<(), String>> },
    GetProviderStorage { provider_id: String, key: String, resp: oneshot::Sender<Result<Option<String>, String>> },
    SetProviderStorage { provider_id: String, key: String, value: String, resp: oneshot::Sender<Result<(), String>> },
    GetFeedCache { provider_id: String, module_id: String, resp: oneshot::Sender<Result<Option<String>, String>> },
    SetFeedCache { provider_id: String, module_id: String, payload_json: String, ttl_seconds: i64, resp: oneshot::Sender<Result<(), String>> },
    ToggleSavedAlbum {
        id: String,
        title: String,
        artist: Option<String>,
        cover_art_url: Option<String>,
        provider_id: String,
        resp: oneshot::Sender<Result<bool, String>>,
    },
    GetSavedAlbums {
        resp: oneshot::Sender<Result<Vec<queries::SavedAlbum>, String>>,
    },
    ToggleSavedPlaylist {
        id: String,
        title: String,
        author: Option<String>,
        cover_art_url: Option<String>,
        provider_id: String,
        resp: oneshot::Sender<Result<bool, String>>,
    },
    GetSavedPlaylists {
        resp: oneshot::Sender<Result<Vec<queries::SavedPlaylist>, String>>,
    },
    UpsertArtistMetadata {
        id: String,
        name: String,
        avatar_url: Option<String>,
        bio: Option<String>,
        provider_id: String,
        resp: oneshot::Sender<Result<(), String>>,
    },
    GetArtistMetadata {
        query: String,
        resp: oneshot::Sender<Result<Option<queries::ArtistMetadata>, String>>,
    },
    SetFeatureFlag {
        key: String,
        enabled: bool,
        resp: oneshot::Sender<Result<(), String>>,
    },
    ResetFeatureFlags {
        resp: oneshot::Sender<Result<(), String>>,
    },
    GetMarkovAutoplayCandidate {
        seed_artist: Option<String>,
        seed_album_id: Option<i64>,
        seed_track_id: Option<i64>,
        seed_file_path: Option<String>,
        seed_duration_ms: Option<u64>,
        resp: oneshot::Sender<Result<Option<LocalTrack>, String>>,
    },
    Quit,
}

pub fn start_db_thread(mut conn: Connection, rx: Receiver<DbRequest>) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        while let Ok(req) = rx.recv() {
            match req {
                DbRequest::RecordPlaybackEvent { title, artist, album, cover_art_url, provider_id, source_id, duration_ms, resp } => {
                    let res = queries::record_playback_event(&conn, &title, &artist, album.as_deref(), cover_art_url.as_deref(), &provider_id, &source_id, duration_ms).map_err(|e| e.to_string());
                    let _ = resp.send(res);
                }
                DbRequest::RecordResolutionResult { provider_id, success, resp } => {
                    let res = queries::record_resolution_result(&conn, &provider_id, success).map_err(|e| e.to_string());
                    let _ = resp.send(res);
                }
                DbRequest::GetCanonicalQuickPicks { mood, limit, resp } => {
                    let res = queries::get_canonical_quick_picks(&conn, mood.as_deref(), limit).map_err(|e| e.to_string());
                    let _ = resp.send(res);
                }
                DbRequest::GetCanonicalKeepListening { mood, limit, resp } => {
                    let res = queries::get_canonical_keep_listening(&conn, mood.as_deref(), limit).map_err(|e| e.to_string());
                    let _ = resp.send(res);
                }
                DbRequest::GetCanonicalForgottenFavorites { mood, limit, resp } => {
                    let res = queries::get_canonical_forgotten_favorites(&conn, mood.as_deref(), limit).map_err(|e| e.to_string());
                    let _ = resp.send(res);
                }
                DbRequest::GetCanonicalDiscoverSeeds { mood, limit, resp } => {
                    let res = queries::get_canonical_discover_seeds(&conn, mood.as_deref(), limit).map_err(|e| e.to_string());
                    let _ = resp.send(res);
                }
                DbRequest::ToggleCanonicalLike {
                    canonical_key,
                    title,
                    artist,
                    album,
                    cover_art_url,
                    provider_id,
                    source_id,
                    local_track_id,
                    duration_ms,
                    resp,
                } => {
                    let res = queries::toggle_canonical_like(
                        &conn,
                        &canonical_key,
                        title.as_deref(),
                        artist.as_deref(),
                        album.as_deref(),
                        cover_art_url.as_deref(),
                        provider_id.as_deref(),
                        source_id.as_deref(),
                        local_track_id,
                        duration_ms,
                    ).map_err(|e| e.to_string());
                    let _ = resp.send(res);
                }
                DbRequest::GetCanonicalLikedSongs { limit, resp } => {
                    let res = queries::get_canonical_liked_songs(&conn, limit).map_err(|e| e.to_string());
                    let _ = resp.send(res);
                }
                DbRequest::GetExtensionMetrics { resp } => {
                    let res = queries::get_extension_metrics(&conn).map_err(|e| e.to_string());
                    let _ = resp.send(res);
                }
                DbRequest::GetHeavyRotation7d { limit, resp } => {
                    let res = queries::get_heavy_rotation_7d(&conn, limit).map_err(|e| e.to_string());
                    let _ = resp.send(res);
                }
                DbRequest::GetIncompletePlaybackSessions { limit, resp } => {
                    let res = queries::get_incomplete_playback_sessions(&conn, limit).map_err(|e| e.to_string());
                    let _ = resp.send(res);
                }
                DbRequest::GetColdStartLocalArtists { limit, resp } => {
                    let res = queries::get_cold_start_local_artists(&conn, limit).map_err(|e| e.to_string());
                    let _ = resp.send(res);
                }
                DbRequest::GetLocalTracks { limit, offset, resp } => {
                    let _ = resp.send(queries::get_local_tracks(&conn, limit, offset));
                }
                DbRequest::GetAlbums { limit, offset, resp } => {
                    let _ = resp.send(queries::get_albums(&conn, limit, offset));
                }
                DbRequest::GetRecentAlbums { limit, resp } => {
                    let _ = resp.send(queries::get_recent_albums(&conn, limit));
                }
                DbRequest::GetAlbumTracks { album_id, limit, offset, resp } => {
                    let _ = resp.send(queries::get_album_tracks(&conn, album_id, limit, offset));
                }
                DbRequest::GetPlaylists { limit, offset, resp } => {
                    let _ = resp.send(queries::get_playlists(&conn, limit, offset));
                }
                DbRequest::SaveQueueAsPlaylist { name, tracks, resp } => {
                    let _ = resp.send(queries::save_queue_as_playlist(&mut conn, &name, &tracks));
                }
                DbRequest::CreatePlaylist { name, resp } => {
                    let _ = resp.send(queries::create_playlist(&conn, &name));
                }
                DbRequest::AddToPlaylist { playlist_id, track_id, resp } => {
                    let _ = resp.send(queries::add_to_playlist(&conn, playlist_id, track_id));
                }
                DbRequest::GetPlaylistTracks { playlist_id, limit, offset, resp } => {
                    let _ = resp.send(queries::get_playlist_tracks(&conn, playlist_id, limit, offset));
                }
                DbRequest::RemoveFromPlaylist { playlist_id, track_id, resp } => {
                    let _ = resp.send(queries::remove_from_playlist(&mut conn, playlist_id, track_id));
                }
                DbRequest::DeletePlaylist { playlist_id, resp } => {
                    let _ = resp.send(queries::delete_playlist(&conn, playlist_id));
                }
                DbRequest::RenamePlaylist { playlist_id, new_name, resp } => {
                    let _ = resp.send(queries::rename_playlist(&conn, playlist_id, &new_name));
                }
                DbRequest::ReorderPlaylistTrack { playlist_id, from_pos, to_pos, resp } => {
                    let _ = resp.send(queries::reorder_playlist_track(&mut conn, playlist_id, from_pos, to_pos));
                }
                DbRequest::ClearLocalLibrary { resp } => {
                    let _ = resp.send(queries::clear_local_library(&conn));
                }
                DbRequest::GetSetting { key, resp } => {
                    let _ = resp.send(queries::get_setting(&conn, &key));
                }
                DbRequest::GetAllSettings { resp } => {
                    let _ = resp.send(queries::get_all_settings(&conn));
                }
                DbRequest::SetSetting { key, value, resp } => {
                    let _ = resp.send(queries::set_setting(&conn, &key, &value));
                }
                DbRequest::FactoryReset { resp } => {
                    let _ = resp.send(queries::factory_reset(&conn));
                }
                DbRequest::InsertTracks { tracks, resp } => {
                    let _ = resp.send(queries::insert_tracks(&mut conn, tracks));
                }
                DbRequest::LoadAudioCache { path, resp } => {
                    let _ = resp.send(queries::load_audio_cache(&conn, &path));
                }
                DbRequest::RemoveTrackByPath { path, resp } => {
                    let _ = resp.send(queries::remove_track_by_path(&conn, &path));
                }
                DbRequest::SearchLibrary { query, limit, resp } => {
                    let _ = resp.send(queries::search_library(&conn, &query, limit));
                }
                DbRequest::SyncProviders { providers, resp } => {
                    let _ = resp.send(queries::sync_providers(&conn, providers));
                }
                DbRequest::GetProviders { resp } => {
                    let _ = resp.send(queries::get_providers(&conn));
                }
                DbRequest::DeleteProvider { provider_id, resp } => {
                    let res = queries::delete_provider(&conn, &provider_id);
                    let _ = resp.send(res);
                }
                DbRequest::ToggleProvider { provider_id, enabled, resp } => {
                    let _ = resp.send(queries::toggle_provider(&conn, &provider_id, enabled));
                }
                DbRequest::SaveProviderSettings { provider_id, settings_json, resp } => {
                    let _ = resp.send(queries::save_provider_settings(&conn, &provider_id, &settings_json));
                }
                DbRequest::GetProviderStorage { provider_id, key, resp } => {
                    let _ = resp.send(queries::get_provider_storage(&conn, &provider_id, &key));
                }
                DbRequest::SetProviderStorage { provider_id, key, value, resp } => {
                    let _ = resp.send(queries::set_provider_storage(&conn, &provider_id, &key, &value));
                }
                DbRequest::GetFeedCache { provider_id, module_id, resp } => {
                    let res = queries::get_feed_cache(&conn, &provider_id, &module_id).map_err(|e| e.to_string());
                    let _ = resp.send(res);
                }
                DbRequest::SetFeedCache { provider_id, module_id, payload_json, ttl_seconds, resp } => {
                    let res = queries::set_feed_cache(&conn, &provider_id, &module_id, &payload_json, ttl_seconds).map_err(|e| e.to_string());
                    let _ = resp.send(res);
                }
                DbRequest::ToggleSavedAlbum { id, title, artist, cover_art_url, provider_id, resp } => {
                    let res = queries::toggle_saved_album(&conn, &id, &title, artist.as_deref(), cover_art_url.as_deref(), &provider_id).map_err(|e| e.to_string());
                    let _ = resp.send(res);
                }
                DbRequest::GetSavedAlbums { resp } => {
                    let res = queries::get_saved_albums(&conn).map_err(|e| e.to_string());
                    let _ = resp.send(res);
                }
                DbRequest::ToggleSavedPlaylist { id, title, author, cover_art_url, provider_id, resp } => {
                    let res = queries::toggle_saved_playlist(&conn, &id, &title, author.as_deref(), cover_art_url.as_deref(), &provider_id).map_err(|e| e.to_string());
                    let _ = resp.send(res);
                }
                DbRequest::GetSavedPlaylists { resp } => {
                    let res = queries::get_saved_playlists(&conn).map_err(|e| e.to_string());
                    let _ = resp.send(res);
                }
                DbRequest::UpsertArtistMetadata { id, name, avatar_url, bio, provider_id, resp } => {
                    let res = queries::upsert_artist_metadata(&conn, &id, &name, avatar_url.as_deref(), bio.as_deref(), &provider_id).map_err(|e| e.to_string());
                    let _ = resp.send(res);
                }
                DbRequest::GetArtistMetadata { query, resp } => {
                    let res = queries::get_artist_metadata(&conn, &query).map_err(|e| e.to_string());
                    let _ = resp.send(res);
                }
                DbRequest::SetFeatureFlag { key, enabled, resp } => {
                    let res = queries::set_feature_flag(&conn, &key, enabled).map_err(|e| e.to_string());
                    let _ = resp.send(res);
                }
                DbRequest::ResetFeatureFlags { resp } => {
                    let res = queries::reset_feature_flags(&conn).map_err(|e| e.to_string());
                    let _ = resp.send(res);
                }
                DbRequest::GetMarkovAutoplayCandidate { seed_artist, seed_album_id, seed_track_id, seed_file_path, seed_duration_ms, resp } => {
                    let res = queries::get_markov_autoplay_candidate(
                        &conn,
                        seed_artist.as_deref(),
                        seed_album_id,
                        seed_track_id,
                        seed_file_path.as_deref(),
                        seed_duration_ms,
                    ).map_err(|e| e.to_string());
                    let _ = resp.send(res);
                }
                DbRequest::Quit => {
                    break;
                }
            }
        }
    })
}
