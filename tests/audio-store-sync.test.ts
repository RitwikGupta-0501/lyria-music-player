// @ts-nocheck
import { describe, it, expect, mock } from "bun:test";

let syncHandler: ((e: { payload: any }) => void) | null = null;

mock.module("@tauri-apps/api/core", () => ({
    invoke: mock(() => Promise.resolve()),
    transformCallback: mock(() => 0),
    Channel: class {},
    convertFileSrc: (s: string) => s,
    isTauri: () => true,
}));

mock.module("@tauri-apps/api/event", () => ({
    listen: mock((event: string, handler: (e: { payload: any }) => void) => {
        if (event === "player-sync") {
            syncHandler = handler;
        }
        return Promise.resolve(() => {});
    }),
    emit: mock(() => Promise.resolve()),
}));

describe("AudioStore — queue exhaustion emits Stopped (BUG-06)", () => {
    it("sets playbackState to Stopped when player-sync Stopped is received", async () => {
        const { audioStore } = await import("../src/lib/stores/audio.svelte");
        audioStore.playbackState = "Playing";

        await audioStore.init();

        // Simulate backend emitting Stopped after queue exhaustion
        if (syncHandler) {
            syncHandler({
                payload: { state: "Stopped", position: 0, duration: 0, track: "" },
            });
        }

        expect(audioStore.playbackState).toBe("Stopped");
        expect(audioStore.currentTime).toBe(0);
    });
});
