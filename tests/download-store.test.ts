// @ts-nocheck
import { describe, it, expect, mock } from "bun:test";

let mockInvokedCalls: Array<{ cmd: string; args: any }> = [];
let mockQueueData: any[] = [];

mock.module("@tauri-apps/api/core", () => ({
    invoke: mock((cmd, args) => {
        mockInvokedCalls.push({ cmd, args });
        if (cmd === "search_download_candidates") {
            return Promise.resolve([
                {
                    native_id: "test-native-123",
                    title: "Test Track",
                    artist: "Test Artist",
                    album: "Test Album",
                    duration_ms: 210000,
                    artwork_url: "https://example.com/art.jpg",
                    codec: "Flac",
                    bitrate_kbps: 1411,
                    extra_info: "Lossless FLAC"
                }
            ]);
        }
        if (cmd === "download_track") {
            return Promise.resolve("job-uuid-abc");
        }
        if (cmd === "retry_download_item") {
            const item = mockQueueData.flatMap(q => q.items).find(i => i.id === args?.itemId);
            if (item) { item.status = "queued"; item.error_message = null; }
            return Promise.resolve("dl-123");
        }
        if (cmd === "retry_download" || cmd === "retry_all_failed_downloads") {
            mockQueueData.flatMap(q => q.items).forEach(i => {
                if (i.status === "failed") { i.status = "queued"; i.error_message = null; }
            });
            return Promise.resolve();
        }
        if (cmd === "dismiss_download_item") {
            mockQueueData.forEach(q => {
                q.items = q.items.filter(i => i.id !== args?.itemId);
            });
            return Promise.resolve();
        }
        if (cmd === "get_active_download_queue" || cmd === "get_download_queue") {
            return Promise.resolve(mockQueueData);
        }
        if (cmd === "get_downloaded_tracks") {
            return Promise.resolve([
                {
                    id: 99,
                    title: "Downloaded Offline Hit",
                    artist: "Artist Offline",
                    album_id: null,
                    track_number: 1,
                    file_path: "/music/Lyria/OfflineHit.flac"
                }
            ]);
        }
        return Promise.resolve();
    }),
    listen: mock(() => Promise.resolve(() => {})),
    transformCallback: mock(() => 0),
    Channel: class {},
    convertFileSrc: (s: string) => s
}));

mock.module("@tauri-apps/api/event", () => ({
    listen: mock(() => Promise.resolve(() => {}))
}));

describe("DownloadStore — Candidate Search & Enqueue Lifecycle", () => {
    it("opens candidate search modal and searches candidates via Tauri invoke", async () => {
        const { downloadStore } = await import("../src/lib/stores/download.svelte.js");
        mockInvokedCalls = [];

        downloadStore.openCandidateSearch({ title: "Test Track", artist: "Test Artist" }, "monochrome");
        expect(downloadStore.isCandidateSearchOpen).toBe(true);
        expect(downloadStore.targetTrackInfo?.title).toBe("Test Track");
        expect(downloadStore.targetTrackInfo?.artist).toBe("Test Artist");

        await downloadStore.executeCandidateSearch("Test Track Test Artist", "monochrome");
        expect(downloadStore.searchResults.length).toBe(1);
        expect(downloadStore.searchResults[0].native_id).toBe("test-native-123");
        expect(downloadStore.searchResults[0].codec).toBe("Flac");

        const searchCall = mockInvokedCalls.find(c => c.cmd === "search_download_candidates");
        expect(searchCall).toBeDefined();
        expect(searchCall?.args).toEqual({
            query: "Test Track Test Artist",
            providerId: "monochrome"
        });
    });

    it("enqueues track with chosen candidate native_id and closes search modal", async () => {
        const { downloadStore } = await import("../src/lib/stores/download.svelte.js");
        mockInvokedCalls = [];

        downloadStore.openCandidateSearch({ title: "Test Track", artist: "Test Artist" }, "monochrome");

        const chosenCandidate = {
            native_id: "test-native-123",
            title: "Test Track (Live)",
            artist: "Test Artist",
            album: "Live Album",
            duration_ms: 215000,
            codec: "Flac"
        };

        const jobId = await downloadStore.selectCandidateAndDownload(chosenCandidate);
        expect(jobId).toBe("job-uuid-abc");
        expect(downloadStore.isCandidateSearchOpen).toBe(false);

        const dlCall = mockInvokedCalls.find(c => c.cmd === "download_track");
        expect(dlCall).toBeDefined();
        expect(dlCall?.args.nativeId).toBe("test-native-123");
        expect(dlCall?.args.preferredProvider).toBe("monochrome");
    });

    it("sanitizes unknown artist in openCandidateSearch to omit 'Unknown Artist'", async () => {
        const { downloadStore, sanitizeArtist } = await import("../src/lib/stores/download.svelte.js");
        expect(sanitizeArtist("Unknown Artist")).toBe("");
        expect(sanitizeArtist("unknown")).toBe("");
        expect(sanitizeArtist("  Unknown  ")).toBe("");
        expect(sanitizeArtist("Pink Floyd")).toBe("Pink Floyd");

        downloadStore.openCandidateSearch({ title: "Comfortably Numb", artist: "Unknown Artist" }, "monochrome");
        expect(downloadStore.searchQuery).toBe("Comfortably Numb");
        expect(downloadStore.targetTrackInfo?.artist).toBe("");

        downloadStore.openCandidateSearch({ title: "Echoes", artist: "unknown" }, "monochrome");
        expect(downloadStore.searchQuery).toBe("Echoes");

        downloadStore.openCandidateSearch({ title: "Time", artist: "" }, "monochrome");
        expect(downloadStore.searchQuery).toBe("Time");

        downloadStore.openCandidateSearch({ title: "Time", artist: "Pink Floyd" }, "monochrome");
        expect(downloadStore.searchQuery).toBe("Time Pink Floyd");
    });

    it("sanitizes unknown artist in downloadTrack and passes empty artist to backend", async () => {
        const { downloadStore } = await import("../src/lib/stores/download.svelte.js");
        mockInvokedCalls = [];

        await downloadStore.downloadTrack({
            title: "Comfortably Numb",
            artist: "Unknown Artist",
        });

        const dlCall = mockInvokedCalls.find(c => c.cmd === "download_track");
        expect(dlCall).toBeDefined();
        expect(dlCall?.args.title).toBe("Comfortably Numb");
        expect(dlCall?.args.artist).toBe("");
    });

    it("fetches downloaded tracks into libraryStore", async () => {
        const { libraryStore } = await import("../src/lib/stores/library.svelte.js");
        mockInvokedCalls = [];

        const tracks = await libraryStore.fetchDownloadedTracks();
        expect(tracks.length).toBe(1);
        expect(tracks[0].title).toBe("Downloaded Offline Hit");
        expect(tracks[0].file_path).toBe("/music/Lyria/OfflineHit.flac");
        expect(libraryStore.downloadedTracks.length).toBe(1);
        expect(mockInvokedCalls.some(c => c.cmd === "get_downloaded_tracks")).toBe(true);
    });

    it("handles retryDownloadItem, retryDownload, dismissDownloadItem, and retryAllFailed", async () => {
        const { downloadStore } = await import("../src/lib/stores/download.svelte.js");
        mockInvokedCalls = [];

        // Mock queue with a failed item
        mockQueueData = [
            {
                download: {
                    id: "dl-123",
                    entity_type: "track",
                    title: "Echoes",
                    artist: "Pink Floyd",
                    total_items: 1,
                    completed_items: 0,
                    status: "failed",
                },
                items: [
                    {
                        id: "item-echoes",
                        download_id: "dl-123",
                        track_title: "Echoes",
                        artist: "Pink Floyd",
                        total_bytes: 5000000,
                        downloaded_bytes: 0,
                        status: "failed",
                        error_message: "Network failure",
                        retry_count: 0,
                    }
                ]
            }
        ];
        downloadStore.queue = JSON.parse(JSON.stringify(mockQueueData));

        // 1. Retry individual item
        await downloadStore.retryDownloadItem("item-echoes");
        expect(mockInvokedCalls.some(c => c.cmd === "retry_download_item" && c.args.itemId === "item-echoes")).toBe(true);
        expect(downloadStore.queue[0].items[0].status).toBe("queued");
        expect(downloadStore.queue[0].items[0].error_message).toBeNull();

        // 2. Retry download job
        mockInvokedCalls = [];
        downloadStore.queue[0].items[0].status = "failed";
        downloadStore.queue[0].items[0].error_message = "HTTP 500";
        await downloadStore.retryDownload("dl-123");
        expect(mockInvokedCalls.some(c => c.cmd === "retry_download" && c.args.downloadId === "dl-123")).toBe(true);
        expect(downloadStore.queue[0].items[0].status).toBe("queued");

        // 3. Retry all failed
        mockInvokedCalls = [];
        downloadStore.queue[0].items[0].status = "failed";
        await downloadStore.retryAllFailed();
        expect(mockInvokedCalls.some(c => c.cmd === "retry_all_failed_downloads")).toBe(true);
        expect(downloadStore.queue[0].items[0].status).toBe("queued");

        // 4. Dismiss item
        mockInvokedCalls = [];
        await downloadStore.dismissDownloadItem("item-echoes");
        expect(mockInvokedCalls.some(c => c.cmd === "dismiss_download_item" && c.args.itemId === "item-echoes")).toBe(true);
    });
});
