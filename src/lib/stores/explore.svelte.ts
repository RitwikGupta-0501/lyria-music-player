export interface DrawerCollectionTrack {
    id: string | number;
    title: string;
    artist?: string | null;
    album?: string | null;
    file_path?: string | null;
    cover_art_url?: string | null;
    provider_id?: string;
    duration_ms?: number | null;
    track_number?: number | null;
    liked?: boolean;
    canonical_key?: string;
}

export interface DrawerCollection {
    kind: 'album' | 'playlist';
    source: 'local' | 'remote';
    id: string | number;
    title: string;
    subtitle?: string;
    cover_art_url?: string | null;
    cover_art_path?: string | null;
    provider_id?: string;
    tracks?: DrawerCollectionTrack[];
    rawLocal?: unknown;
    rawRemote?: AlbumDetailResult | null;
}

import { invoke } from "@tauri-apps/api/core";
import { audioStore } from "./audio.svelte";
import { libraryStore } from "./library.svelte";
import { toastStore } from "./toast.svelte";
import { settingsStore } from "./settings.svelte";
import type { AdjacentHorizonPayload, RadioMixCard } from "./home.svelte";

export interface TrackResult {
    id: string;
    title: string;
    artist: string;
    album?: string | null;
    cover_art_url?: string | null;
    stream_url?: string | null;
    file_path?: string | null;
    is_local?: boolean;
    quality_hint?: string | null;
    duration_ms?: number | null;
    provider_id?: string;
    provider_name?: string;
    sources?: Array<{
        provider_id: string;
        provider_name: string;
    }>;
}

export interface AlbumItem {
    id: string;
    title: string;
    artist: string;
    year?: string | null;
    cover_art_url?: string | null;
    is_local?: boolean;
    provider_id?: string;
    provider_name?: string;
}

export interface PlaylistItem {
    id: string;
    title: string;
    author?: string | null;
    item_count?: number | null;
    cover_art_url?: string | null;
    provider_id?: string;
    provider_name?: string;
}

export interface GenreItem {
    id: string;
    title: string;
    endpoint_params?: string | null;
    color_hex?: string | null;
    provider_id?: string;
    provider_name?: string;
}

export interface ArtistItem {
    id: string;
    name: string;
    avatar_url?: string | null;
    subscribers?: string | null;
    provider_id?: string;
    provider_name?: string;
}

export interface EditorialSpotlight {
    id: string;
    title: string;
    artist: string;
    cover_art_url?: string | null;
    description?: string | null;
    release_year?: string | null;
    track_count?: number | null;
    provider_id?: string;
    provider_name?: string;
}

export interface TopResultItem {
    id: string;
    title: string;
    subtitle: string;
    item_type: string; // "artist", "album", "song"
    cover_art_url?: string | null;
    is_local?: boolean;
    provider_id?: string | null;
    provider_name?: string | null;
}

export type SearchItem = 
    | { type: "TopResult"; data: TopResultItem }
    | { type: "Track"; data: TrackResult }
    | { type: "Album"; data: AlbumItem }
    | { type: "Artist"; data: ArtistItem }
    | { type: "Playlist"; data: PlaylistItem };

export interface SearchCategorySection {
    category: string; // "Top Result", "Songs", "Albums", "Artists", "Playlists", "Videos"
    items: SearchItem[];
}


export interface AlbumDetailResult {
    id: string;
    title: string;
    artist: string;
    year?: string | null;
    description?: string | null;
    cover_art_url?: string | null;
    track_count?: number | null;
    tracks: TrackResult[];
    provider_id?: string;
}

export interface ArtistDetailResult {
    id: string;
    name: string;
    avatar_url?: string | null;
    banner_url?: string | null;
    subscribers?: string | null;
    bio?: string | null;
    top_tracks: TrackResult[];
    albums: AlbumItem[];
    singles: AlbumItem[];
    videos?: TrackResult[];
    featured_on?: PlaylistItem[];
    similar_artists?: ArtistItem[];
    provider_id?: string;
}

export interface CategorizedSearchResult {
    sections: SearchCategorySection[];
    continuation_token?: string | null;
}

export interface SourceOption {
    id: string;
    name: string;
    badge: string;
}

export const AVAILABLE_SOURCES: SourceOption[] = [
    { id: "local", name: "Local Library", badge: "LOSSLESS" },
    { id: "youtube-wasm", name: "YouTube Music", badge: "WASM" },
];

export type ModuleItem =
    | { type: "Track"; data: TrackResult }
    | { type: "Album"; data: AlbumItem }
    | { type: "Playlist"; data: PlaylistItem }
    | { type: "Genre"; data: GenreItem }
    | { type: "Artist"; data: ArtistItem }
    | { type: "Spotlight"; data: EditorialSpotlight };

export interface ModuleData {
    items: ModuleItem[];
}

export interface AggregatedModule {
    provider_id: string;
    provider_name: string;
    module: {
        id: string;
        name: string;
        layout: string;
    };
}

export interface CategoryPalette {
    [key: string]: string;
}

const CATEGORY_COLORS: CategoryPalette = {
    "Electronic": "#D48B38",
    "Classical": "#946342",
    "Jazz & Soul": "#C69055",
    "Ambient": "#5C768D",
    "Rock": "#A84C3C",
    "Hip-Hop": "#B5733A",
    "Focus": "#4A6B6C",
    "Chill": "#6A7B6E",
};

export class ExploreStore {
    spotlights = $state<EditorialSpotlight[]>([]);
    activeSpotlightIndex = $state<number>(0);
    categoryGrid = $state<GenreItem[]>([]);
    rankedTracks = $state<TrackResult[]>([]);
    newReleases2x2 = $state<AlbumItem[]>([]);

    activeCategory = $state<GenreItem | null>(null);
    categoryTracks = $state<TrackResult[]>([]);

    searchQuery = $state("");
    activeSearchFilter = $state<string>("all"); // "all", "songs", "albums", "artists", "playlists"
    activeSourceFilters = $state<string[]>(["local", "youtube-wasm"]);
    isSourceMenuOpen = $state(false);

    rawSections = $state<SearchCategorySection[]>([]);
    isSearching = $state(false);
    activeDrawerCollection = $state<DrawerCollection | null>(null);
    isLoadingCollection = $state(false);
    selectedRemoteAlbum = $state<AlbumDetailResult | null>(null);
    selectedRemotePlaylist = $state<AlbumDetailResult | null>(null);
    isLoadingPlaylist = $state(false);
    selectedArtist = $state<ArtistDetailResult | null>(null);
    isLoadingArtist = $state(false);
    isLoadingAlbum = $state(false);
    lastFetchedAt = $state<number | null>(null);
    readonly CACHE_TTL_MS = 15 * 60 * 1000;

    openLocalAlbum(album: any) {
        this.activeDrawerCollection = {
            kind: 'album',
            source: 'local',
            id: album.id,
            title: album.title,
            subtitle: album.artist || 'Unknown Artist',
            cover_art_path: album.cover_art_path,
            rawLocal: album,
            tracks: [],
        };
        this.selectedRemoteAlbum = null;
        this.selectedRemotePlaylist = null;
    }

    openLocalPlaylist(playlist: any) {
        this.activeDrawerCollection = {
            kind: 'playlist',
            source: 'local',
            id: playlist.id,
            title: playlist.name,
            subtitle: 'Local Playlist',
            rawLocal: playlist,
            tracks: [],
        };
        this.selectedRemoteAlbum = null;
        this.selectedRemotePlaylist = null;
    }

    async openFavorites() {
        const liked = await libraryStore.fetchLikedSongs();
        const tracks = liked.map((s, idx) => ({
            id: s.last_source_id || s.local_track_id || idx,
            title: s.title,
            artist: s.artist,
            album: s.album,
            file_path: s.file_path,
            cover_art_url: s.cover_art_url,
            provider_id: s.last_provider_id || (s.file_path ? undefined : "youtube-wasm"),
            duration_ms: s.duration_ms,
            canonical_key: s.canonical_key,
            liked: true,
        }));
        this.activeDrawerCollection = {
            kind: 'playlist',
            source: 'local',
            id: 'favorites',
            title: 'Liked Songs',
            subtitle: `${tracks.length} ${tracks.length === 1 ? 'Favorite Song' : 'Favorite Songs'}`,
            tracks,
        };
        this.selectedRemoteAlbum = null;
        this.selectedRemotePlaylist = null;
    }

    closeDrawerCollection() {
        this.activeDrawerCollection = null;
        this.selectedRemoteAlbum = null;
        this.selectedRemotePlaylist = null;
    }

    async openAlbum(album: { id: string; title?: string; artist?: string; cover_art_url?: string | null; is_local?: boolean; provider_id?: string }) {
        if (album.is_local || album.id.startsWith("local-")) {
            return;
        }

        const fallbackProvider = (settingsStore.defaultRemoteProvider && settingsStore.defaultRemoteProvider !== "local")
            ? settingsStore.defaultRemoteProvider
            : "youtube-wasm";
        const pId = (album.provider_id && album.provider_id !== "local")
            ? album.provider_id
            : fallbackProvider;

        const albumTitle = album.title || album.id || "Album";
        const albumArtist = album.artist || "Unknown Artist";

        // Instantly populate drawer stub with card metadata so drawer opens immediately with title and cover art!
        this.activeDrawerCollection = {
            kind: 'album',
            source: 'remote',
            id: album.id,
            title: albumTitle,
            subtitle: albumArtist,
            cover_art_url: album.cover_art_url || null,
            provider_id: pId,
            tracks: [],
        };
        this.selectedRemoteAlbum = {
            id: album.id,
            title: albumTitle,
            artist: albumArtist,
            cover_art_url: album.cover_art_url || null,
            tracks: [],
            provider_id: pId,
        };
        this.selectedRemotePlaylist = null;
        this.isLoadingCollection = true;
        this.isLoadingAlbum = true;

        try {
            let res: AlbumDetailResult | null = null;

            // 1. If album.id is a real remote browse ID (e.g. MPREb_... or OLAK5uy_... or starts with MPRE or OLAK or VL)
            if (album.id && (album.id.startsWith("MPRE") || album.id.startsWith("OLAK") || album.id.startsWith("VL") || album.id.startsWith("FE"))) {
                res = await invoke<AlbumDetailResult>("browse_provider_album", {
                    providerId: pId,
                    albumId: album.id,
                }).catch(() => null);
            }

            // 2. If browse_provider_album failed or album.id was a plain title
            if (!res) {
                const searchQuery = `${albumArtist} ${albumTitle}`.trim();
                const searchRes = await invoke<CategorizedSearchResult>("search_provider_categorized", {
                    providerId: pId,
                    query: searchQuery,
                    filter: "albums",
                }).catch(() => null);

                let remoteAlbumId: string | null = null;
                let foundCover: string | null = null;

                if (searchRes && searchRes.sections) {
                    for (const sec of searchRes.sections) {
                        for (const item of sec.items) {
                            if (item.type === "Album" && item.data.id) {
                                remoteAlbumId = item.data.id;
                                foundCover = item.data.cover_art_url || null;
                                break;
                            }
                            if (item.type === "TopResult" && item.data.item_type === "album" && item.data.id) {
                                remoteAlbumId = item.data.id;
                                foundCover = item.data.cover_art_url || null;
                                break;
                            }
                        }
                        if (remoteAlbumId) break;
                    }
                }

                if (remoteAlbumId) {
                    res = await invoke<AlbumDetailResult>("browse_provider_album", {
                        providerId: pId,
                        albumId: remoteAlbumId,
                    }).catch(() => null);
                }

                // 3. Fallback track query
                if (!res) {
                    const trackResults = await invoke<any[]>("search_provider", {
                        providerId: pId,
                        query: searchQuery,
                    }).catch(() => []);

                    if (trackResults && trackResults.length > 0) {
                        const tracks = trackResults.slice(0, 20).map((t: any) => ({
                            id: t.id,
                            title: t.title,
                            artist: t.artist || albumArtist,
                            album: albumTitle,
                            cover_art_url: t.cover_art_url || album.cover_art_url || foundCover,
                            duration_ms: t.duration_ms,
                            stream_url: t.stream_url,
                            provider_id: pId,
                        }));

                        res = {
                            id: album.id,
                            title: albumTitle,
                            artist: albumArtist,
                            cover_art_url: album.cover_art_url || foundCover || (tracks[0] && tracks[0].cover_art_url) || null,
                            tracks,
                            provider_id: pId,
                        };
                    }
                }
            }

            if (res && this.activeDrawerCollection?.id === album.id) {
                this.activeDrawerCollection = {
                    ...this.activeDrawerCollection,
                    title: res.title || albumTitle,
                    subtitle: res.artist || albumArtist,
                    cover_art_url: res.cover_art_url || album.cover_art_url || null,
                    tracks: res.tracks || [],
                    rawRemote: res,
                };
                this.selectedRemoteAlbum = {
                    ...res,
                    title: res.title || albumTitle,
                    artist: res.artist || albumArtist,
                    cover_art_url: res.cover_art_url || album.cover_art_url || null,
                };
                document.dispatchEvent(new CustomEvent("echo:navigate-album"));
            }
        } catch (e) {
            console.error("Failed to browse album:", e);
            toastStore.error("Failed to load album details");
        } finally {
            this.isLoadingCollection = false;
            this.isLoadingAlbum = false;
        }
    }

    async openRadioMix(card: { id: string; title: string; subtitle?: string; covers?: string[]; seed: any }) {
        const fallbackProvider = (settingsStore.defaultRemoteProvider && settingsStore.defaultRemoteProvider !== "local")
            ? settingsStore.defaultRemoteProvider
            : "youtube-wasm";
        const pId = (card.seed?.provider_id && card.seed?.provider_id !== "local")
            ? card.seed.provider_id
            : fallbackProvider;

        this.activeDrawerCollection = {
            kind: "playlist",
            source: "remote",
            id: card.id,
            title: card.title,
            subtitle: card.subtitle || "Radio Station",
            cover_art_url: (card.covers && card.covers[0]) || null,
            provider_id: pId,
            tracks: [],
        };
        this.isLoadingCollection = true;
        this.isLoadingPlaylist = true;

        try {
            let tracks: any[] = [];
            const radioResult: any = await invoke("get_radio_stream", {
                providerId: pId,
                seed: card.seed,
            }).catch(() => null);

            if (radioResult && radioResult.tracks && radioResult.tracks.length > 0) {
                tracks = radioResult.tracks.map((t: any) => ({
                    id: t.id,
                    title: t.title,
                    artist: t.artist,
                    album: t.album || "",
                    cover_art_url: t.cover_art_url || (card.covers && card.covers[0]) || null,
                    duration_ms: t.duration_ms,
                    stream_url: t.stream_url,
                    provider_id: pId,
                }));
            } else {
                const cleanArt = (card.seed?.artist || "")
                    .replace(/\s*•\s*[\d.]+[MK]?\s*views.*$/i, "")
                    .replace(/\s*•.*$/i, "")
                    .replace(/\s*-\s*Topic$/i, "")
                    .replace(/\s*VEVO$/i, "")
                    .trim();
                
                const cleanTit = (card.seed?.title || card.title || "")
                    .replace(/\s*•\s*[\d.]+[MK]?\s*views.*$/i, "")
                    .replace(/\s*•.*$/i, "")
                    .trim();

                const query = (!cleanArt || cleanArt.toLowerCase() === "unknown" || cleanTit.toLowerCase().includes(cleanArt.toLowerCase()))
                    ? cleanTit
                    : `${cleanArt} ${cleanTit}`;

                let searchResults = await invoke<any[]>("search_provider", {
                    providerId: pId,
                    query,
                }).catch(() => []);

                if ((!searchResults || searchResults.length === 0) && cleanTit && cleanTit !== query) {
                    searchResults = await invoke<any[]>("search_provider", {
                        providerId: pId,
                        query: cleanTit,
                    }).catch(() => []);
                }

                if (searchResults && searchResults.length > 0) {
                    tracks = searchResults.map((t: any) => ({
                        id: t.id,
                        title: t.title,
                        artist: t.artist,
                        album: t.album || "",
                        cover_art_url: t.cover_art_url || (card.covers && card.covers[0]) || null,
                        duration_ms: t.duration_ms,
                        stream_url: t.stream_url,
                        provider_id: pId,
                    }));
                }
            }

            if (this.activeDrawerCollection?.id === card.id) {
                this.activeDrawerCollection = {
                    ...this.activeDrawerCollection,
                    tracks,
                };
            }
        } catch (e) {
            console.error("Failed to load radio mix tracks:", e);
        } finally {
            this.isLoadingCollection = false;
            this.isLoadingPlaylist = false;
        }
    }

    async openHorizon(payload: AdjacentHorizonPayload) {
        const fallbackProvider = (settingsStore.defaultRemoteProvider && settingsStore.defaultRemoteProvider !== "local")
            ? settingsStore.defaultRemoteProvider
            : "youtube-wasm";
        const pId = (payload.seed?.provider_id && payload.seed?.provider_id !== "local")
            ? payload.seed.provider_id
            : fallbackProvider;

        const initialTracks = (payload.preview_tracks || []).map((t: any) => ({
            id: t.sources?.[0]?.remote_track_id || t.canonical_key,
            title: t.title,
            artist: t.artist,
            album: t.album || "",
            cover_art_url: t.cover_art_url,
            duration_ms: t.duration_ms,
            provider_id: pId,
        }));

        const horizonId = `horizon-${payload.suggested_genre.toLowerCase().replace(/\s+/g, "-")}`;
        this.activeDrawerCollection = {
            kind: "playlist",
            source: "remote",
            id: horizonId,
            title: payload.suggested_genre,
            subtitle: payload.tagline,
            cover_art_url: (payload.preview_tracks && payload.preview_tracks[0]?.cover_art_url) || null,
            provider_id: pId,
            tracks: initialTracks,
        };
        this.isLoadingCollection = true;
        this.isLoadingPlaylist = true;

        try {
            const radioResult: any = await invoke("get_radio_stream", {
                providerId: pId,
                seed: payload.seed,
            }).catch(() => null);

            let moreTracks: any[] = [];
            if (radioResult && radioResult.tracks && radioResult.tracks.length > 0) {
                moreTracks = radioResult.tracks.map((t: any) => ({
                    id: t.id,
                    title: t.title,
                    artist: t.artist,
                    album: t.album || "",
                    cover_art_url: t.cover_art_url || null,
                    duration_ms: t.duration_ms,
                    stream_url: t.stream_url,
                    provider_id: pId,
                }));
            } else {
                const searchResults = await invoke<any[]>("search_provider", {
                    providerId: pId,
                    query: payload.suggested_genre,
                }).catch(() => []);

                if (searchResults && searchResults.length > 0) {
                    moreTracks = searchResults.map((t: any) => ({
                        id: t.id,
                        title: t.title,
                        artist: t.artist,
                        album: t.album || "",
                        cover_art_url: t.cover_art_url || null,
                        duration_ms: t.duration_ms,
                        stream_url: t.stream_url,
                        provider_id: pId,
                    }));
                }
            }

            if (this.activeDrawerCollection?.id === horizonId) {
                const combined = [...initialTracks];
                for (const mt of moreTracks) {
                    if (!combined.some(ct => ct.title.toLowerCase() === mt.title.toLowerCase() && ct.artist.toLowerCase() === mt.artist.toLowerCase())) {
                        combined.push(mt);
                    }
                }
                this.activeDrawerCollection = {
                    ...this.activeDrawerCollection,
                    tracks: combined,
                };
            }
        } catch (e) {
            console.error("Failed to load horizon tracks:", e);
        } finally {
            this.isLoadingCollection = false;
            this.isLoadingPlaylist = false;
        }
    }

    async openPlaylist(playlist: { id: string; title?: string; author?: string; cover_art_url?: string | null; provider_id?: string }) {
        const pId = playlist.provider_id || "youtube-wasm";
        // Instantly populate drawer stub so drawer opens immediately with title and cover art!
        this.activeDrawerCollection = {
            kind: 'playlist',
            source: 'remote',
            id: playlist.id,
            title: playlist.title || "Playlist",
            subtitle: playlist.author || "Curated Playlist",
            cover_art_url: playlist.cover_art_url || null,
            provider_id: pId,
            tracks: [],
        };
        this.selectedRemotePlaylist = {
            id: playlist.id,
            title: playlist.title || "Playlist",
            artist: playlist.author || "Curated Playlist",
            cover_art_url: playlist.cover_art_url || null,
            tracks: [],
            provider_id: pId,
        };
        this.selectedRemoteAlbum = null;
        this.isLoadingCollection = true;
        this.isLoadingPlaylist = true;

        try {
            const res = await invoke<AlbumDetailResult>("browse_provider_album", {
                providerId: pId,
                albumId: playlist.id,
            });
            if (res && this.activeDrawerCollection?.id === playlist.id) {
                this.activeDrawerCollection = {
                    ...this.activeDrawerCollection,
                    title: res.title || playlist.title || "Playlist",
                    subtitle: res.artist || playlist.author || "Curated Playlist",
                    cover_art_url: res.cover_art_url || playlist.cover_art_url || null,
                    tracks: res.tracks || [],
                    rawRemote: res,
                };
                this.selectedRemotePlaylist = {
                    ...res,
                    title: res.title || playlist.title || "Playlist",
                    artist: res.artist || playlist.author || "Curated Playlist",
                    cover_art_url: res.cover_art_url || playlist.cover_art_url || null,
                };
                document.dispatchEvent(new CustomEvent("echo:navigate-playlist"));
            }
        } catch (e) {
            console.error("Failed to browse playlist:", e);
            toastStore.error("Failed to load playlist details");
        } finally {
            this.isLoadingCollection = false;
            this.isLoadingPlaylist = false;
        }
    }

    async openArtist(artist: { id: string; name?: string; provider_id?: string }) {
        this.isLoadingArtist = true;
        const artistName = artist.name || artist.id;
        const pId = (artist.provider_id && artist.provider_id !== "local")
            ? artist.provider_id
            : ((settingsStore.defaultRemoteProvider && settingsStore.defaultRemoteProvider !== "local")
                ? settingsStore.defaultRemoteProvider
                : "youtube-wasm");

        // Immediately navigate with provisional state
        this.selectedArtist = {
            id: artist.id,
            name: artistName,
            top_tracks: [],
            albums: [],
            singles: [],
            provider_id: pId,
        };
        document.dispatchEvent(new CustomEvent("echo:navigate-artist"));

        try {
            let res: ArtistDetailResult | null = null;

            // 1. If artist.id is already a remote channel ID (starts with UC or MPRE or FE)
            if (artist.id && (artist.id.startsWith("UC") || artist.id.startsWith("FE") || artist.id.startsWith("MPRE"))) {
                res = await invoke<ArtistDetailResult>("browse_provider_artist", {
                    providerId: pId,
                    artistId: artist.id,
                }).catch(() => null);
            }

            // 2. If browse_provider_artist failed or artist.id was a plain name
            if (!res) {
                // Search for the artist profile on the provider
                const searchRes = await invoke<CategorizedSearchResult>("search_provider_categorized", {
                    providerId: pId,
                    query: artistName,
                    filter: "artists",
                }).catch(() => null);

                let remoteArtistId: string | null = null;
                let avatarUrl: string | null = null;
                let subscribers: string | null = null;

                if (searchRes && searchRes.sections) {
                    for (const sec of searchRes.sections) {
                        for (const item of sec.items) {
                            if (item.type === "Artist" && item.data.id) {
                                remoteArtistId = item.data.id;
                                avatarUrl = item.data.avatar_url || null;
                                subscribers = item.data.subscribers || null;
                                break;
                            }
                            if (item.type === "TopResult" && item.data.item_type === "artist" && item.data.id) {
                                remoteArtistId = item.data.id;
                                avatarUrl = item.data.cover_art_url || null;
                                break;
                            }
                        }
                        if (remoteArtistId) break;
                    }
                }

                if (remoteArtistId) {
                    res = await invoke<ArtistDetailResult>("browse_provider_artist", {
                        providerId: pId,
                        artistId: remoteArtistId,
                    }).catch(() => null);
                }

                // 3. Fallback synthesis: If still no full profile, search tracks & albums for this artist
                if (!res) {
                    const trackResults = await invoke<any[]>("search_provider", {
                        providerId: pId,
                        query: artistName,
                    }).catch(() => []);

                    // Also find local albums matching this artist
                    const localAlbums: AlbumItem[] = libraryStore.albums
                        .filter(a => a.artist && a.artist.toLowerCase().includes(artistName.toLowerCase()))
                        .map(a => ({
                            id: `local-${a.id}`,
                            title: a.title,
                            artist: a.artist || artistName,
                            cover_art_url: a.cover_art_path ? (a.cover_art_path.startsWith("/") ? `asset://localhost/${encodeURIComponent(a.cover_art_path)}` : a.cover_art_path) : null,
                            is_local: true,
                        }));

                    res = {
                        id: artist.id,
                        name: artistName,
                        avatar_url: avatarUrl,
                        subscribers: subscribers,
                        top_tracks: trackResults.slice(0, 15),
                        albums: localAlbums,
                        singles: [],
                        provider_id: pId,
                    };
                }
            }

            if (res) {
                this.selectedArtist = res;
            }
        } catch (e) {
            console.error("Failed to browse artist:", e);
            toastStore.error("Failed to load artist page");
        } finally {
            this.isLoadingArtist = false;
        }
    }

    private _searchDebounceTimer: ReturnType<typeof setTimeout> | null = null;

    isLoading = $state(false);
    isCategoryLoading = $state(false);
    isLoaded = $state(false);

    activeSpotlight = $derived(
        this.spotlights.length > 0
            ? this.spotlights[this.activeSpotlightIndex] ?? this.spotlights[0]
            : null
    );

    filteredRankedTracks = $derived(this.rankedTracks.slice(0, 5));
    filteredNewReleases = $derived(this.newReleases2x2.slice(0, 4));

    // Dynamic filtering based on active source filters and active category filter
    searchSections = $derived.by(() => {
        const allowedSources = new Set(this.activeSourceFilters);
        const filter = this.activeSearchFilter;

        return this.rawSections
            .map(sec => {
                // Filter items by source
                const filteredItems = sec.items.filter(item => {
                    const pid = item.data.provider_id || "youtube-wasm";
                    return allowedSources.has(pid);
                });

                return {
                    category: sec.category,
                    items: filteredItems,
                };
            })
            .filter(sec => {
                if (sec.items.length === 0) return false;
                if (filter === "all") return true;
                if (filter === "songs" && (sec.category === "Songs" || sec.category === "Videos")) return true;
                if (filter === "albums" && sec.category === "Albums") return true;
                if (filter === "artists" && sec.category === "Artists") return true;
                if (filter === "playlists" && sec.category === "Playlists") return true;
                return false;
            });
    });

    // Helper derivations for search results
    topResultSection = $derived(
        this.searchSections.find(s => s.category.toLowerCase().includes("top result"))
    );
    songsSection = $derived(
        this.searchSections.find(s => s.category.toLowerCase() === "songs" || s.category.toLowerCase() === "videos")
    );
    albumsSection = $derived(
        this.searchSections.find(s => s.category.toLowerCase() === "albums")
    );
    artistsSection = $derived(
        this.searchSections.find(s => s.category.toLowerCase() === "artists")
    );
    playlistsSection = $derived(
        this.searchSections.find(s => s.category.toLowerCase() === "playlists")
    );

    totalSearchResultCount = $derived(
        this.searchSections.reduce((acc, s) => acc + s.items.length, 0)
    );

    nextSpotlight() {
        if (this.spotlights.length <= 1) return;
        this.activeSpotlightIndex = (this.activeSpotlightIndex + 1) % this.spotlights.length;
    }

    prevSpotlight() {
        if (this.spotlights.length <= 1) return;
        this.activeSpotlightIndex =
            (this.activeSpotlightIndex - 1 + this.spotlights.length) % this.spotlights.length;
    }

    setSpotlightIndex(index: number) {
        if (index >= 0 && index < this.spotlights.length) {
            this.activeSpotlightIndex = index;
        }
    }

    async init(force = false) {
        if (this.isLoaded && !force && this.lastFetchedAt && (Date.now() - this.lastFetchedAt < this.CACHE_TTL_MS)) {
            return;
        }
        await this.loadExplore(force);
    }

    setSearchQuery(query: string) {
        this.searchQuery = query;
        if (this._searchDebounceTimer) {
            clearTimeout(this._searchDebounceTimer);
            this._searchDebounceTimer = null;
        }

        const trimmed = query.trim();
        if (!trimmed) {
            this.rawSections = [];
            this.isSearching = false;
            return;
        }

        this.isSearching = true;
        this._searchDebounceTimer = setTimeout(async () => {
            await this.performSearch(trimmed);
        }, 260);
    }

    setSearchFilter(filter: string) {
        if (this.activeSearchFilter === filter) return;
        this.activeSearchFilter = filter;
        if (this.searchQuery.trim()) {
            this.performSearch(this.searchQuery.trim());
        }
    }

    toggleSourceFilter(sourceId: string) {
        if (this.activeSourceFilters.includes(sourceId)) {
            if (this.activeSourceFilters.length > 1) {
                this.activeSourceFilters = this.activeSourceFilters.filter(id => id !== sourceId);
            }
        } else {
            this.activeSourceFilters = [...this.activeSourceFilters, sourceId];
        }
    }

    isSourceActive(sourceId: string): boolean {
        return this.activeSourceFilters.includes(sourceId);
    }

    private searchCache = new Map<string, SearchCategorySection[]>();

    async performSearch(query: string) {
        const trimmed = query.trim();
        if (!trimmed) {
            this.clearSearch();
            return;
        }

        const cacheKey = `${trimmed.toLowerCase()}:${this.activeSearchFilter}`;
        if (this.searchCache.has(cacheKey)) {
            this.rawSections = this.searchCache.get(cacheKey)!;
            this.isSearching = false;
            return;
        }

        this.isSearching = true;
        try {
            const filterArg = this.activeSearchFilter === "all" ? null : this.activeSearchFilter;
            
            // 1. Parallel dispatch: Local SQLite + Remote WASM
            const localTracksPromise = (async () => {
                try {
                    const local = await invoke<any[]>("search_library", { query: trimmed, limit: 15 });
                    return local || [];
                } catch (e) {
                    console.error("Local search error:", e);
                    return [];
                }
            })();

            const remotePromise = (async () => {
                try {
                    const res = await invoke<CategorizedSearchResult>("search_provider_categorized", {
                        providerId: "youtube-wasm",
                        query: trimmed,
                        filter: filterArg,
                    });
                    return res?.sections || [];
                } catch (e) {
                    console.error("Remote search error:", e);
                    return [];
                }
            })();

            const [localTracksRaw, remoteSectionsRaw] = await Promise.all([
                localTracksPromise,
                remotePromise,
            ]);

            if (this.searchQuery.trim() !== trimmed) return;

            // 2. Format Local Tracks into SearchItems
            const localTrackItems: SearchItem[] = localTracksRaw.map(t => ({
                type: "Track",
                data: {
                    id: `local-${t.id}`,
                    title: t.title,
                    artist: t.artist || "Unknown Artist",
                    album: null,
                    cover_art_url: null,
                    file_path: t.file_path,
                    is_local: true,
                    provider_id: "local",
                    provider_name: "Local Library",
                }
            }));

            // 3. Format Local Albums matching query
            const localAlbumMatches = libraryStore.albums
                .filter(a => a.title.toLowerCase().includes(trimmed.toLowerCase()) || (a.artist && a.artist.toLowerCase().includes(trimmed.toLowerCase())))
                .map(a => ({
                    type: "Album" as const,
                    data: {
                        id: `local-album-${a.id}`,
                        title: a.title,
                        artist: a.artist || "Unknown Artist",
                        cover_art_url: a.cover_art_path,
                        is_local: true,
                        provider_id: "local",
                        provider_name: "Local Library",
                    }
                }));

            // 4. Merge Sections: Prepend local items to Songs & Albums shelves
            const combinedSections: SearchCategorySection[] = [];
            let songsAdded = false;
            let albumsAdded = false;

            for (const rSec of remoteSectionsRaw) {
                if (rSec.category === "Songs" || rSec.category === "Videos") {
                    const mergedSongs = [...localTrackItems, ...rSec.items];
                    combinedSections.push({
                        category: "Songs",
                        items: mergedSongs,
                    });
                    songsAdded = true;
                } else if (rSec.category === "Albums") {
                    const mergedAlbums = [...localAlbumMatches, ...rSec.items];
                    combinedSections.push({
                        category: "Albums",
                        items: mergedAlbums,
                    });
                    albumsAdded = true;
                } else {
                    combinedSections.push(rSec);
                }
            }

            // If remote had no Songs shelf but we have local tracks
            if (!songsAdded && localTrackItems.length > 0) {
                combinedSections.push({
                    category: "Songs",
                    items: localTrackItems,
                });
            }

            // If remote had no Albums shelf but we have local albums
            if (!albumsAdded && localAlbumMatches.length > 0) {
                combinedSections.push({
                    category: "Albums",
                    items: localAlbumMatches,
                });
            }

            this.searchCache.set(cacheKey, combinedSections);
            this.rawSections = combinedSections;

            // Opportunistically prefetch categories in background when on "all"
            if (this.activeSearchFilter === "all") {
                this.prefetchCategory(trimmed, "songs");
                this.prefetchCategory(trimmed, "albums");
                this.prefetchCategory(trimmed, "artists");
                this.prefetchCategory(trimmed, "playlists");
            }
        } catch (e) {
            console.error("Categorized search failed:", e);
        } finally {
            if (this.searchQuery.trim() === trimmed) {
                this.isSearching = false;
            }
        }
    }

    private async prefetchCategory(query: string, category: string) {
        const cacheKey = `${query.toLowerCase()}:${category}`;
        if (this.searchCache.has(cacheKey)) return;

        try {
            const res = await invoke<CategorizedSearchResult>("search_provider_categorized", {
                providerId: "youtube-wasm",
                query,
                filter: category,
            });
            if (res && Array.isArray(res.sections) && res.sections.length > 0) {
                this.searchCache.set(cacheKey, res.sections);
            }
        } catch {
            // Silently ignore prefetch background errors
        }
    }

    clearSearch() {
        this.searchQuery = "";
        this.rawSections = [];
        this.searchCache.clear();
        this.isSearching = false;
        this.activeSearchFilter = "all";
        if (this._searchDebounceTimer) {
            clearTimeout(this._searchDebounceTimer);
            this._searchDebounceTimer = null;
        }
    }

    async loadExplore(force = false) {
        if (this.isLoading) return;
        if (!this.isLoaded) {
            this.isLoading = true;
        }
        try {
            const modules = await invoke<AggregatedModule[]>("get_explore_feed");

            const allSpotlights: EditorialSpotlight[] = [];
            const allGenres: GenreItem[] = [];
            const allTracks: TrackResult[] = [];
            const allAlbums: AlbumItem[] = [];

            for (const agg of (modules || [])) {
                try {
                    const data = await invoke<ModuleData>("fetch_provider_module", {
                        providerId: agg.provider_id,
                        moduleId: agg.module.id,
                    });

                    if (data && data.items) {
                        for (const item of data.items) {
                            if (item.type === "Spotlight") {
                                allSpotlights.push({
                                    ...item.data,
                                    provider_id: agg.provider_id,
                                    provider_name: agg.provider_name,
                                    cover_art_url: item.data.cover_art_url || "https://images.unsplash.com/photo-1618005182384-a83a8bd57fbe?q=80&w=1200&auto=format&fit=crop",
                                });
                            } else if (item.type === "Genre") {
                                const title = item.data.title;
                                const color = item.data.color_hex || CATEGORY_COLORS[title] || "#B58E62";
                                allGenres.push({
                                    ...item.data,
                                    color_hex: color,
                                    provider_id: agg.provider_id,
                                    provider_name: agg.provider_name,
                                });
                            } else if (item.type === "Track") {
                                allTracks.push({
                                    ...item.data,
                                    provider_id: agg.provider_id,
                                    provider_name: agg.provider_name,
                                });
                            } else if (item.type === "Album") {
                                allAlbums.push({
                                    ...item.data,
                                    provider_id: agg.provider_id,
                                    provider_name: agg.provider_name,
                                });
                            }
                        }
                    }
                } catch (err) {
                    console.error(`Failed to fetch module data for ${agg.provider_id}/${agg.module.id}:`, err);
                }
            }

            if (allSpotlights.length > 0) {
                this.spotlights = allSpotlights;
                this.activeSpotlightIndex = 0;
            } else {
                this.spotlights = [{
                    id: "default-spotlight",
                    title: "Echoes of Eternity",
                    artist: "Kavinsky & Daft Punk",
                    cover_art_url: "https://images.unsplash.com/photo-1618005182384-a83a8bd57fbe?q=80&w=1200&auto=format&fit=crop",
                    description: "Curated Master Edition • 24-bit 96kHz Lossless",
                    release_year: "2026",
                    provider_name: "Echo Curated",
                }];
            }

            if (allGenres.length > 0) {
                this.categoryGrid = allGenres;
            } else {
                this.categoryGrid = Object.keys(CATEGORY_COLORS).map((title, i) => ({
                    id: `genre-${i}`,
                    title,
                    color_hex: CATEGORY_COLORS[title],
                }));
            }

            this.rankedTracks = allTracks;
            this.newReleases2x2 = allAlbums;
            this.isLoaded = true;
            this.lastFetchedAt = Date.now();
        } catch (e) {
            console.error("Failed to load explore feed:", e);
        } finally {
            this.isLoading = false;
        }
    }

    async selectCategory(category: GenreItem) {
        this.activeCategory = category;
        this.isCategoryLoading = true;
        this.categoryTracks = [];

        try {
            const pId = category.provider_id || "youtube-wasm";
            const endpoint = category.endpoint_params || category.id;
            
            const data = await invoke<ModuleData>("fetch_provider_module", {
                providerId: pId,
                moduleId: endpoint,
            });

            if (data && data.items) {
                this.categoryTracks = data.items
                    .filter((i): i is { type: "Track"; data: TrackResult } => i.type === "Track")
                    .map(i => ({
                        ...i.data,
                        provider_id: pId,
                    }));
            }
        } catch (e) {
            console.error(`Failed to load tracks for category ${category.title}:`, e);
        } finally {
            this.isCategoryLoading = false;
        }
    }

    closeCategory() {
        this.activeCategory = null;
        this.categoryTracks = [];
    }

    async playTrack(track: TrackResult, providerId?: string) {
        if (track.is_local || track.file_path) {
            const localCanonical = {
                id: track.id ? Number(track.id.toString().replace("local-", "")) : -1,
                track_id: track.id ? Number(track.id.toString().replace("local-", "")) : -1,
                title: track.title,
                artist: track.artist || "Unknown Artist",
                file_path: track.file_path,
                is_local: true,
                duration_ms: track.duration_ms || 210000,
            };
            await audioStore.handleTrackClick(localCanonical);
            return;
        }

        const pId = providerId || track.provider_id || "youtube-wasm";
        const canonicalTrack = {
            id: track.id,
            remote_track_id: track.id,
            title: track.title,
            artist: track.artist || "Unknown Artist",
            album: track.album || undefined,
            duration_ms: track.duration_ms || 210000,
            cover_art_url: track.cover_art_url || undefined,
            provider_id: pId,
        };

        await audioStore.handleTrackClick(canonicalTrack);
    }

    async playSpotlight() {
        const spot = this.activeSpotlight;
        if (!spot) return;
        
        try {
            const pId = spot.provider_id || "youtube-wasm";
            const tracks = await invoke<TrackResult[]>("search_provider", {
                providerId: pId,
                query: `${spot.title} ${spot.artist}`,
            });
            if (tracks && tracks.length > 0) {
                const canonicalTracks = tracks.map(t => ({
                    id: t.id,
                    title: t.title,
                    artist: t.artist || spot.artist,
                    album: t.album || spot.title,
                    duration_ms: t.duration_ms || 210000,
                    cover_art_url: t.cover_art_url || spot.cover_art_url || undefined,
                    provider_id: pId,
                }));
                await audioStore.setQueue(canonicalTracks, 0);
            }
        } catch (e) {
            console.error("Failed to play spotlight:", e);
        }
    }

    async playAlbum(album: AlbumItem) {
        const pId = album.provider_id || "youtube-wasm";
        try {
            const tracks = await invoke<TrackResult[]>("search_provider", {
                providerId: pId,
                query: `${album.title} ${album.artist}`,
            });
            if (tracks && tracks.length > 0) {
                const canonicalTracks = tracks.map(t => ({
                    id: t.id,
                    title: t.title,
                    artist: t.artist || album.artist,
                    album: t.album || album.title,
                    duration_ms: t.duration_ms || 210000,
                    cover_art_url: t.cover_art_url || album.cover_art_url || undefined,
                    provider_id: pId,
                }));
                await audioStore.setQueue(canonicalTracks, 0);
            }
        } catch (e) {
            console.error("Failed to play album:", e);
        }
    }
}

export const exploreStore = new ExploreStore();
