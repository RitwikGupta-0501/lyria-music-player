use std::collections::HashMap;
use std::sync::RwLock;
use serde::{Serialize, Deserialize};
use rusqlite::Connection;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeatureFlag {
    ShuffleMode,
    ReorderQueue,
    VirtualScrolling,
    PersistentQueue,
    GlassyPlayerBar,
    ExperimentalDsp,
    ExploreSpotlightCarousel,
    OfflineArtworkCaching,
}

impl FeatureFlag {
    pub const ALL: &'static [FeatureFlag] = &[
        FeatureFlag::ShuffleMode,
        FeatureFlag::ReorderQueue,
        FeatureFlag::VirtualScrolling,
        FeatureFlag::PersistentQueue,
        FeatureFlag::GlassyPlayerBar,
        FeatureFlag::ExperimentalDsp,
        FeatureFlag::ExploreSpotlightCarousel,
        FeatureFlag::OfflineArtworkCaching,
    ];

    pub fn key(&self) -> &'static str {
        match self {
            FeatureFlag::ShuffleMode => "shuffle_mode",
            FeatureFlag::ReorderQueue => "reorder_queue",
            FeatureFlag::VirtualScrolling => "virtual_scrolling",
            FeatureFlag::PersistentQueue => "persistent_queue",
            FeatureFlag::GlassyPlayerBar => "glassy_player_bar",
            FeatureFlag::ExperimentalDsp => "experimental_dsp",
            FeatureFlag::ExploreSpotlightCarousel => "explore_spotlight_carousel",
            FeatureFlag::OfflineArtworkCaching => "offline_artwork_caching",
        }
    }

    pub fn from_key(k: &str) -> Option<Self> {
        let norm = k.trim().to_lowercase().replace('-', "_");
        match norm.as_str() {
            "shuffle_mode" | "shufflemode" => Some(FeatureFlag::ShuffleMode),
            "reorder_queue" | "reorderqueue" => Some(FeatureFlag::ReorderQueue),
            "virtual_scrolling" | "virtualscrolling" => Some(FeatureFlag::VirtualScrolling),
            "persistent_queue" | "persistentqueue" => Some(FeatureFlag::PersistentQueue),
            "glassy_player_bar" | "glassyplayerbar" => Some(FeatureFlag::GlassyPlayerBar),
            "experimental_dsp" | "experimentaldsp" => Some(FeatureFlag::ExperimentalDsp),
            "explore_spotlight_carousel" | "explorespotlightcarousel" => Some(FeatureFlag::ExploreSpotlightCarousel),
            "offline_artwork_caching" | "offlineartworkcaching" => Some(FeatureFlag::OfflineArtworkCaching),
            _ => None,
        }
    }

    pub fn descriptor(&self) -> FlagDescriptor {
        match self {
            FeatureFlag::ShuffleMode => FlagDescriptor {
                key: self.key(),
                name: "Shuffle Playback Mode",
                description: "Enables Fisher-Yates and algorithmic queue shuffling capabilities.",
                category: "Audio & Queue",
                stage: FlagStage::Stable,
                default_enabled: true,
            },
            FeatureFlag::ReorderQueue => FlagDescriptor {
                key: self.key(),
                name: "Queue Reordering",
                description: "Allows manual drag-and-drop or programmatic reordering of tracks in the queue.",
                category: "Audio & Queue",
                stage: FlagStage::Stable,
                default_enabled: true,
            },
            FeatureFlag::VirtualScrolling => FlagDescriptor {
                key: self.key(),
                name: "Virtual Scrolling",
                description: "Enables DOM virtualization for massive library and playlist track lists.",
                category: "Performance",
                stage: FlagStage::Stable,
                default_enabled: true,
            },
            FeatureFlag::PersistentQueue => FlagDescriptor {
                key: self.key(),
                name: "Persistent Queue State",
                description: "Preserves playback queue and current position across application restarts.",
                category: "Audio & Queue",
                stage: FlagStage::Stable,
                default_enabled: true,
            },
            FeatureFlag::GlassyPlayerBar => FlagDescriptor {
                key: self.key(),
                name: "Liquid Glass Player Bar",
                description: "Applies liquid glassmorphic backdrop-filter blur and specular borders to the player bar.",
                category: "Appearance",
                stage: FlagStage::Beta,
                default_enabled: true,
            },
            FeatureFlag::ExploreSpotlightCarousel => FlagDescriptor {
                key: self.key(),
                name: "Explore Spotlight Carousel",
                description: "Displays the top editorial spotlight banner carousel in the Explore view.",
                category: "Explore & Discovery",
                stage: FlagStage::Stable,
                default_enabled: true,
            },
            FeatureFlag::ExperimentalDsp => FlagDescriptor {
                key: self.key(),
                name: "Experimental DSP & Equalizer",
                description: "Unlocks real-time parametric EQ and advanced audio digital signal processing hooks.",
                category: "Audio Engine",
                stage: FlagStage::Experimental,
                default_enabled: false,
            },
            FeatureFlag::OfflineArtworkCaching => FlagDescriptor {
                key: self.key(),
                name: "Aggressive Offline Artwork Caching",
                description: "Pre-caches and stores local copies of remote artwork thumbnails on disk.",
                category: "Network & Storage",
                stage: FlagStage::Beta,
                default_enabled: true,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FlagStage {
    Stable,
    Beta,
    Experimental,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagDescriptor {
    pub key: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub category: &'static str,
    pub stage: FlagStage,
    pub default_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagDetail {
    pub key: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub stage: FlagStage,
    pub enabled: bool,
    pub is_overridden_by_env: bool,
}

pub struct FeatureFlags {
    state: RwLock<HashMap<FeatureFlag, bool>>,
    env_overrides: RwLock<HashMap<FeatureFlag, bool>>,
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self::new()
    }
}

impl FeatureFlags {
    pub fn new() -> Self {
        let mut defaults = HashMap::new();
        let mut envs = HashMap::new();

        for flag in FeatureFlag::ALL {
            let desc = flag.descriptor();
            let mut val = desc.default_enabled;

            // Check Environment Variable Overrides (ECHO_FF_<NAME>=1/0/true/false)
            let env_var_name = format!("ECHO_FF_{}", desc.key.to_uppercase());
            if let Ok(env_val) = std::env::var(&env_var_name) {
                let trimmed = env_val.trim().to_lowercase();
                if trimmed == "1" || trimmed == "true" || trimmed == "yes" || trimmed == "on" {
                    val = true;
                    envs.insert(*flag, true);
                } else if trimmed == "0" || trimmed == "false" || trimmed == "no" || trimmed == "off" {
                    val = false;
                    envs.insert(*flag, false);
                }
            }

            defaults.insert(*flag, val);
        }

        FeatureFlags {
            state: RwLock::new(defaults),
            env_overrides: RwLock::new(envs),
        }
    }

    pub fn init_from_db(&self, conn: &Connection) {
        if let Ok(mut stmt) = conn.prepare("SELECT key, enabled FROM feature_flags") {
            if let Ok(rows) = stmt.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? != 0))
            }) {
                let envs = self.env_overrides.read().unwrap();
                let mut state = self.state.write().unwrap();

                for item in rows.flatten() {
                    if let Some(flag) = FeatureFlag::from_key(&item.0) {
                        // Environment variables take precedence over DB
                        if !envs.contains_key(&flag) {
                            state.insert(flag, item.1);
                        }
                    }
                }
            }
        }
    }

    pub fn is_enabled(&self, flag: FeatureFlag) -> bool {
        self.state
            .read()
            .ok()
            .and_then(|map| map.get(&flag).copied())
            .unwrap_or_else(|| flag.descriptor().default_enabled)
    }

    pub fn set_flag(&self, flag: FeatureFlag, enabled: bool) {
        if let Ok(mut map) = self.state.write() {
            map.insert(flag, enabled);
        }
    }

    pub fn reset_defaults(&self) {
        if let Ok(mut map) = self.state.write() {
            let envs = self.env_overrides.read().unwrap();
            for flag in FeatureFlag::ALL {
                let val = if let Some(env_val) = envs.get(flag) {
                    *env_val
                } else {
                    flag.descriptor().default_enabled
                };
                map.insert(*flag, val);
            }
        }
    }

    pub fn get_all_details(&self) -> Vec<FlagDetail> {
        let state = self.state.read().unwrap();
        let envs = self.env_overrides.read().unwrap();

        FeatureFlag::ALL.iter().map(|flag| {
            let desc = flag.descriptor();
            let is_env = envs.contains_key(flag);
            let enabled = state.get(flag).copied().unwrap_or(desc.default_enabled);

            FlagDetail {
                key: desc.key.to_string(),
                name: desc.name.to_string(),
                description: desc.description.to_string(),
                category: desc.category.to_string(),
                stage: desc.stage,
                enabled,
                is_overridden_by_env: is_env,
            }
        }).collect()
    }

    pub fn get_enabled_flags(&self) -> Vec<FeatureFlag> {
        let state = self.state.read().unwrap();
        FeatureFlag::ALL
            .iter()
            .filter(|f| state.get(f).copied().unwrap_or(false))
            .copied()
            .collect()
    }

    pub fn enable(&self, flag: FeatureFlag) {
        self.set_flag(flag, true);
    }

    pub fn disable(&self, flag: FeatureFlag) {
        self.set_flag(flag, false);
    }
}

lazy_static::lazy_static! {
    pub static ref FEATURE_FLAGS: FeatureFlags = FeatureFlags::new();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_flags_defaults() {
        let ff = FeatureFlags::new();
        assert!(ff.is_enabled(FeatureFlag::ShuffleMode));
        assert!(ff.is_enabled(FeatureFlag::ReorderQueue));
        assert!(!ff.is_enabled(FeatureFlag::ExperimentalDsp));
    }

    #[test]
    fn test_feature_flags_toggle() {
        let ff = FeatureFlags::new();
        ff.set_flag(FeatureFlag::ExperimentalDsp, true);
        assert!(ff.is_enabled(FeatureFlag::ExperimentalDsp));
        ff.set_flag(FeatureFlag::ExperimentalDsp, false);
        assert!(!ff.is_enabled(FeatureFlag::ExperimentalDsp));
    }

    #[test]
    fn test_key_conversion() {
        assert_eq!(FeatureFlag::from_key("shuffle_mode"), Some(FeatureFlag::ShuffleMode));
        assert_eq!(FeatureFlag::from_key("experimental-dsp"), Some(FeatureFlag::ExperimentalDsp));
        assert_eq!(FeatureFlag::from_key("non_existent"), None);
    }
}
