use super::models::{LyricLine, LyricsFormat, LyricsResult};
use regex::Regex;

/// Parses raw LRC or plain text into a structured `LyricsResult`.
pub fn parse_lrc(
    provider_id: &str,
    provider_name: &str,
    raw_content: &str,
) -> LyricsResult {
    let mut offset_ms: i64 = 0;
    let mut raw_lines_with_times: Vec<(i64, String)> = Vec::new();
    let mut has_timestamp = false;

    // Matches tags like [offset:+500] or [offset:-300]
    let offset_regex = Regex::new(r"^\[offset:\s*([+-]?\d+)\s*\]").unwrap();
    // Matches timestamps like [01:23.45] or [01:23.456] or [01:23]
    let timestamp_regex = Regex::new(r"\[(\d{1,2}):(\d{1,2})(?:[.:](\d{1,3}))?\]").unwrap();

    for line in raw_content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Check for [offset:...]
        if let Some(caps) = offset_regex.captures(trimmed) {
            if let Ok(val) = caps[1].parse::<i64>() {
                offset_ms = val;
            }
            continue;
        }

        // Check for other ID tags like [ar:Artist], [ti:Title] - skip them
        if trimmed.starts_with('[') && trimmed.contains(':') && !trimmed.starts_with("[0") && !trimmed.starts_with("[1") && !trimmed.starts_with("[2") && !trimmed.starts_with("[3") && !trimmed.starts_with("[4") && !trimmed.starts_with("[5") && !trimmed.starts_with("[6") && !trimmed.starts_with("[7") && !trimmed.starts_with("[8") && !trimmed.starts_with("[9") {
            // It might be a metadata tag like [ar:...], [al:...], [by:...]
            // Check if it matches a timestamp first; if not, ignore metadata tag
            if !timestamp_regex.is_match(trimmed) {
                continue;
            }
        }

        // Extract all timestamps from the line
        let mut timestamps = Vec::new();
        let mut last_end = 0;

        for mat in timestamp_regex.find_iter(trimmed) {
            if let Some(caps) = timestamp_regex.captures(mat.as_str()) {
                let mins: u64 = caps[1].parse().unwrap_or(0);
                let secs: u64 = caps[2].parse().unwrap_or(0);
                let sub: u64 = if let Some(sub_match) = caps.get(3) {
                    let sub_str = sub_match.as_str();
                    if sub_str.len() == 1 {
                        sub_str.parse::<u64>().unwrap_or(0) * 100
                    } else if sub_str.len() == 2 {
                        sub_str.parse::<u64>().unwrap_or(0) * 10
                    } else {
                        sub_str.parse::<u64>().unwrap_or(0)
                    }
                } else {
                    0
                };

                let time_ms = (mins * 60 * 1000) + (secs * 1000) + sub;
                timestamps.push(time_ms as i64);
                has_timestamp = true;
            }
            last_end = mat.end();
        }

        if !timestamps.is_empty() {
            let lyric_text = trimmed[last_end..].trim().to_string();
            for t in timestamps {
                raw_lines_with_times.push((t, lyric_text.clone()));
            }
        }
    }

    if has_timestamp && !raw_lines_with_times.is_empty() {
        // Sort chronologically
        raw_lines_with_times.sort_by_key(|&(t, _)| t);

        // Apply offset and adjust
        let mut lines = Vec::with_capacity(raw_lines_with_times.len());
        for (raw_t, text) in raw_lines_with_times {
            let adjusted = raw_t + offset_ms;
            let final_time_ms = if adjusted < 0 { 0 } else { adjusted as u64 };
            lines.push(LyricLine {
                time_ms: final_time_ms,
                text,
                end_time_ms: None,
            });
        }

        // Compute end_time_ms based on the next line's start time
        for i in 0..lines.len() {
            if i + 1 < lines.len() {
                lines[i].end_time_ms = Some(lines[i + 1].time_ms);
            }
        }

        LyricsResult {
            provider_id: provider_id.to_string(),
            provider_name: provider_name.to_string(),
            format: LyricsFormat::Synced,
            lines,
            raw_plain: None,
            raw_lrc: Some(raw_content.to_string()),
            offset_ms,
            instrumental: false,
            copyright: None,
        }
    } else {
        // Plain text fallback
        let plain_lines: Vec<LyricLine> = raw_content
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .map(|text| LyricLine {
                time_ms: 0,
                text,
                end_time_ms: None,
            })
            .collect();

        LyricsResult {
            provider_id: provider_id.to_string(),
            provider_name: provider_name.to_string(),
            format: LyricsFormat::Plain,
            lines: plain_lines,
            raw_plain: Some(raw_content.to_string()),
            raw_lrc: None,
            offset_ms: 0,
            instrumental: false,
            copyright: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_standard_lrc() {
        let lrc = r#"
[ti:Never Gonna Give You Up]
[ar:Rick Astley]
[al:Whenever You Need Somebody]
[00:18.50]We're no strangers to love
[00:22.80]You know the rules and so do I
[00:27.10]A full commitment's what I'm thinking of
"#;
        let result = parse_lrc("test-provider", "Test Provider", lrc);
        assert_eq!(result.format, LyricsFormat::Synced);
        assert_eq!(result.lines.len(), 3);
        assert_eq!(result.lines[0].time_ms, 18500);
        assert_eq!(result.lines[0].text, "We're no strangers to love");
        assert_eq!(result.lines[0].end_time_ms, Some(22800));
        assert_eq!(result.lines[1].time_ms, 22800);
        assert_eq!(result.lines[1].end_time_ms, Some(27100));
        assert_eq!(result.lines[2].time_ms, 27100);
        assert_eq!(result.lines[2].end_time_ms, None);
    }

    #[test]
    fn test_parse_offset_and_multi_brackets() {
        let lrc = r#"
[offset:+500]
[00:10.00][00:20.00]Chorus repeats here
"#;
        let result = parse_lrc("test-provider", "Test Provider", lrc);
        assert_eq!(result.format, LyricsFormat::Synced);
        assert_eq!(result.lines.len(), 2);
        assert_eq!(result.offset_ms, 500);
        assert_eq!(result.lines[0].time_ms, 10500);
        assert_eq!(result.lines[0].text, "Chorus repeats here");
        assert_eq!(result.lines[1].time_ms, 20500);
        assert_eq!(result.lines[1].text, "Chorus repeats here");
    }

    #[test]
    fn test_parse_plain_text() {
        let plain = "First line of the song\nSecond line of the song\nThird line";
        let result = parse_lrc("test-provider", "Test Provider", plain);
        assert_eq!(result.format, LyricsFormat::Plain);
        assert_eq!(result.lines.len(), 3);
        assert_eq!(result.lines[0].text, "First line of the song");
        assert_eq!(result.lines[0].time_ms, 0);
    }
}
