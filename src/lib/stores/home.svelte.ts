import { invoke } from "@tauri-apps/api/core";
import { audioStore } from "./audio.svelte";
import { toastStore } from "./toast.svelte";

export interface CanonicalSong {
    id: number;
    canonical_key: string;
    title: string;
    artist: string;
    album: string | null;
    cover_art_url: string | null;
    play_count: number;
    last_played_at: string | null;
    liked: boolean;
    local_track_id: number | null;
    local_file_path: string | null;
    last_provider_id: string;
    last_source_id: string;
    duration_ms: number | null;
}

export interface HomeFeedPayload {
    quick_picks: CanonicalSong[];
    keep_listening: CanonicalSong[];
    forgotten_favorites: CanonicalSong[];
    discover_seeds: CanonicalSong[];
}

export interface DailyDiscoverShelf {
    seed: CanonicalSong;
    contextTag: string;
    tracks: any[];
    loading: boolean;
}

class HomeStore {
    quickPicks = $state<CanonicalSong[]>([]);
    keepListening = $state<CanonicalSong[]>([]);
    forgottenFavorites = $state<CanonicalSong[]>([]);
    discoverShelves = $state<DailyDiscoverShelf[]>([]);
    
    phase1Loaded = $state(false);
    phase2Loaded = $state(false);
    isRefreshing = $state(false);

    async init() {
        if (!this.phase1Loaded) {
            await this.loadHome();
        }
    }

    async loadHome(force = false) {
        if (this.isRefreshing) return;
        this.isRefreshing = true;

        try {
            // ── Phase 1: Fast Local & Cached Telemetry (Instant Render) ────────
            const feed: HomeFeedPayload = await invoke("get_home_feed");
            this.quickPicks = feed.quick_picks;
            this.keepListening = feed.keep_listening;
            this.forgottenFavorites = feed.forgotten_favorites;
            this.phase1Loaded = true;

            // ── Phase 2: Background Discovery (Seeds -> Online Recommendations) ─
            if (feed.discover_seeds && feed.discover_seeds.length > 0) {
                this.loadDailyDiscover(feed.discover_seeds);
            }
            this.phase2Loaded = true;
        } catch (e) {
            console.error("Failed to load home feed:", e);
        } finally {
            this.isRefreshing = false;
        }
    }

    private async loadDailyDiscover(seeds: CanonicalSong[]) {
        const shelves: DailyDiscoverShelf[] = seeds.map(s => ({
            seed: s,
            contextTag: `Because you listen to ${s.title}`,
            tracks: [],
            loading: true,
        }));
        this.discoverShelves = shelves;

        function sanitizeSeedQuery(artist?: string | null, title?: string | null): string {
            const a = (artist || "").trim();
            let t = (title || "").trim();
            t = t.replace(/\s*[\(\[](official\s*(music\s*)?video|audio|remastered|lyric\s*video|official\s*audio|hd|4k)[\)\]]/gi, "").trim();
            if (a && t.toLowerCase().startsWith(a.toLowerCase())) {
                return t;
            }
            return a ? `${a} ${t}` : t;
        }

        // Fetch related tracks for each seed in parallel
        for (let i = 0; i < seeds.length; i++) {
            const seed = seeds[i];
            const providerId = (seed.last_provider_id && seed.last_provider_id !== "local") ? seed.last_provider_id : "youtube-wasm";
            const query = sanitizeSeedQuery(seed.artist, seed.title);
            
            try {
                const results: any[] = await invoke("search_provider", {
                    providerId,
                    query,
                });
                if (results && results.length > 0) {
                    this.discoverShelves[i].tracks = results.slice(0, 10).map(r => ({
                        ...r,
                        provider_id: providerId,
                    }));
                }
            } catch (e) {
                console.error(`Failed to fetch related for ${seed.title}:`, e);
            } finally {
                this.discoverShelves[i].loading = false;
            }
        }
    }

    async toggleLike(song: CanonicalSong) {
        try {
            const newLiked = await invoke<boolean>("toggle_track_like", {
                canonicalKey: song.canonical_key,
            });
            song.liked = newLiked;
        } catch (e) {
            console.error("Failed to toggle like:", e);
        }
    }

    async playCanonicalSong(song: CanonicalSong) {
        // 1. If local lossless copy exists on disk, play immediately
        if (song.local_file_path) {
            audioStore.setQueue([{
                id: `local-${song.local_track_id}`,
                title: song.title,
                artist: song.artist,
                album: song.album || '',
                file_path: song.local_file_path,
                provider_id: 'local',
                cover_art_url: song.cover_art_url,
                duration_ms: song.duration_ms,
            }], 0);
            this.recordPlay(song);
            return;
        }

        // 2. If remote stream, resolve fresh authenticated URL on-demand
        try {
            toastStore.show(`Resolving stream for ${song.title}...`, 'info', 1500);
            const resolved: any = await invoke("search_provider", {
                providerId: song.last_provider_id || 'youtube-wasm',
                query: `${song.artist} ${song.title}`,
            });

            const target = (resolved && resolved[0]) ? resolved[0] : null;
            if (target && target.stream_url) {
                audioStore.setQueue([{
                    id: `remote-${song.last_provider_id}-${song.last_source_id}`,
                    title: song.title,
                    artist: song.artist,
                    album: song.album || '',
                    file_path: target.stream_url,
                    stream_url: target.stream_url,
                    provider_id: song.last_provider_id,
                    cover_art_url: song.cover_art_url || target.cover_art_url,
                    duration_ms: song.duration_ms || target.duration_ms,
                }], 0);
                this.recordPlay(song);
            } else {
                toastStore.show(`Could not resolve stream for ${song.title}`, 'error');
            }
        } catch (e) {
            console.error("Failed to resolve canonical track:", e);
            toastStore.show(`Failed to play ${song.title}`, 'error');
        }
    }

    async recordPlay(song: CanonicalSong) {
        try {
            await invoke("record_track_play", {
                title: song.title,
                artist: song.artist,
                album: song.album,
                coverArtUrl: song.cover_art_url,
                providerId: song.last_provider_id,
                sourceId: song.last_source_id,
                durationMs: song.duration_ms,
            });
        } catch (e) {
            console.error("Failed to record play event:", e);
        }
    }
}

export const homeStore = new HomeStore();
