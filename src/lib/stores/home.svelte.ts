import { invoke } from "@tauri-apps/api/core";
import { audioStore } from "./audio.svelte";
import { toastStore } from "./toast.svelte";
import { libraryStore } from "./library.svelte";
import { settingsStore } from "./settings.svelte";

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

export interface IncompleteSessionItem {
    session_id: string;
    session_type: string; // "album" | "playlist" | "queue"
    title: string;
    subtitle: string;
    cover_art_url?: string | null;
    current_track_index: number;
    total_tracks: number;
    progress_percent: number;
    last_source_id?: string | null;
    provider_id: string;
}

export interface HeavyRotationArtistItem {
    artist: string;
    total_plays: number;
    total_duration_ms: number;
    avatar_url?: string | null;
}

export interface HeavyRotationAlbumItem {
    album_title: string;
    artist: string;
    cover_art_url?: string | null;
    total_plays: number;
}

export interface HeavyRotationShelf {
    artists: HeavyRotationArtistItem[];
    albums: HeavyRotationAlbumItem[];
}

export interface ColdStartSeedItem {
    artist: string;
    track_title: string;
    album_title?: string | null;
    cover_art_url?: string | null;
    file_path?: string | null;
    track_id: number;
}

export interface CanonicalSeedV1 {
    abi_version: number;
    canonical_key: string;
    title: string;
    artist: string;
    album?: string | null;
    isrc?: string | null;
    duration_ms?: number | null;
    native_id?: string | null;
    provider_id?: string | null;
}

export interface RadioMixCard {
    id: string;
    title: string;
    subtitle: string;
    category: "artist" | "temporal_mood";
    covers: string[];
    gradient_start: string;
    gradient_end: string;
    seed: CanonicalSeedV1;
}

export interface AdjacentHorizonPayload {
    dominant_genre_or_artist: string;
    suggested_genre: string;
    tagline: string;
    description: string;
    accent_color: string;
    seed: CanonicalSeedV1;
    preview_tracks: FederatedTrack[];
}

export interface HomeLocalShelves {
    quick_picks: FederatedTrack[];
    keep_listening: FederatedTrack[];
    jump_back_in: IncompleteSessionItem[];
    heavy_rotation: HeavyRotationShelf;
    forgotten_favorites: FederatedTrack[];
    cold_start_seeds: ColdStartSeedItem[];
}

export interface FederatedShelfResult {
    tracks: FederatedTrack[];
    failed_providers: string[];
    is_partial: boolean;
}

class HomeStore {
    constructor() {
        libraryStore.onLikeToggled = (key, isLiked) => {
            this.syncLikedStatus(key, isLiked);
        };
    }
    // ── Reactive Svelte 5 Rune State ─────────────────────────────
    quickPicks = $state<FederatedTrack[]>([]);
    keepListening = $state<FederatedTrack[]>([]);
    jumpBackIn = $state<IncompleteSessionItem[]>([]);
    heavyRotation = $state<HeavyRotationShelf>({ artists: [], albums: [] });
    forgottenFavorites = $state<FederatedTrack[]>([]);
    coldStartSeeds = $state<ColdStartSeedItem[]>([]);

    dailyDiscover = $state<FederatedTrack[]>([]);
    radioMixes = $state<RadioMixCard[]>([]);
    adjacentHorizons = $state<AdjacentHorizonPayload[]>([]);
    activeHorizonIndex = $state(0);
    adjacentHorizon = $derived(
        this.adjacentHorizons.length > 0
            ? this.adjacentHorizons[this.activeHorizonIndex] || this.adjacentHorizons[0]
            : null
    );
    failedProviders = $state<string[]>([]);

    nextHorizon() {
        if (this.adjacentHorizons.length > 0) {
            this.activeHorizonIndex = (this.activeHorizonIndex + 1) % this.adjacentHorizons.length;
        }
    }

    prevHorizon() {
        if (this.adjacentHorizons.length > 0) {
            this.activeHorizonIndex = (this.activeHorizonIndex - 1 + this.adjacentHorizons.length) % this.adjacentHorizons.length;
        }
    }

    selectHorizon(index: number) {
        if (index >= 0 && index < this.adjacentHorizons.length) {
            this.activeHorizonIndex = index;
        }
    }

    currentMood = $state("All");
    isLoadingLocal = $state(false);
    isLoadingRemote = $state(false);
    errorLocal = $state<string | null>(null);
    errorRemote = $state<string | null>(null);
    phase1Loaded = $state(false);
    phase2Loaded = $state(false);

    private currentFetchId = 0;

    // Derived flags for clean zero-state transitions
    hasTelemetry = $derived(
        this.quickPicks.length > 0 ||
        this.keepListening.length > 0 ||
        this.heavyRotation.artists.length > 0
    );

    async init() {
        if (!this.phase1Loaded || (!this.phase2Loaded && !this.isLoadingRemote)) {
            await this.loadHome();
        }
    }

    async selectMood(mood: string) {
        if (this.currentMood === mood) return;
        this.currentMood = mood;
        await this.loadHome(true);
    }

    async loadHome(force = false) {
        const fetchId = ++this.currentFetchId;
        const moodParam = this.currentMood === "All" ? null : this.currentMood;

        // ── Phase 1: Fast Local SQLite Telemetry Read (<10ms) ────────
        this.isLoadingLocal = true;
        this.errorLocal = null;
        try {
            const local: HomeLocalShelves = await invoke("get_home_local_shelves", { mood: moodParam });
            if (fetchId !== this.currentFetchId) return;

            this.quickPicks = local.quick_picks;
            this.keepListening = local.keep_listening;
            this.jumpBackIn = local.jump_back_in;
            this.heavyRotation = local.heavy_rotation;
            this.forgottenFavorites = local.forgotten_favorites;
            this.coldStartSeeds = local.cold_start_seeds;
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

        // ── Phase 2: Remote Federated Discovery & Radio Compilation ──
        this.isLoadingRemote = true;
        this.errorRemote = null;

        try {
            // Concurrently fetch Radios, Adjacent Horizons, and Federated Daily Discover
            const [radiosRes, horizonRes, remoteRes] = await Promise.allSettled([
                invoke<RadioMixCard[]>("get_home_radios", { mood: moodParam }),
                invoke<AdjacentHorizonPayload[]>("get_home_adjacent_horizons").catch(async () => {
                    const single = await invoke<AdjacentHorizonPayload | null>("get_home_adjacent_horizon");
                    return single ? [single] : [];
                }),
                invoke<FederatedShelfResult>("get_home_remote_shelves", { mood: moodParam }),
            ]);

            if (fetchId !== this.currentFetchId) return;

            if (radiosRes.status === "fulfilled") {
                this.radioMixes = radiosRes.value;
            }
            if (horizonRes.status === "fulfilled") {
                this.adjacentHorizons = Array.isArray(horizonRes.value) ? horizonRes.value : (horizonRes.value ? [horizonRes.value] : []);
                if (this.activeHorizonIndex >= this.adjacentHorizons.length) {
                    this.activeHorizonIndex = 0;
                }
            }
            if (remoteRes.status === "fulfilled") {
                this.dailyDiscover = remoteRes.value.tracks;
                this.failedProviders = remoteRes.value.failed_providers;
            }

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

    syncLikedStatus(canonicalKey: string, isLiked: boolean) {
        const keyNorm = canonicalKey.toLowerCase().trim();
        const updateList = (list: FederatedTrack[]) => {
            for (const item of list) {
                if (item.canonical_key.toLowerCase().trim() === keyNorm) {
                    item.liked = isLiked;
                }
            }
        };

        updateList(this.quickPicks);
        updateList(this.keepListening);
        updateList(this.forgottenFavorites);
        updateList(this.dailyDiscover);
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
            this.syncLikedStatus(track.canonical_key, newLiked);

            // Sync global libraryStore liked songs for PlayerBar and Playlist views
            await libraryStore.fetchLikedSongs();
        } catch (e) {
            console.error("Failed to toggle like:", e);
        }
    }

    async playFederatedTrack(track: FederatedTrack) {
        // 1. Local Source with known file path
        const localSource = track.sources.find(s => s.type === "Local" || (s as any).file_path);
        if (localSource && (localSource as any).file_path) {
            audioStore.handleTrackClick({
                id: (localSource as any).track_id || track.canonical_key,
                title: track.title,
                artist: track.artist,
                album: track.album || "",
                file_path: (localSource as any).file_path,
                provider_id: "local",
                cover_art_url: track.cover_art_url,
                duration_ms: track.duration_ms,
            });
            this.recordPlay(track, "local", (localSource as any).file_path);
            return;
        }

        // 2. Remote Source with known remote_track_id and valid remote provider
        const remoteSource = track.sources.find(s => s.type === "Remote" && (s as any).provider_id && (s as any).provider_id !== "local");
        const rawProviderId = (remoteSource as any)?.provider_id;
        const rawRemoteId = (remoteSource as any)?.remote_track_id;

        if (rawProviderId && rawRemoteId && !rawRemoteId.includes("::")) {
            audioStore.handleTrackClick({
                id: `remote-${rawProviderId}-${rawRemoteId}`,
                remote_track_id: rawRemoteId,
                title: track.title,
                artist: track.artist,
                album: track.album || "",
                stream_url: (remoteSource as any)?.stream_url || null,
                provider_id: rawProviderId,
                cover_art_url: track.cover_art_url,
                duration_ms: track.duration_ms,
            });
            this.recordPlay(track, rawProviderId, rawRemoteId);
            return;
        }

        // 3. Fallback resolution via configured defaultRemoteProvider
        const fallbackProvider = settingsStore.defaultRemoteProvider;
        if (!fallbackProvider || fallbackProvider === "local") {
            toastStore.show(`Local track file not found for ${track.title}`, "error");
            return;
        }

        try {
            toastStore.show(`Resolving ${track.title}...`, "info", 1500);
            
            // Sanitize query to remove YouTube telemetry artifacts, view counts, and redundant artist names
            const cleanArt = (track.artist || "")
                .replace(/\s*•\s*[\d.]+[MK]?\s*views.*$/i, "")
                .replace(/\s*•.*$/i, "")
                .replace(/\s*-\s*Topic$/i, "")
                .replace(/\s*VEVO$/i, "")
                .trim();
            
            const cleanTit = (track.title || "")
                .replace(/\s*•\s*[\d.]+[MK]?\s*views.*$/i, "")
                .replace(/\s*•.*$/i, "")
                .trim();

            const queryStr = (!cleanArt || cleanArt.toLowerCase() === "unknown" || cleanTit.toLowerCase().includes(cleanArt.toLowerCase()))
                ? cleanTit
                : `${cleanArt} ${cleanTit}`;

            let searchResults = await invoke<any[]>("search_provider", {
                providerId: fallbackProvider,
                query: queryStr,
            });

            if (!searchResults || searchResults.length === 0) {
                // Secondary fallback: search just the cleaned title
                if (cleanTit && cleanTit !== queryStr) {
                    searchResults = await invoke<any[]>("search_provider", {
                        providerId: fallbackProvider,
                        query: cleanTit,
                    });
                }
            }

            const target = (searchResults && searchResults.length > 0) ? searchResults[0] : null;
            if (target && target.id) {
                await audioStore.handleTrackClick({
                    id: `remote-${fallbackProvider}-${target.id}`,
                    remote_track_id: target.id,
                    title: track.title,
                    artist: track.artist,
                    album: track.album || target.album || "",
                    provider_id: fallbackProvider,
                    cover_art_url: track.cover_art_url || target.cover_art_url,
                    duration_ms: track.duration_ms || target.duration_ms,
                });
                this.recordPlay(track, fallbackProvider, target.id);
            } else {
                toastStore.show(`Could not resolve track for ${track.title}`, "error");
            }
        } catch (e) {
            console.error("Failed to resolve track:", e);
            toastStore.show(`Failed to play ${track.title}`, "error");
        }
    }

    async playRadioMix(card: RadioMixCard) {
        const fallbackProvider = settingsStore.defaultRemoteProvider;
        if (!fallbackProvider || fallbackProvider === "local") {
            toastStore.show(`Radios require an enabled streaming provider in Settings`, "info", 2500);
            return;
        }

        toastStore.show(`Starting ${card.title}...`, "info", 2000);
        try {
            // First check if a WASM provider can resolve a radio stream for this seed
            const radioResult: any = await invoke("get_radio_stream", {
                providerId: fallbackProvider,
                seed: card.seed,
            }).catch(() => null);

            if (radioResult && radioResult.tracks && radioResult.tracks.length > 0) {
                const tracks = radioResult.tracks.map((t: any) => ({
                    id: `remote-${fallbackProvider}-${t.id}`,
                    remote_track_id: t.id,
                    title: t.title,
                    artist: t.artist,
                    album: t.album || "",
                    provider_id: fallbackProvider,
                    cover_art_url: t.cover_art_url,
                    duration_ms: t.duration_ms,
                }));
                audioStore.setQueue(tracks, 0);
                return;
            }

            // Fallback: search for the seed artist / mood query
            const query = card.seed.artist || card.seed.title || card.title;
            const searchResults = await invoke<any[]>("search_provider", {
                providerId: fallbackProvider,
                query,
            });

            if (searchResults && searchResults.length > 0) {
                const tracks = searchResults.map((t: any) => ({
                    id: `remote-${fallbackProvider}-${t.id}`,
                    remote_track_id: t.id,
                    title: t.title,
                    artist: t.artist,
                    album: t.album || "",
                    provider_id: fallbackProvider,
                    cover_art_url: t.cover_art_url,
                    duration_ms: t.duration_ms,
                }));
                audioStore.setQueue(tracks, 0);
            } else {
                toastStore.show(`Could not start ${card.title}`, "error");
            }
        } catch (e) {
            console.error("Failed to start radio mix:", e);
            toastStore.show(`Failed to start ${card.title}`, "error");
        }
    }

    async resumeSession(session: IncompleteSessionItem) {
        if (session.session_type === "album" && session.last_source_id) {
            try {
                const albumId = parseInt(session.last_source_id, 10);
                const tracks: any = await invoke("get_album_tracks", { albumId, limit: 100, offset: 0 });
                if (tracks && tracks.length > 0) {
                    const queueTracks = tracks.map((t: any) => ({
                        id: `local-${t.id}`,
                        title: t.title,
                        artist: t.artist,
                        album: session.title,
                        file_path: t.file_path,
                        provider_id: "local",
                        cover_art_url: session.cover_art_url,
                    }));
                    const startIdx = Math.min(session.current_track_index, queueTracks.length - 1);
                    audioStore.setQueue(queueTracks, startIdx);
                    toastStore.show(`Resumed ${session.title}`, "info", 1500);
                }
            } catch (e) {
                console.error("Failed to resume album session:", e);
            }
        } else {
            toastStore.show(`Resuming ${session.title}...`, "info", 1500);
        }
    }

    async playColdStartSeed(seed: ColdStartSeedItem) {
        if (seed.file_path) {
            audioStore.setQueue([{
                id: `local-${seed.track_id}`,
                title: seed.track_title,
                artist: seed.artist,
                album: seed.album_title || "",
                file_path: seed.file_path,
                provider_id: "local",
                cover_art_url: seed.cover_art_url,
            }], 0);
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
