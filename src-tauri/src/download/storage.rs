use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

/// Resolves the default downloads directory:
/// 1. First tries OS standard audio directory `~/Music/Lyria`
/// 2. Falls back to `~/Downloads/Lyria`
/// 3. Falls back to data directory `Lyria/downloads`
pub fn get_default_downloads_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let base_music = app
        .path()
        .audio_dir()
        .or_else(|_| app.path().download_dir())
        .or_else(|_| app.path().data_dir())
        .map_err(|e| format!("Failed to resolve OS directory: {}", e))?;

    let lyria_downloads = base_music.join("Lyria");
    if !lyria_downloads.exists() {
        std::fs::create_dir_all(&lyria_downloads)
            .map_err(|e| format!("Failed to create downloads directory {:?}: {}", lyria_downloads, e))?;
    }

    Ok(lyria_downloads)
}

/// Sanitizes path components by stripping or replacing forbidden OS characters
/// across Windows, macOS, and Linux: \ / : * ? " < > | and control characters.
pub fn sanitize_filename_component(name: &str) -> String {
    let filtered: String = name
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect();

    // Trim whitespace and trailing periods (Windows forbids filenames ending in dot or space)
    let trimmed = filtered.trim().trim_end_matches('.');
    if trimmed.is_empty() || trimmed.chars().all(|c| c == '_') {
        "Unknown".to_string()
    } else {
        trimmed.to_string()
    }
}

/// Constructs a cleanly organized path on disk:
/// `{base_dir}/{SanitizedArtist}/{SanitizedAlbum}/{01 - }{SanitizedTitle}.{ext}`
pub fn build_track_destination_path(
    base_dir: &Path,
    artist: &str,
    album: Option<&str>,
    track_number: Option<u32>,
    title: &str,
    extension: &str,
) -> PathBuf {
    let clean_artist = sanitize_filename_component(artist);
    let clean_album = sanitize_filename_component(album.unwrap_or("Singles & EP"));
    let clean_title = sanitize_filename_component(title);
    let clean_ext = sanitize_filename_component(extension);

    let filename = if let Some(num) = track_number {
        format!("{:02} - {}.{}", num, clean_title, clean_ext)
    } else {
        format!("{}.{}", clean_title, clean_ext)
    };

    base_dir.join(clean_artist).join(clean_album).join(filename)
}

/// Builds the temporary `.part` file path located in the exact same parent
/// directory as the destination file to ensure atomic `rename` on all filesystems.
pub fn build_temp_file_path(dest_path: &Path) -> PathBuf {
    let file_name = dest_path
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("track.audio");
    let temp_name = format!("{}.part", file_name);

    if let Some(parent) = dest_path.parent() {
        parent.join(temp_name)
    } else {
        PathBuf::from(temp_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename_component() {
        assert_eq!(sanitize_filename_component("AC/DC"), "AC_DC");
        assert_eq!(sanitize_filename_component("What?: The Album*"), "What__ The Album_");
        assert_eq!(sanitize_filename_component("Trailing Dot..."), "Trailing Dot");
        assert_eq!(sanitize_filename_component("   Spaces   "), "Spaces");
        assert_eq!(sanitize_filename_component(":::"), "Unknown");
        assert_eq!(sanitize_filename_component(""), "Unknown");
    }

    #[test]
    fn test_build_track_destination_path() {
        let base = Path::new("/home/user/Music/Lyria");
        let path = build_track_destination_path(
            base,
            "The Weeknd",
            Some("After Hours"),
            Some(4),
            "Blinding Lights",
            "flac",
        );

        assert_eq!(
            path,
            Path::new("/home/user/Music/Lyria/The Weeknd/After Hours/04 - Blinding Lights.flac")
        );

        // Test without album and track number
        let single_path = build_track_destination_path(
            base,
            "Daft Punk",
            None,
            None,
            "One More Time",
            "opus",
        );

        assert_eq!(
            single_path,
            Path::new("/home/user/Music/Lyria/Daft Punk/Singles & EP/One More Time.opus")
        );
    }

    #[test]
    fn test_build_temp_file_path() {
        let dest = Path::new("/home/user/Music/track.flac");
        let temp = build_temp_file_path(dest);
        assert_eq!(temp, Path::new("/home/user/Music/track.flac.part"));
    }
}
