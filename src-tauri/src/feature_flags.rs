use std::collections::HashMap;
use std::sync::RwLock;
use serde::{Serialize, Deserialize};
use rusqlite::Connection;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeatureFlag {
    // Pages (Top-Level)
    PageHome,
    PageExplore,

    // Home Page Sections
    HomeQuickPicks,
    HomeDailyDiscover,
    HomeHeavyRotation,
    HomeAdjacentHorizons,
    HomeRadioMix,
    HomeJumpBackIn,
    HomeForgottenFavorites,

    // Explore Page Sections
    ExploreSpotlightCarousel,
    ExploreCategoryGrid,
    ExploreNewReleases,
    ExploreTopCharts,
    ExploreThematicCollections,
}

impl FeatureFlag {
    pub const ALL: &'static [FeatureFlag] = &[
        FeatureFlag::PageHome,
        FeatureFlag::PageExplore,
        FeatureFlag::HomeQuickPicks,
        FeatureFlag::HomeDailyDiscover,
        FeatureFlag::HomeHeavyRotation,
        FeatureFlag::HomeAdjacentHorizons,
        FeatureFlag::HomeRadioMix,
        FeatureFlag::HomeJumpBackIn,
        FeatureFlag::HomeForgottenFavorites,
        FeatureFlag::ExploreSpotlightCarousel,
        FeatureFlag::ExploreCategoryGrid,
        FeatureFlag::ExploreNewReleases,
        FeatureFlag::ExploreTopCharts,
        FeatureFlag::ExploreThematicCollections,
    ];

    pub fn key(&self) -> &'static str {
        match self {
            FeatureFlag::PageHome => "page_home",
            FeatureFlag::PageExplore => "page_explore",
            FeatureFlag::HomeQuickPicks => "home_quick_picks",
            FeatureFlag::HomeDailyDiscover => "home_daily_discover",
            FeatureFlag::HomeHeavyRotation => "home_heavy_rotation",
            FeatureFlag::HomeAdjacentHorizons => "home_adjacent_horizons",
            FeatureFlag::HomeRadioMix => "home_radio_mix",
            FeatureFlag::HomeJumpBackIn => "home_jump_back_in",
            FeatureFlag::HomeForgottenFavorites => "home_forgotten_favorites",
            FeatureFlag::ExploreSpotlightCarousel => "explore_spotlight_carousel",
            FeatureFlag::ExploreCategoryGrid => "explore_category_grid",
            FeatureFlag::ExploreNewReleases => "explore_new_releases",
            FeatureFlag::ExploreTopCharts => "explore_top_charts",
            FeatureFlag::ExploreThematicCollections => "explore_thematic_collections",
        }
    }

    pub fn from_key(k: &str) -> Option<Self> {
        let norm = k.trim().to_lowercase().replace('-', "_");
        match norm.as_str() {
            "page_home" | "pagehome" => Some(FeatureFlag::PageHome),
            "page_explore" | "pageexplore" => Some(FeatureFlag::PageExplore),
            "home_quick_picks" | "homequickpicks" => Some(FeatureFlag::HomeQuickPicks),
            "home_daily_discover" | "homedailydiscover" => Some(FeatureFlag::HomeDailyDiscover),
            "home_heavy_rotation" | "homeheavyrotation" => Some(FeatureFlag::HomeHeavyRotation),
            "home_adjacent_horizons" | "homeadjacenthorizons" => Some(FeatureFlag::HomeAdjacentHorizons),
            "home_radio_mix" | "homeradiomix" => Some(FeatureFlag::HomeRadioMix),
            "home_jump_back_in" | "homejumpbackin" => Some(FeatureFlag::HomeJumpBackIn),
            "home_forgotten_favorites" | "homeforgottenfavorites" => Some(FeatureFlag::HomeForgottenFavorites),
            "explore_spotlight_carousel" | "explorespotlightcarousel" => Some(FeatureFlag::ExploreSpotlightCarousel),
            "explore_category_grid" | "explorecategorygrid" => Some(FeatureFlag::ExploreCategoryGrid),
            "explore_new_releases" | "explorenewreleases" => Some(FeatureFlag::ExploreNewReleases),
            "explore_top_charts" | "exploretopcharts" => Some(FeatureFlag::ExploreTopCharts),
            "explore_thematic_collections" | "explorethematiccollections" => Some(FeatureFlag::ExploreThematicCollections),
            _ => None,
        }
    }

    pub fn descriptor(&self) -> FlagDescriptor {
        match self {
            FeatureFlag::PageHome => FlagDescriptor {
                key: self.key(),
                name: "Home Page",
                description: "Enables the dynamic Home recommendations feed in navigation.",
                category: "Pages",
                stage: FlagStage::Stable,
                default_enabled: true,
            },
            FeatureFlag::PageExplore => FlagDescriptor {
                key: self.key(),
                name: "Explore Page",
                description: "Enables the Explore catalog, charts, and online discovery view.",
                category: "Pages",
                stage: FlagStage::Stable,
                default_enabled: true,
            },
            FeatureFlag::HomeQuickPicks => FlagDescriptor {
                key: self.key(),
                name: "Quick Picks Grid",
                description: "Displays the personalized 2x3 seed grid on the Home page.",
                category: "Home Page Sections",
                stage: FlagStage::Stable,
                default_enabled: true,
            },
            FeatureFlag::HomeDailyDiscover => FlagDescriptor {
                key: self.key(),
                name: "Daily Discover Carousel",
                description: "Displays the daily algorithmic discovery carousel on the Home page.",
                category: "Home Page Sections",
                stage: FlagStage::Stable,
                default_enabled: true,
            },
            FeatureFlag::HomeHeavyRotation => FlagDescriptor {
                key: self.key(),
                name: "Heavy Rotation Shelf",
                description: "Displays your 7-day most played artists and albums shelf.",
                category: "Home Page Sections",
                stage: FlagStage::Stable,
                default_enabled: true,
            },
            FeatureFlag::HomeAdjacentHorizons => FlagDescriptor {
                key: self.key(),
                name: "Adjacent Horizons Discovery",
                description: "Displays the dynamic musical frontiers and genre discovery card.",
                category: "Home Page Sections",
                stage: FlagStage::Stable,
                default_enabled: true,
            },
            FeatureFlag::HomeRadioMix => FlagDescriptor {
                key: self.key(),
                name: "Radio Mixes Carousel",
                description: "Displays algorithmic artist and genre radio mixes on the Home page.",
                category: "Home Page Sections",
                stage: FlagStage::Stable,
                default_enabled: true,
            },
            FeatureFlag::HomeJumpBackIn => FlagDescriptor {
                key: self.key(),
                name: "Jump Back In Shelf",
                description: "Displays recently played albums, sessions, and playlists on the Home page.",
                category: "Home Page Sections",
                stage: FlagStage::Stable,
                default_enabled: true,
            },
            FeatureFlag::HomeForgottenFavorites => FlagDescriptor {
                key: self.key(),
                name: "Forgotten Favorites Shelf",
                description: "Displays past favorites you haven't listened to in a while.",
                category: "Home Page Sections",
                stage: FlagStage::Stable,
                default_enabled: true,
            },
            FeatureFlag::ExploreSpotlightCarousel => FlagDescriptor {
                key: self.key(),
                name: "Spotlight Carousel",
                description: "Displays the top editorial spotlight banner carousel in Explore.",
                category: "Explore Page Sections",
                stage: FlagStage::Stable,
                default_enabled: true,
            },
            FeatureFlag::ExploreCategoryGrid => FlagDescriptor {
                key: self.key(),
                name: "Category & Genre Hub",
                description: "Displays the mood, vibe, and genre category matrix in Explore.",
                category: "Explore Page Sections",
                stage: FlagStage::Stable,
                default_enabled: true,
            },
            FeatureFlag::ExploreNewReleases => FlagDescriptor {
                key: self.key(),
                name: "New Release Radar",
                description: "Displays the new release album radar shelf in Explore.",
                category: "Explore Page Sections",
                stage: FlagStage::Stable,
                default_enabled: true,
            },
            FeatureFlag::ExploreTopCharts => FlagDescriptor {
                key: self.key(),
                name: "Top Charts Ledger",
                description: "Displays trending tracks and top charts ledger in Explore.",
                category: "Explore Page Sections",
                stage: FlagStage::Stable,
                default_enabled: true,
            },
            FeatureFlag::ExploreThematicCollections => FlagDescriptor {
                key: self.key(),
                name: "Thematic Collections Shelf",
                description: "Displays curated thematic playlists and collections in Explore.",
                category: "Explore Page Sections",
                stage: FlagStage::Stable,
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

    pub fn is_enabled_str(&self, key: &str) -> bool {
        if let Some(flag) = FeatureFlag::from_key(key) {
            self.is_enabled(flag)
        } else {
            false
        }
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
        assert!(ff.is_enabled(FeatureFlag::PageHome));
        assert!(ff.is_enabled(FeatureFlag::PageExplore));
        assert!(ff.is_enabled(FeatureFlag::HomeQuickPicks));
    }

    #[test]
    fn test_feature_flags_toggle() {
        let ff = FeatureFlags::new();
        ff.set_flag(FeatureFlag::PageHome, false);
        assert!(!ff.is_enabled(FeatureFlag::PageHome));
        ff.set_flag(FeatureFlag::PageHome, true);
        assert!(ff.is_enabled(FeatureFlag::PageHome));
    }

    #[test]
    fn test_key_conversion() {
        assert_eq!(FeatureFlag::from_key("page_home"), Some(FeatureFlag::PageHome));
        assert_eq!(FeatureFlag::from_key("home-quick-picks"), Some(FeatureFlag::HomeQuickPicks));
        assert_eq!(FeatureFlag::from_key("non_existent"), None);
    }
}
