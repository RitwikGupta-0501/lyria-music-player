use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LyricsQuery {
    pub title: String,
    pub artist: String,
    #[serde(default)]
    pub album: Option<String>,
    #[serde(default)]
    pub duration_ms: Option<u64>,
    #[serde(default)]
    pub isrc: Option<String>,
    #[serde(default)]
    pub track_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LyricsFormat {
    Synced,
    Plain,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LyricLine {
    pub time_ms: u64,
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_time_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LyricsResult {
    pub provider_id: String,
    pub provider_name: String,
    pub format: LyricsFormat,
    #[serde(default)]
    pub lines: Vec<LyricLine>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_plain: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_lrc: Option<String>,
    #[serde(default)]
    pub offset_ms: i64,
    #[serde(default)]
    pub instrumental: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub copyright: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LyricsProviderMeta {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub is_preferred: bool,
    pub priority: i32,
    pub status: String,
}
