import { invoke } from "@tauri-apps/api/core";

export type FeatureFlagKey = 
    // Pages
    | "page_home"
    | "page_explore"
    // Home Sections
    | "home_quick_picks"
    | "home_daily_discover"
    | "home_heavy_rotation"
    | "home_adjacent_horizons"
    | "home_radio_mix"
    | "home_jump_back_in"
    | "home_forgotten_favorites"
    // Explore Sections
    | "explore_spotlight_carousel"
    | "explore_category_grid"
    | "explore_new_releases"
    | "explore_top_charts"
    | "explore_thematic_collections";

export type FlagStage = "stable" | "beta" | "experimental";

export interface FlagDetail {
    key: FeatureFlagKey;
    name: string;
    description: string;
    category: string;
    stage: FlagStage;
    enabled: boolean;
    is_overridden_by_env: boolean;
}

export class FeatureFlagsStore {
    loaded = $state(false);
    flags = $state<Record<string, boolean>>({
        page_home: true,
        page_explore: true,
        home_quick_picks: true,
        home_daily_discover: true,
        home_heavy_rotation: true,
        home_adjacent_horizons: true,
        home_radio_mix: true,
        home_jump_back_in: true,
        home_forgotten_favorites: true,
        explore_spotlight_carousel: true,
        explore_category_grid: true,
        explore_new_releases: true,
        explore_top_charts: true,
        explore_thematic_collections: true,
    });
    details = $state<FlagDetail[]>([]);

    async init() {
        try {
            const list = await invoke<FlagDetail[]>("get_feature_flags_detail");
            if (Array.isArray(list)) {
                this.details = list;
                const nextMap: Record<string, boolean> = {};
                for (const d of list) {
                    nextMap[d.key] = d.enabled;
                }
                this.flags = nextMap;
            }
        } catch (e) {
            console.error("Failed to load feature flags:", e);
        } finally {
            this.loaded = true;
        }
    }

    isEnabled(key: FeatureFlagKey | string): boolean {
        return this.flags[key] ?? true;
    }

    async toggle(key: FeatureFlagKey | string, enabled: boolean) {
        this.flags[key] = enabled;
        const item = this.details.find(d => d.key === key);
        if (item) {
            item.enabled = enabled;
        }
        try {
            await invoke("set_feature_flag", { flag: key, enabled });
        } catch (e) {
            console.error(`Failed to set feature flag '${key}':`, e);
        }
    }

    async resetToDefaults() {
        try {
            await invoke("reset_feature_flags");
            await this.init();
        } catch (e) {
            console.error("Failed to reset feature flags:", e);
        }
    }
}

export const flagsStore = new FeatureFlagsStore();
