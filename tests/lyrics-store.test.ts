// @ts-nocheck
import { describe, it, expect, mock } from "bun:test";

mock.module("@tauri-apps/api/core", () => ({
    invoke: mock((cmd, args) => {
        if (cmd === "get_lyrics_providers") {
            return Promise.resolve([
                { id: "lrclib-wasm", name: "LRCLIB Lyrics Provider", version: "1.0.0", author: "Lyria", is_preferred: true, priority: 10, status: "enabled" },
                { id: "netease-wasm", name: "NetEase Lyrics Provider", version: "1.0.0", author: "Lyria", is_preferred: false, priority: 9, status: "enabled" }
            ]);
        }
        if (cmd === "get_lyrics") {
            return Promise.resolve({
                provider_id: "lrclib-wasm",
                provider_name: "LRCLIB Lyrics Provider",
                format: "synced",
                lines: [
                    { time_ms: 1000, text: "Line 1", end_time_ms: 3000 },
                    { time_ms: 3000, text: "Line 2", end_time_ms: 6000 },
                    { time_ms: 6000, text: "Line 3", end_time_ms: 9000 }
                ],
                offset_ms: 0,
                instrumental: false
            });
        }
        return Promise.resolve();
    }),
    listen: mock(() => Promise.resolve(() => {})),
    transformCallback: mock(() => 0),
    Channel: class {},
    convertFileSrc: (s: string) => s,
    isTauri: () => true,
}));

mock.module("@tauri-apps/api/event", () => ({
    listen: mock(() => Promise.resolve(() => {})),
    emit: mock(() => Promise.resolve()),
}));

describe("LyricsStore — Synchronized Active Line Derivation & Seeking", () => {
    it("computes activeLineIndex derived from audioStore.currentTime", async () => {
        const { lyricsStore } = await import("../src/lib/stores/lyrics.svelte");
        const { audioStore } = await import("../src/lib/stores/audio.svelte");

        lyricsStore.currentLyrics = {
            provider_id: "lrclib-wasm",
            provider_name: "LRCLIB",
            format: "synced",
            lines: [
                { time_ms: 1000, text: "First line", end_time_ms: 3000 },
                { time_ms: 3000, text: "Second line", end_time_ms: 5000 },
                { time_ms: 5000, text: "Third line", end_time_ms: 8000 }
            ],
            offset_ms: 0,
            instrumental: false
        };

        expect(lyricsStore.hasLyrics).toBe(true);
        expect(lyricsStore.isSynced).toBe(true);

        // Before first line
        audioStore.currentTime = 0.5;
        expect(lyricsStore.activeLineIndex).toBe(-1);

        // During first line
        audioStore.currentTime = 1.5;
        expect(lyricsStore.activeLineIndex).toBe(0);

        // Exactly at second line
        audioStore.currentTime = 3.0;
        expect(lyricsStore.activeLineIndex).toBe(1);

        // During third line
        audioStore.currentTime = 6.2;
        expect(lyricsStore.activeLineIndex).toBe(2);
    });

    it("seekToLine delegates seek to audioStore with correct seconds", async () => {
        const { lyricsStore } = await import("../src/lib/stores/lyrics.svelte");
        const { audioStore } = await import("../src/lib/stores/audio.svelte");

        let soughtSeconds = -1;
        const originalSeek = audioStore.seek;
        audioStore.seek = (s) => {
            soughtSeconds = s;
            return Promise.resolve();
        };

        lyricsStore.currentLyrics = {
            provider_id: "lrclib-wasm",
            provider_name: "LRCLIB",
            format: "synced",
            lines: [
                { time_ms: 1500, text: "Line 1" },
                { time_ms: 4200, text: "Line 2" }
            ],
            offset_ms: 0,
            instrumental: false
        };

        lyricsStore.seekToLine(1);
        expect(soughtSeconds).toBe(4.2);

        audioStore.seek = originalSeek;
    });

    it("handles plain format and empty lyrics gracefully", async () => {
        const { lyricsStore } = await import("../src/lib/stores/lyrics.svelte");

        lyricsStore.currentLyrics = {
            provider_id: "lrclib-wasm",
            provider_name: "LRCLIB",
            format: "plain",
            lines: [],
            raw_plain: "Plain text lyrics line 1\nPlain text line 2",
            offset_ms: 0,
            instrumental: false
        };

        expect(lyricsStore.hasLyrics).toBe(true);
        expect(lyricsStore.isSynced).toBe(false);
        expect(lyricsStore.activeLineIndex).toBe(-1);
    });
});
