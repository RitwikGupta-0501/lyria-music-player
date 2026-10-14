use rusqlite::{Connection, Result as SqlResult};

pub fn init_db<P: AsRef<std::path::Path>>(db_path: P) -> SqlResult<Connection> {
    let conn = Connection::open(db_path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS albums (
            id INTEGER PRIMARY KEY,
            title TEXT NOT NULL,
            artist TEXT,
            cover_art_path TEXT,
            UNIQUE(title, artist)
        )",
        [],
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tracks (
            id INTEGER PRIMARY KEY,
            title TEXT NOT NULL,
            artist TEXT,
            album_id INTEGER,
            genre TEXT,
            track_number INTEGER,
            file_path TEXT UNIQUE NOT NULL,
            FOREIGN KEY(album_id) REFERENCES albums(id)
        )",
        [],
    )?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_tracks_album_id ON tracks(album_id)", [])?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS playlists (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS playlist_tracks (
            id INTEGER PRIMARY KEY,
            playlist_id INTEGER,
            track_id INTEGER,
            position INTEGER,
            FOREIGN KEY(playlist_id) REFERENCES playlists(id) ON DELETE CASCADE,
            FOREIGN KEY(track_id) REFERENCES tracks(id) ON DELETE CASCADE
        )",
        [],
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS providers (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            author TEXT NOT NULL,
            version TEXT NOT NULL,
            file_path TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'enabled',
            error_message TEXT,
            checksum TEXT,
            capabilities TEXT,
            homepage TEXT,
            settings_schema TEXT,
            priority INTEGER NOT NULL DEFAULT 0,
            icon TEXT,
            settings TEXT,
            imported_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    // Migration to add settings and description columns if they don't exist
    let _ = conn.execute("ALTER TABLE providers ADD COLUMN settings TEXT", []);
    let _ = conn.execute("ALTER TABLE providers ADD COLUMN description TEXT", []);

    conn.execute(
        "CREATE TABLE IF NOT EXISTS provider_storage (
            provider_id TEXT NOT NULL,
            key TEXT NOT NULL,
            value TEXT NOT NULL,
            PRIMARY KEY(provider_id, key),
            FOREIGN KEY(provider_id) REFERENCES providers(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // Queue persistence tables (Phase 5)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS queue_state (
            id INTEGER PRIMARY KEY,
            current_position INTEGER NOT NULL DEFAULT 0,
            repeat_mode TEXT NOT NULL DEFAULT 'Off',
            queue_mode TEXT NOT NULL DEFAULT 'Normal',
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS queued_tracks (
            id INTEGER PRIMARY KEY,
            queue_state_id INTEGER NOT NULL,
            instance_id TEXT NOT NULL UNIQUE,
            track_id INTEGER NOT NULL DEFAULT -1,
            position INTEGER NOT NULL,
            -- Remote source fields (NULL for local tracks)
            stream_url TEXT,
            provider_id TEXT,
            remote_track_id TEXT,
            quality_hint TEXT,
            cached_title TEXT,
            cached_artist TEXT,
            cover_art_url TEXT,
            duration_ms INTEGER,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(queue_state_id) REFERENCES queue_state(id) ON DELETE CASCADE
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS shuffle_order (
            id INTEGER PRIMARY KEY,
            queue_state_id INTEGER NOT NULL,
            instance_id TEXT NOT NULL,
            position INTEGER NOT NULL,
            seed INTEGER NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(queue_state_id) REFERENCES queue_state(id) ON DELETE CASCADE
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS queue_history (
            id INTEGER PRIMARY KEY,
            action TEXT NOT NULL,
            from_position INTEGER,
            to_position INTEGER,
            track_id INTEGER,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS queue_snapshots (
            id INTEGER PRIMARY KEY,
            queue_state_id INTEGER NOT NULL,
            description TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(queue_state_id) REFERENCES queue_state(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // Migrations for existing DBs
    let _ = conn.execute("ALTER TABLE queued_tracks ADD COLUMN remote_track_id TEXT", []);
    let _ = conn.execute("ALTER TABLE queued_tracks ADD COLUMN duration_ms INTEGER", []);
    
    let _ = conn.execute("DROP TABLE IF EXISTS provider_states", []);
    let _ = conn.execute(
        "CREATE TABLE IF NOT EXISTS providers (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            author TEXT NOT NULL,
            version TEXT NOT NULL,
            file_path TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'enabled',
            error_message TEXT,
            checksum TEXT,
            capabilities TEXT,
            homepage TEXT,
            settings_schema TEXT,
            priority INTEGER NOT NULL DEFAULT 0,
            icon TEXT,
            imported_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    );


    // Universal Multi-Source Telemetry & Personalization tables
    conn.execute(
        "CREATE TABLE IF NOT EXISTS song_telemetry (
            id INTEGER PRIMARY KEY,
            canonical_key TEXT NOT NULL UNIQUE,
            title TEXT NOT NULL,
            artist TEXT NOT NULL,
            album TEXT,
            genre TEXT,
            cover_art_url TEXT,
            play_count INTEGER NOT NULL DEFAULT 1,
            last_played_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            liked INTEGER NOT NULL DEFAULT 0,
            local_track_id INTEGER,
            last_provider_id TEXT NOT NULL DEFAULT 'local',
            last_source_id TEXT NOT NULL,
            duration_ms INTEGER,
            FOREIGN KEY(local_track_id) REFERENCES tracks(id) ON DELETE SET NULL
        )",
        [],
    )?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_song_telemetry_rank ON song_telemetry(play_count DESC, last_played_at DESC)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_song_telemetry_last_played ON song_telemetry(last_played_at DESC)", [])?;

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
    conn.execute("CREATE INDEX IF NOT EXISTS idx_playback_events_played_at ON playback_events(played_at DESC)", [])?;

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

    
    // Recommendation Cache (Granular Per-Seed Keying)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS recommendation_cache (
            seed_canonical_key TEXT NOT NULL,
            provider_id TEXT NOT NULL,
            shelf_type TEXT NOT NULL,
            payload_json TEXT NOT NULL,
            fetched_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            ttl_seconds INTEGER NOT NULL DEFAULT 21600,
            PRIMARY KEY(seed_canonical_key, provider_id, shelf_type)
        )",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_rec_cache_lookup 
         ON recommendation_cache(seed_canonical_key, provider_id, shelf_type, fetched_at)",
        [],
    )?;

    // User-Defined Deduplication Overrides
    conn.execute(
        "CREATE TABLE IF NOT EXISTS dedup_overrides (
            canonical_key_a TEXT NOT NULL,
            canonical_key_b TEXT NOT NULL,
            should_merge INTEGER NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY(canonical_key_a, canonical_key_b)
        )",
        [],
    )?;

    // Extension Metrics Migrations
    let _ = conn.execute("ALTER TABLE extension_metrics ADD COLUMN consecutive_failures INTEGER NOT NULL DEFAULT 0", []);
    let _ = conn.execute("ALTER TABLE extension_metrics ADD COLUMN backoff_until TIMESTAMP", []);


    // Explore Feed Cache (24-Hour Editorial TTL)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS feed_cache (
            provider_id TEXT NOT NULL,
            module_id TEXT NOT NULL,
            payload_json TEXT NOT NULL,
            fetched_at INTEGER NOT NULL,
            ttl_seconds INTEGER NOT NULL DEFAULT 86400,
            PRIMARY KEY(provider_id, module_id)
        )",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_feed_cache_lookup 
         ON feed_cache(provider_id, module_id, fetched_at)",
        [],
    )?;

    // Saved Remote & Local Albums
    conn.execute(
        "CREATE TABLE IF NOT EXISTS saved_albums (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            artist TEXT,
            cover_art_url TEXT,
            provider_id TEXT NOT NULL DEFAULT 'local',
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    // Saved Remote & Local Playlists
    conn.execute(
        "CREATE TABLE IF NOT EXISTS saved_playlists (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            author TEXT,
            cover_art_url TEXT,
            provider_id TEXT NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    // Cached Artist Metadata & Avatars
    conn.execute(
        "CREATE TABLE IF NOT EXISTS artist_metadata (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            avatar_url TEXT,
            bio TEXT,
            provider_id TEXT NOT NULL DEFAULT 'youtube-wasm',
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_artist_metadata_name ON artist_metadata(name)",
        [],
    )?;

    // Feature Flags & Runtime Experiments
    conn.execute(
        "CREATE TABLE IF NOT EXISTS feature_flags (
            key TEXT PRIMARY KEY,
            enabled INTEGER NOT NULL,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    // Downloads & Offline Manager (Phase 2)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS downloads (
            id TEXT PRIMARY KEY,
            entity_type TEXT NOT NULL,
            entity_id TEXT,
            title TEXT NOT NULL,
            artist TEXT NOT NULL,
            album TEXT,
            total_items INTEGER NOT NULL DEFAULT 1,
            completed_items INTEGER NOT NULL DEFAULT 0,
            status TEXT NOT NULL DEFAULT 'queued',
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_downloads_status ON downloads(status)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_downloads_created_at ON downloads(created_at DESC)", [])?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS download_items (
            id TEXT PRIMARY KEY,
            download_id TEXT NOT NULL,
            track_title TEXT NOT NULL,
            artist TEXT NOT NULL,
            album TEXT,
            track_number INTEGER,
            provider_id TEXT,
            download_url TEXT,
            dest_file_path TEXT,
            temp_file_path TEXT,
            total_bytes INTEGER DEFAULT 0,
            downloaded_bytes INTEGER DEFAULT 0,
            status TEXT NOT NULL DEFAULT 'queued',
            error_message TEXT,
            retry_count INTEGER DEFAULT 0,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(download_id) REFERENCES downloads(id) ON DELETE CASCADE
        )",
        [],
    )?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_download_items_download_id ON download_items(download_id)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_download_items_status ON download_items(status)", [])?;
    // Lyrics Cache for Sandboxed Extensions and Offline Playback
    conn.execute(
        "CREATE TABLE IF NOT EXISTS lyrics_cache (
            canonical_key TEXT NOT NULL,
            provider_id TEXT NOT NULL,
            format TEXT NOT NULL,
            lyrics_json TEXT NOT NULL,
            instrumental INTEGER NOT NULL DEFAULT 0,
            is_manual INTEGER NOT NULL DEFAULT 0,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY(canonical_key, provider_id)
        )",
        [],
    )?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_lyrics_cache_canonical ON lyrics_cache(canonical_key)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_lyrics_cache_provider ON lyrics_cache(provider_id)", [])?;

    // Foreign Key Cascade Optimization Indexes
    conn.execute("CREATE INDEX IF NOT EXISTS idx_playlist_tracks_playlist ON playlist_tracks(playlist_id)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_playlist_tracks_track ON playlist_tracks(track_id)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_queued_tracks_qsid ON queued_tracks(queue_state_id)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_shuffle_order_qsid ON shuffle_order(queue_state_id)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_queue_snapshots_qsid ON queue_snapshots(queue_state_id)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_song_telemetry_track ON song_telemetry(local_track_id)", [])?;

    // Migrations
    let _ = conn.execute("ALTER TABLE tracks ADD COLUMN genre TEXT", []);
    let _ = conn.execute("ALTER TABLE song_telemetry ADD COLUMN genre TEXT", []);
    let _ = conn.execute("ALTER TABLE lyrics_cache ADD COLUMN is_manual INTEGER NOT NULL DEFAULT 0", []);
    let _ = conn.execute("ALTER TABLE download_items ADD COLUMN native_id TEXT", []);

    // Consolidate "Unknown Album" entries into a single canonical ("Unknown Album", "Various Artists")
    let _ = (|| -> SqlResult<()> {
        conn.execute(
            "INSERT OR IGNORE INTO albums (title, artist) VALUES ('Unknown Album', 'Various Artists')",
            [],
        )?;
        let canonical_id: Option<i64> = conn.query_row(
            "SELECT id FROM albums WHERE title = 'Unknown Album' AND artist = 'Various Artists' LIMIT 1",
            [],
            |row| row.get(0),
        ).ok();
        if let Some(canonical_id) = canonical_id {
            let _ = conn.execute(
                "UPDATE tracks SET album_id = ?1 WHERE album_id IN (
                    SELECT id FROM albums WHERE title = 'Unknown Album' AND id != ?1
                ) OR album_id IS NULL",
                [canonical_id],
            );
            let _ = conn.execute(
                "DELETE FROM albums WHERE title = 'Unknown Album' AND id != ?1",
                [canonical_id],
            );
        }
        Ok(())
    })();

    Ok(conn)
}
