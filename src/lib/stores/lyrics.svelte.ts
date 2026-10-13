import { invoke } from "@tauri-apps/api/core";
import { audioStore } from "./audio.svelte";

export interface LyricLine {
    time_ms: number;
    text: string;
    end_time_ms?: number;
}

export interface LyricsResult {
    provider_id: string;
    provider_name: string;
    format: "synced" | "plain";
    lines: LyricLine[];
    raw_plain?: string;
    raw_lrc?: string;
    offset_ms: number;
    instrumental: boolean;
    copyright?: string;
}

export interface LyricsProviderMeta {
    id: string;
    name: string;
    version: string;
    author: string;
    is_preferred: boolean;
    priority: number;
    status: string;
}

export class LyricsStore {
    currentLyrics = $state<LyricsResult | null>(null);
    isLoading = $state(false);
    error = $state<string | null>(null);
    providers = $state<LyricsProviderMeta[]>([]);
    preferredProvider = $state<string | null>(null);
    activeTrackKey = $state<string | null>(null);
    autoScroll = $state(true);

    // Active line index derived directly from audioStore.currentTime via O(log N) binary search
    activeLineIndex = $derived.by(() => {
        const lines = this.currentLyrics?.lines;
        if (!this.currentLyrics || this.currentLyrics.format !== "synced" || !lines?.length) {
            return -1;
        }
        const curMs = audioStore.currentTime * 1000;
        let low = 0;
        let high = lines.length - 1;
        let active = -1;

        while (low <= high) {
            const mid = (low + high) >> 1;
            if (lines[mid].time_ms <= curMs) {
                active = mid;
                low = mid + 1;
            } else {
                high = mid - 1;
            }
        }
        return active;
    });

    hasLyrics = $derived(
        !!this.currentLyrics && (this.currentLyrics.lines?.length > 0 || !!this.currentLyrics.raw_plain)
    );

    isSynced = $derived(
        this.currentLyrics?.format === "synced" && (this.currentLyrics.lines?.length ?? 0) > 0
    );

    async init() {
        await this.fetchProviders();
    }

    async fetchProviders() {
        try {
            const metas = await invoke<LyricsProviderMeta[]>("get_lyrics_providers");
            this.providers = metas || [];
            const preferred = this.providers.find((p) => p.is_preferred);
            this.preferredProvider = preferred ? preferred.id : null;
        } catch (e) {
            console.error("Failed to load lyrics providers:", e);
        }
    }

    async fetchForCurrentTrack(forceProvider?: string) {
        const track = audioStore.currentQueueTrack;
        if (!track || !track.title) {
            this.currentLyrics = null;
            this.activeTrackKey = null;
            this.error = null;
            return;
        }

        const trackKey = `${track.artist || ""}::${track.title}`;
        if (!forceProvider && this.activeTrackKey === trackKey && this.currentLyrics) {
            return;
        }

        this.activeTrackKey = trackKey;
        this.isLoading = true;
        this.error = null;

        try {
            const query = {
                title: track.title,
                artist: track.artist || "Unknown",
                album: undefined,
                duration_ms: audioStore.duration > 0 ? Math.round(audioStore.duration * 1000) : undefined,
            };

            const res = await invoke<LyricsResult | null>("get_lyrics", {
                query,
                providerOverride: forceProvider || null,
            });

            if (this.activeTrackKey === trackKey) {
                this.currentLyrics = res;
            }
        } catch (err: unknown) {
            if (this.activeTrackKey === trackKey) {
                this.error = typeof err === "string" ? err : (err as Error)?.message || "Failed to load lyrics";
                this.currentLyrics = null;
            }
        } finally {
            if (this.activeTrackKey === trackKey) {
                this.isLoading = false;
            }
        }
    }

    seekToLine(index: number, preRollMs = 0) {
        if (!this.currentLyrics || !this.currentLyrics.lines || index < 0 || index >= this.currentLyrics.lines.length) {
            return;
        }
        const line = this.currentLyrics.lines[index];
        // Subtract lead-in pre-roll if provided so the vocal attack / consonant isn't clipped
        const targetMs = Math.max(0, line.time_ms - preRollMs);
        audioStore.seek(targetMs / 1000);
    }

    async setPreferredProvider(providerId: string | null) {
        this.preferredProvider = providerId;
        try {
            await invoke("set_preferred_lyrics_provider", { providerId });
            await this.fetchProviders();
            await this.fetchForCurrentTrack();
        } catch (e) {
            console.error("Failed to set preferred lyrics provider:", e);
        }
    }

    async clearCache() {
        try {
            await invoke("clear_lyrics_cache", { canonicalKey: null });
            this.currentLyrics = null;
            this.activeTrackKey = null;
            await this.fetchForCurrentTrack();
        } catch (e) {
            console.error("Failed to clear lyrics cache:", e);
        }
    }
}

export const lyricsStore = new LyricsStore();
