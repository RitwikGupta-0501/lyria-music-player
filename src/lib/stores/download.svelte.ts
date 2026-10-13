import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { toastStore } from "./toast.svelte";

export interface DownloadCandidateItem {
    native_id: string;
    title: string;
    artist: string;
    album?: string | null;
    duration_ms?: number | null;
    artwork_url?: string | null;
    codec?: string | null;
    bitrate_kbps?: number | null;
    extra_info?: string | null;
}

export interface ProviderCandidate {
    id: string;
    name: string;
    priority: number;
    is_preferred: boolean;
}

export interface DownloadRecord {
    id: string;
    entity_type: string;
    entity_id?: string | null;
    title: string;
    artist: string;
    album?: string | null;
    total_items: number;
    completed_items: number;
    status: string;
    created_at?: string | null;
    updated_at?: string | null;
}

export interface DownloadItemRecord {
    id: string;
    download_id: string;
    track_title: string;
    artist: string;
    album?: string | null;
    track_number?: number | null;
    provider_id?: string | null;
    download_url?: string | null;
    dest_file_path?: string | null;
    temp_file_path?: string | null;
    total_bytes: number;
    downloaded_bytes: number;
    status: string;
    error_message?: string | null;
    retry_count: number;
    created_at?: string | null;
    updated_at?: string | null;
    native_id?: string | null;
}

export interface DownloadJobSummary {
    download: DownloadRecord;
    items: DownloadItemRecord[];
}

export interface DownloadProgressPayload {
    item_id: string;
    download_id: string;
    downloaded_bytes: number;
    total_bytes: number;
    speed_bytes_per_sec: number;
}

export interface DownloadStatusPayload {
    item_id: string;
    download_id: string;
    status: string;
    error?: string | null;
}

export class DownloadStore {
    providers = $state<ProviderCandidate[]>([]);
    preferredProvider = $state<string | null>(null);
    queue = $state<DownloadJobSummary[]>([]);
    progressMap = $state<Record<string, DownloadProgressPayload>>({});
    downloadsDir = $state<string>("");

    // Manual Candidate Picker State
    isCandidateSearchOpen = $state(false);
    searchQuery = $state("");
    searchProviderId = $state<string | null>(null);
    searchResults = $state<DownloadCandidateItem[]>([]);
    isSearching = $state(false);
    searchError = $state<string | null>(null);
    targetTrackInfo = $state<{
        title: string;
        artist: string;
        album?: string | null;
        duration_ms?: number | null;
    } | null>(null);

    private unlisteners: UnlistenFn[] = [];

    async init() {
        await this.fetchProviders();
        await this.fetchQueue();
        await this.fetchDownloadsDir();
        await this.setupEventListeners();
    }

    async setupEventListeners() {
        const unlistenProgress = await listen<DownloadProgressPayload>(
            "download://progress",
            (event) => {
                this.progressMap[event.payload.item_id] = event.payload;
            }
        );

        const unlistenStatus = await listen<DownloadStatusPayload>(
            "download://status",
            async (event) => {
                const foundItem = this.findItemById(event.payload.item_id);
                if (event.payload.status === "completed") {
                    if (foundItem) {
                        toastStore.success(`Downloaded "${foundItem.track_title}"`);
                    }
                } else if (event.payload.status === "failed") {
                    toastStore.error(`Failed to download ${foundItem ? `"${foundItem.track_title}"` : "track"}: ${event.payload.error || "Unknown error"}`);
                }
                await this.fetchQueue();
            }
        );

        this.unlisteners.push(unlistenProgress, unlistenStatus);
    }

    findItemById(itemId: string): DownloadItemRecord | null {
        for (const job of this.queue) {
            for (const item of job.items) {
                if (item.id === itemId) return item;
            }
        }
        return null;
    }

    get activeCount(): number {
        let count = 0;
        for (const job of this.queue) {
            for (const item of job.items) {
                if (item.status === "downloading" || item.status === "queued") {
                    count++;
                }
            }
        }
        return count;
    }

    isTrackDownloading(title?: string | null, artist?: string | null): boolean {
        if (!title) return false;
        const normTitle = title.trim().toLowerCase();
        const normArtist = artist ? artist.trim().toLowerCase() : "";
        for (const job of this.queue) {
            for (const item of job.items) {
                if (item.status === "downloading" || item.status === "queued") {
                    if (item.track_title.trim().toLowerCase() === normTitle) {
                        if (!normArtist || item.artist.trim().toLowerCase() === normArtist) {
                            return true;
                        }
                    }
                }
            }
        }
        return false;
    }

    isTrackDownloaded(title?: string | null, artist?: string | null): boolean {
        if (!title) return false;
        const normTitle = title.trim().toLowerCase();
        const normArtist = artist ? artist.trim().toLowerCase() : "";
        for (const job of this.queue) {
            for (const item of job.items) {
                if (item.status === "completed") {
                    if (item.track_title.trim().toLowerCase() === normTitle) {
                        if (!normArtist || item.artist.trim().toLowerCase() === normArtist) {
                            return true;
                        }
                    }
                }
            }
        }
        return false;
    }

    async fetchProviders() {
        try {
            const list = await invoke<ProviderCandidate[]>("get_download_providers");
            this.providers = list || [];
            const pref = await invoke<string | null>("get_preferred_download_provider");
            this.preferredProvider = pref || (this.providers[0]?.id ?? null);
        } catch (e) {
            console.error("Failed to fetch download providers:", e);
        }
    }

    async setPreferredProvider(providerId: string | null) {
        try {
            await invoke("set_preferred_download_provider", { providerId });
            this.preferredProvider = providerId;
        } catch (e) {
            console.error("Failed to set preferred download provider:", e);
        }
    }

    async fetchQueue() {
        try {
            const queue = await invoke<DownloadJobSummary[]>("get_download_queue");
            this.queue = queue || [];
        } catch (e) {
            console.error("Failed to fetch download queue:", e);
        }
    }

    async downloadTrack(params: {
        title: string;
        artist: string;
        album?: string | null;
        track_number?: number | null;
        duration_ms?: number | null;
        native_id?: string | null;
        preferred_provider?: string | null;
    }): Promise<string> {
        toastStore.info(`Starting download: "${params.title}"`);
        const downloadId = await invoke<string>("download_track", {
            title: params.title,
            artist: params.artist,
            album: params.album ?? null,
            trackNumber: params.track_number ?? null,
            durationMs: params.duration_ms ?? null,
            nativeId: params.native_id ?? null,
            preferredProvider: params.preferred_provider ?? this.preferredProvider,
        });

        await this.fetchQueue();
        return downloadId;
    }

    async downloadAlbum(params: {
        album_title: string;
        artist: string;
        tracks: Array<{
            title: string;
            track_number?: number | null;
            duration_ms?: number | null;
            native_id?: string | null;
        }>;
        preferred_provider?: string | null;
    }): Promise<string> {
        toastStore.info(`Starting album download: "${params.album_title}" (${params.tracks.length} tracks)`);
        const downloadId = await invoke<string>("download_album", {
            albumTitle: params.album_title,
            artist: params.artist,
            tracks: params.tracks.map((t) => ({
                title: t.title,
                track_number: t.track_number ?? null,
                duration_ms: t.duration_ms ?? null,
                native_id: t.native_id ?? null,
            })),
            preferredProvider: params.preferred_provider ?? this.preferredProvider,
        });

        await this.fetchQueue();
        return downloadId;
    }

    openCandidateSearch(track: {
        title: string;
        artist: string;
        album?: string | null;
        duration_ms?: number | null;
    }, providerId?: string) {
        this.targetTrackInfo = track;
        this.searchQuery = `${track.title} ${track.artist}`.trim();
        this.searchProviderId = providerId || this.preferredProvider || (this.providers[0]?.id ?? null);
        this.searchResults = [];
        this.searchError = null;
        this.isCandidateSearchOpen = true;

        if (this.searchProviderId) {
            this.executeCandidateSearch(this.searchQuery, this.searchProviderId);
        }
    }

    closeCandidateSearch() {
        this.isCandidateSearchOpen = false;
        this.targetTrackInfo = null;
        this.searchResults = [];
        this.searchError = null;
    }

    async executeCandidateSearch(query: string, providerId: string) {
        if (!query.trim()) return;

        this.isSearching = true;
        this.searchError = null;

        try {
            const results = await invoke<DownloadCandidateItem[]>("search_download_candidates", {
                providerId,
                query: query.trim(),
            });
            this.searchResults = results || [];
        } catch (e) {
            this.searchError = typeof e === "string" ? e : (e as Error).message;
            this.searchResults = [];
        } finally {
            this.isSearching = false;
        }
    }

    async selectCandidateAndDownload(candidate: DownloadCandidateItem): Promise<string | undefined> {
        if (!this.targetTrackInfo) return;

        const downloadId = await this.downloadTrack({
            title: this.targetTrackInfo.title,
            artist: this.targetTrackInfo.artist,
            album: this.targetTrackInfo.album,
            duration_ms: candidate.duration_ms ?? this.targetTrackInfo.duration_ms,
            native_id: candidate.native_id,
            preferred_provider: this.searchProviderId,
        });

        this.closeCandidateSearch();
        return downloadId;
    }

    async pauseDownload(downloadId: string) {
        await invoke("pause_download", { downloadId });
        await this.fetchQueue();
    }

    async resumeDownload(downloadId: string, preferredProvider?: string) {
        await invoke("resume_download", {
            downloadId,
            preferredProvider: preferredProvider || null,
        });
        await this.fetchQueue();
    }

    async cancelDownload(downloadId: string) {
        await invoke("cancel_download", { downloadId });
        await this.fetchQueue();
    }

    async fetchDownloadsDir() {
        try {
            const dir = await invoke<string>("get_downloads_dir");
            this.downloadsDir = dir || "";
        } catch (e) {
            console.error("Failed to fetch downloads dir:", e);
        }
    }

    async openDownloadsFolder() {
        try {
            await invoke("open_downloads_folder");
        } catch (e) {
            console.error("Failed to open downloads folder:", e);
        }
    }

    async clearCompletedDownloads() {
        try {
            await invoke("clear_completed_downloads");
            await this.fetchQueue();
            toastStore.info("Cleared completed downloads history");
        } catch (e) {
            console.error("Failed to clear completed downloads:", e);
        }
    }

    cleanup() {
        for (const unlisten of this.unlisteners) {
            unlisten();
        }
        this.unlisteners = [];
    }
}

export const downloadStore = new DownloadStore();
