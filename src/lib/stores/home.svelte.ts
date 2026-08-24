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
    // ── Reactive Svelte 5 Rune State ─────────────────────────────
    quickPicks = $state<FederatedTrack[]>([]);
    keepListening = $state<FederatedTrack[]>([]);
    jumpBackIn = $state<IncompleteSessionItem[]>([]);
    heavyRotation = $state<HeavyRotationShelf>({ artists: [], albums: [] });
    forgottenFavorites = $state<FederatedTrack[]>([]);
    coldStartSeeds = $state<ColdStartSeedItem[]>([]);

    dailyDiscover = $state<FederatedTrack[]>([]);
    radioMixes = $state<RadioMixCard[]>([]);
    adjacentHorizon = $state<AdjacentHorizonPayload | null>(null);
    failedProviders = $state<string[]>([]);

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
        if (!this.phase1Loaded) {
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
                invoke<AdjacentHorizonPayload | null>("get_home_adjacent_horizon"),
                invoke<FederatedShelfResult>("get_home_remote_shelves", { mood: moodParam }),
            ]);

            if (fetchId !== this.currentFetchId) return;

            if (radiosRes.status === "fulfilled") {
                this.radioMixes = radiosRes.value;
            }
            if (horizonRes.status === "fulfilled") {
                this.adjacentHorizon = horizonRes.value;
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

    async playRadioMix(card: RadioMixCard) {
        toastStore.show(`Starting ${card.title}...`, "info", 2000);
        try {
            // First check if a WASM provider can resolve a radio stream for this seed
            const radioResult: any = await invoke("get_radio_stream", {
                providerId: "youtube-wasm",
                seed: card.seed,
            }).catch(() => null);

            if (radioResult && radioResult.tracks && radioResult.tracks.length > 0) {
                const tracks = radioResult.tracks.map((t: any) => ({
                    id: `remote-youtube-wasm-${t.id}`,
                    title: t.title,
                    artist: t.artist,
                    album: t.album || "",
                    file_path: t.stream_url || "",
                    stream_url: t.stream_url,
                    provider_id: "youtube-wasm",
                    cover_art_url: t.cover_art_url,
                    duration_ms: t.duration_ms,
                }));
                audioStore.setQueue(tracks, 0);
                return;
            }

            // Fallback: search for the seed artist / mood query
            const query = card.seed.artist || card.seed.title || card.title;
            const searchResults: any = await invoke("search_provider", {
                providerId: "youtube-wasm",
                query,
            });

            if (searchResults && searchResults.length > 0) {
                const tracks = searchResults.map((t: any) => ({
                    id: `remote-youtube-wasm-${t.id}`,
                    title: t.title,
                    artist: t.artist,
                    album: t.album || "",
                    file_path: t.stream_url || "",
                    stream_url: t.stream_url,
                    provider_id: "youtube-wasm",
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
