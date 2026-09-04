import { invoke } from "@tauri-apps/api/core";

export type FeatureFlagKey = 
    | "shuffle_mode"
    | "reorder_queue"
    | "virtual_scrolling"
    | "persistent_queue"
    | "glassy_player_bar"
    | "experimental_dsp"
    | "explore_spotlight_carousel"
    | "offline_artwork_caching";

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
        shuffle_mode: true,
        reorder_queue: true,
        virtual_scrolling: true,
        persistent_queue: true,
        glassy_player_bar: true,
        experimental_dsp: false,
        explore_spotlight_carousel: true,
        offline_artwork_caching: true,
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
        return this.flags[key] ?? false;
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
