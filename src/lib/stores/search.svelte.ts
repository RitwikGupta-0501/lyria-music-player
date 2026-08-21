import { invoke } from "@tauri-apps/api/core";
import { audioStore } from "./audio.svelte";
import { toastStore } from "./toast.svelte";
import { homeStore } from "./home.svelte";

export interface ResolvedUrlResult {
    provider_id: string;
    provider_name: string;
    track: {
        id: string;
        title: string;
        artist: string;
        album: string | null;
        cover_art_url: string | null;
        stream_url: string | null;
        quality_hint: string | null;
        duration_ms: number | null;
    };
}

class SearchStore {
    query = $state("");
    activeFilter = $state<"all" | "local" | "online" | "tracks" | "albums">("all");
    
    localTracks = $state<any[]>([]);
    localAlbums = $state<any[]>([]);
    remoteTracks = $state<any[]>([]);
    resolvedUrl = $state<ResolvedUrlResult | null>(null);
    
    isSearching = $state(false);
    isResolvingUrl = $state(false);
    
    private debounceTimer: ReturnType<typeof setTimeout> | null = null;
    private currentRequestId = 0;

    handleQueryChange(newQuery: string) {
        this.query = newQuery;
        if (this.debounceTimer) clearTimeout(this.debounceTimer);

        const trimmed = newQuery.trim();
        if (!trimmed) {
            this.clear();
            return;
        }

        // Check if query is an HTTP URL
        if (trimmed.startsWith("http://") || trimmed.startsWith("https://")) {
            this.resolveUrl(trimmed);
            return;
        } else {
            this.resolvedUrl = null;
        }

        this.debounceTimer = setTimeout(() => {
            this.performSearch(trimmed);
        }, 220);
    }

    async resolveUrl(url: string) {
        this.isResolvingUrl = true;
        this.resolvedUrl = null;
        try {
            const res = await invoke<ResolvedUrlResult | null>("resolve_stream_url", { url });
            this.resolvedUrl = res;
        } catch (e) {
            console.error("Failed to resolve URL via extensions:", e);
        } finally {
            this.isResolvingUrl = false;
        }
    }

    async performSearch(query: string) {
        const reqId = ++this.currentRequestId;
        this.isSearching = true;

        try {
            // 1. Instant Local Library Search
            const local = await invoke<any[]>("search_library", { query, limit: 15 });
            if (reqId !== this.currentRequestId) return;
            this.localTracks = local || [];

            // 2. Parallel Remote Extension Search
            const providers: any[] = await invoke("get_providers");
            const enabled = providers.filter(p => p.status === 'enabled' && (p.capabilities?.includes('search') || p.id === 'youtube-wasm'));

            const promises = enabled.map(p =>
                invoke<any[]>("search_provider", { providerId: p.id, query })
                    .then(res => (res || []).map(r => ({ ...r, provider_id: p.id, provider_name: p.name })))
                    .catch(e => {
                        console.error(`Search error for ${p.name}:`, e);
                        return [];
                    })
            );

            const allRemote = (await Promise.all(promises)).flat();
            if (reqId !== this.currentRequestId) return;
            this.remoteTracks = allRemote;
        } catch (e) {
            console.error("Search execution failed:", e);
        } finally {
            if (reqId === this.currentRequestId) {
                this.isSearching = false;
            }
        }
    }

    async playResolvedUrl() {
        if (!this.resolvedUrl) return;
        const { provider_id, track } = this.resolvedUrl;
        
        try {
            toastStore.show(`Resolving stream for ${track.title}...`, 'info', 1500);
            const resolved: any = await invoke("resolve_track", {
                providerId: provider_id,
                trackId: track.id,
            });

            if (resolved && resolved.stream_url) {
                const trackPayload = {
                    id: track.id,
                    remote_track_id: track.id,
                    title: track.title,
                    artist: track.artist,
                    album: track.album || '',
                    file_path: resolved.stream_url,
                    stream_url: resolved.stream_url,
                    provider_id,
                    cover_art_url: track.cover_art_url,
                    duration_ms: track.duration_ms,
                };
                await audioStore.handleTrackClick(trackPayload);

                homeStore.recordPlay({
                    id: 0,
                    canonical_key: '',
                    title: track.title,
                    artist: track.artist,
                    album: track.album || null,
                    cover_art_url: track.cover_art_url || null,
                    play_count: 1,
                    last_played_at: null,
                    liked: false,
                    local_track_id: null,
                    local_file_path: null,
                    last_provider_id: provider_id,
                    last_source_id: track.id,
                    duration_ms: track.duration_ms || null,
                });
            }
        } catch (e) {
            console.error("Failed to play resolved URL track:", e);
            toastStore.show(`Failed to play ${track.title}`, 'error');
        }
    }

    async playLocalTrack(track: any) {
        const trackPayload = {
            id: track.id,
            track_id: track.id,
            title: track.title,
            artist: track.artist,
            album: track.album || '',
            file_path: track.file_path,
            track_number: track.track_number,
            is_local: true,
        };
        await audioStore.handleTrackClick(trackPayload);

        homeStore.recordPlay({
            id: 0,
            canonical_key: '',
            title: track.title,
            artist: track.artist,
            album: track.album || null,
            cover_art_url: null,
            play_count: 1,
            last_played_at: null,
            liked: false,
            local_track_id: track.id,
            local_file_path: track.file_path,
            last_provider_id: 'local',
            last_source_id: track.file_path,
            duration_ms: null,
        });
    }

    async playRemoteTrack(track: any) {
        const providerId = track.provider_id || "youtube-wasm";
        try {
            const trackPayload = {
                id: track.id,
                remote_track_id: track.id,
                title: track.title,
                artist: track.artist,
                album: track.album || undefined,
                duration_ms: track.duration_ms || 210000,
                cover_art_url: track.cover_art_url || undefined,
                provider_id: providerId,
            };
            await audioStore.handleTrackClick(trackPayload);

            homeStore.recordPlay({
                id: 0,
                canonical_key: '',
                title: track.title,
                artist: track.artist,
                album: track.album || null,
                cover_art_url: track.cover_art_url || null,
                play_count: 1,
                last_played_at: null,
                liked: false,
                local_track_id: null,
                local_file_path: null,
                last_provider_id: providerId,
                last_source_id: track.id,
                duration_ms: track.duration_ms || null,
            });
        } catch (e) {
            console.error("Failed to play remote track:", e);
            toastStore.show(`Failed to play ${track.title}`, 'error');
        }
    }

    clear() {
        this.query = "";
        this.localTracks = [];
        this.remoteTracks = [];
        this.resolvedUrl = null;
        this.isSearching = false;
        this.isResolvingUrl = false;
    }
}

export const searchStore = new SearchStore();
