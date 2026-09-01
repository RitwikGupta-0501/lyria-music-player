export function getCanonicalKey(
    track: { title: string; artist?: string | null; canonical_key?: string },
    fallbackArtist?: string | null
): string {
    if (track.canonical_key && track.canonical_key.trim()) {
        return track.canonical_key.trim().toLowerCase();
    }
    const artist = (track.artist || fallbackArtist || "unknown").trim().toLowerCase();
    const title = (track.title || "unknown").trim().toLowerCase();
    return `${artist}::${title}`;
}

import { invoke } from "@tauri-apps/api/core";
import { convertFileSrc } from '@tauri-apps/api/core';
import { toastStore } from './toast.svelte';

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
    likedSongs = $state<any[]>([]);
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

    async fetchLikedSongs(): Promise<any[]> {
        try {
            const songs = await invoke<any[]>("get_liked_songs", { limit: 500 });
            this.likedSongs = songs || [];
            return this.likedSongs;
        } catch (e) {
            console.error("Failed to fetch liked songs:", e);
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

    async scanDirectory(path: string) {
        if (!path) return;
        this.isScanning = true;
        this.lastScanResult = null;
        try {
            const added = await invoke<number>("scan_local_directory", { path });
            this.lastScanResult = added;
            await this.fetchAlbums();
            await this.fetchRecentAlbums();
            toastStore.success(`Scan complete: ${added} new track${added !== 1 ? 's' : ''} added.`);
        } catch (e) {
            console.error("Scan error:", e);
            toastStore.error(`Scan failed: ${e}`);
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
    
    async createPlaylist(name: string) {
        await invoke("create_playlist", { name });
        await this.fetchPlaylists();
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
