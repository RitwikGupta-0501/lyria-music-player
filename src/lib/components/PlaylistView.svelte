<script lang="ts">
    import { libraryStore, type Playlist, type SavedPlaylist } from "$lib/stores/library.svelte";
    import { exploreStore } from "$lib/stores/explore.svelte";
    import { Plus } from "phosphor-svelte";
    import { createVirtualizer } from "@tanstack/svelte-virtual";
    import { onMount } from "svelte";
    import PlaylistCard from "./PlaylistCard.svelte";
    import FavoritesCard from "./FavoritesCard.svelte";
    import LibraryHeader from "./LibraryHeader.svelte";
    import PromptModal from "./PromptModal.svelte";

    type GridItem = 
        | { kind: "favorites" }
        | { kind: "playlist"; data: Playlist | SavedPlaylist; isRemote: boolean };

    let { activeView = $bindable("playlists"), onSelectPlaylist } = $props<{ activeView?: string, onSelectPlaylist: (p: Playlist) => void }>();

    let mosaics = $state<Record<string | number, string[]>>({});
    let trackCounts = $state<Record<string | number, number>>({});
    let promptOpen = $state(false);

    let containerWidth = $state(0);
    let cols = $derived(Math.max(1, Math.floor((containerWidth + 32) / 222))); // 190px + 32px gap

    let allItems = $derived.by(() => {
        const list: GridItem[] = [
            { kind: "favorites" },
            ...libraryStore.savedPlaylists.map(p => ({
                kind: "playlist" as const,
                data: p,
                isRemote: true,
            })),
            ...libraryStore.playlists.map(p => ({
                kind: "playlist" as const,
                data: p,
                isRemote: false,
            }))
        ];
        return list;
    });

    let rows = $derived.by(() => {
        const result: GridItem[][] = [];
        const items = allItems;
        for (let i = 0; i < items.length; i += cols) {
            result.push(items.slice(i, i + cols));
        }
        return result;
    });

    let mainContent = $state<HTMLElement | null>(null);
    onMount(() => {
        mainContent = document.querySelector('.main-content');
    });

    let virtStore = $derived.by(() => {
        const mc = mainContent;
        const rowCount = rows.length;
        const cw = containerWidth;
        const cCount = cols;
        const colWidth = cCount > 0 ? (cw - (cCount - 1) * 32) / cCount : 0;
        const rowHeight = colWidth > 0 ? colWidth + 52 + 32 : 320; // card + text + gap

        return createVirtualizer({
            count: rowCount,
            getScrollElement: () => mc,
            estimateSize: () => rowHeight,
            overscan: 5,
        });
    });

    async function handleCreate(name: string) {
        if (!name || !name.trim()) return;
        await libraryStore.createPlaylist(name.trim());
        promptOpen = false;
        await loadMosaics();
    }

    function handleCreatePrompt() {
        promptOpen = true;
    }

    async function loadMosaics() {
        const entries = await Promise.all(
            libraryStore.playlists.map(async p => {
                const tracks = await libraryStore.getPlaylistTracks(p.id);
                trackCounts[p.id] = tracks.length;
                return [p.id, await libraryStore.getPlaylistArtworkMosaic(p.id)] as const;
            })
        );
        mosaics = Object.fromEntries(entries);
    }

    $effect(() => {
        if (libraryStore.playlists) {
            loadMosaics();
            libraryStore.fetchLikedSongs();
        }
    });
</script>

<LibraryHeader bind:activeView>
    {#snippet actions()}
        <div style="display: flex; gap: 1rem; align-items: center;">
            <span class="text-muted">{libraryStore.playlists.length + 1} Playlists</span>
            <div style="display: flex; gap: 0.5rem;">
                <button class="ghost" onclick={handleCreatePrompt}>
                    <Plus size={16} />
                    New Playlist
                </button>
            </div>
        </div>
    {/snippet}
</LibraryHeader>

<div class="playlist-grid">
    <div bind:clientWidth={containerWidth} style="position: relative; width: 100%; height: {$virtStore.getTotalSize()}px;">
        {#each $virtStore.getVirtualItems() as virtualRow (virtualRow.index)}
            {@const r = virtualRow.index}
            {@const rowItems = rows[r]}
            <div class="virtual-row" style="position: absolute; top: 0; left: 0; width: 100%; transform: translateY({virtualRow.start}px); grid-template-columns: repeat({cols}, minmax(0, 1fr));">
                {#each rowItems as item, idx (`${item.kind}-${item.kind === 'playlist' ? item.data.id : 'favorites'}`)}
                    {#if item.kind === "favorites"}
                        <FavoritesCard
                            selected={exploreStore.activeDrawerCollection?.id === "favorites"}
                            onclick={() => exploreStore.openFavorites()}
                        />
                    {:else}
                        {@const p = item.data}
                        <PlaylistCard
                            playlist={p}
                            mosaicUrls={mosaics[p.id]}
                            trackCount={trackCounts[p.id]}
                            selected={exploreStore.activeDrawerCollection?.id === String(p.id)}
                            onclick={() => {
                                if (item.isRemote) {
                                    const remote = p as SavedPlaylist;
                                    exploreStore.openPlaylist({
                                        id: String(remote.id),
                                        title: remote.title,
                                        author: remote.author || undefined,
                                        cover_art_url: remote.cover_art_url,
                                        provider_id: remote.provider_id,
                                    });
                                } else {
                                    onSelectPlaylist(p as Playlist);
                                }
                            }}
                        />
                    {/if}
                {/each}
            </div>
        {/each}
    </div>
</div>

{#if promptOpen}
    <PromptModal
        title="Create Playlist"
        defaultValue="New Playlist"
        onSubmit={handleCreate}
        onClose={() => promptOpen = false}
    />
{/if}

<style>
    .playlist-grid {
        padding: 2.5rem 2.5rem var(--player-clearance, 10rem) 2.5rem;
        scroll-padding-bottom: var(--player-scroll-padding, 10rem);
        width: 100%;
        box-sizing: border-box;
    }

    @media (max-width: 900px) {
        .playlist-grid {
            padding: 1.5rem 1.25rem 10rem 1.25rem;
        }
    }
    .virtual-row {
        display: grid;
        gap: 2rem;
    }
</style>
