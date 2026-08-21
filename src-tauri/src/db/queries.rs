use rusqlite::Result as SqlResult;
use serde::{Serialize, Deserialize};
use rusqlite::Connection;
use crate::{Album, LocalTrack, Playlist};
use super::TrackData;

pub fn get_local_tracks(conn: &Connection, limit: u32, offset: u32) -> Result<Vec<LocalTrack>, String> {
    let mut tracks = Vec::new();
    let mut stmt = conn.prepare("SELECT id, title, artist, album_id, track_number, file_path FROM tracks ORDER BY id LIMIT ?1 OFFSET ?2").map_err(|e| e.to_string())?;
    let track_iter = stmt.query_map([&limit, &offset], |row| {
        Ok(LocalTrack {
            id: row.get(0)?,
            title: row.get(1)?,
            artist: row.get(2)?,
            album_id: row.get(3)?,
            track_number: row.get(4)?,
            file_path: row.get(5)?,
        })
    }).map_err(|e| e.to_string())?;

    for t in track_iter.flatten() {
        tracks.push(t);
    }
    Ok(tracks)
}

pub fn get_albums(conn: &Connection, limit: u32, offset: u32) -> Result<Vec<Album>, String> {
    let mut albums = Vec::new();
    let mut stmt = conn.prepare("SELECT id, title, artist, cover_art_path FROM albums ORDER BY artist, title LIMIT ?1 OFFSET ?2").map_err(|e| e.to_string())?;
    let album_iter = stmt.query_map([&limit, &offset], |row| {
        Ok(Album {
            id: row.get(0)?,
            title: row.get(1)?,
            artist: row.get(2)?,
            cover_art_path: row.get(3)?,
        })
    }).map_err(|e| e.to_string())?;

    for a in album_iter.flatten() {
        albums.push(a);
    }
    Ok(albums)
}


pub fn get_recent_albums(conn: &Connection, limit: u32) -> Result<Vec<Album>, String> {
    let mut albums = Vec::new();
    let mut stmt = conn.prepare("SELECT id, title, artist, cover_art_path FROM albums ORDER BY id DESC LIMIT ?1").map_err(|e| e.to_string())?;
    let album_iter = stmt.query_map([&limit], |row| {
        Ok(Album {
            id: row.get(0)?,
            title: row.get(1)?,
            artist: row.get(2)?,
            cover_art_path: row.get(3)?,
        })
    }).map_err(|e| e.to_string())?;

    for a in album_iter.flatten() {
        albums.push(a);
    }
    Ok(albums)
}

pub fn get_album_tracks(conn: &Connection, album_id: i64, limit: u32, offset: u32) -> Result<Vec<LocalTrack>, String> {
    let mut tracks = Vec::new();
    let mut stmt = conn.prepare("SELECT id, title, artist, album_id, track_number, file_path FROM tracks WHERE album_id = ?1 ORDER BY track_number LIMIT ?2 OFFSET ?3").map_err(|e| e.to_string())?;
    let track_iter = stmt.query_map([&album_id, &(limit as i64), &(offset as i64)], |row| {
        Ok(LocalTrack {
            id: row.get(0)?,
            title: row.get(1)?,
            artist: row.get(2)?,
            album_id: row.get(3)?,
            track_number: row.get(4)?,
            file_path: row.get(5)?,
        })
    }).map_err(|e| e.to_string())?;

    for t in track_iter.flatten() {
        tracks.push(t);
    }
    Ok(tracks)
}

pub fn get_playlists(conn: &Connection, limit: u32, offset: u32) -> Result<Vec<Playlist>, String> {
    let mut playlists = Vec::new();
    let mut stmt = conn.prepare("SELECT id, name FROM playlists ORDER BY created_at LIMIT ?1 OFFSET ?2").map_err(|e| e.to_string())?;
    let playlist_iter = stmt.query_map([&limit, &offset], |row| {
        Ok(Playlist {
            id: row.get(0)?,
            name: row.get(1)?,
        })
    }).map_err(|e| e.to_string())?;

    for p in playlist_iter.flatten() {
        playlists.push(p);
    }
    Ok(playlists)
}

pub fn create_playlist(conn: &Connection, name: &str) -> Result<i64, String> {
    conn.execute("INSERT INTO playlists (name) VALUES (?1)", [&name]).map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

pub fn add_to_playlist(conn: &Connection, playlist_id: i64, track_id: i64) -> Result<(), String> {
    let mut stmt = conn.prepare("SELECT COALESCE(MAX(position), 0) FROM playlist_tracks WHERE playlist_id = ?1").map_err(|e| e.to_string())?;
    let max_pos: i64 = stmt.query_row([&playlist_id], |row| row.get(0)).unwrap_or(0);
    
    conn.execute("INSERT INTO playlist_tracks (playlist_id, track_id, position) VALUES (?1, ?2, ?3)", [&playlist_id, &track_id, &(max_pos + 1)]).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_playlist_tracks(conn: &Connection, playlist_id: i64, limit: u32, offset: u32) -> Result<Vec<LocalTrack>, String> {
    let mut tracks = Vec::new();
    let mut stmt = conn.prepare("SELECT t.id, t.title, t.artist, t.album_id, t.track_number, t.file_path FROM tracks t JOIN playlist_tracks pt ON t.id = pt.track_id WHERE pt.playlist_id = ?1 ORDER BY pt.position LIMIT ?2 OFFSET ?3").map_err(|e| e.to_string())?;
    let track_iter = stmt.query_map([&playlist_id, &(limit as i64), &(offset as i64)], |row| {
        Ok(LocalTrack {
            id: row.get(0)?,
            title: row.get(1)?,
            artist: row.get(2)?,
            album_id: row.get(3)?,
            track_number: row.get(4)?,
            file_path: row.get(5)?,
        })
    }).map_err(|e| e.to_string())?;

    for t in track_iter.flatten() {
        tracks.push(t);
    }
    Ok(tracks)
}

pub fn remove_from_playlist(conn: &mut Connection, playlist_id: i64, track_id: i64) -> Result<(), String> {
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    
    let position: Option<i64> = tx.query_row(
        "SELECT position FROM playlist_tracks WHERE playlist_id = ?1 AND track_id = ?2",
        [&playlist_id, &track_id],
        |row| row.get(0)
    ).ok();

    if let Some(pos) = position {
        tx.execute(
            "DELETE FROM playlist_tracks WHERE playlist_id = ?1 AND track_id = ?2",
            [&playlist_id, &track_id]
        ).map_err(|e| e.to_string())?;

        tx.execute(
            "UPDATE playlist_tracks SET position = position - 1 WHERE playlist_id = ?1 AND position > ?2",
            [&playlist_id, &pos]
        ).map_err(|e| e.to_string())?;
    }
    
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

pub fn delete_playlist(conn: &Connection, playlist_id: i64) -> Result<(), String> {
    conn.execute("DELETE FROM playlists WHERE id = ?1", [&playlist_id]).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn rename_playlist(conn: &Connection, playlist_id: i64, new_name: &str) -> Result<(), String> {
    conn.execute("UPDATE playlists SET name = ?1 WHERE id = ?2", (new_name, &playlist_id)).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn reorder_playlist_track(conn: &mut Connection, playlist_id: i64, from_pos: i64, to_pos: i64) -> Result<(), String> {
    if from_pos == to_pos {
        return Ok(());
    }

    let tx = conn.transaction().map_err(|e| e.to_string())?;

    tx.execute(
        "UPDATE playlist_tracks SET position = -1 WHERE playlist_id = ?1 AND position = ?2",
        [&playlist_id, &from_pos]
    ).map_err(|e| e.to_string())?;

    if from_pos < to_pos {
        tx.execute(
            "UPDATE playlist_tracks SET position = position - 1 WHERE playlist_id = ?1 AND position > ?2 AND position <= ?3",
            [&playlist_id, &from_pos, &to_pos]
        ).map_err(|e| e.to_string())?;
    } else {
        tx.execute(
            "UPDATE playlist_tracks SET position = position + 1 WHERE playlist_id = ?1 AND position >= ?2 AND position < ?3",
            [&playlist_id, &to_pos, &from_pos]
        ).map_err(|e| e.to_string())?;
    }

    tx.execute(
        "UPDATE playlist_tracks SET position = ?1 WHERE playlist_id = ?2 AND position = -1",
        [&to_pos, &playlist_id]
    ).map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

pub fn clear_local_library(conn: &Connection) -> Result<(), String> {
    conn.execute("DELETE FROM tracks", []).map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM albums", []).map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM playlists", []).map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM playlist_tracks", []).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_setting(conn: &Connection, key: &str) -> Result<Option<String>, String> {
    let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1").map_err(|e| e.to_string())?;
    let mut rows = stmt.query([key]).map_err(|e| e.to_string())?;
    if let Ok(Some(row)) = rows.next() {
        let value: String = row.get(0).map_err(|e| e.to_string())?;
        return Ok(Some(value));
    }
    Ok(None)
}

pub fn get_all_settings(conn: &Connection) -> Result<std::collections::HashMap<String, String>, String> {
    let mut stmt = conn.prepare("SELECT key, value FROM settings").map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    }).map_err(|e| e.to_string())?;

    let mut map = std::collections::HashMap::new();
    for (k, v) in rows.flatten() {
        map.insert(k, v);
    }
    Ok(map)
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<(), String> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = ?2",
        (key, value),
    ).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn factory_reset(conn: &Connection) -> Result<(), String> {
    let _ = conn.execute("DELETE FROM tracks", []);
    let _ = conn.execute("DELETE FROM settings", []);
    let _ = conn.execute("DELETE FROM albums", []);
    let _ = conn.execute("DELETE FROM playlists", []);
    let _ = conn.execute("DELETE FROM playlist_tracks", []);
    Ok(())
}

pub fn insert_tracks(conn: &mut Connection, tracks: Vec<TrackData>) -> Result<usize, String> {
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let mut added = 0;
    
    // Pre-prepare inside the transaction to avoid compiling on every loop
    let mut album_select = tx.prepare("SELECT id FROM albums WHERE title = ?1 AND artist = ?2").map_err(|e| e.to_string())?;
    let mut album_cache: std::collections::HashMap<(String, String), Option<i64>> = std::collections::HashMap::new();

    for track in tracks {
        let album_title = track.album.unwrap_or_else(|| "Unknown Album".to_string());
        let album_artist = track.artist.clone().unwrap_or_else(|| "Unknown Artist".to_string());
        let cache_key = (album_title.clone(), album_artist.clone());

        let album_id = if let Some(&id) = album_cache.get(&cache_key) {
            id
        } else {
            let _ = tx.execute(
                "INSERT OR IGNORE INTO albums (title, artist) VALUES (?1, ?2)",
                (&album_title, &album_artist),
            );
            let id: Option<i64> = album_select.query_row((&album_title, &album_artist), |row| row.get(0)).ok();
            album_cache.insert(cache_key, id);
            id
        };

        let res = tx.execute(
            "INSERT OR IGNORE INTO tracks (title, artist, album_id, track_number, file_path) VALUES (?1, ?2, ?3, ?4, ?5)",
            (&track.title, &track.artist, &album_id, &track.track_number, &track.file_path),
        );
        
        if res.is_ok() && res.unwrap() > 0 {
            added += 1;
        }
    }
    drop(album_select);
    tx.commit().map_err(|e| e.to_string())?;
    Ok(added)
}

pub fn load_audio_cache(conn: &Connection, path: &str) -> Result<(), String> {
    let _ = conn.execute(
        "INSERT OR IGNORE INTO tracks (title, file_path) VALUES (?1, ?2)",
        ("Unknown Title", path),
    );
    Ok(())
}

pub fn remove_track_by_path(conn: &Connection, path: &str) -> Result<(), String> {
    let album_id: Option<i64> = conn
        .query_row("SELECT album_id FROM tracks WHERE file_path = ?1", [path], |r| r.get(0))
        .ok()
        .flatten();

    conn.execute("DELETE FROM tracks WHERE file_path = ?1", [path])
        .map_err(|e| e.to_string())?;

    // Remove orphaned album if this was its last track.
    if let Some(album_id) = album_id {
        let remaining: i64 = conn
            .query_row("SELECT COUNT(*) FROM tracks WHERE album_id = ?1", [album_id], |r| r.get(0))
            .unwrap_or(0);
        if remaining == 0 {
            let _ = conn.execute("DELETE FROM albums WHERE id = ?1", [album_id]);
        }
    }

    Ok(())
}

pub fn search_library(conn: &Connection, query: &str, limit: u32) -> Result<Vec<LocalTrack>, String> {
    let mut tracks = Vec::new();
    let like_query = format!("%{}%", query);
    
    let mut stmt = conn.prepare(
        "SELECT t.id, t.title, t.artist, t.album_id, t.track_number, t.file_path 
         FROM tracks t 
         LEFT JOIN albums a ON t.album_id = a.id
         WHERE t.title LIKE ?1 OR t.artist LIKE ?1 OR a.title LIKE ?1
         ORDER BY t.title 
         LIMIT ?2"
    ).map_err(|e| e.to_string())?;

    let track_iter = stmt.query_map(rusqlite::params![&like_query, &limit], |row| {
        Ok(LocalTrack {
            id: row.get(0)?,
            title: row.get(1)?,
            artist: row.get(2)?,
            album_id: row.get(3)?,
            track_number: row.get(4)?,
            file_path: row.get(5)?,
        })
    }).map_err(|e| e.to_string())?;

    for t in track_iter.flatten() {
        tracks.push(t);
    }
    Ok(tracks)
}

pub fn sync_providers(conn: &Connection, providers: Vec<crate::ProviderInfo>) -> Result<(), String> {
    if providers.is_empty() {
        let _ = conn.execute("DELETE FROM providers", []);
        return Ok(());
    }

    // Prune ghost extensions no longer on disk
    let active_ids: Vec<String> = providers.iter().map(|p| p.id.clone()).collect();
    let placeholders = (1..=active_ids.len()).map(|i| format!("?{}", i)).collect::<Vec<_>>().join(", ");
    let delete_query = format!("DELETE FROM providers WHERE id NOT IN ({})", placeholders);
    let params: Vec<&dyn rusqlite::ToSql> = active_ids.iter().map(|id| id as &dyn rusqlite::ToSql).collect();
    let _ = conn.execute(&delete_query, rusqlite::params_from_iter(params));

    for p in providers {
        conn.execute(
            "INSERT INTO providers (id, name, author, version, file_path, status, error_message, checksum, capabilities, homepage, settings_schema, priority, icon)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
             ON CONFLICT(id) DO UPDATE SET
                name=excluded.name,
                author=excluded.author,
                version=excluded.version,
                file_path=excluded.file_path,
                error_message=excluded.error_message,
                checksum=excluded.checksum,
                capabilities=excluded.capabilities,
                homepage=excluded.homepage,
                settings_schema=excluded.settings_schema,
                priority=excluded.priority,
                icon=excluded.icon,
                updated_at=CURRENT_TIMESTAMP",
            rusqlite::params![
                p.id, p.name, p.author, p.version, p.file_path,
                p.status, p.error_message, p.checksum,
                p.capabilities.map(|c| serde_json::to_string(&c).unwrap_or_default()),
                p.homepage, p.settings_schema, p.priority, p.icon
            ],
        ).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn delete_provider(conn: &Connection, provider_id: &str) -> Result<Option<String>, String> {
    let file_path: Option<String> = conn.query_row(
        "SELECT file_path FROM providers WHERE id = ?1",
        [provider_id],
        |row| row.get(0),
    ).ok();

    conn.execute("DELETE FROM providers WHERE id = ?1", [provider_id]).map_err(|e| e.to_string())?;
    let _ = conn.execute("DELETE FROM extension_metrics WHERE provider_id = ?1", [provider_id]);

    Ok(file_path)
}

pub fn get_providers(conn: &Connection) -> Result<Vec<crate::ProviderInfo>, String> {
    let mut stmt = conn.prepare("SELECT id, name, author, version, file_path, status, error_message, checksum, capabilities, homepage, settings_schema, priority, icon, settings FROM providers").map_err(|e| e.to_string())?;
    let mut rows = stmt.query([]).map_err(|e| e.to_string())?;

    let mut providers = Vec::new();
    while let Ok(Some(row)) = rows.next() {
        let id: String = row.get(0).unwrap_or_default();
        let capabilities_str: Option<String> = row.get(8).unwrap_or(None);
        let capabilities: Option<Vec<String>> = capabilities_str.and_then(|s| serde_json::from_str(&s).ok());
        
        providers.push(crate::ProviderInfo {
            id,
            name: row.get(1).unwrap_or_default(),
            author: row.get(2).unwrap_or_default(),
            version: row.get(3).unwrap_or_default(),
            file_path: row.get(4).unwrap_or_default(),
            status: row.get(5).unwrap_or_else(|_| "enabled".to_string()),
            error_message: row.get(6).unwrap_or(None),
            checksum: row.get(7).unwrap_or(None),
            capabilities,
            homepage: row.get(9).unwrap_or(None),
            settings_schema: row.get(10).unwrap_or(None),
            priority: row.get(11).unwrap_or(0),
            icon: row.get(12).unwrap_or(None),
            settings: row.get(13).unwrap_or(None),
        });
    }

    Ok(providers)
}

pub fn toggle_provider(conn: &Connection, provider_id: &str, enabled: bool) -> Result<(), String> {
    let status = if enabled { "enabled" } else { "disabled" };
    conn.execute(
        "UPDATE providers SET status = ?1 WHERE id = ?2",
        rusqlite::params![status, provider_id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn save_provider_settings(conn: &Connection, provider_id: &str, settings_json: &str) -> Result<(), String> {
    conn.execute(
        "UPDATE providers SET settings = ?1 WHERE id = ?2",
        rusqlite::params![settings_json, provider_id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_provider_storage(conn: &Connection, provider_id: &str, key: &str) -> Result<Option<String>, String> {
    let mut stmt = conn.prepare("SELECT value FROM provider_storage WHERE provider_id = ?1 AND key = ?2").map_err(|e| e.to_string())?;
    let mut rows = stmt.query(rusqlite::params![provider_id, key]).map_err(|e| e.to_string())?;
    if let Ok(Some(row)) = rows.next() {
        let value: String = row.get(0).map_err(|e| e.to_string())?;
        Ok(Some(value))
    } else {
        Ok(None)
    }
}

pub fn set_provider_storage(conn: &Connection, provider_id: &str, key: &str, value: &str) -> Result<(), String> {
    conn.execute(
        "INSERT INTO provider_storage (provider_id, key, value) VALUES (?1, ?2, ?3)
         ON CONFLICT(provider_id, key) DO UPDATE SET value=excluded.value",
        rusqlite::params![provider_id, key, value],
    ).map_err(|e| e.to_string())?;
    Ok(())
}


use crate::db::canonical::make_canonical_key;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CanonicalSong {
    pub id: i64,
    pub canonical_key: String,
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub cover_art_url: Option<String>,
    pub play_count: i64,
    pub last_played_at: Option<String>,
    pub liked: bool,
    pub local_track_id: Option<i64>,
    pub local_file_path: Option<String>,
    pub last_provider_id: String,
    pub last_source_id: String,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExtensionMetric {
    pub provider_id: String,
    pub total_plays: i64,
    pub total_duration_ms: i64,
    pub total_resolutions: i64,
    pub failed_resolutions: i64,
    pub last_used_at: String,
}

#[allow(clippy::too_many_arguments)]
pub fn record_playback_event(
    conn: &Connection,
    title: &str,
    artist: &str,
    album: Option<&str>,
    cover_art_url: Option<&str>,
    provider_id: &str,
    source_id: &str,
    duration_ms: Option<u64>,
) -> SqlResult<()> {
    let canonical_key = make_canonical_key(title, artist);

    // 1. Check if a local file exists matching title and artist
    let local_match: Option<(i64, Option<String>)> = conn.query_row(
        "SELECT t.id, a.cover_art_path FROM tracks t LEFT JOIN albums a ON t.album_id = a.id WHERE LOWER(t.title) = LOWER(?1) AND LOWER(t.artist) = LOWER(?2) LIMIT 1",
        [title, artist],
        |row| Ok((row.get(0)?, row.get(1)?))
    ).ok();

    let local_track_id = local_match.as_ref().map(|m| m.0);
    let resolved_cover = cover_art_url.map(|s| s.to_string()).or_else(|| local_match.and_then(|m| m.1));

    // 2. Upsert into song_telemetry
    conn.execute(
        "INSERT INTO song_telemetry (
            canonical_key, title, artist, album, cover_art_url, play_count, last_played_at, liked, local_track_id, last_provider_id, last_source_id, duration_ms
        ) VALUES (?1, ?2, ?3, ?4, ?5, 1, CURRENT_TIMESTAMP, 0, ?6, ?7, ?8, ?9)
        ON CONFLICT(canonical_key) DO UPDATE SET
            play_count = play_count + 1,
            last_played_at = CURRENT_TIMESTAMP,
            title = CASE WHEN length(?2) > 0 THEN ?2 ELSE title END,
            artist = CASE WHEN length(?3) > 0 THEN ?3 ELSE artist END,
            album = COALESCE(?4, album),
            cover_art_url = COALESCE(?5, cover_art_url),
            local_track_id = COALESCE(?6, local_track_id),
            last_provider_id = ?7,
            last_source_id = ?8,
            duration_ms = COALESCE(?9, duration_ms)",
        rusqlite::params![
            canonical_key,
            title,
            artist,
            album,
            resolved_cover,
            local_track_id,
            provider_id,
            source_id,
            duration_ms.map(|d| d as i64),
        ],
    )?;

    // 3. Append to playback_events
    conn.execute(
        "INSERT INTO playback_events (canonical_key, provider_id, source_id, duration_ms) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![canonical_key, provider_id, source_id, duration_ms.map(|d| d as i64)],
    )?;

    // 4. Update extension_metrics
    conn.execute(
        "INSERT INTO extension_metrics (provider_id, total_plays, total_duration_ms, total_resolutions, failed_resolutions, last_used_at)
         VALUES (?1, 1, ?2, 0, 0, CURRENT_TIMESTAMP)
         ON CONFLICT(provider_id) DO UPDATE SET
            total_plays = total_plays + 1,
            total_duration_ms = total_duration_ms + ?2,
            last_used_at = CURRENT_TIMESTAMP",
        rusqlite::params![provider_id, duration_ms.unwrap_or(0) as i64],
    )?;

    Ok(())
}

pub fn record_resolution_result(conn: &Connection, provider_id: &str, success: bool) -> SqlResult<()> {
    let failed_inc = if success { 0 } else { 1 };
    conn.execute(
        "INSERT INTO extension_metrics (provider_id, total_plays, total_duration_ms, total_resolutions, failed_resolutions, last_used_at)
         VALUES (?1, 0, 0, 1, ?2, CURRENT_TIMESTAMP)
         ON CONFLICT(provider_id) DO UPDATE SET
            total_resolutions = total_resolutions + 1,
            failed_resolutions = failed_resolutions + ?2,
            last_used_at = CURRENT_TIMESTAMP",
        rusqlite::params![provider_id, failed_inc],
    )?;
    Ok(())
}

pub fn get_canonical_quick_picks(conn: &Connection, limit: usize) -> SqlResult<Vec<CanonicalSong>> {
    let mut stmt = conn.prepare(
        "SELECT st.id, st.canonical_key, st.title, st.artist, st.album, st.cover_art_url, st.play_count, st.last_played_at, st.liked, st.local_track_id, t.file_path, st.last_provider_id, st.last_source_id, st.duration_ms
         FROM song_telemetry st
         LEFT JOIN tracks t ON st.local_track_id = t.id
         ORDER BY st.play_count DESC, st.last_played_at DESC
         LIMIT ?1"
    )?;

    let rows = stmt.query_map([limit as i64], map_canonical_row)?;
    let mut results = Vec::new();
    for r in rows {
        results.push(r?);
    }
    Ok(results)
}

pub fn get_canonical_keep_listening(conn: &Connection, limit: usize) -> SqlResult<Vec<CanonicalSong>> {
    let mut stmt = conn.prepare(
        "SELECT st.id, st.canonical_key, st.title, st.artist, st.album, st.cover_art_url, st.play_count, st.last_played_at, st.liked, st.local_track_id, t.file_path, st.last_provider_id, st.last_source_id, st.duration_ms
         FROM song_telemetry st
         LEFT JOIN tracks t ON st.local_track_id = t.id
         WHERE st.last_played_at >= datetime('now', '-14 days')
         ORDER BY st.last_played_at DESC
         LIMIT ?1"
    )?;

    let rows = stmt.query_map([limit as i64], map_canonical_row)?;
    let mut results = Vec::new();
    for r in rows {
        results.push(r?);
    }
    Ok(results)
}

pub fn get_canonical_forgotten_favorites(conn: &Connection, limit: usize) -> SqlResult<Vec<CanonicalSong>> {
    let mut stmt = conn.prepare(
        "SELECT st.id, st.canonical_key, st.title, st.artist, st.album, st.cover_art_url, st.play_count, st.last_played_at, st.liked, st.local_track_id, t.file_path, st.last_provider_id, st.last_source_id, st.duration_ms
         FROM song_telemetry st
         LEFT JOIN tracks t ON st.local_track_id = t.id
         WHERE (st.play_count >= 2 OR st.liked = 1)
           AND (st.last_played_at IS NULL OR st.last_played_at <= datetime('now', '-30 days'))
         ORDER BY st.play_count DESC
         LIMIT ?1"
    )?;

    let rows = stmt.query_map([limit as i64], map_canonical_row)?;
    let mut results = Vec::new();
    for r in rows {
        results.push(r?);
    }
    Ok(results)
}

pub fn get_canonical_liked_songs(conn: &Connection, limit: usize) -> SqlResult<Vec<CanonicalSong>> {
    let mut stmt = conn.prepare(
        "SELECT st.id, st.canonical_key, st.title, st.artist, st.album, st.cover_art_url, st.play_count, st.last_played_at, st.liked, st.local_track_id, t.file_path, st.last_provider_id, st.last_source_id, st.duration_ms
         FROM song_telemetry st
         LEFT JOIN tracks t ON st.local_track_id = t.id
         WHERE st.liked = 1
         ORDER BY st.last_played_at DESC
         LIMIT ?1"
    )?;

    let rows = stmt.query_map([limit as i64], map_canonical_row)?;
    let mut results = Vec::new();
    for r in rows {
        results.push(r?);
    }
    Ok(results)
}

pub fn get_canonical_discover_seeds(conn: &Connection, limit: usize) -> SqlResult<Vec<CanonicalSong>> {
    let mut stmt = conn.prepare(
        "SELECT st.id, st.canonical_key, st.title, st.artist, st.album, st.cover_art_url, st.play_count, st.last_played_at, st.liked, st.local_track_id, t.file_path, st.last_provider_id, st.last_source_id, st.duration_ms
         FROM song_telemetry st
         LEFT JOIN tracks t ON st.local_track_id = t.id
         WHERE st.liked = 1 OR st.play_count >= 2
         ORDER BY RANDOM()
         LIMIT ?1"
    )?;

    let rows = stmt.query_map([limit as i64], map_canonical_row)?;
    let mut results = Vec::new();
    for r in rows {
        results.push(r?);
    }
    Ok(results)
}

pub fn toggle_canonical_like(
    conn: &Connection,
    canonical_key: &str,
    title: Option<&str>,
    artist: Option<&str>,
    album: Option<&str>,
    cover_art_url: Option<&str>,
    provider_id: Option<&str>,
    source_id: Option<&str>,
    local_track_id: Option<i64>,
    duration_ms: Option<u64>,
) -> SqlResult<bool> {
    let exists: bool = conn.query_row(
        "SELECT 1 FROM song_telemetry WHERE canonical_key = ?1",
        [canonical_key],
        |_| Ok(true),
    ).unwrap_or(false);

    if exists {
        conn.execute(
            "UPDATE song_telemetry SET liked = CASE WHEN liked = 1 THEN 0 ELSE 1 END WHERE canonical_key = ?1",
            [canonical_key],
        )?;
    } else {
        let parts: Vec<&str> = canonical_key.split("::").collect();
        let fallback_artist = if parts.len() > 1 { parts[0] } else { "Unknown Artist" };
        let fallback_title = if parts.len() > 1 { parts[1] } else { canonical_key };

        let actual_title = title.unwrap_or(fallback_title);
        let actual_artist = artist.unwrap_or(fallback_artist);
        let actual_provider = provider_id.unwrap_or("local");
        let actual_source = source_id.unwrap_or(canonical_key);

        conn.execute(
            "INSERT INTO song_telemetry (
                canonical_key, title, artist, album, cover_art_url,
                play_count, liked, local_track_id, last_provider_id,
                last_source_id, duration_ms, last_played_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, 0, 1, ?6, ?7, ?8, ?9, CURRENT_TIMESTAMP)",
            rusqlite::params![
                canonical_key,
                actual_title,
                actual_artist,
                album,
                cover_art_url,
                local_track_id,
                actual_provider,
                actual_source,
                duration_ms.map(|d| d as i64),
            ],
        )?;
    }

    let liked: i64 = conn.query_row(
        "SELECT liked FROM song_telemetry WHERE canonical_key = ?1",
        [canonical_key],
        |row| row.get(0),
    ).unwrap_or(0);

    Ok(liked == 1)
}

pub fn get_extension_metrics(conn: &Connection) -> SqlResult<Vec<ExtensionMetric>> {
    let mut stmt = conn.prepare(
        "SELECT provider_id, total_plays, total_duration_ms, total_resolutions, failed_resolutions, last_used_at FROM extension_metrics ORDER BY total_plays DESC"
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(ExtensionMetric {
            provider_id: row.get(0)?,
            total_plays: row.get(1)?,
            total_duration_ms: row.get(2)?,
            total_resolutions: row.get(3)?,
            failed_resolutions: row.get(4)?,
            last_used_at: row.get(5)?,
        })
    })?;

    let mut results = Vec::new();
    for r in rows {
        results.push(r?);
    }
    Ok(results)
}

fn map_canonical_row(row: &rusqlite::Row) -> rusqlite::Result<CanonicalSong> {
    let duration_i64: Option<i64> = row.get(13)?;
    let liked_i64: i64 = row.get(8)?;
    Ok(CanonicalSong {
        id: row.get(0)?,
        canonical_key: row.get(1)?,
        title: row.get(2)?,
        artist: row.get(3)?,
        album: row.get(4)?,
        cover_art_url: row.get(5)?,
        play_count: row.get(6)?,
        last_played_at: row.get(7)?,
        liked: liked_i64 == 1,
        local_track_id: row.get(9)?,
        local_file_path: row.get(10)?,
        last_provider_id: row.get(11)?,
        last_source_id: row.get(12)?,
        duration_ms: duration_i64.map(|d| d as u64),
    })
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonical_telemetry_and_extension_metrics() {
        let conn = Connection::open_in_memory().unwrap();
        init_db_in_memory(&conn).unwrap();

        // 1. Record play from YouTube
        record_playback_event(
            &conn,
            "Never Gonna Give You Up (Official Music Video)",
            "Rick Astley - Topic",
            Some("Whenever You Need Somebody"),
            Some("https://img.youtube.com/art.jpg"),
            "youtube-wasm",
            "dQw4w9WgXcQ",
            Some(213000),
        ).unwrap();

        // 2. Record play of same song from Local/other provider
        record_playback_event(
            &conn,
            "Never Gonna Give You Up [Official Audio]",
            "Rick Astley",
            Some("Whenever You Need Somebody"),
            None,
            "local",
            "/music/rick.mp3",
            Some(213000),
        ).unwrap();

        // 3. Verify deduplicated Quick Picks
        let qp = get_canonical_quick_picks(&conn, 10).unwrap();
        assert_eq!(qp.len(), 1);
        assert_eq!(qp[0].canonical_key, "rick astley::never gonna give you up");
        assert_eq!(qp[0].play_count, 2);

        // 4. Verify Extension Metrics
        let metrics = get_extension_metrics(&conn).unwrap();
        assert_eq!(metrics.len(), 2);
        let yt_m = metrics.iter().find(|m| m.provider_id == "youtube-wasm").unwrap();
        assert_eq!(yt_m.total_plays, 1);
        assert_eq!(yt_m.total_duration_ms, 213000);

        // 5. Test Like Toggle
        let is_liked = toggle_canonical_like(&conn, "rick astley::never gonna give you up", None, None, None, None, None, None, None, None).unwrap();
        assert!(is_liked);
    }

    fn init_db_in_memory(conn: &Connection) -> rusqlite::Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tracks (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                artist TEXT,
                album_id INTEGER,
                track_number INTEGER,
                file_path TEXT UNIQUE NOT NULL
            )",
            [],
        )?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS albums (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                artist TEXT,
                cover_art_path TEXT
            )",
            [],
        )?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS song_telemetry (
                id INTEGER PRIMARY KEY,
                canonical_key TEXT NOT NULL UNIQUE,
                title TEXT NOT NULL,
                artist TEXT NOT NULL,
                album TEXT,
                cover_art_url TEXT,
                play_count INTEGER NOT NULL DEFAULT 1,
                last_played_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                liked INTEGER NOT NULL DEFAULT 0,
                local_track_id INTEGER,
                last_provider_id TEXT NOT NULL DEFAULT 'local',
                last_source_id TEXT NOT NULL,
                duration_ms INTEGER
            )",
            [],
        )?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS playback_events (
                id INTEGER PRIMARY KEY,
                canonical_key TEXT NOT NULL,
                provider_id TEXT NOT NULL,
                source_id TEXT NOT NULL,
                played_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                duration_ms INTEGER
            )",
            [],
        )?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS extension_metrics (
                provider_id TEXT PRIMARY KEY,
                total_plays INTEGER NOT NULL DEFAULT 0,
                total_duration_ms INTEGER NOT NULL DEFAULT 0,
                total_resolutions INTEGER NOT NULL DEFAULT 0,
                failed_resolutions INTEGER NOT NULL DEFAULT 0,
                last_used_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        Ok(())
    }
}
