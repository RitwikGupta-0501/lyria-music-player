import { invoke } from "@tauri-apps/api/core";
import { audioStore } from "./audio.svelte";

export interface TrackResult {
    id: string;
    title: string;
    artist: string;
    album?: string | null;
    cover_art_url?: string | null;
    stream_url?: string | null;
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
    searchSections = $state<SearchCategorySection[]>([]);
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
            this.searchSections = [];
            this.isSearching = false;
            return;
        }

        this.isSearching = true;
        this._searchDebounceTimer = setTimeout(async () => {
            await this.performSearch(trimmed);
        }, 300);
    }

    setSearchFilter(filter: string) {
        if (this.activeSearchFilter === filter) return;
        this.activeSearchFilter = filter;
        if (this.searchQuery.trim()) {
            this.performSearch(this.searchQuery.trim());
        }
    }

    async performSearch(query: string) {
        this.isSearching = true;
        try {
            const filterArg = this.activeSearchFilter === "all" ? null : this.activeSearchFilter;
            const res = await invoke<CategorizedSearchResult>("search_provider_categorized", {
                providerId: "youtube-wasm",
                query: query,
                filter: filterArg,
            });

            if (this.searchQuery.trim() === query) {
                // Decorate provenance and deduplicate track items
                this.searchSections = (res.sections || []).map(sec => ({
                    ...sec,
                    items: sec.items.map(item => {
                        if (item.type === "Track") {
                            return {
                                type: "Track",
                                data: {
                                    ...item.data,
                                    provider_id: "youtube-wasm",
                                    provider_name: "YouTube Music",
                                }
                            };
                        } else if (item.type === "Album") {
                            return {
                                type: "Album",
                                data: {
                                    ...item.data,
                                    provider_id: "youtube-wasm",
                                    provider_name: "YouTube Music",
                                }
                            };
                        } else if (item.type === "Artist") {
                            return {
                                type: "Artist",
                                data: {
                                    ...item.data,
                                    provider_id: "youtube-wasm",
                                    provider_name: "YouTube Music",
                                }
                            };
                        } else if (item.type === "Playlist") {
                            return {
                                type: "Playlist",
                                data: {
                                    ...item.data,
                                    provider_id: "youtube-wasm",
                                    provider_name: "YouTube Music",
                                }
                            };
                        }
                        return item;
                    })
                }));
            }
        } catch (e) {
            console.error("Categorized search failed:", e);
        } finally {
            if (this.searchQuery.trim() === query) {
                this.isSearching = false;
            }
        }
    }

    clearSearch() {
        this.searchQuery = "";
        this.searchSections = [];
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
                this.activeSpotlightIndex = 0;
            }

            this.categoryGrid = allGenres.length > 0 ? allGenres.slice(0, 8) : [
                { id: "electronic", title: "Electronic", color_hex: "#D48B38" },
                { id: "classical", title: "Classical", color_hex: "#946342" },
                { id: "jazz-soul", title: "Jazz & Soul", color_hex: "#C69055" },
                { id: "ambient", title: "Ambient", color_hex: "#5C768D" },
                { id: "rock", title: "Rock", color_hex: "#A84C3C" },
                { id: "hip-hop", title: "Hip-Hop", color_hex: "#B5733A" },
                { id: "focus", title: "Focus", color_hex: "#4A6B6C" },
                { id: "chill", title: "Chill", color_hex: "#6A7B6E" },
            ];

            this.rankedTracks = allTracks.length > 0 ? allTracks.slice(0, 10) : [
                { id: "track-1", title: "Instant Crush (Master Flac)", artist: "Daft Punk ft. Julian Casablancas", duration_ms: 337000 },
                { id: "track-2", title: "Nightcall (Drive OST)", artist: "Kavinsky", duration_ms: 259000 },
                { id: "track-3", title: "Midnight City", artist: "M83", duration_ms: 243000 },
                { id: "track-4", title: "Resonance", artist: "HOME", duration_ms: 212000 },
                { id: "track-5", title: "Genesis", artist: "Justice", duration_ms: 234000 },
            ];

            this.newReleases2x2 = allAlbums.length > 0 ? allAlbums.slice(0, 4) : [
                { id: "alb-1", title: "Random Access Memories", artist: "Daft Punk" },
                { id: "alb-2", title: "OutRun (Deluxe)", artist: "Kavinsky" },
                { id: "alb-3", title: "Hurry Up, We're Dreaming", artist: "M83" },
                { id: "alb-4", title: "Cross", artist: "Justice" },
            ];

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
            const providerId = category.provider_id || "youtube-wasm";
            const query = category.endpoint_params || `genre ${category.title} mix`;
            const results = await invoke<TrackResult[]>("search_provider", {
                providerId,
                query,
            });

            this.categoryTracks = (results || []).map(t => ({
                ...t,
                provider_id: providerId,
                provider_name: category.provider_name || "YouTube Music",
            }));
        } catch (e) {
            console.error("Failed to fetch category tracks:", e);
        } finally {
            this.isCategoryLoading = false;
        }
    }

    closeCategory() {
        this.activeCategory = null;
        this.categoryTracks = [];
    }

    async playTrack(track: TrackResult, providerId?: string) {
        const pId = providerId || track.provider_id || "youtube-wasm";
        const canonicalTrack = {
            id: track.id,
            title: track.title,
            artist: track.artist || "Unknown Artist",
            album: track.album || undefined,
            duration_ms: track.duration_ms || 210000,
            cover_art_url: track.cover_art_url || undefined,
            provider_id: pId,
        };

        await audioStore.setQueue([canonicalTrack], 0);
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
