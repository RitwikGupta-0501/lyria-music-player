//! Adversarial and fault-tolerance test suite for Echo recommendation engine.

use std::collections::HashMap;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

use crate::db::canonical::{
    compute_dedup_confidence, make_canonical_key, CandidateTrack,
};
use crate::providers::recommendations::ProviderErrorType;

#[tokio::test]
async fn test_order_invariant_user_override() {
    let mut overrides = HashMap::new();
    let track_a = CandidateTrack {
        canonical_key: "artist_a::song_one".into(),
        title: "Song One".into(),
        artist: "Artist A".into(),
        album: None,
        isrc: None,
        duration_ms: Some(200_000),
    };
    let track_b = CandidateTrack {
        canonical_key: "artist_b::completely_different".into(),
        title: "Completely Different".into(),
        artist: "Artist B".into(),
        album: None,
        isrc: None,
        duration_ms: Some(150_000),
    };

    // Forward override key
    let pair1 = crate::db::canonical::canonical_override_key(&track_a.canonical_key, &track_b.canonical_key);
    overrides.insert(pair1, true);

    // Assert (track_a, track_b) matches
    let res1 = compute_dedup_confidence(&track_a, &track_b, &overrides);
    assert!(res1.is_match);
    assert_eq!(res1.confidence, 1.0);
    assert_eq!(res1.reason, "user_override");

    // Assert reverse order (track_b, track_a) matches identically
    let res2 = compute_dedup_confidence(&track_b, &track_a, &overrides);
    assert!(res2.is_match);
    assert_eq!(res2.confidence, 1.0);
    assert_eq!(res2.reason, "user_override");
}

#[tokio::test]
async fn test_canonical_key_cache_collision() {
    // Variations of Cafe with diacritics and noise
    let title_1 = "Café Del Mar (Official Audio)";
    let artist_1 = "Energy 52";

    let title_2 = "Cafe Del Mar [Lyrics]";
    let artist_2 = "Energy 52";

    let key_1 = make_canonical_key(title_1, artist_1);
    let key_2 = make_canonical_key(title_2, artist_2);

    assert_eq!(key_1, key_2, "Canonical keys must match exactly across diacritics and tags");
}

#[tokio::test]
async fn test_continuous_gaussian_duration_scoring() {
    let base = CandidateTrack {
        canonical_key: "artist::track".into(),
        title: "Track Name".into(),
        artist: "Artist Name".into(),
        album: None,
        isrc: None,
        duration_ms: Some(200_000),
    };

    let empty_overrides = HashMap::new();

    // 1. Identical duration: delta = 0 -> penalty = 1.0
    let same_dur = CandidateTrack {
        duration_ms: Some(200_000),
        ..base.clone()
    };
    let score_same = compute_dedup_confidence(&base, &same_dur, &empty_overrides);
    assert!(score_same.is_match);
    assert!((score_same.confidence - 1.0).abs() < 0.01);

    // 2. 5s difference: delta = 5000ms -> penalty = exp(-0.5 * 0.25) ≈ 0.88
    let slight_diff = CandidateTrack {
        duration_ms: Some(205_000),
        ..base.clone()
    };
    let score_slight = compute_dedup_confidence(&base, &slight_diff, &empty_overrides);
    assert!(score_slight.is_match);
    assert!(score_slight.confidence > 0.85);

    // 3. 60s difference: delta = 60000ms -> penalty = exp(-0.5 * 36) ≈ 0.000000015
    let huge_diff = CandidateTrack {
        duration_ms: Some(260_000),
        ..base.clone()
    };
    let score_huge = compute_dedup_confidence(&base, &huge_diff, &empty_overrides);
    assert!(!score_huge.is_match);
    assert!(score_huge.confidence < 0.85);
}

#[tokio::test]
async fn test_cancellation_token_behavior() {
    let token = CancellationToken::new();
    let token_clone = token.clone();

    let task = tokio::spawn(async move {
        tokio::select! {
            _ = token_clone.cancelled() => Err("cancelled"),
            _ = tokio::time::sleep(Duration::from_secs(5)) => Ok("done"),
        }
    });

    // Cancel token immediately
    token.cancel();

    let res = task.await.expect("task join failed");
    assert_eq!(res, Err("cancelled"));
}

#[tokio::test]
async fn test_circuit_breaker_error_types() {
    let err_429 = ProviderErrorType::Http429;
    let err_timeout = ProviderErrorType::Timeout;
    let err_panic = ProviderErrorType::WasmPanic("memory bounds exceeded".into());

    assert!(format!("{:?}", err_429).contains("Http429"));
    assert!(format!("{:?}", err_timeout).contains("Timeout"));
    assert!(format!("{:?}", err_panic).contains("WasmPanic"));
}

