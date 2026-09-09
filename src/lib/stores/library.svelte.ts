import { audioStore } from "./audio.svelte";
import { invoke } from "@tauri-apps/api/core";
import { convertFileSrc } from "@tauri-apps/api/core";
import { toastStore } from "./toast.svelte";
import { settingsStore } from "./settings.svelte";
import { normalizeCanonicalString, isCanonicalEntityMatch } from "$lib/utils/format";

export interface SavedAlbum {
    id: string;
    title: string;
    artist?: string | null;
    cover_art_url?: string | null;
    provider_id: string;
    created_at?: string | null;
}

export interface SavedPlaylist {
    id: string;
    title: string;
    author?: string | null;
    cover_art_url?: string | null;
    provider_id: string;
    created_at?: string | null;
}

export function getCanonicalKey(
    track: { title: string; artist?: string | null; canonical_key?: string },
    fallbackArtist?: string | null
): string {
    if (track.canonical_key && track.canonical_key.trim()) {
        return normalizeCanonicalString(track.canonical_key);
    }
    const artist = normalizeCanonicalString(track.artist || fallbackArtist || "unknown");
    const title = normalizeCanonicalString(track.title || "unknown");
    return `${artist}::${title}`;
}

export interface LikedSong {
    canonical_key: string;
    title: string;
    artist?: string | null;
    album?: string | null;
    cover_art_url?: string | null;
    last_provider_id?: string | null;
    last_source_id?: string | null;
    local_track_id?: number | null;
    local_file_path?: string | null;
    file_path?: string | null;
    duration_ms?: number | null;
}

export interface Album {
    id: number;
    title: string;
    artist: string | null;
    cover_art_path: string | null;
}

export interface LocalTrack {
    id: number;
    title: string;
    artist: string | null;
    album_id: number | null;
    track_number: number | null;
    file_path: string;
}

export interface Playlist {
    id: number;
    name: string;
}

export class LibraryStore {
    albums = $state<Album[]>([]);
    recentAlbums = $state<Album[]>([]);
    playlists = $state<Playlist[]>([]);
    likedSongs = $state<LikedSong[]>([]);
    savedAlbums = $state<SavedAlbum[]>([]);
    savedPlaylists = $state<SavedPlaylist[]>([]);
    isScanning = $state(false);
    lastScanResult = $state<number | null>(null);
    onLikeToggled: ((canonicalKey: string, isLiked: boolean) => void) | null = null;

    async fetchAlbums() {
        try {
            this.albums = await invoke("get_albums", { limit: 500, offset: 0 });
        } catch (e) {
            console.error(e);
        }
    }

    async fetchRecentAlbums() {
        try {
            this.recentAlbums = await invoke("get_recent_albums", { limit: 20 });
        } catch (e) {
            console.error(e);
        }
    }

    async fetchPlaylists() {
        try {
            this.playlists = await invoke("get_playlists", { limit: 500, offset: 0 });
        } catch (e) {
            console.error(e);
        }
    }

    async fetchLikedSongs(): Promise<LikedSong[]> {
        try {
            const songs = await invoke<LikedSong[]>("get_liked_songs", { limit: 500 });
            this.likedSongs = songs || [];
            return this.likedSongs;
        } catch (e) {
            console.error("Failed to fetch liked songs:", e);
            return [];
        }
    }

    async fetchSavedAlbums(): Promise<SavedAlbum[]> {
        try {
            const albums = await invoke<SavedAlbum[]>("get_saved_albums");
            this.savedAlbums = albums || [];
            return this.savedAlbums;
        } catch (e) {
            console.error("Failed to fetch saved albums:", e);
            return [];
        }
    }

    async fetchSavedPlaylists(): Promise<SavedPlaylist[]> {
        try {
            const playlists = await invoke<SavedPlaylist[]>("get_saved_playlists");
            this.savedPlaylists = playlists || [];
            return this.savedPlaylists;
        } catch (e) {
            console.error("Failed to fetch saved playlists:", e);
            return [];
        }
    }

    async toggleLike(track: { 
        title: string; 
        artist?: string; 
        album?: string;
        canonical_key?: string;
        cover_art_url?: string;
        provider_id?: string;
        id?: any;
        local_track_id?: number;
        duration_ms?: number;
    }, fallbackArtist?: string | null): Promise<boolean> {
        try {
            const key = getCanonicalKey(track, fallbackArtist);
            const isLiked = await invoke<boolean>("toggle_track_like", { 
                canonicalKey: key,
                title: track.title,
                artist: track.artist,
                album: track.album,
                coverArtUrl: track.cover_art_url,
                providerId: track.provider_id,
                sourceId: String(track.id || key),
                localTrackId: track.local_track_id,
                durationMs: track.duration_ms,
            });
            await this.fetchLikedSongs();
            if (this.onLikeToggled) {
                this.onLikeToggled(key, isLiked);
            }
            return isLiked;
        } catch (e) {
            console.error("Failed to toggle like:", e);
            return false;
        }
    }

    async toggleSaveAlbum(album: { 
        id: string; 
        title: string; 
        artist?: string | null; 
        cover_art_url?: string | null; 
        provider_id?: string 
    }): Promise<boolean> {
        try {
            const isSaved = await invoke<boolean>("toggle_save_album", {
                id: album.id,
                title: album.title,
                artist: album.artist || null,
                coverArtUrl: album.cover_art_url || null,
                providerId: album.provider_id || "local",
            });
            await this.fetchSavedAlbums();
            if (isSaved) {
                toastStore.show(`Added "${album.title}" to Albums`, "success");
            } else {
                toastStore.show(`Removed "${album.title}" from Albums`, "info");
            }
            return isSaved;
        } catch (e) {
            console.error("Failed to toggle save album:", e);
            toastStore.show("Failed to update saved album", "error");
            return false;
        }
    }

    async toggleSavePlaylist(playlist: { 
        id: string; 
        title: string; 
        author?: string | null; 
        cover_art_url?: string | null; 
        provider_id?: string 
    }): Promise<boolean> {
        try {
            const isSaved = await invoke<boolean>("toggle_save_playlist", {
                id: playlist.id,
                title: playlist.title,
                author: playlist.author || null,
                coverArtUrl: playlist.cover_art_url || null,
                providerId: playlist.provider_id || settingsStore.getEffectiveRemoteProvider(),
            });
            await this.fetchSavedPlaylists();
            if (isSaved) {
                toastStore.show(`Saved "${playlist.title}" to Playlists`, "success");
            } else {
                toastStore.show(`Removed "${playlist.title}" from Playlists`, "info");
            }
            return isSaved;
        } catch (e) {
            console.error("Failed to toggle save playlist:", e);
            toastStore.show("Failed to update saved playlist", "error");
            return false;
        }
    }

    isLikedSong(canonicalKey: string): boolean {
        if (!canonicalKey) return false;
        const target = normalizeCanonicalString(canonicalKey);
        return this.likedSongs.some(s => normalizeCanonicalString(s.canonical_key) === target);
    }

    isAlbumSaved(id: string, title?: string, artist?: string): boolean {
        const cleanId = String(id);
        const query = { title, artist };
        return this.savedAlbums.some(a => {
            if (String(a.id) === cleanId) return true;
            if (isCanonicalEntityMatch(a, query)) return true;
            return false;
        });
    }

    isPlaylistSaved(id: string, title?: string, author?: string): boolean {
        const cleanId = id ? String(id).replace(/^(VL|playlist:)/, "") : "";
        const query = { title, artist: author };
        return this.savedPlaylists.some(p => {
            if (String(p.id) === cleanId || String(p.id) === String(id)) return true;
            const cleanPId = p.id ? String(p.id).replace(/^(VL|playlist:)/, "") : "";
            if (cleanId && cleanPId && cleanId === cleanPId) return true;
            if (isCanonicalEntityMatch({ title: p.title, artist: p.author }, query)) return true;
            return false;
        });
    }

    async scanDirectory(path: string) {
        this.isScanning = true;
        this.lastScanResult = null;
        try {
            const count = await invoke<number>("scan_local_directory", { path });
            this.lastScanResult = count;
            await this.fetchAlbums();
            await this.fetchPlaylists();
        } catch (e) {
            console.error(e);
        } finally {
            this.isScanning = false;
        }
    }

    async getAlbumTracks(albumId: number): Promise<LocalTrack[]> {
        return await invoke("get_album_tracks", { albumId, limit: 500, offset: 0 });
    }

    async getPlaylistTracks(playlistId: number): Promise<LocalTrack[]> {
        return await invoke("get_playlist_tracks", { playlistId, limit: 500, offset: 0 });
    }

    async saveQueueAsPlaylist(name: string, tracks: any[]): Promise<number> {
        const id = await invoke<number>("save_queue_as_playlist", { name, tracks });
        await this.fetchPlaylists();
        toastStore.show(`Saved ${tracks.length} track${tracks.length === 1 ? "" : "s"} to "${name}"`, "success");
        return id;
    }

    async createPlaylist(name: string): Promise<number> {
        const id = await invoke<number>("create_playlist", { name });
        await this.fetchPlaylists();
        return id;
    }

    async addToPlaylist(playlistId: number, trackId: number) {
        await invoke("add_to_playlist", { playlistId, trackId });
    }

    async removeFromPlaylist(playlistId: number, trackId: number) {
        await invoke("remove_from_playlist", { playlistId, trackId });
    }

    async deletePlaylist(playlistId: number) {
        await invoke("delete_playlist", { playlistId });
        await this.fetchPlaylists();
    }

    async renamePlaylist(playlistId: number, newName: string) {
        await invoke("rename_playlist", { playlistId, newName });
        await this.fetchPlaylists();
    }

    async reorderPlaylistTrack(playlistId: number, fromPos: number, toPos: number) {
        await invoke("reorder_playlist_track", { playlistId, fromPos, toPos });
    }

    async getPlaylistArtworkMosaic(playlistId: number): Promise<string[]> {
        const tracks = await this.getPlaylistTracks(playlistId);
        const artworkUrls: string[] = [];
        for (const track of tracks) {
            if (artworkUrls.length >= 4) break;
            const url = await this.getArtworkUrl(track.id, track.file_path);
            if (url && !artworkUrls.includes(url)) {
                artworkUrls.push(url);
            }
        }
        return artworkUrls;
    }

    async getArtworkUrl(trackId: number, filePath: string): Promise<string | null> {
        try {
            const cachedPath = await invoke<string | null>("extract_and_cache_artwork", { trackId, filePath });
            if (cachedPath) {
                return convertFileSrc(cachedPath);
            }
        } catch (e) {
            console.error(e);
        }
        return null;
    }

    async playPlaylist(playlistId: number) {
        try {
            const tracks = await this.getPlaylistTracks(playlistId);
            if (tracks && tracks.length > 0) {
                const queueTracks = tracks.map(t => ({
                    id: t.id,
                    track_id: t.id,
                    title: t.title,
                    artist: t.artist || "Unknown Artist",
                    file_path: t.file_path,
                    is_local: true,
                    duration_ms: 210000,
                }));
                await audioStore.setQueue(queueTracks, 0);
            }
        } catch (e) {
            console.error("Failed to play local playlist:", e);
        }
    }

    async playAlbum(albumId: number) {
        try {
            const tracks = await this.getAlbumTracks(albumId);
            if (tracks && tracks.length > 0) {
                const queueTracks = tracks.map(t => ({
                    id: t.id,
                    track_id: t.id,
                    title: t.title,
                    artist: t.artist || "Unknown Artist",
                    file_path: t.file_path,
                    is_local: true,
                    duration_ms: 210000,
                }));
                await audioStore.setQueue(queueTracks, 0);
            }
        } catch (e) {
            console.error("Failed to play local album:", e);
        }
    }
    
    async clearLibrary() {
        try {
            await invoke("clear_local_library");
            this.albums = [];
            this.playlists = [];
        } catch(e) {
            console.error(e);
        }
    }
}

export const libraryStore = new LibraryStore();
