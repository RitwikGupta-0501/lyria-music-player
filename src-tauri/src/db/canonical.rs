//! Canonical song normalization and deduplication engine for Echo.
//! Normalizes titles and artists across local files and remote streaming providers
//! to form a deterministic fingerprint (canonical_key), and provides multi-factor
//! continuous Gaussian confidence scoring for track deduplication.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use unicode_normalization::UnicodeNormalization;
use crate::queue::TrackSourceInfo;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CandidateTrack {
    pub canonical_key: String,
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub isrc: Option<String>,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FederatedTrack {
    pub canonical_key: String,
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub isrc: Option<String>,
    pub cover_art_url: Option<String>,
    pub duration_ms: Option<u64>,
    pub play_count: u64,
    pub seed_provenance: Option<String>,
    pub sources: Vec<TrackSourceInfo>,
    pub liked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DedupMatchResult {
    pub is_match: bool,
    pub confidence: f32,
    pub reason: &'static str,
}

/// Strips diacritics and accents from a string after NFD decomposition, then normalizes via NFKC.
fn strip_diacritics(s: &str) -> String {
    s.nfd()
        .filter(|c| !unicode_normalization::char::is_combining_mark(*c))
        .nfkc()
        .collect()
}

pub fn clean_title(title: &str) -> String {
    // 1. Unicode NFKC normalization + diacritic stripping
    let mut s = strip_diacritics(title).nfkc().collect::<String>();

    // 2. Lowercase for case-insensitive normalization
    s = s.to_lowercase();

    // 3. Remove common YouTube/streaming noise patterns in parentheses or brackets
    let noise_patterns = [
        "(official video)",
        "[official video]",
        "(official music video)",
        "[official music video]",
        "(official audio)",
        "[official audio]",
        "(official visualizer)",
        "[official visualizer]",
        "(lyrics)",
        "[lyrics]",
        "(lyric video)",
        "[lyric video]",
        "(audio)",
        "[audio]",
        "(video)",
        "[video]",
        "(visualizer)",
        "[visualizer]",
        "(official hd video)",
        "(hd)",
        "[hd]",
        "(4k)",
        "[4k]",
        "(hq)",
        "[hq]",
        "(explicit)",
        "[explicit]",
        "(clean)",
        "[clean]",
    ];

    for pat in &noise_patterns {
        s = s.replace(pat, "");
    }

    // 4. Remove "(feat. ...)" or "[feat. ...]" or "(ft. ...)"
    s = remove_parenthesized_feat(&s);

    // 5. Remove remastered annotations e.g. "(remastered 2021)" or "- remastered"
    s = remove_remastered_tags(&s);

    // 6. Clean up special characters, punctuation, and extra whitespace
    s = s.replace(&['(', ')', '[', ']', '{', '}', '"', '\''][..], " ");
    s = s.split_whitespace().collect::<Vec<&str>>().join(" ");

    s.trim().to_string()
}

pub fn clean_artist_for_display(raw: &str) -> String {
    let s = raw.trim();
    if s.is_empty() {
        return "Unknown Artist".to_string();
    }

    // Split by bullet '•' or '|' or '·' if present
    for delimiter in ['•', '|', '·'] {
        if s.contains(delimiter) {
            let parts: Vec<&str> = s.split(delimiter).collect();
            let noise_words = ["view", "views", "video", "audio", "subscribers", "subscriber", "mins", "min", "sec", "hours", "official", "lyrics", "hd", "4k"];
            for part in &parts {
                let p_trimmed = part.trim();
                let p_lower = p_trimmed.to_lowercase();
                let is_timestamp = p_trimmed.contains(':') && p_trimmed.chars().all(|c| c.is_ascii_digit() || c == ':');
                let is_noise = noise_words.iter().any(|k| p_lower.contains(k)) || is_timestamp;
                if !is_noise && p_trimmed.len() > 1 {
                    return clean_artist_for_display(p_trimmed);
                }
            }
        }
    }

    let mut result = s.to_string();
    let lower = result.to_lowercase();
    if lower.ends_with(" - topic") {
        result = result[..result.len() - 8].trim().to_string();
    } else if lower.ends_with(" vevo") {
        result = result[..result.len() - 5].trim().to_string();
    } else if lower.ends_with("vevo") {
        result = result[..result.len() - 4].trim().to_string();
    }

    if result.to_lowercase().starts_with("video - ") || result.to_lowercase().starts_with("video • ") {
        result = result[8..].trim().to_string();
    }

    result
}

pub fn split_artist_names(raw: &str) -> Vec<String> {
    let clean = clean_artist_for_display(raw);
    if clean.is_empty() || clean.eq_ignore_ascii_case("unknown artist") {
        return Vec::new();
    }

    // Split on common collaboration and feature delimiters
    let normalized = clean
        .replace(" feat. ", " ; ")
        .replace(" feat ", " ; ")
        .replace(" ft. ", " ; ")
        .replace(" ft ", " ; ")
        .replace(" with ", " ; ")
        .replace(" & ", " ; ")
        .replace(" / ", " ; ")
        .replace(", ", " ; ");

    let mut names = Vec::new();
    for part in normalized.split(';') {
        let trimmed = part.trim();
        if trimmed.len() > 1 && !trimmed.eq_ignore_ascii_case("the") && !trimmed.eq_ignore_ascii_case("unknown") {
            names.push(trimmed.to_string());
        }
    }

    if names.is_empty() {
        names.push(clean);
    }
    names
}

pub fn clean_album_title_for_display(raw: &str) -> Option<String> {
    let mut s = raw.trim().to_string();
    if s.is_empty() {
        return None;
    }

    let alpha_count = s.chars().filter(|c| c.is_alphanumeric()).count();
    if alpha_count < 2 {
        return None;
    }
    if s.eq_ignore_ascii_case("unknown") || s.eq_ignore_ascii_case("unknown album") || s == "&" || s == "/" || s == "•" {
        return None;
    }

    let noise_suffixes = [
        "(original motion picture soundtrack)",
        "[original motion picture soundtrack]",
        "(original soundtrack)",
        "[original soundtrack]",
        "(deluxe edition)",
        "[deluxe edition]",
        "(deluxe version)",
        "[deluxe version]",
        "(expanded edition)",
        "[expanded edition]",
        "(special edition)",
        "[special edition]",
        "(bonus track edition)",
        "[bonus track edition]",
        "(remastered)",
        "[remastered]",
        "(anniversary edition)",
        "[anniversary edition]",
    ];

    for noise in &noise_suffixes {
        if s.to_lowercase().contains(noise) {
            let lower = s.to_lowercase();
            if let Some(pos) = lower.find(noise) {
                s = s[..pos].trim().to_string();
            }
        }
    }

    s = s.trim_end_matches(&['-', '–', '—', '/', ':', '|'][..]).trim().to_string();

    if s.chars().filter(|c| c.is_alphanumeric()).count() < 2 {
        None
    } else {
        Some(s)
    }
}

pub fn clean_artist(artist: &str) -> String {
    // Clean raw display name / remove video bullet metadata first
    let display_cleaned = clean_artist_for_display(artist);

    // 1. Unicode NFKC normalization + diacritic stripping
    let mut s = strip_diacritics(&display_cleaned).nfkc().collect::<String>();

    // 2. Lowercase
    s = s.to_lowercase();

    // 3. Remove YouTube auto-generated channel suffix e.g. "Rick Astley - Topic" -> "Rick Astley"
    if s.ends_with(" - topic") {
        s = s.trim_end_matches(" - topic").to_string();
    }
    if s.ends_with(" vevo") {
        s = s.trim_end_matches(" vevo").to_string();
    }
    if s.ends_with("vevo") {
        s = s.trim_end_matches("vevo").to_string();
    }

    // 4. Remove "feat. ..." or "ft. ..."
    if let Some(idx) = s.find(" feat.") {
        s = s[..idx].to_string();
    } else if let Some(idx) = s.find(" ft.") {
        s = s[..idx].to_string();
    } else if let Some(idx) = s.find(" feat ") {
        s = s[..idx].to_string();
    } else if let Some(idx) = s.find(" ft ") {
        s = s[..idx].to_string();
    }

    // 5. Clean punctuation and whitespace
    s = s.replace(&['"', '\'', ',', '&'][..], " ");
    s = s.split_whitespace().collect::<Vec<&str>>().join(" ");

    s.trim().to_string()
}

pub fn make_canonical_key(title: &str, artist: &str) -> String {
    let a = clean_artist(artist);
    let t = clean_title(title);
    format!("{}::{}", a, t)
}

/// Generates an order-invariant lexicographical tuple key for user overrides.
pub fn canonical_override_key(key_a: &str, key_b: &str) -> (String, String) {
    if key_a <= key_b {
        (key_a.to_string(), key_b.to_string())
    } else {
        (key_b.to_string(), key_a.to_string())
    }
}

/// Multi-factor confidence scoring engine for deduplicating tracks across heterogeneous providers.
pub fn compute_dedup_confidence(
    track_a: &CandidateTrack,
    track_b: &CandidateTrack,
    overrides: &HashMap<(String, String), bool>,
) -> DedupMatchResult {
    // 0. Check Order-Invariant User Overrides
    let pair = canonical_override_key(&track_a.canonical_key, &track_b.canonical_key);
    if let Some(&forced) = overrides.get(&pair) {
        return DedupMatchResult {
            is_match: forced,
            confidence: if forced { 1.0 } else { 0.0 },
            reason: "user_override",
        };
    }

    // 1. ISRC Exact Match Short-Circuit
    if let (Some(isrc_a), Some(isrc_b)) = (&track_a.isrc, &track_b.isrc) {
        if !isrc_a.is_empty() && !isrc_b.is_empty() && isrc_a.eq_ignore_ascii_case(isrc_b) {
            return DedupMatchResult {
                is_match: true,
                confidence: 1.0,
                reason: "isrc_match",
            };
        }
    }

    // 2. Continuous Gaussian Multi-Attribute Scoring
    let title_a = clean_title(&track_a.title);
    let title_b = clean_title(&track_b.title);
    let title_sim = strsim::sorensen_dice(&title_a, &title_b) as f32;

    let artist_a = clean_artist(&track_a.artist);
    let artist_b = clean_artist(&track_b.artist);
    let artist_sim = strsim::sorensen_dice(&artist_a, &artist_b) as f32;

    let duration_penalty = match (track_a.duration_ms, track_b.duration_ms) {
        (Some(d_a), Some(d_b)) => {
            let diff_sec = (d_a as f32 - d_b as f32).abs() / 1000.0;
            // Gaussian decay: e^(-0.5 * (diff / 10)^2)
            (-0.5 * (diff_sec / 10.0).powi(2)).exp()
        }
        _ => 0.85, // Neutral score for missing duration
    };

    let confidence = (title_sim * 0.50) + (artist_sim * 0.30) + (duration_penalty * 0.20);

    DedupMatchResult {
        is_match: confidence >= 0.85,
        confidence,
        reason: "fuzzy_heuristic",
    }
}

fn remove_parenthesized_feat(s: &str) -> String {
    let mut result = s.to_string();
    for prefix in &["(feat.", "(ft.", "[feat.", "[ft.", "(feat ", "(ft "] {
        while let Some(start) = result.find(prefix) {
            let close_char = if prefix.starts_with('(') { ')' } else { ']' };
            if let Some(end) = result[start..].find(close_char) {
                result.replace_range(start..start + end + 1, "");
            } else {
                result.replace_range(start.., "");
            }
        }
    }
    result
}

fn remove_remastered_tags(s: &str) -> String {
    let mut result = s.to_string();
    for tag in &["- remastered", "- 20", "- live", "(remastered", "[remastered"] {
        if let Some(pos) = result.find(tag) {
            if tag.starts_with('(') || tag.starts_with('[') {
                let close_char = if tag.starts_with('(') { ')' } else { ']' };
                if let Some(end) = result[pos..].find(close_char) {
                    result.replace_range(pos..pos + end + 1, "");
                } else {
                    result.replace_range(pos.., "");
                }
            } else {
                result = result[..pos].to_string();
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_title_and_accents() {
        assert_eq!(clean_title("Never Gonna Give You Up (Official Music Video)"), "never gonna give you up");
        assert_eq!(clean_title("Never Gonna Give You Up [Official Audio]"), "never gonna give you up");
        assert_eq!(clean_title("Never Gonna Give You Up (Lyrics)"), "never gonna give you up");
        assert_eq!(clean_title("Never Gonna Give You Up (feat. Other Artist) [4K]"), "never gonna give you up");
        assert_eq!(clean_title("Bohemian Rhapsody - Remastered 2011"), "bohemian rhapsody");
        assert_eq!(clean_title("Café del Mar"), "cafe del mar");
        assert_eq!(clean_title("Hélène"), "helene");
    }

    #[test]
    fn test_clean_artist() {
        assert_eq!(clean_artist("Rick Astley - Topic"), "rick astley");
        assert_eq!(clean_artist("RickAstleyVEVO"), "rickastley");
        assert_eq!(clean_artist("Rick Astley feat. Somebody"), "rick astley");
        assert_eq!(clean_artist("Queen"), "queen");
        assert_eq!(clean_artist("Beyoncé"), "beyonce");
    }

    #[test]
    fn test_canonical_key_deduplication() {
        let key1 = make_canonical_key("Never Gonna Give You Up (Official Music Video)", "Rick Astley - Topic");
        let key2 = make_canonical_key("Never Gonna Give You Up [Official Audio]", "Rick Astley");
        let key3 = make_canonical_key("Café del Mar", "José Padilla");
        let key4 = make_canonical_key("Cafe Del Mar", "Jose Padilla");

        assert_eq!(key1, "rick astley::never gonna give you up");
        assert_eq!(key2, key1);
        assert_eq!(key3, "jose padilla::cafe del mar");
        assert_eq!(key4, key3);
    }

    #[test]
    fn test_isrc_exact_match() {
        let track_a = CandidateTrack {
            canonical_key: "queen::bohemian rhapsody".into(),
            title: "Bohemian Rhapsody (2011 Mix)".into(),
            artist: "Queen".into(),
            album: None,
            isrc: Some("GBUM71029604".into()),
            duration_ms: Some(354000),
        };
        let track_b = CandidateTrack {
            canonical_key: "queen::bohemian rhapsody live at wembley".into(),
            title: "Bohemian Rhapsody (Live at Wembley)".into(),
            artist: "Queen".into(),
            album: None,
            isrc: Some("GBUM71029604".into()),
            duration_ms: Some(360000),
        };
        let overrides = HashMap::new();
        let res = compute_dedup_confidence(&track_a, &track_b, &overrides);
        assert!(res.is_match);
        assert_eq!(res.confidence, 1.0);
        assert_eq!(res.reason, "isrc_match");
    }

    #[test]
    fn test_continuous_gaussian_duration_penalty() {
        let track_a = CandidateTrack {
            canonical_key: "queen::bohemian rhapsody".into(),
            title: "Bohemian Rhapsody".into(),
            artist: "Queen".into(),
            album: None,
            isrc: None,
            duration_ms: Some(354000),
        };
        // 5s difference
        let track_b = CandidateTrack {
            canonical_key: "queen::bohemian rhapsody".into(),
            title: "Bohemian Rhapsody".into(),
            artist: "Queen".into(),
            album: None,
            isrc: None,
            duration_ms: Some(359000),
        };
        let overrides = HashMap::new();
        let res1 = compute_dedup_confidence(&track_a, &track_b, &overrides);
        assert!(res1.is_match);
        assert!(res1.confidence >= 0.95);

        // 60s difference (e.g. extended mix)
        let track_c = CandidateTrack {
            canonical_key: "queen::bohemian rhapsody".into(),
            title: "Bohemian Rhapsody".into(),
            artist: "Queen".into(),
            album: None,
            isrc: None,
            duration_ms: Some(414000),
        };
        let res2 = compute_dedup_confidence(&track_a, &track_c, &overrides);
        assert!(!res2.is_match);
        assert!(res2.confidence < 0.85);
    }

    #[test]
    fn test_order_invariant_user_override() {
        let track_a = CandidateTrack {
            canonical_key: "track_a".into(),
            title: "Track A".into(),
            artist: "Artist A".into(),
            album: None,
            isrc: None,
            duration_ms: Some(180000),
        };
        let track_b = CandidateTrack {
            canonical_key: "track_b".into(),
            title: "Totally Different".into(),
            artist: "Different Artist".into(),
            album: None,
            isrc: None,
            duration_ms: Some(250000),
        };
        
        let mut overrides = HashMap::new();
        // Insert with key_b first
        let pair = canonical_override_key("track_b", "track_a");
        overrides.insert(pair, true);

        // Evaluate passing track_a, track_b
        let res = compute_dedup_confidence(&track_a, &track_b, &overrides);
        assert!(res.is_match);
        assert_eq!(res.confidence, 1.0);
        assert_eq!(res.reason, "user_override");
    }
}
