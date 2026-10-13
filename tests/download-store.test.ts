// @ts-nocheck
import { describe, it, expect, mock } from "bun:test";

let mockInvokedCalls: Array<{ cmd: string; args: any }> = [];

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
        if (cmd === "get_active_download_queue") {
            return Promise.resolve([]);
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
});
