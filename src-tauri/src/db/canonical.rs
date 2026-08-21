//! Canonical song normalization and deduplication engine for Echo.
//! Normalizes titles and artists across local files and remote streaming providers
//! to form a deterministic fingerprint (canonical_key).

pub fn clean_title(title: &str) -> String {
    let mut s = title.to_string();

    // 1. Lowercase for case-insensitive normalization
    s = s.to_lowercase();

    // 2. Remove common YouTube/streaming noise patterns in parentheses or brackets
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

    // 3. Remove "(feat. ...)" or "[feat. ...]" or "(ft. ...)"
    s = remove_parenthesized_feat(&s);

    // 4. Remove remastered annotations e.g. "(remastered 2021)" or "- remastered"
    s = remove_remastered_tags(&s);

    // 5. Clean up special characters, punctuation, and extra whitespace
    s = s.replace(&['(', ')', '[', ']', '{', '}', '"', '\''][..], " ");
    s = s.split_whitespace().collect::<Vec<&str>>().join(" ");

    s.trim().to_string()
}

pub fn clean_artist(artist: &str) -> String {
    let mut s = artist.to_string();

    // 1. Lowercase
    s = s.to_lowercase();

    // 2. Remove YouTube auto-generated channel suffix e.g. "Rick Astley - Topic" -> "Rick Astley"
    if s.ends_with(" - topic") {
        s = s.trim_end_matches(" - topic").to_string();
    }
    if s.ends_with(" vevo") {
        s = s.trim_end_matches(" vevo").to_string();
    }
    if s.ends_with("vevo") {
        s = s.trim_end_matches("vevo").to_string();
    }

    // 3. Remove "feat. ..." or "ft. ..."
    if let Some(idx) = s.find(" feat.") {
        s = s[..idx].to_string();
    } else if let Some(idx) = s.find(" ft.") {
        s = s[..idx].to_string();
    } else if let Some(idx) = s.find(" feat ") {
        s = s[..idx].to_string();
    } else if let Some(idx) = s.find(" ft ") {
        s = s[..idx].to_string();
    }

    // 4. Clean punctuation and whitespace
    s = s.replace(&['"', '\'', ',', '&'][..], " ");
    s = s.split_whitespace().collect::<Vec<&str>>().join(" ");

    s.trim().to_string()
}

pub fn make_canonical_key(title: &str, artist: &str) -> String {
    let a = clean_artist(artist);
    let t = clean_title(title);
    format!("{}::{}", a, t)
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
    fn test_clean_title() {
        assert_eq!(clean_title("Never Gonna Give You Up (Official Music Video)"), "never gonna give you up");
        assert_eq!(clean_title("Never Gonna Give You Up [Official Audio]"), "never gonna give you up");
        assert_eq!(clean_title("Never Gonna Give You Up (Lyrics)"), "never gonna give you up");
        assert_eq!(clean_title("Never Gonna Give You Up (feat. Other Artist) [4K]"), "never gonna give you up");
        assert_eq!(clean_title("Bohemian Rhapsody - Remastered 2011"), "bohemian rhapsody");
        assert_eq!(clean_title("Plain Song Title"), "plain song title");
    }

    #[test]
    fn test_clean_artist() {
        assert_eq!(clean_artist("Rick Astley - Topic"), "rick astley");
        assert_eq!(clean_artist("RickAstleyVEVO"), "rickastley");
        assert_eq!(clean_artist("Rick Astley feat. Somebody"), "rick astley");
        assert_eq!(clean_artist("Queen"), "queen");
    }

    #[test]
    fn test_canonical_key_deduplication() {
        let key1 = make_canonical_key("Never Gonna Give You Up (Official Music Video)", "Rick Astley - Topic");
        let key2 = make_canonical_key("Never Gonna Give You Up [Official Audio]", "Rick Astley");
        let key3 = make_canonical_key("Never Gonna Give You Up", "Rick Astley");

        assert_eq!(key1, "rick astley::never gonna give you up");
        assert_eq!(key2, key1);
        assert_eq!(key3, key1);
    }
}
