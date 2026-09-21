// @ts-nocheck
import { describe, it, expect, mock } from "bun:test";

const listeners: Record<string, (e?: any) => void> = {};

mock.module("@tauri-apps/api/core", () => ({
    invoke: mock(() => Promise.resolve()),
    transformCallback: mock(() => 0),
    Channel: class {},
    convertFileSrc: (s: string) => s,
    isTauri: () => true,
}));

mock.module("@tauri-apps/api/event", () => ({
    listen: mock((event: string, handler: (e?: any) => void) => {
        listeners[event] = handler;
        return Promise.resolve(() => {
            delete listeners[event];
        });
    }),
    emit: mock(() => Promise.resolve()),
}));

describe("AudioStore — OS Hardware Media Keys (MPRIS / SMTC)", () => {
    it("wires media-play-pause, media-next, and media-prev event listeners", async () => {
        const { audioStore } = await import("../src/lib/stores/audio.svelte");

        let toggled = false;
        let skippedForward = false;
        let skippedBackward = false;

        audioStore.togglePlay = () => { toggled = true; };
        audioStore.skipForward = () => { skippedForward = true; };
        audioStore.skipBackward = () => { skippedBackward = true; };

        await audioStore.init();

        expect(typeof listeners["media-play-pause"]).toBe("function");
        expect(typeof listeners["media-next"]).toBe("function");
        expect(typeof listeners["media-prev"]).toBe("function");

        listeners["media-play-pause"]();
        expect(toggled).toBe(true);

        listeners["media-next"]();
        expect(skippedForward).toBe(true);

        listeners["media-prev"]();
        expect(skippedBackward).toBe(true);
    });
});
