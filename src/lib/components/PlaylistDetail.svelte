<script lang="ts">
    import {
        libraryStore,
        type Playlist,
        type LocalTrack,
    } from "$lib/stores/library.svelte";
    import { audioStore } from "$lib/stores/audio.svelte";
    import { toastStore } from "$lib/stores/toast.svelte";
    import {
        DotsThreeIcon,
        PlayIcon,
        PauseIcon,
        PlusIcon,
        TrashIcon,
        PencilIcon,
    } from "phosphor-svelte";
    import { createVirtualizer } from "@tanstack/svelte-virtual";
    import { exploreStore, type AlbumDetailResult } from "$lib/stores/explore.svelte";

    let { playlist = null, remotePlaylist = null, onBack, onDeleted } = $props<{ 
        playlist?: Playlist | null; 
        remotePlaylist?: AlbumDetailResult | null; 
        onBack: () => void; 
        onDeleted: () => void; 
    }>();

    let effectiveRemote = $derived(remotePlaylist || exploreStore.selectedRemotePlaylist);
    let tracks = $state<any[]>([]);
    let artUrls = $state<string[]>([]);
    let isEditingName = $state(false);
    let editName = $state("");

    let playlistTitle = $derived(effectiveRemote?.title || playlist?.name || "Playlist");
    let playlistAuthor = $derived(effectiveRemote?.artist || "Curated Playlist");
    let artUrl = $derived(effectiveRemote?.cover_art_url || (artUrls.length > 0 ? artUrls[0] : null));

    let activeDropdown = $state<number | null>(null);

    let scrollContainer = $state<HTMLElement | null>(null);
    let virtStore = $derived.by(() => {
        const container = scrollContainer;
        return createVirtualizer({
            count: tracks.length,
            getScrollElement: () => container,
            estimateSize: () => 52,
            overscan: 10,
        });
    });

    async function loadData() {
        if (effectiveRemote) {
            tracks = (effectiveRemote.tracks || []).map((t: any, idx: number) => ({
                id: t.id || idx,
                title: t.title,
                artist: t.artist,
                album: effectiveRemote.title,
                file_path: t.id,
                duration_ms: t.duration_ms,
                cover_art_url: t.cover_art_url || effectiveRemote.cover_art_url,
            }));
        } else if (playlist) {
            tracks = await libraryStore.getPlaylistTracks(playlist.id);
            artUrls = await libraryStore.getPlaylistArtworkMosaic(playlist.id);
            if (!isEditingName) {
                editName = playlist.name;
            }
        }
    }

    $effect(() => {
        loadData();

        const closeDropdowns = () => {
            activeDropdown = null;
        };
        document.addEventListener("click", closeDropdowns);
        return () => document.removeEventListener("click", closeDropdowns);
    });

    function toggleDropdown(e: Event, index: number) {
        e.stopPropagation();
        activeDropdown = activeDropdown === index ? null : index;
    }

    async function removeTrack(e: Event, trackId: number) {
        e.stopPropagation();
        if (playlist) {
            await libraryStore.removeFromPlaylist(playlist.id, trackId);
            await loadData();
        }
        activeDropdown = null;
    }

    async function deletePlaylist() {
        if (playlist && confirm("Are you sure you want to delete this playlist?")) {
            await libraryStore.deletePlaylist(playlist.id);
            onDeleted();
        }
    }

    async function saveName() {
        if (playlist && editName.trim() && editName !== playlist.name) {
            await libraryStore.renamePlaylist(playlist.id, editName);
        }
        isEditingName = false;
    }

    async function playTrack(index: number) {
        if (tracks.length === 0) return;

        const current = tracks[index];

        if (effectiveRemote) {
            const pId = current.provider_id || effectiveRemote.provider_id || "youtube-wasm";
            const trackPayload = {
                id: current.id,
                title: current.title,
                artist: current.artist,
                album: effectiveRemote.title,
                remote_track_id: current.id,
                provider_id: pId,
                cover_art_url: current.cover_art_url || effectiveRemote.cover_art_url,
                duration_ms: current.duration_ms,
            };

            if (audioStore.queue.length > 0) {
                if (audioStore.trackClickBehavior === "interrupt") {
                    await audioStore.playInterrupt(trackPayload);
                    return;
                } else if (audioStore.trackClickBehavior === "append") {
                    await audioStore.addToQueue(trackPayload);
                    toastStore.show("Added to queue", "info", 1500);
                    return;
                }
            }
            const queueTracks = tracks.map((t) => ({
                id: t.id,
                title: t.title,
                artist: t.artist,
                album: effectiveRemote.title,
                remote_track_id: t.id,
                provider_id: t.provider_id || pId,
                cover_art_url: t.cover_art_url || effectiveRemote.cover_art_url,
                duration_ms: t.duration_ms,
            }));
            await audioStore.setQueue(queueTracks, index);
            return;
        }

        if (playlist) {
            if (audioStore.queue.length > 0) {
                const trackPayload = {
                    id: current.id,
                    title: current.title,
                    artist: current.artist,
                    album: playlist.name,
                    file_path: current.file_path,
                };

                if (audioStore.trackClickBehavior === "interrupt") {
                    await audioStore.playInterrupt(trackPayload);
                    return;
                } else if (audioStore.trackClickBehavior === "append") {
                    await audioStore.addToQueue(trackPayload);
                    toastStore.show("Added to queue", "info", 1500);
                    return;
                }
            }

            const queueTracks = tracks.map((t) => ({
                id: t.id,
                title: t.title,
                artist: t.artist,
                album: playlist.name,
                file_path: t.file_path,
            }));

            await audioStore.setQueue(queueTracks, index);
        }
    }
</script>

<div class="view-album">
    <div class="album-header">
        <div class="art-container">
            {#if artUrl}
                <img src={artUrl} alt={playlistTitle} class="art-img" loading="eager" />
            {:else}
                <div class="art-placeholder font-headline-lg">
                    <span>{playlistTitle.charAt(0).toUpperCase()}</span>
                </div>
            {/if}
        </div>
        <div class="album-info">
            {#if isEditingName && playlist}
                <div class="edit-name-row">
                    <input 
                        type="text" 
                        bind:value={editName} 
                        onkeydown={(e) => e.key === 'Enter' && saveName()}
                        onblur={saveName}
                        class="edit-name-input"
                    />
                    <button class="save-name-btn" onclick={saveName}>Save</button>
                </div>
            {:else}
                <h3 class="album-title font-headline-lg">
                    {playlistTitle}
                    {#if playlist}
                        <button class="icon-action-btn" onclick={() => isEditingName = true} title="Rename playlist">
                            <PencilIcon size={14} />
                        </button>
                    {/if}
                </h3>
            {/if}
            <p class="album-artist">
                {playlistAuthor} • {tracks.length} {tracks.length === 1 ? "track" : "tracks"}
            </p>
            {#if playlist}
                <button class="delete-playlist-link" onclick={deletePlaylist}>
                    <TrashIcon size={13} />
                    <span>Delete Playlist</span>
                </button>
            {/if}
        </div>
    </div>

    <div class="track-list" bind:this={scrollContainer}>
        {#if exploreStore.isLoadingPlaylist && tracks.length === 0}
            <div class="album-tracks-loading">
                <div class="track-skeleton-row"></div>
                <div class="track-skeleton-row"></div>
                <div class="track-skeleton-row"></div>
                <div class="track-skeleton-row"></div>
                <div class="track-skeleton-row"></div>
            </div>
        {/if}
        <div
            style="position: relative; width: 100%; height: {$virtStore.getTotalSize()}px;"
        >
            {#each $virtStore.getVirtualItems() as row (row.index)}
                {@const i = row.index}
                {@const track = tracks[i]}
                <!-- svelte-ignore a11y_click_events_have_key_events -->
                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <div
                    class="track-row"
                    class:active={audioStore.currentTrack === track.title ||
                        audioStore.currentTrack === track.file_path}
                    style="position: absolute; top: 0; left: 0; width: 100%; transform: translateY({row.start}px);"
                    onclick={() => playTrack(i)}
                >
                    <div class="track-left">
                        <div class="track-status">
                            {#if audioStore.currentTrack === track.title || audioStore.currentTrack === track.file_path}
                                {#if audioStore.playbackState === "Playing"}
                                    <div class="playing-visualizer">
                                        <div class="bar"></div>
                                        <div class="bar"></div>
                                        <div class="bar"></div>
                                        <div class="bar"></div>
                                    </div>
                                {:else}
                                    <PauseIcon
                                        size={18}
                                        weight="bold"
                                        color="var(--echo-primary)"
                                    />
                                {/if}
                            {:else}
                                <span class="track-number">{i + 1}</span>
                                <PlayIcon
                                    size={18}
                                    weight="bold"
                                    class="track-play-icon"
                                />
                            {/if}
                        </div>

                        <div class="track-details">
                            <span class="track-name">{track.title}</span>
                            <span class="track-artist-sub">{track.artist || "Unknown Artist"}</span>
                        </div>
                    </div>

                    <div class="track-right">
                        {#if playlist}
                            <button
                                class="more-btn"
                                onclick={(e) => toggleDropdown(e, i)}
                                aria-label="More options"
                            >
                                <DotsThreeIcon size={20} weight="bold" />
                            </button>

                            {#if activeDropdown === i}
                                <div class="dropdown glass-panel">
                                    <button
                                        class="dropdown-row text-danger"
                                        onclick={(e) => removeTrack(e, track.id)}
                                    >
                                        <TrashIcon size={14} />
                                        Remove from Playlist
                                    </button>
                                </div>
                            {/if}
                        {/if}
                    </div>
                </div>
            {/each}
        </div>
    </div>
</div>

<style>
    .view-album {
        display: flex;
        flex-direction: column;
        position: absolute;
        inset: 0;
        padding: 1.5rem;
        padding-bottom: 8rem;
    }

    .album-header {
        display: flex;
        align-items: center;
        gap: 1.5rem;
        margin-bottom: 2rem;
    }

    .art-container {
        width: 7rem;
        height: 7rem;
        flex-shrink: 0;
        border-radius: 1rem;
        background-color: #27272a;
        box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.5);
        border: 1px solid rgba(255, 255, 255, 0.1);
        overflow: hidden;
    }

    .art-img {
        width: 100%;
        height: 100%;
        object-fit: cover;
    }

    .art-placeholder {
        width: 100%;
        height: 100%;
        background-color: var(--echo-raised);
        display: flex;
        align-items: center;
        justify-content: center;
        color: rgba(255, 255, 255, 0.4);
        font-weight: 700;
        font-size: 2rem;
    }

    .album-info {
        display: flex;
        flex-direction: column;
        gap: 0.25rem;
    }

    .album-title {
        font-size: 1.4rem;
        color: var(--echo-text-1);
        margin: 0;
        line-height: 1.2;
        display: flex;
        align-items: center;
        gap: 0.5rem;
    }

    .icon-action-btn {
        background: transparent;
        border: none;
        color: var(--echo-text-2);
        cursor: pointer;
        padding: 0.2rem;
        display: inline-flex;
        align-items: center;
        border-radius: 4px;
        transition: color 0.15s ease, background 0.15s ease;
    }

    .icon-action-btn:hover {
        color: #fff;
        background: rgba(255, 255, 255, 0.08);
    }

    .edit-name-row {
        display: flex;
        align-items: center;
        gap: 0.5rem;
    }

    .edit-name-input {
        background: rgba(255, 255, 255, 0.08);
        border: 1px solid rgba(255, 255, 255, 0.15);
        border-radius: 6px;
        color: #fff;
        padding: 0.35rem 0.6rem;
        font-size: 1rem;
        font-weight: 600;
    }

    .save-name-btn {
        background: #B58E62;
        color: #121212;
        border: none;
        border-radius: 6px;
        padding: 0.35rem 0.75rem;
        font-size: 0.8rem;
        font-weight: 700;
        cursor: pointer;
    }

    .album-artist {
        font-size: 0.875rem;
        color: var(--echo-text-2);
        margin: 0;
        font-family: var(--echo-font-body);
    }

    .delete-playlist-link {
        display: inline-flex;
        align-items: center;
        gap: 0.35rem;
        background: transparent;
        border: none;
        color: rgba(239, 68, 68, 0.8);
        font-size: 0.75rem;
        cursor: pointer;
        padding: 0;
        margin-top: 0.3rem;
        transition: color 0.15s ease;
    }

    .delete-playlist-link:hover {
        color: rgb(239, 68, 68);
        text-decoration: underline;
    }

    .track-list {
        flex: 1;
        display: flex;
        flex-direction: column;
        overflow-y: auto;
        padding-right: 0.5rem;
    }

    .album-tracks-loading {
        display: flex;
        flex-direction: column;
        gap: 0.6rem;
        padding: 0.5rem 0;
    }

    .track-skeleton-row {
        height: 42px;
        width: 100%;
        border-radius: 8px;
        background: linear-gradient(90deg, rgba(255, 255, 255, 0.03) 0%, rgba(255, 255, 255, 0.07) 50%, rgba(255, 255, 255, 0.03) 100%);
        background-size: 200% 100%;
        animation: track-skeleton-anim 1.5s infinite;
    }

    @keyframes track-skeleton-anim {
        0% { background-position: 200% 0; }
        100% { background-position: -200% 0; }
    }

    .track-row {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 0.75rem;
        border-radius: 0.75rem;
        cursor: pointer;
        transition: all 0.2s ease;
        position: relative;
    }

    .track-row:hover {
        background-color: rgba(255, 255, 255, 0.03);
    }

    .track-left {
        display: flex;
        align-items: center;
        gap: 0.8rem;
    }

    .track-status {
        width: 2rem;
        font-size: 0.875rem;
        color: var(--echo-text-2);
        display: flex;
        align-items: center;
    }

    .track-number {
        display: block;
    }

    :global(.track-play-icon) {
        display: none;
        color: var(--echo-primary-dark);
    }

    .track-row:hover .track-number {
        display: none;
    }

    .track-row:not(.active):hover :global(.track-play-icon) {
        display: block;
    }

    .track-details {
        display: flex;
        flex-direction: column;
        gap: 0.1rem;
    }

    .track-name {
        font-size: 0.9375rem;
        font-weight: 500;
        color: var(--echo-text-1);
        transition: color 0.2s ease;
        font-family: var(--echo-font-body);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        max-width: 220px;
    }

    .track-artist-sub {
        font-size: 0.75rem;
        color: var(--echo-text-3);
    }

    .track-row:hover .track-name {
        color: #ffffff;
    }

    .track-row.active .track-number {
        width: 1.5rem;
        text-align: right;
        color: var(--echo-text-3);
        font-variant-numeric: tabular-nums;
        font-size: 0.875rem;
    }

    .playing-visualizer {
        display: flex;
        align-items: flex-end;
        justify-content: center;
        gap: 2px;
        height: 14px;
        width: 1.5rem;
    }

    .playing-visualizer .bar {
        width: 3px;
        background-color: var(--echo-primary-dark);
        border-radius: 2px;
        transform-origin: bottom;
    }

    .playing-visualizer .bar:nth-child(1) { height: 100%; animation: eq-bar-1 1.2s ease-in-out infinite; }
    .playing-visualizer .bar:nth-child(2) { height: 100%; animation: eq-bar-2 1.5s ease-in-out infinite; }
    .playing-visualizer .bar:nth-child(3) { height: 100%; animation: eq-bar-3 1.1s ease-in-out infinite; }
    .playing-visualizer .bar:nth-child(4) { height: 100%; animation: eq-bar-4 1.4s ease-in-out infinite; }

    @keyframes eq-bar-1 { 0%, 100% { transform: scaleY(0.3); } 25% { transform: scaleY(0.9); } 50% { transform: scaleY(0.5); } 75% { transform: scaleY(1); } }
    @keyframes eq-bar-2 { 0%, 100% { transform: scaleY(0.6); } 25% { transform: scaleY(0.2); } 50% { transform: scaleY(1); } 75% { transform: scaleY(0.4); } }
    @keyframes eq-bar-3 { 0%, 100% { transform: scaleY(0.8); } 25% { transform: scaleY(0.4); } 50% { transform: scaleY(0.9); } 75% { transform: scaleY(0.3); } }
    @keyframes eq-bar-4 { 0%, 100% { transform: scaleY(0.4); } 25% { transform: scaleY(1); } 50% { transform: scaleY(0.3); } 75% { transform: scaleY(0.8); } }

    .track-right {
        display: flex;
        align-items: center;
        position: relative;
    }

    .more-btn {
        background: transparent;
        border: none;
        color: var(--echo-text-2);
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 0.25rem;
        border-radius: 0.25rem;
    }

    .more-btn:hover {
        color: var(--echo-text-1);
        background-color: rgba(255, 255, 255, 0.05);
    }

    .dropdown {
        position: absolute;
        right: 0.5rem;
        top: 2.25rem;
        width: 190px;
        border-radius: 10px;
        z-index: 20;
        overflow: hidden;
        display: flex;
        flex-direction: column;
        background: rgba(22, 22, 28, 0.95) !important;
    }

    .dropdown-row {
        width: 100%;
        text-align: left;
        background: transparent;
        border: none;
        border-radius: 0;
        color: var(--echo-text-1);
        font-size: 0.8rem;
        font-weight: 400;
        padding: 0.5rem 0.875rem;
        cursor: pointer;
        display: flex;
        align-items: center;
        gap: 0.4rem;
        transition: background 0.1s ease;
    }

    .dropdown-row:hover {
        background: rgba(255, 255, 255, 0.06);
    }

    .dropdown-row.text-danger {
        color: rgb(239, 68, 68);
    }
</style>
