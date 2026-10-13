use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum DownloadQualityPreference {
    #[default]
    High,         // ~320kbps MP3 / ~256kbps Opus / AAC
    Lossless,     // FLAC / ALAC / WAV
    Standard,     // ~128kbps - ~192kbps
    SmallestSize, // Lowest bitrate stream for bandwidth conservation
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AudioCodec {
    Flac,
    Opus,
    Aac,
    Mp3,
    Vorbis,
    Wav,
    M4a,
    #[serde(untagged)]
    Other(String),
}

impl AudioCodec {
    pub fn extension(&self) -> &str {
        match self {
            Self::Flac => "flac",
            Self::Opus => "opus",
            Self::Aac => "aac",
            Self::Mp3 => "mp3",
            Self::Vorbis => "ogg",
            Self::Wav => "wav",
            Self::M4a => "m4a",
            Self::Other(ext) => ext.as_str(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DownloadCandidateItem {
    pub native_id: String,
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub duration_ms: Option<u64>,
    pub artwork_url: Option<String>,
    pub codec: Option<AudioCodec>,
    pub bitrate_kbps: Option<u32>,
    pub extra_info: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AlbumTrackDownloadInput {
    pub title: String,
    pub track_number: Option<u32>,
    pub duration_ms: Option<u64>,
    #[serde(default)]
    pub native_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DownloadQueryInput {
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub duration_ms: Option<u64>,
    pub isrc: Option<String>,
    pub native_id: Option<String>,
    #[serde(default)]
    pub quality_preference: DownloadQualityPreference,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DownloadSourceResult {
    pub download_url: String,
    pub codec: AudioCodec,
    pub bitrate_kbps: Option<u32>,
    pub sample_rate_hz: Option<u32>,
    pub bit_depth: Option<u8>,
    pub content_length_bytes: Option<u64>,
    pub headers: Option<HashMap<String, String>>,
    pub cover_art_url: Option<String>,
    pub lyrics_lrc: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProviderCandidate {
    pub id: String,
    pub name: String,
    pub priority: i32,
    pub is_preferred: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProviderAttemptError {
    pub provider_id: String,
    pub provider_name: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DownloadResolutionError {
    NoDownloadProvidersAvailable,
    CapabilityMissing { provider_id: String },
    CircuitBreakerTripped { provider_id: String, cooldown_remaining_secs: u64 },
    AllProvidersFailed { attempts: Vec<ProviderAttemptError> },
    Sandbox(String),
    InvalidUrl(String),
}

impl std::fmt::Display for DownloadResolutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoDownloadProvidersAvailable => {
                write!(f, "No enabled extensions declare the 'download' capability")
            }
            Self::CapabilityMissing { provider_id } => {
                write!(f, "Provider '{}' does not declare the 'download' capability", provider_id)
            }
            Self::CircuitBreakerTripped { provider_id, cooldown_remaining_secs } => {
                write!(
                    f,
                    "Provider '{}' is temporarily in cooldown ({}s remaining) due to consecutive failures",
                    provider_id, cooldown_remaining_secs
                )
            }
            Self::AllProvidersFailed { attempts } => {
                write!(f, "All download providers failed to resolve source: {:?}", attempts)
            }
            Self::Sandbox(err) => write!(f, "Sandbox execution error: {}", err),
            Self::InvalidUrl(url) => write!(f, "Download URL rejected: {}", url),
        }
    }
}

impl std::error::Error for DownloadResolutionError {}
