<script lang="ts">
    import { libraryStore, type Playlist } from "$lib/stores/library.svelte";
    import { exploreStore } from "$lib/stores/explore.svelte";
    import { Plus, ListPlus, Playlist as PlaylistIcon, Play, Heart } from "phosphor-svelte";
    import { createVirtualizer } from "@tanstack/svelte-virtual";
    import { onMount } from "svelte";
    import LibraryHeader from "./LibraryHeader.svelte";
    import PromptModal from "./PromptModal.svelte";

    interface DisplayPlaylist {
        id: number | string;
        name: string;
        isFavorites?: boolean;
    }

    let { activeView = $bindable("playlists"), onSelectPlaylist } = $props<{ activeView?: string, onSelectPlaylist: (p: Playlist) => void }>();

    let mosaics = $state<Record<number, string[]>>({});
    let trackCounts = $state<Record<number, number>>({});
    let promptOpen = $state(false);

    let containerWidth = $state(0);
    let cols = $derived(Math.max(1, Math.floor((containerWidth + 32) / 222))); // 190px + 32px gap

    let allPlaylists = $derived.by(() => {
        const list: DisplayPlaylist[] = [
            {
                id: "favorites",
                name: "Liked Songs",
                isFavorites: true,
            },
            ...libraryStore.playlists.map(p => ({
                id: p.id,
                name: p.name,
                isFavorites: false,
            }))
        ];
        return list;
    });

    let rows = $derived.by(() => {
        const result = [];
        const playlists = allPlaylists;
        for (let i = 0; i < playlists.length; i += cols) {
            result.push(playlists.slice(i, i + cols));
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
            {@const rowPlaylists = rows[r]}
            <div class="virtual-row" style="position: absolute; top: 0; left: 0; width: 100%; transform: translateY({virtualRow.start}px); grid-template-columns: repeat({cols}, minmax(0, 1fr));">
                {#each rowPlaylists as playlist (playlist.id)}
                    <!-- svelte-ignore a11y_click_events_have_key_events -->
                    <!-- svelte-ignore a11y_no_static_element_interactions -->
                    <div 
                        class="playlist-card group" 
                        class:favorites-card={playlist.isFavorites}
                        onclick={() => {
                            if (playlist.isFavorites) {
                                exploreStore.openFavorites();
                            } else {
                                onSelectPlaylist(playlist as Playlist);
                            }
                        }}
                    >
                        <div class="art-container" class:favorites-art-container={playlist.isFavorites}>
                            {#if playlist.isFavorites}
                                <div class="favorites-art-gradient">
                                    <Heart size={64} weight="fill" color="#D4A86E" class="favorites-heart-icon" />
                                </div>
                            {:else if mosaics[playlist.id] && mosaics[playlist.id].length >= 4}
                                <div class="mosaic-grid">
                                    <img src={mosaics[playlist.id][0]} alt="Cover" class="mosaic-img" />
                                    <img src={mosaics[playlist.id][1]} alt="Cover" class="mosaic-img" />
                                    <img src={mosaics[playlist.id][2]} alt="Cover" class="mosaic-img" />
                                    <img src={mosaics[playlist.id][3]} alt="Cover" class="mosaic-img" />
                                </div>
                            {:else if mosaics[playlist.id] && mosaics[playlist.id].length > 0}
                                <img src={mosaics[playlist.id][0]} alt={playlist.name} class="art-img" />
                            {:else}
                                <div class="art-placeholder">
                                    <PlaylistIcon size={56} weight="thin" color="rgba(255, 255, 255, 0.35)" />
                                </div>
                            {/if}

                            <div class="play-overlay">
                                <div class="play-btn">
                                    <Play weight="fill" size={28} />
                                </div>
                            </div>
                        </div>

                        <h3 class="card-title">{playlist.name}</h3>
                        {#if playlist.isFavorites}
                            <p class="card-artist">{libraryStore.likedSongs.length} {libraryStore.likedSongs.length === 1 ? 'Favorite Song' : 'Favorite Songs'}</p>
                        {:else}
                            <p class="card-artist">{trackCounts[playlist.id] ?? 0} {trackCounts[playlist.id] === 1 ? 'track' : 'tracks'}</p>
                        {/if}
                    </div>
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
        padding: 2.5rem;
        padding-bottom: 10rem;
    }
    .virtual-row {
        display: grid;
        gap: 2rem;
    }
    .playlist-card {
        display: flex;
        flex-direction: column;
        cursor: pointer;
        width: 100%;
        min-width: 0;
    }

    .art-container {
        width: 100%;
        aspect-ratio: 1;
        border-radius: 1.5rem;
        background-color: #27272a;
        border: 1px solid rgba(255, 255, 255, 0.1);
        overflow: hidden;
        box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.5), 0 4px 6px -4px rgba(0, 0, 0, 0.5);
        margin-bottom: 0.85rem;
        position: relative;
        transition: border-color 0.25s ease, box-shadow 0.25s ease;
    }

    .art-container.favorites-art-container {
        border: 1px solid rgba(181, 142, 98, 0.22);
        background: #121215;
        box-shadow: 0 10px 18px -3px rgba(0, 0, 0, 0.6), 0 4px 6px -4px rgba(0, 0, 0, 0.4);
    }

    .playlist-card:hover .art-container.favorites-art-container {
        border-color: rgba(212, 168, 110, 0.55);
        box-shadow: 0 14px 28px -4px rgba(0, 0, 0, 0.8), 0 0 20px rgba(181, 142, 98, 0.16);
    }

    .favorites-art-gradient {
        width: 100%;
        height: 100%;
        background: 
            radial-gradient(circle at 20% 20%, rgba(212, 168, 110, 0.16) 0%, rgba(181, 142, 98, 0.05) 45%, transparent 72%),
            linear-gradient(145deg, #1a1a1e 0%, #121215 55%, #0a0a0c 100%);
        display: flex;
        align-items: center;
        justify-content: center;
        transition: transform 0.6s cubic-bezier(0.4, 0, 0.2, 1);
    }

    :global(.favorites-heart-icon) {
        filter: drop-shadow(0 4px 16px rgba(212, 168, 110, 0.35));
    }

    .playlist-card:hover .favorites-art-gradient {
        transform: scale(1.05);
    }

    .art-img {
        width: 100%;
        height: 100%;
        object-fit: cover;
        display: block;
        transition: transform 0.6s cubic-bezier(0.4, 0, 0.2, 1);
    }

    .playlist-card:hover .art-img {
        transform: scale(1.05);
    }

    .mosaic-grid {
        display: grid;
        grid-template-columns: 1fr 1fr;
        grid-template-rows: 1fr 1fr;
        width: 100%;
        height: 100%;
        transition: transform 0.6s cubic-bezier(0.4, 0, 0.2, 1);
    }

    .playlist-card:hover .mosaic-grid {
        transform: scale(1.05);
    }

    .mosaic-img {
        width: 100%;
        height: 100%;
        object-fit: cover;
    }

    .art-placeholder {
        width: 100%;
        height: 100%;
        display: flex;
        align-items: center;
        justify-content: center;
        background: #1e1e24;
    }

    .play-overlay {
        position: absolute;
        inset: 0;
        background-color: rgba(0, 0, 0, 0.4);
        backdrop-filter: blur(4px);
        -webkit-backdrop-filter: blur(4px);
        opacity: 0;
        transition: opacity 0.25s ease;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .playlist-card:hover .play-overlay {
        opacity: 1;
    }

    .play-btn {
        width: 48px;
        height: 48px;
        border-radius: 50%;
        background-color: var(--echo-primary, #B58E62);
        color: #000000;
        display: flex;
        align-items: center;
        justify-content: center;
        box-shadow: 0 4px 14px rgba(0, 0, 0, 0.5);
        transform: translateY(6px);
        transition: transform 0.25s cubic-bezier(0.34, 1.56, 0.64, 1), background-color 0.2s ease;
    }

    .playlist-card:hover .play-btn {
        transform: translateY(0);
    }

    .play-btn:hover {
        background-color: #c9a276;
    }

    .card-title {
        font-family: var(--echo-font-body, inherit);
        font-size: 0.95rem;
        font-weight: 600;
        color: var(--echo-text-1, #FFFFFF);
        margin: 0;
        line-height: 1.3;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        width: 100%;
    }

    .card-artist {
        font-size: 0.8rem;
        color: var(--echo-text-2, rgba(255, 255, 255, 0.6));
        margin: 0.2rem 0 0 0;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        width: 100%;
    }
</style>
