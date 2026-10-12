// @ts-nocheck
import { describe, it, expect, mock } from "bun:test";

mock.module("@tauri-apps/api/core", () => ({
    invoke: mock(() => Promise.resolve()),
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

describe("AudioStore — $derived setting delegation (BUG-04)", () => {
    it("setTrackClickBehavior delegates to settingsStore without direct assignment", async () => {
        const { settingsStore } = await import("../src/lib/stores/settings.svelte");
        const { audioStore } = await import("../src/lib/stores/audio.svelte");

        // Seed settings state directly
        settingsStore.settings = { track_click_behavior: "interrupt" };

        // Calling audioStore.setTrackClickBehavior must not throw "Cannot assign to a derived variable"
        await expect(
            audioStore.setTrackClickBehavior("append")
        ).resolves.toBeUndefined();

        expect(settingsStore.trackClickBehavior).toBe("append");
    });

    it("setQueueCompletionBehavior delegates without direct $derived write", async () => {
        const { settingsStore } = await import("../src/lib/stores/settings.svelte");
        const { audioStore } = await import("../src/lib/stores/audio.svelte");

        settingsStore.settings = { queue_completion_behavior: "retain_stopped" };

        await expect(
            audioStore.setQueueCompletionBehavior("collapse_idle")
        ).resolves.toBeUndefined();

        expect(settingsStore.queueCompletionBehavior).toBe("collapse_idle");
    });
});
