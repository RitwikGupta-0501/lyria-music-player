// @ts-nocheck
import { describe, it, expect, mock } from "bun:test";

const invokedCommands: Array<{ cmd: string; args: any }> = [];

mock.module("@tauri-apps/api/core", () => ({
    invoke: mock((cmd: string, args: any) => {
        invokedCommands.push({ cmd, args });
        if (cmd === "search_library") {
            return Promise.resolve([]);
        }
        if (cmd === "get_providers") {
            return Promise.resolve([
                {
                    id: "youtube-wasm",
                    name: "YouTube Provider",
                    status: "enabled",
                    capabilities: ["search", "resolve", "explore", "stream"],
                },
                {
                    id: "lrclib-wasm",
                    name: "LRCLIB Lyrics Provider",
                    status: "enabled",
                    capabilities: ["lyrics"],
                },
                {
                    id: "netease-wasm",
                    name: "NetEase Lyrics Provider",
                    status: "enabled",
                    capabilities: ["lyrics", "search"],
                },
                {
                    id: "disabled-song-provider",
                    name: "Disabled Provider",
                    status: "disabled",
                    capabilities: ["search"],
                },
            ]);
        }
        if (cmd === "search_provider") {
            return Promise.resolve([
                {
                    id: "yt-1",
                    title: "Test Track",
                    artist: "Test Artist",
                    album: "Test Album",
                    duration_ms: 200000,
                },
            ]);
        }
        if (cmd === "get_all_settings") {
            return Promise.resolve({});
        }
        return Promise.resolve([]);
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

describe("Search & Explore — Lyrics Extension Exclusion", () => {
    it("SearchStore.performSearch excludes providers with lyrics capability", async () => {
        const { searchStore } = await import("../src/lib/stores/search.svelte");

        invokedCommands.length = 0;
        await searchStore.performSearch("Radiohead");

        // Find all search_provider invocations
        const searchProviderCalls = invokedCommands.filter((c) => c.cmd === "search_provider");

        // Should invoke youtube-wasm
        expect(searchProviderCalls.some((c) => c.args.providerId === "youtube-wasm")).toBe(true);

        // Should strictly NOT invoke lrclib-wasm or netease-wasm even if netease declared 'search'
        expect(searchProviderCalls.some((c) => c.args.providerId === "lrclib-wasm")).toBe(false);
        expect(searchProviderCalls.some((c) => c.args.providerId === "netease-wasm")).toBe(false);
        expect(searchProviderCalls.some((c) => c.args.providerId === "disabled-song-provider")).toBe(false);
    });

    it("ExploreStore.updateAvailableSources excludes providers declaring lyrics capability", async () => {
        const { exploreStore } = await import("../src/lib/stores/explore.svelte");

        await exploreStore.updateAvailableSources();

        const sourceIds = exploreStore.availableSources.map((s) => s.id);
        expect(sourceIds).toContain("local");
        expect(sourceIds).toContain("youtube-wasm");
        expect(sourceIds).not.toContain("lrclib-wasm");
        expect(sourceIds).not.toContain("netease-wasm");
        expect(sourceIds).not.toContain("disabled-song-provider");

        // Verify getEffectiveRemoteSongProvider returns valid song provider
        const effectiveRemote = exploreStore.getEffectiveRemoteSongProvider();
        expect(effectiveRemote).toBe("youtube-wasm");
    });
});
