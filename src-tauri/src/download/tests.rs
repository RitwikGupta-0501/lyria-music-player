use std::sync::Arc;
use std::time::Duration;

use crate::download::models::*;
use crate::download::resolver::{CircuitBreaker, DownloadResolver};

#[test]
fn test_download_models_serialization_roundtrip() {
    let query = DownloadQueryInput {
        title: "Starboy".to_string(),
        artist: "The Weeknd".to_string(),
        album: Some("Starboy".to_string()),
        duration_ms: Some(230000),
        isrc: Some("USUG11600863".to_string()),
        native_id: Some("track_123".to_string()),
        quality_preference: DownloadQualityPreference::Lossless,
    };

    let serialized = serde_json::to_string(&query).expect("serialize query");
    let deserialized: DownloadQueryInput =
        serde_json::from_str(&serialized).expect("deserialize query");
    assert_eq!(query, deserialized);
    assert_eq!(
        deserialized.quality_preference,
        DownloadQualityPreference::Lossless
    );

    let result = DownloadSourceResult {
        download_url: "https://cdn.example.com/audio/starboy.flac".to_string(),
        codec: AudioCodec::Flac,
        bitrate_kbps: Some(1411),
        sample_rate_hz: Some(44100),
        bit_depth: Some(16),
        content_length_bytes: Some(40500000),
        headers: Some(
            [("Authorization".to_string(), "Bearer token123".to_string())]
                .into_iter()
                .collect(),
        ),
        cover_art_url: Some("https://cdn.example.com/art/starboy.jpg".to_string()),
        lyrics_lrc: Some("[00:01.00] I'm tryna put you in the worst mood...".to_string()),
    };

    let res_json = serde_json::to_string(&result).expect("serialize result");
    let res_de: DownloadSourceResult =
        serde_json::from_str(&res_json).expect("deserialize result");
    assert_eq!(result, res_de);
    assert_eq!(res_de.codec.extension(), "flac");
}

#[test]
fn test_codec_file_extensions() {
    assert_eq!(AudioCodec::Flac.extension(), "flac");
    assert_eq!(AudioCodec::Opus.extension(), "opus");
    assert_eq!(AudioCodec::Aac.extension(), "aac");
    assert_eq!(AudioCodec::Mp3.extension(), "mp3");
    assert_eq!(AudioCodec::Vorbis.extension(), "ogg");
    assert_eq!(AudioCodec::Wav.extension(), "wav");
    assert_eq!(AudioCodec::M4a.extension(), "m4a");
    assert_eq!(AudioCodec::Other("alac".to_string()).extension(), "alac");
}

#[test]
fn test_candidate_sorting_with_and_without_preferred() {
    let cb = Arc::new(CircuitBreaker::default());
    let resolver = DownloadResolver::new(cb);

    let candidates = vec![
        ProviderCandidate {
            id: "provider_low".to_string(),
            name: "Low Priority".to_string(),
            priority: 1,
            is_preferred: false,
        },
        ProviderCandidate {
            id: "provider_high".to_string(),
            name: "High Priority".to_string(),
            priority: 10,
            is_preferred: false,
        },
        ProviderCandidate {
            id: "provider_mid".to_string(),
            name: "Mid Priority".to_string(),
            priority: 5,
            is_preferred: false,
        },
    ];

    // 1. Without preferred provider: sorted by priority descending
    let sorted_no_pref = resolver.sort_candidates(candidates.clone(), None);
    assert_eq!(sorted_no_pref[0].id, "provider_high");
    assert_eq!(sorted_no_pref[1].id, "provider_mid");
    assert_eq!(sorted_no_pref[2].id, "provider_low");

    // 2. With preferred provider: preferred is always first
    let sorted_pref = resolver.sort_candidates(candidates, Some("provider_mid"));
    assert_eq!(sorted_pref[0].id, "provider_mid");
    assert!(sorted_pref[0].is_preferred);
    assert_eq!(sorted_pref[1].id, "provider_high");
    assert_eq!(sorted_pref[2].id, "provider_low");
}

#[tokio::test]
async fn test_resolver_fallback_to_secondary_when_primary_fails() {
    let cb = Arc::new(CircuitBreaker::default());
    let resolver = DownloadResolver::new(cb);

    let candidates = vec![
        ProviderCandidate {
            id: "flac_vault".to_string(),
            name: "FLAC Vault".to_string(),
            priority: 10,
            is_preferred: false,
        },
        ProviderCandidate {
            id: "backup_archive".to_string(),
            name: "Backup Archive".to_string(),
            priority: 5,
            is_preferred: false,
        },
    ];

    let query = DownloadQueryInput {
        title: "Blinding Lights".to_string(),
        artist: "The Weeknd".to_string(),
        album: None,
        duration_ms: None,
        isrc: None,
        native_id: None,
        quality_preference: DownloadQualityPreference::High,
    };

    // Primary provider "flac_vault" fails, secondary "backup_archive" succeeds
    let (winner_id, source) = resolver
        .resolve_with_fallback(
            candidates,
            Some("flac_vault"),
            &query,
            |pid, _| async move {
                if pid == "flac_vault" {
                    Err("Track not available on FLAC Vault".to_string())
                } else if pid == "backup_archive" {
                    Ok(DownloadSourceResult {
                        download_url: "https://archive.org/audio/blinding_lights.mp3".to_string(),
                        codec: AudioCodec::Mp3,
                        bitrate_kbps: Some(320),
                        sample_rate_hz: Some(44100),
                        bit_depth: None,
                        content_length_bytes: Some(8500000),
                        headers: None,
                        cover_art_url: None,
                        lyrics_lrc: None,
                    })
                } else {
                    Err("Unknown provider".to_string())
                }
            },
        )
        .await
        .expect("fallback should succeed");

    assert_eq!(winner_id, "backup_archive");
    assert_eq!(source.codec, AudioCodec::Mp3);
    assert_eq!(source.bitrate_kbps, Some(320));
}

#[tokio::test]
async fn test_resolver_all_providers_fail() {
    let cb = Arc::new(CircuitBreaker::default());
    let resolver = DownloadResolver::new(cb);

    let candidates = vec![
        ProviderCandidate {
            id: "p1".to_string(),
            name: "Provider 1".to_string(),
            priority: 1,
            is_preferred: false,
        },
        ProviderCandidate {
            id: "p2".to_string(),
            name: "Provider 2".to_string(),
            priority: 2,
            is_preferred: false,
        },
    ];

    let query = DownloadQueryInput {
        title: "Song".to_string(),
        artist: "Artist".to_string(),
        album: None,
        duration_ms: None,
        isrc: None,
        native_id: None,
        quality_preference: DownloadQualityPreference::Standard,
    };

    let result = resolver
        .resolve_with_fallback(candidates, None, &query, |_pid, _| async move {
            Err("Rate limit exceeded".to_string())
        })
        .await;

    match result {
        Err(DownloadResolutionError::AllProvidersFailed { attempts }) => {
            assert_eq!(attempts.len(), 2);
            assert_eq!(attempts[0].message, "Rate limit exceeded");
            assert_eq!(attempts[1].message, "Rate limit exceeded");
        }
        other => panic!("Expected AllProvidersFailed, got {:?}", other),
    }
}

#[tokio::test]
async fn test_circuit_breaker_trips_and_bypasses_provider() {
    let cb = Arc::new(CircuitBreaker::new(2, Duration::from_secs(60)));
    let resolver = DownloadResolver::new(cb.clone());

    // Record 2 failures for p1 -> should trip breaker
    cb.record_failure("p1");
    assert!(cb.is_available("p1")); // 1 failure, threshold is 2
    cb.record_failure("p1");
    assert!(!cb.is_available("p1")); // 2 failures, tripped!

    let candidates = vec![
        ProviderCandidate {
            id: "p1".to_string(),
            name: "Provider 1".to_string(),
            priority: 10,
            is_preferred: true,
        },
        ProviderCandidate {
            id: "p2".to_string(),
            name: "Provider 2".to_string(),
            priority: 5,
            is_preferred: false,
        },
    ];

    let query = DownloadQueryInput {
        title: "Song".to_string(),
        artist: "Artist".to_string(),
        album: None,
        duration_ms: None,
        isrc: None,
        native_id: None,
        quality_preference: DownloadQualityPreference::High,
    };

    // p1 should be skipped immediately without invoking its closure!
    let (winner, _) = resolver
        .resolve_with_fallback(candidates, Some("p1"), &query, |pid, _| async move {
            if pid == "p1" {
                panic!("Tripped provider p1 should never be invoked!");
            }
            Ok(DownloadSourceResult {
                download_url: "https://example.com/audio.opus".to_string(),
                codec: AudioCodec::Opus,
                bitrate_kbps: Some(256),
                sample_rate_hz: Some(48000),
                bit_depth: None,
                content_length_bytes: None,
                headers: None,
                cover_art_url: None,
                lyrics_lrc: None,
            })
        })
        .await
        .expect("should succeed with p2");

    assert_eq!(winner, "p2");
}

#[tokio::test]
async fn test_empty_candidates_returns_no_providers() {
    let cb = Arc::new(CircuitBreaker::default());
    let resolver = DownloadResolver::new(cb);

    let query = DownloadQueryInput {
        title: "Song".to_string(),
        artist: "Artist".to_string(),
        album: None,
        duration_ms: None,
        isrc: None,
        native_id: None,
        quality_preference: DownloadQualityPreference::High,
    };

    let result = resolver
        .resolve_with_fallback(vec![], None, &query, |_pid, _| async move {
            Ok(DownloadSourceResult {
                download_url: "url".to_string(),
                codec: AudioCodec::Mp3,
                bitrate_kbps: None,
                sample_rate_hz: None,
                bit_depth: None,
                content_length_bytes: None,
                headers: None,
                cover_art_url: None,
                lyrics_lrc: None,
            })
        })
        .await;

    match result {
        Err(DownloadResolutionError::NoDownloadProvidersAvailable) => (),
        other => panic!("Expected NoDownloadProvidersAvailable, got {:?}", other),
    }
}

#[test]
fn test_download_db_enqueue_and_progress_lifecycle() {
    let temp_db = tempfile::NamedTempFile::new().expect("temp file");
    let mut conn = crate::db::schema::init_db(temp_db.path()).expect("init db");

    let download_id = "job-album-123".to_string();
    let download = crate::db::queries::DownloadRecord {
        id: download_id.clone(),
        entity_type: "album".to_string(),
        entity_id: Some("album-yt-456".to_string()),
        title: "Random Access Memories".to_string(),
        artist: "Daft Punk".to_string(),
        album: Some("Random Access Memories".to_string()),
        total_items: 2,
        completed_items: 0,
        status: "queued".to_string(),
        created_at: None,
        updated_at: None,
    };

    let items = vec![
        crate::db::queries::DownloadItemRecord {
            id: "item-1".to_string(),
            download_id: download_id.clone(),
            track_title: "Give Life Back to Music".to_string(),
            artist: "Daft Punk".to_string(),
            album: Some("Random Access Memories".to_string()),
            track_number: Some(1),
            provider_id: Some("flac_downloader".to_string()),
            download_url: Some("https://example.com/track1.flac".to_string()),
            dest_file_path: None,
            temp_file_path: Some("/tmp/track1.flac.part".to_string()),
            total_bytes: 45000000,
            downloaded_bytes: 0,
            status: "queued".to_string(),
            error_message: None,
            retry_count: 0,
            created_at: None,
            updated_at: None,
        },
        crate::db::queries::DownloadItemRecord {
            id: "item-2".to_string(),
            download_id: download_id.clone(),
            track_title: "Get Lucky".to_string(),
            artist: "Daft Punk".to_string(),
            album: Some("Random Access Memories".to_string()),
            track_number: Some(2),
            provider_id: Some("flac_downloader".to_string()),
            download_url: Some("https://example.com/track2.flac".to_string()),
            dest_file_path: None,
            temp_file_path: Some("/tmp/track2.flac.part".to_string()),
            total_bytes: 38000000,
            downloaded_bytes: 0,
            status: "queued".to_string(),
            error_message: None,
            retry_count: 0,
            created_at: None,
            updated_at: None,
        },
    ];

    // 1. Enqueue job and items
    crate::db::queries::enqueue_download_job(&mut conn, &download, &items)
        .expect("enqueue download job");

    // 2. Query queue
    let queue = crate::db::queries::get_download_queue(&conn).expect("get download queue");
    assert_eq!(queue.len(), 1);
    assert_eq!(queue[0].download.title, "Random Access Memories");
    assert_eq!(queue[0].items.len(), 2);

    // 3. Update progress for item 1
    crate::db::queries::update_download_item_progress(&conn, "item-1", 15000000, 45000000)
        .expect("update progress");

    let active_items = crate::db::queries::get_active_download_items(&conn).expect("get active");
    assert_eq!(active_items.len(), 2);
    let item1 = active_items.iter().find(|i| i.id == "item-1").unwrap();
    assert_eq!(item1.downloaded_bytes, 15000000);

    // 4. Complete item 1 and ingest into library
    let track_data1 = crate::db::TrackData {
        title: "Give Life Back to Music".to_string(),
        artist: Some("Daft Punk".to_string()),
        album: Some("Random Access Memories".to_string()),
        track_number: Some(1),
        file_path: "/music/Daft Punk/Random Access Memories/01 - Give Life Back to Music.flac".to_string(),
    };

    let track_id1 = crate::db::queries::complete_download_item(
        &mut conn,
        "item-1",
        &track_data1.file_path,
        &track_data1,
    )
    .expect("complete item 1");
    assert!(track_id1 > 0);

    // Verify parent download status is updated to 'downloading' with completed_items == 1
    let queue = crate::db::queries::get_download_queue(&conn).expect("get queue");
    assert_eq!(queue[0].download.completed_items, 1);
    assert_eq!(queue[0].download.status, "downloading");

    // Verify local tracks library contains the track
    let local_tracks = crate::db::queries::get_local_tracks(&conn, 10, 0).expect("get local tracks");
    assert_eq!(local_tracks.len(), 1);
    assert_eq!(local_tracks[0].title, "Give Life Back to Music");

    // 5. Complete item 2 -> parent download should transition to 'completed'
    let track_data2 = crate::db::TrackData {
        title: "Get Lucky".to_string(),
        artist: Some("Daft Punk".to_string()),
        album: Some("Random Access Memories".to_string()),
        track_number: Some(2),
        file_path: "/music/Daft Punk/Random Access Memories/02 - Get Lucky.flac".to_string(),
    };

    let track_id2 = crate::db::queries::complete_download_item(
        &mut conn,
        "item-2",
        &track_data2.file_path,
        &track_data2,
    )
    .expect("complete item 2");
    assert!(track_id2 > track_id1);

    let queue = crate::db::queries::get_download_queue(&conn).expect("get queue");
    assert_eq!(queue[0].download.completed_items, 2);
    assert_eq!(queue[0].download.status, "completed");

    // 6. Delete job should cascade delete items
    crate::db::queries::delete_download_job(&conn, &download_id).expect("delete job");
    let queue_after = crate::db::queries::get_download_queue(&conn).expect("get queue after");
    assert!(queue_after.is_empty());
}

