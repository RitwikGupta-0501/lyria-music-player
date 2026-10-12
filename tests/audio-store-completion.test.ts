// @ts-nocheck
import { describe, it, expect, mock } from "bun:test";

let invokedCommands: string[] = [];

mock.module("@tauri-apps/api/core", () => ({
    invoke: mock((cmd: string) => {
        invokedCommands.push(cmd);
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

describe("AudioStore — Queue Completion Behavior", () => {
    it("clears queue and resets player when set to collapse_idle", async () => {
        const { audioStore } = await import("../src/lib/stores/audio.svelte");
        const { settingsStore } = await import("../src/lib/stores/settings.svelte");

        settingsStore.settings["queue_completion_behavior"] = "collapse_idle";
        audioStore.currentTime = 45;
        audioStore.duration = 180;
        audioStore.currentTrack = "Test Track";
        audioStore.queue = [
            {
                instanceId: "inst-1",
                title: "Test Track",
                artist: "Artist",
                trackNumber: 1,
                source: { type: "Local", file_path: "/test.mp3" },
            },
        ];

        invokedCommands = [];
        await audioStore.handleQueueCompletion();

        expect(audioStore.currentTime).toBe(0);
        expect(audioStore.duration).toBe(0);
        expect(audioStore.currentTrack).toBe("None");
        expect(audioStore.queue.length).toBe(0);
        expect(invokedCommands).toContain("clear_queue");
    });

    it("pauses at the end of track when set to pause_end", async () => {
        const { audioStore } = await import("../src/lib/stores/audio.svelte");
        const { settingsStore } = await import("../src/lib/stores/settings.svelte");

        settingsStore.settings["queue_completion_behavior"] = "pause_end";
        audioStore.currentTime = 120;
        audioStore.duration = 180;

        await audioStore.handleQueueCompletion();

        expect(audioStore.playbackState).toBe("Paused");
        expect(audioStore.currentTime).toBe(180);
    });

    it("resets position to 0 but retains metadata on retain_stopped", async () => {
        const { audioStore } = await import("../src/lib/stores/audio.svelte");
        const { settingsStore } = await import("../src/lib/stores/settings.svelte");

        settingsStore.settings["queue_completion_behavior"] = "retain_stopped";
        audioStore.currentTime = 175;
        audioStore.duration = 180;
        audioStore.currentTrack = "Keep Track";

        await audioStore.handleQueueCompletion();

        expect(audioStore.currentTime).toBe(0);
        expect(audioStore.duration).toBe(180);
        expect(audioStore.currentTrack).toBe("Keep Track");
    });
});
