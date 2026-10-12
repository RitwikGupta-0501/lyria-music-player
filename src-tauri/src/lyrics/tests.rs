use super::models::*;
use super::parser::parse_lrc;

#[test]
fn test_lrc_parser_standard_and_centiseconds() {
    let lrc = r#"
[ti:Bohemian Rhapsody]
[ar:Queen]
[00:00.50]Is this the real life?
[00:03.20]Is this just fantasy?
[00:07.85]Caught in a landslide, no escape from reality
"#;
    let result = parse_lrc("lrclib-wasm", "LRCLIB Provider", lrc);
    assert_eq!(result.format, LyricsFormat::Synced);
    assert_eq!(result.provider_id, "lrclib-wasm");
    assert_eq!(result.lines.len(), 3);
    assert_eq!(result.lines[0].time_ms, 500);
    assert_eq!(result.lines[0].text, "Is this the real life?");
    assert_eq!(result.lines[0].end_time_ms, Some(3200));
    assert_eq!(result.lines[1].time_ms, 3200);
    assert_eq!(result.lines[1].text, "Is this just fantasy?");
    assert_eq!(result.lines[1].end_time_ms, Some(7850));
    assert_eq!(result.lines[2].time_ms, 7850);
    assert_eq!(result.lines[2].end_time_ms, None);
}

#[test]
fn test_lrc_parser_multi_bracket_expansion() {
    let lrc = r#"
[00:10.00][00:20.00][00:30.00]Never gonna let you down
"#;
    let result = parse_lrc("test-prov", "Test", lrc);
    assert_eq!(result.format, LyricsFormat::Synced);
    assert_eq!(result.lines.len(), 3);
    assert_eq!(result.lines[0].time_ms, 10000);
    assert_eq!(result.lines[1].time_ms, 20000);
    assert_eq!(result.lines[2].time_ms, 30000);
    assert_eq!(result.lines[0].text, "Never gonna let you down");
    assert_eq!(result.lines[1].text, "Never gonna let you down");
    assert_eq!(result.lines[2].text, "Never gonna let you down");
}

#[test]
fn test_lrc_parser_offset_tag_handling() {
    let lrc = r#"
[offset:+1500]
[00:05.00]Line one delayed by 1.5 seconds
"#;
    let result = parse_lrc("test-prov", "Test", lrc);
    assert_eq!(result.format, LyricsFormat::Synced);
    assert_eq!(result.offset_ms, 1500);
    assert_eq!(result.lines[0].time_ms, 6500);

    let lrc_neg = r#"
[offset:-8000]
[00:05.00]Line one clamped to zero
"#;
    let result_neg = parse_lrc("test-prov", "Test", lrc_neg);
    assert_eq!(result_neg.offset_ms, -8000);
    assert_eq!(result_neg.lines[0].time_ms, 0); // Clamped
}

#[test]
fn test_lrc_parser_plain_fallback() {
    let plain = r#"
Yesterday
All my troubles seemed so far away
Now it looks as though they're here to stay
"#;
    let result = parse_lrc("test-prov", "Test", plain);
    assert_eq!(result.format, LyricsFormat::Plain);
    assert_eq!(result.lines.len(), 3);
    assert_eq!(result.lines[0].text, "Yesterday");
    assert_eq!(result.lines[0].time_ms, 0);
    assert_eq!(result.raw_plain.unwrap().trim(), plain.trim());
}

#[test]
fn test_title_cleaning_for_lyrics_search() {
    let raw = "Hotel California (Remastered 2013) [Official Music Video]";
    let cleaned = crate::db::canonical::clean_title(raw);
    assert_eq!(cleaned, "hotel california");

    let feat = "Blinding Lights (feat. Rosalía)";
    let cleaned_feat = crate::db::canonical::clean_title(feat);
    assert_eq!(cleaned_feat, "blinding lights");
}

#[test]
fn test_sqlite_lyrics_cache_operations() {
    let conn = crate::db::schema::init_db(":memory:").unwrap();

    let canonical_key = "queen::bohemian rhapsody";
    let lyrics = LyricsResult {
        provider_id: "lrclib".to_string(),
        provider_name: "LRCLIB".to_string(),
        format: LyricsFormat::Synced,
        lines: vec![
            LyricLine {
                time_ms: 1000,
                text: "Mama, just killed a man".to_string(),
                end_time_ms: Some(5000),
            },
        ],
        raw_plain: None,
        raw_lrc: Some("[00:01.00]Mama, just killed a man".to_string()),
        offset_ms: 0,
        instrumental: false,
        copyright: None,
    };

    // Save
    crate::db::queries::save_cached_lyrics(&conn, canonical_key, &lyrics).unwrap();

    // Retrieve
    let cached = crate::db::queries::get_cached_lyrics(&conn, canonical_key, Some("lrclib")).unwrap();
    assert!(cached.is_some());
    let hit = cached.unwrap();
    assert_eq!(hit.provider_id, "lrclib");
    assert_eq!(hit.lines.len(), 1);
    assert_eq!(hit.lines[0].text, "Mama, just killed a man");

    // Clear
    crate::db::queries::clear_lyrics_cache(&conn, Some(canonical_key)).unwrap();
    let after_clear = crate::db::queries::get_cached_lyrics(&conn, canonical_key, None).unwrap();
    assert!(after_clear.is_none());
}
