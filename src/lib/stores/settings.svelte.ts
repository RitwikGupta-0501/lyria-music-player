import { invoke } from "@tauri-apps/api/core";
import { DEFAULT_KEYMAP, type KeyAction, type KeyBinding } from "./keymap";

export class SettingsStore {
    loaded = $state(false);
    settings = $state<Record<string, string>>({});

    // Typed derived getters for all application settings
    glassyPlayerBar = $derived(this.settings["glassy_player_bar"] === "true");
    keepPlayingOnQueueClear = $derived(this.settings["keep_playing_on_queue_clear"] === "true");
    trackClickBehavior = $derived(
        (this.settings["track_click_behavior"] as "interrupt" | "clear" | "append") || "interrupt"
    );
    queueCompletionBehavior = $derived(
        (this.settings["queue_completion_behavior"] as "retain_stopped" | "pause_end" | "collapse_idle") || "retain_stopped"
    );
    logCollectionEnabled = $derived(this.settings["log_collection_enabled"] !== "false");
    discoveryArtistDiversityCeiling = $derived(
        parseInt(this.settings["discovery_artist_diversity_ceiling"] || "3", 10)
    );
    defaultRemoteProvider = $derived(this.settings["default_remote_provider"] || "local");

    async init() {
        try {
            const allSettings = await invoke<Record<string, string>>("get_all_settings");
            this.settings = allSettings || {};
            
            // Sync log collection active flag in Rust logger
            await invoke("set_log_collection_enabled", { enabled: this.logCollectionEnabled });
        } catch (e) {
            console.error("Failed to load settings:", e);
        } finally {
            this.loaded = true;
        }
    }

    async setSetting(key: string, value: string) {
        this.settings[key] = value;
        try {
            await invoke("set_setting", { key, value });
        } catch (e) {
            console.error(`Failed to save setting '${key}':`, e);
        }
    }

    async setGlassyPlayerBar(val: boolean) {
        await this.setSetting("glassy_player_bar", val ? "true" : "false");
    }

    async setKeepPlayingOnQueueClear(val: boolean) {
        await this.setSetting("keep_playing_on_queue_clear", val ? "true" : "false");
    }

    async setTrackClickBehavior(behavior: "interrupt" | "clear" | "append") {
        await this.setSetting("track_click_behavior", behavior);
    }

    async setQueueCompletionBehavior(behavior: "retain_stopped" | "pause_end" | "collapse_idle") {
        await this.setSetting("queue_completion_behavior", behavior);
    }

    async setLogCollectionEnabled(enabled: boolean) {
        await this.setSetting("log_collection_enabled", enabled ? "true" : "false");
        try {
            await invoke("set_log_collection_enabled", { enabled });
        } catch (e) {
            console.error("Failed to update logger active state:", e);
        }
    }

    async setDiscoveryArtistDiversityCeiling(ceiling: number) {
        await this.setSetting("discovery_artist_diversity_ceiling", ceiling.toString());
    }

    async setDefaultRemoteProvider(providerId: string) {
        await this.setSetting("default_remote_provider", providerId);
    }

    getKeymap(): Record<KeyAction, KeyBinding> {
        const customJson = this.settings["custom_keymap"];
        const keymap: Record<KeyAction, KeyBinding> = {
            refreshRecommendations: { ...DEFAULT_KEYMAP.refreshRecommendations.binding },
            playPause: { ...DEFAULT_KEYMAP.playPause.binding },
            prevTrack: { ...DEFAULT_KEYMAP.prevTrack.binding },
            nextTrack: { ...DEFAULT_KEYMAP.nextTrack.binding },
            seekBack: { ...DEFAULT_KEYMAP.seekBack.binding },
            seekForward: { ...DEFAULT_KEYMAP.seekForward.binding },
            volumeUp: { ...DEFAULT_KEYMAP.volumeUp.binding },
            volumeDown: { ...DEFAULT_KEYMAP.volumeDown.binding },
            toggleShuffle: { ...DEFAULT_KEYMAP.toggleShuffle.binding },
            cycleRepeat: { ...DEFAULT_KEYMAP.cycleRepeat.binding },
            search: { ...DEFAULT_KEYMAP.search.binding },
            escape: { ...DEFAULT_KEYMAP.escape.binding },
        };

        if (customJson) {
            try {
                const parsed = JSON.parse(customJson);
                for (const [action, binding] of Object.entries(parsed)) {
                    if (action in keymap && binding && typeof binding === "object") {
                        keymap[action as KeyAction] = binding as KeyBinding;
                    }
                }
            } catch (e) {
                console.error("Failed to parse custom keymap:", e);
            }
        }

        return keymap;
    }

    getKeybinding(action: KeyAction): KeyBinding {
        return this.getKeymap()[action] || DEFAULT_KEYMAP[action].binding;
    }

    async setKeybinding(action: KeyAction, binding: KeyBinding) {
        const currentMap = this.getKeymap();
        currentMap[action] = binding;
        await this.setSetting("custom_keymap", JSON.stringify(currentMap));
    }

    async resetKeymap() {
        await this.setSetting("custom_keymap", "");
    }
}

export const settingsStore = new SettingsStore();
