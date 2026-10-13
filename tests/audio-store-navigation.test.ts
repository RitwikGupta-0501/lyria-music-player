// @ts-nocheck
import { describe, it, expect, mock } from "bun:test";

let invokedCommands: Array<{ cmd: string; args?: any }> = [];

mock.module("@tauri-apps/api/core", () => ({
    invoke: mock((cmd: string, args?: any) => {
        invokedCommands.push({ cmd, args });
        if (cmd === "skip_forward") {
            // Simulate queue at terminus with repeat off (same track returned)
            return Promise.resolve({
                tracks: [
                    { instanceId: "track-1", title: "Track 1", source: { type: "Local", file_path: "/1.mp3" } },
                    { instanceId: "track-2", title: "Track 2", source: { type: "Local", file_path: "/2.mp3" } },
                ],
                current_position: 1,
                current_track: { instanceId: "track-2", title: "Track 2", source: { type: "Local", file_path: "/2.mp3" } },
                repeat_mode: "Off",
                queue_mode: "Normal",
            });
        }
        if (cmd === "get_next_track") {
            return Promise.resolve({
                instanceId: "track-3",
                title: "Track 3",
                source: { type: "Local", file_path: "/3.mp3" },
            });
        }
        return Promise.resolve();
    }),
    transformCallback: mock(() => 0),
    Channel: class {},
    convertFileSrc: (s: string) => s,
    isTauri: () => true,
}));

mock.module("@tauri-apps/api/event", () => ({
    listen: mock(() => Promise.resolve(() => {})),
    emit: mock(() => Promise.resolve()),
}));

describe("AudioStore — Navigation & Gapless Seek", () => {
    it("seek() triggers seek_audio", async () => {
        const { audioStore } = await import("../src/lib/stores/audio.svelte");

        audioStore.duration = 200;
        invokedCommands = [];

        await audioStore.seek(50);

        expect(invokedCommands.some(c => c.cmd === "seek_audio" && c.args?.position === 50)).toBe(true);
    });

    it("skipForward() does not replay last track from 0:00 when at terminus with RepeatMode::Off", async () => {
        const { audioStore } = await import("../src/lib/stores/audio.svelte");
        const { settingsStore } = await import("../src/lib/stores/settings.svelte");

        settingsStore.autoplay = false;
        audioStore.repeatMode = "Off";
        audioStore.queue = [
            { instanceId: "track-1", title: "Track 1", source: { type: "Local", file_path: "/1.mp3" } },
            { instanceId: "track-2", title: "Track 2", source: { type: "Local", file_path: "/2.mp3" } },
        ];
        audioStore.currentPosition = 1;
        audioStore.currentQueueId = "track-2";

        invokedCommands = [];
        await audioStore.skipForward();

        // Should invoke skip_forward, but NOT load_audio
        expect(invokedCommands.some(c => c.cmd === "skip_forward")).toBe(true);
        expect(invokedCommands.some(c => c.cmd === "load_audio")).toBe(false);
    });
});
