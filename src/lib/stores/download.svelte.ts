import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

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
                await this.fetchQueue();
            }
        );

        this.unlisteners.push(unlistenProgress, unlistenStatus);
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

    cleanup() {
        for (const unlisten of this.unlisteners) {
            unlisten();
        }
        this.unlisteners = [];
    }
}

export const downloadStore = new DownloadStore();
