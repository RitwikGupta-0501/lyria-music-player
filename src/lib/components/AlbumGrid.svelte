<script lang="ts">
    import { libraryStore, type Album, type SavedAlbum } from "$lib/stores/library.svelte";
import { exploreStore } from "$lib/stores/explore.svelte";
    import { open } from "@tauri-apps/plugin-dialog";
    import { FolderOpen, CircleNotch } from "phosphor-svelte";
    import { flip } from "svelte/animate";
    import { cubicOut } from "svelte/easing";
    import AlbumCard from "./AlbumCard.svelte";
    import { createVirtualizer } from "@tanstack/svelte-virtual";
    import { onMount } from "svelte";
    import LibraryHeader from "./LibraryHeader.svelte";
    import { isCanonicalEntityMatch } from "$lib/utils/format";

    let {
        activeView = $bindable("albums"),
        onSelectAlbum,
        selectedAlbumId,
    } = $props<{
        activeView?: string;
        onSelectAlbum: (a: Album) => void;
        selectedAlbumId?: number | string | null;
    }>();

    let containerWidth = $state(0);
    let cols = $derived(Math.max(1, Math.floor((containerWidth + 32) / 222))); // 190px + 32px gap

    let allAlbums = $derived.by(() => {
        const local = libraryStore.albums;
        const saved = libraryStore.savedAlbums;

        const localIds = new Set<string>();
        for (const a of local) {
            localIds.add(String(a.id));
        }

        const remoteSavedOnly = saved.filter(s => {
            // Explicitly exclude any saved entry tagged with provider_id "local"
            if (s.provider_id === "local") return false;
            // Exclude if ID matches a local album ID
            if (localIds.has(String(s.id))) return false;
            // Exclude if title and artist match any local album canonically
            const matchesLocal = local.some(localAlbum => isCanonicalEntityMatch(localAlbum, s));
            return !matchesLocal;
        });

        return [...remoteSavedOnly, ...local];
    });

    let rows = $derived.by(() => {
        const result = [];
        const albums = allAlbums;
        for (let i = 0; i < albums.length; i += cols) {
            result.push(albums.slice(i, i + cols));
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

    async function handleBrowse() {
        const selected = await open({
            directory: true,
            multiple: false,
            title: "Select Music Folder",
        });
        if (selected) {
            await libraryStore.scanDirectory(selected as string);
        }
    }
</script>

<!-- Library header -->
<LibraryHeader bind:activeView>
    {#snippet actions()}
        <div style="display: flex; gap: 1rem; align-items: center;">
            <span class="text-muted">{allAlbums.length} Albums</span>

            <div style="display: flex; gap: 0.5rem; align-items: center;">
                {#if libraryStore.isScanning}
                    <div
                        class="scan-status"
                        style="display: flex; align-items: center; gap: 0.25rem;"
                    >
                        <CircleNotch size={14} class="spin" />
                        <span class="text-muted">Scanning</span>
                    </div>
                {/if}

                <button class="ghost" onclick={handleBrowse}>
                    <FolderOpen size={16} weight="regular" />
                    Add Folder
                </button>
            </div>
        </div>
    {/snippet}
</LibraryHeader>

{#if allAlbums.length === 0 && !libraryStore.isScanning}
    <!-- Empty state / onboarding -->
    <div class="empty-state">
        <div class="empty-icon">
            <FolderOpen size={48} weight="thin" />
        </div>
        <h2 class="empty-heading font-headline-lg">Your library is empty</h2>
        <p class="empty-sub">Add a music folder to start your collection.</p>
        <button
            class="ghost"
            onclick={handleBrowse}
            style="margin-top: 1.75rem;"
        >
            <FolderOpen size={16} />
            Add Music Folder
        </button>
    </div>
{:else}
    <div class="album-grid">
        <div bind:clientWidth={containerWidth} style="position: relative; width: 100%; height: {$virtStore.getTotalSize()}px;">
            {#each $virtStore.getVirtualItems() as virtualRow (virtualRow.index)}
                {@const r = virtualRow.index}
                {@const rowAlbums = rows[r]}
                <div class="virtual-row" style="position: absolute; top: 0; left: 0; width: 100%; transform: translateY({virtualRow.start}px); grid-template-columns: repeat({cols}, minmax(0, 1fr));">
                    {#each rowAlbums as album (album.id)}
                        <div style="view-transition-name: album-{album.id};">
                            <AlbumCard
                                {album}
                                selected={selectedAlbumId === album.id}
                                onclick={() => {
                                    const maybeRemote = album as any;
                                    if (maybeRemote.provider_id && maybeRemote.provider_id !== "local") {
                                        exploreStore.openAlbum({
                                            id: String(maybeRemote.id),
                                            title: maybeRemote.title,
                                            artist: maybeRemote.artist || undefined,
                                            cover_art_url: maybeRemote.cover_art_url,
                                            provider_id: maybeRemote.provider_id,
                                        });
                                    } else {
                                        onSelectAlbum(album as Album);
                                    }
                                }}
                            />
                        </div>
                    {/each}
                </div>
            {/each}
        </div>
    </div>
{/if}

<style>
    /* ── Empty state ── */
    .empty-state {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        min-height: 65vh;
        text-align: center;
        gap: 0.5rem;
    }

    .empty-icon {
        color: var(--echo-text-3);
        margin-bottom: 1.25rem;
        opacity: 0.6;
    }

    .empty-heading {
        font-size: 2rem;
        font-weight: 500;
        color: var(--echo-text-1);
        letter-spacing: -0.025em;
    }

    .empty-sub {
        font-size: 1rem;
        color: var(--echo-text-2);
        max-width: 280px;
        line-height: 1.6;
    }

    /* ── Album grid ── */
    .album-grid {
        padding: 2.5rem 2.5rem var(--player-clearance, 10rem) 2.5rem;
        scroll-padding-bottom: var(--player-scroll-padding, 10rem);
        width: 100%;
        box-sizing: border-box;
    }

    @media (max-width: 900px) {
        .album-grid {
            padding: 1.5rem 1.25rem 10rem 1.25rem;
        }
    }
    .virtual-row {
        display: grid;
        gap: 2rem;
    }
</style>
