import { invoke } from "@tauri-apps/api/core";

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
}

export const settingsStore = new SettingsStore();
