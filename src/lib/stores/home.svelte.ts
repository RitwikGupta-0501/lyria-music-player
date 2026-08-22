import { invoke } from "@tauri-apps/api/core";
import { audioStore } from "./audio.svelte";
import { toastStore } from "./toast.svelte";

export interface TrackSourceInfo {
    type: "Local" | "Remote";
    provider_id?: string;
    remote_track_id?: string;
    stream_url?: string | null;
    quality_hint?: string | null;
    cover_art_url?: string | null;
    duration_ms?: number | null;
    track_id?: number;
    file_path?: string;
    album_id?: number | null;
}

export interface FederatedTrack {
    canonical_key: string;
    title: string;
    artist: string;
    album: string | null;
    isrc: string | null;
    cover_art_url: string | null;
    duration_ms: number | null;
    play_count: number;
    sources: TrackSourceInfo[];
    liked?: boolean;
    seed_provenance?: string | null;
}

export interface HomeLocalShelves {
    quick_picks: FederatedTrack[];
    keep_listening: FederatedTrack[];
    forgotten_favorites: FederatedTrack[];
}

export interface FederatedShelfResult {
    tracks: FederatedTrack[];
    failed_providers: string[];
    is_partial: boolean;
}

class HomeStore {
    quickPicks = $state<FederatedTrack[]>([]);
    keepListening = $state<FederatedTrack[]>([]);
    forgottenFavorites = $state<FederatedTrack[]>([]);
    dailyDiscover = $state<FederatedTrack[]>([]);
    failedProviders = $state<string[]>([]);
    
    isLoadingLocal = $state(false);
    isLoadingRemote = $state(false);
    errorLocal = $state<string | null>(null);
    errorRemote = $state<string | null>(null);
    phase1Loaded = $state(false);
    phase2Loaded = $state(false);

    private currentFetchId = 0;

    async init() {
        if (!this.phase1Loaded) {
            await this.loadHome();
        }
    }

    async loadHome(force = false) {
        const fetchId = ++this.currentFetchId;

        // ── Phase 1: Fast Local Telemetry Read (<10ms) ────────
        this.isLoadingLocal = true;
        this.errorLocal = null;
        try {
            const local: HomeLocalShelves = await invoke("get_home_local_shelves");
            if (fetchId !== this.currentFetchId) return; // Discard stale response
            this.quickPicks = local.quick_picks;
            this.keepListening = local.keep_listening;
            this.forgottenFavorites = local.forgotten_favorites;
            this.phase1Loaded = true;
        } catch (e: any) {
            if (fetchId === this.currentFetchId) {
                this.errorLocal = e?.toString() || "Failed to load local shelves";
                console.error("Failed to load local home shelves:", e);
            }
        } finally {
            if (fetchId === this.currentFetchId) {
                this.isLoadingLocal = false;
            }
        }

        // ── Phase 2: Remote Federated Discovery ────────
        this.isLoadingRemote = true;
        this.errorRemote = null;
        try {
            const remote: FederatedShelfResult = await invoke("get_home_remote_shelves");
            if (fetchId !== this.currentFetchId) return; // Discard stale response
            this.dailyDiscover = remote.tracks;
            this.failedProviders = remote.failed_providers;
            this.phase2Loaded = true;
        } catch (e: any) {
            if (fetchId === this.currentFetchId) {
                this.errorRemote = e?.toString() || "Failed to fetch remote recommendations";
                console.error("Failed to load remote recommendations:", e);
            }
        } finally {
            if (fetchId === this.currentFetchId) {
                this.isLoadingRemote = false;
            }
        }
    }

    async toggleLike(track: FederatedTrack) {
        try {
            const newLiked = await invoke<boolean>("toggle_track_like", {
                canonicalKey: track.canonical_key,
                title: track.title,
                artist: track.artist,
                album: track.album,
                coverArtUrl: track.cover_art_url,
                durationMs: track.duration_ms,
            });
            track.liked = newLiked;
        } catch (e) {
            console.error("Failed to toggle like:", e);
        }
    }

    async playFederatedTrack(track: FederatedTrack) {
        // Find preferred local source first, then remote
        const localSource = track.sources.find(s => s.type === "Local" || s.file_path);
        if (localSource && localSource.file_path) {
            audioStore.setQueue([{
                id: `local-${localSource.track_id || track.canonical_key}`,
                title: track.title,
                artist: track.artist,
                album: track.album || "",
                file_path: localSource.file_path,
                provider_id: "local",
                cover_art_url: track.cover_art_url,
                duration_ms: track.duration_ms,
            }], 0);
            this.recordPlay(track, "local", localSource.file_path);
            return;
        }

        const remoteSource = track.sources.find(s => s.type === "Remote" || s.provider_id);
        const providerId = remoteSource?.provider_id || "youtube-wasm";
        const sourceId = remoteSource?.remote_track_id || track.canonical_key;

        // If remote stream URL is already known
        if (remoteSource?.stream_url) {
            audioStore.setQueue([{
                id: `remote-${providerId}-${sourceId}`,
                title: track.title,
                artist: track.artist,
                album: track.album || "",
                file_path: remoteSource.stream_url,
                stream_url: remoteSource.stream_url,
                provider_id: providerId,
                cover_art_url: track.cover_art_url,
                duration_ms: track.duration_ms,
            }], 0);
            this.recordPlay(track, providerId, sourceId);
            return;
        }

        // Otherwise resolve stream URL on demand
        try {
            toastStore.show(`Resolving stream for ${track.title}...`, "info", 1500);
            const resolved: any = await invoke("search_provider", {
                providerId,
                query: `${track.artist} ${track.title}`,
            });

            const target = (resolved && resolved[0]) ? resolved[0] : null;
            if (target && target.stream_url) {
                audioStore.setQueue([{
                    id: `remote-${providerId}-${target.id || sourceId}`,
                    title: track.title,
                    artist: track.artist,
                    album: track.album || "",
                    file_path: target.stream_url,
                    stream_url: target.stream_url,
                    provider_id: providerId,
                    cover_art_url: track.cover_art_url || target.cover_art_url,
                    duration_ms: track.duration_ms || target.duration_ms,
                }], 0);
                this.recordPlay(track, providerId, target.id || sourceId);
            } else {
                toastStore.show(`Could not resolve stream for ${track.title}`, "error");
            }
        } catch (e) {
            console.error("Failed to resolve stream:", e);
            toastStore.show(`Failed to play ${track.title}`, "error");
        }
    }

    async recordPlay(track: any, providerId: string = "local", sourceId: string = "") {
        try {
            await invoke("record_track_play", {
                title: track.title,
                artist: track.artist,
                album: track.album,
                coverArtUrl: track.cover_art_url,
                providerId,
                sourceId,
                durationMs: track.duration_ms,
            });
        } catch (e) {
            console.error("Failed to record play event:", e);
        }
    }
}

export const homeStore = new HomeStore();
