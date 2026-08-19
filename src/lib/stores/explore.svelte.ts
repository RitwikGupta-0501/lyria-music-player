import { invoke } from "@tauri-apps/api/core";
import { audioStore } from "./audio.svelte";
import { libraryStore } from "./library.svelte";

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

    async init() {
        if (this.isLoaded) return;
        await this.loadExplore();
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
        this.isLoading = true;
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
