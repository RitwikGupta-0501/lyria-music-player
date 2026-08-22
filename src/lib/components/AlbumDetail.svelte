<script lang="ts">
    import { convertFileSrc } from "@tauri-apps/api/core";
    import {
        libraryStore,
        type Album,
        type LocalTrack,
    } from "$lib/stores/library.svelte";
    import { audioStore } from "$lib/stores/audio.svelte";
    import { toastStore } from "$lib/stores/toast.svelte";
    import {
        DotsThreeIcon,
        PlayIcon,
        PauseIcon,
        PlusIcon,
        Waveform,
    } from "phosphor-svelte";
    import { createVirtualizer } from "@tanstack/svelte-virtual";

    import { exploreStore, type AlbumDetailResult } from "$lib/stores/explore.svelte";

    let { album = null, remoteAlbum = null, onBack } = $props<{ 
        album?: Album | null; 
        remoteAlbum?: AlbumDetailResult | null; 
        onBack: () => void 
    }>();

    let effectiveRemote = $derived(remoteAlbum || exploreStore.selectedRemoteAlbum);
    let tracks = $state<any[]>([]);
    let localArtUrl = $state<string | null>(null);
    let artUrl = $derived(
        effectiveRemote?.cover_art_url || 
        (album?.cover_art_path ? convertFileSrc(album.cover_art_path) : localArtUrl)
    );
    let albumTitle = $derived(effectiveRemote?.title || album?.title || "Album");
    let albumArtist = $derived(effectiveRemote?.artist || album?.artist || "Unknown Artist");

    let activeDropdown = $state<number | null>(null);
    let isCreatingPlaylistForTrack = $state<number | null>(null);
    let newPlaylistName = $state("");

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

    $effect(() => {
        if (effectiveRemote) {
            tracks = effectiveRemote.tracks || [];
        } else if (album) {
            libraryStore.getAlbumTracks(album.id).then((t) => {
                tracks = t;
                if (t && t.length > 0 && !album?.cover_art_path) {
                    libraryStore.getArtworkUrl(t[0].id, t[0].file_path).then(url => {
                        localArtUrl = url;
                    });
                }
            });
        }

        const closeDropdowns = () => {
            activeDropdown = null;
            isCreatingPlaylistForTrack = null;
        };
        document.addEventListener("click", closeDropdowns);
        return () => document.removeEventListener("click", closeDropdowns);
    });

    function toggleDropdown(e: Event, index: number) {
        e.stopPropagation();
        activeDropdown = activeDropdown === index ? null : index;
        if (activeDropdown !== index) isCreatingPlaylistForTrack = null;
    }

    async function addTrackToPlaylist(
        e: Event,
        playlistId: number,
        trackId: number,
    ) {
        e.stopPropagation();
        await libraryStore.addToPlaylist(playlistId, trackId);
        activeDropdown = null;
    }

    async function createAndAddPlaylist(e: Event, trackId: number) {
        e.stopPropagation();
        if (!newPlaylistName.trim()) return;
        await libraryStore.createPlaylist(newPlaylistName);
        const newPlaylist = libraryStore.playlists.find(
            (p) => p.name === newPlaylistName,
        );
        if (newPlaylist) {
            await libraryStore.addToPlaylist(newPlaylist.id, trackId);
        }
        newPlaylistName = "";
        isCreatingPlaylistForTrack = null;
        activeDropdown = null;
    }

    async function playTrack(index: number) {
        if (tracks.length === 0) return;

        if (effectiveRemote) {
            const current = tracks[index];
            const pId = current.provider_id || effectiveRemote.provider_id || album?.provider_id || "youtube-wasm";
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

        if (album) {
            if (audioStore.queue.length > 0) {
                const trackPayload = {
                    id: tracks[index].id,
                    title: tracks[index].title,
                    artist: tracks[index].artist,
                    album: album.title,
                    file_path: tracks[index].file_path,
                    track_number: tracks[index].track_number,
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
                album: album.title,
                file_path: t.file_path,
                track_number: t.track_number,
            }));

            await audioStore.setQueue(queueTracks, index);
        }
    }
</script>

<div class="view-album">
    <div class="album-header">
        <div class="art-container">
            {#if artUrl}
                <img src={artUrl} alt={albumTitle} class="art-img" loading="eager" />
            {:else}
                <div class="art-placeholder font-headline-lg">
                    <span>{albumTitle.charAt(0).toUpperCase()}</span>
                </div>
            {/if}
        </div>
        <div class="album-info">
            <h3 class="album-title font-headline-lg">{albumTitle}</h3>
            <p class="album-artist">
                {#if effectiveRemote && effectiveRemote.artist}
                    <button 
                        class="artist-clickable-link" 
                        onclick={() => exploreStore.openArtist({ id: effectiveRemote.artist, name: effectiveRemote.artist })}
                    >
                        {albumArtist}
                    </button>
                {:else}
                    {albumArtist}
                {/if}
            </p>
        </div>
    </div>

    <div class="track-list" bind:this={scrollContainer}>
        {#if exploreStore.isLoadingAlbum && tracks.length === 0}
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
                                <span class="track-number"
                                    >{track.track_number || i + 1}</span
                                >
                                <PlayIcon
                                    size={18}
                                    weight="fill"
                                    class="track-play-icon"
                                />
                            {/if}
                        </div>
                        <div class="track-name">{track.title}</div>
                    </div>

                    <div class="track-right">
                        <!-- svelte-ignore a11y_click_events_have_key_events -->
                        <!-- svelte-ignore a11y_no_static_element_interactions -->
                        <button
                            class="more-btn"
                            onclick={(e) => toggleDropdown(e, i)}
                            title="Add to playlist"
                        >
                            <DotsThreeIcon size={20} weight="bold" />
                        </button>

                        {#if activeDropdown === i}
                            <!-- svelte-ignore a11y_click_events_have_key_events -->
                            <div
                                class="dropdown glass"
                                onclick={(e) => e.stopPropagation()}
                            >
                                <button
                                    class="dropdown-row"
                                    onclick={(e) => {
                                        e.stopPropagation();
                                        audioStore.playNext(track);
                                        toastStore.show(
                                            "Added to play next",
                                            "info",
                                            1500,
                                        );
                                        activeDropdown = null;
                                    }}
                                >
                                    Play next
                                </button>
                                <button
                                    class="dropdown-row"
                                    onclick={(e) => {
                                        e.stopPropagation();
                                        audioStore.addToQueue(track);
                                        toastStore.show(
                                            "Added to queue",
                                            "info",
                                            1500,
                                        );
                                        activeDropdown = null;
                                    }}
                                >
                                    Add to queue
                                </button>
                                <div class="dropdown-divider"></div>

                                {#if libraryStore.playlists.length > 0}
                                    <div class="dropdown-section-label">
                                        Add to playlist
                                    </div>
                                    {#each libraryStore.playlists as playlist}
                                        <button
                                            class="dropdown-row"
                                            onclick={(e) => {
                                                addTrackToPlaylist(
                                                    e,
                                                    playlist.id,
                                                    track.id,
                                                );
                                                toastStore.show(
                                                    `Added to ${playlist.name}`,
                                                    "success",
                                                    1500,
                                                );
                                            }}
                                        >
                                            {playlist.name}
                                        </button>
                                    {/each}
                                    <div class="dropdown-divider"></div>
                                {/if}

                                {#if isCreatingPlaylistForTrack === i}
                                    <div
                                        class="new-playlist-form"
                                        onclick={(e) => e.stopPropagation()}
                                    >
                                        <input
                                            type="text"
                                            bind:value={newPlaylistName}
                                            placeholder="Playlist name"
                                        />
                                        <button
                                            class="primary"
                                            style="padding: 0.35rem 0.65rem; font-size: 0.75rem; border-radius: 7px;"
                                            onclick={(e) =>
                                                createAndAddPlaylist(
                                                    e,
                                                    track.id,
                                                )}
                                        >
                                            Create
                                        </button>
                                    </div>
                                {:else}
                                    <button
                                        class="dropdown-row new-row"
                                        onclick={(e) => {
                                            e.stopPropagation();
                                            isCreatingPlaylistForTrack = i;
                                        }}
                                    >
                                        <PlusIcon size={12} weight="bold" />
                                        New Playlist
                                    </button>
                                {/if}
                            </div>
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
        padding: 1.5rem; /* p-6 */
        padding-bottom: 8rem; /* pb-32 */
    }

    .album-header {
        display: flex;
        align-items: center;
        gap: 1.5rem; /* gap-6 */
        margin-bottom: 2rem; /* mb-8 */
    }

    .art-container {
        width: 7rem; /* w-28 */
        height: 7rem; /* h-28 */
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

    .album-info {
        display: flex;
        flex-direction: column;
    }

    .album-title {
        font-size: 1.5rem; /* text-2xl */
        color: var(--echo-text-1);
        margin-bottom: 0.25rem; /* mb-1 */
        line-height: 1.2;
    }

    .album-artist {
        font-size: 0.875rem; /* text-sm */
        color: var(--echo-text-2);
        margin: 0;
        font-family: var(--echo-font-body);
    }

    .track-list {
        flex: 1;
        display: flex;
        flex-direction: column;
        overflow-y: auto;
        padding-right: 0.5rem;
    }

    .track-row {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 0.75rem; /* p-3 */
        border-radius: 0.75rem; /* rounded-xl */
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
    }

    .track-status {
        width: 2rem; /* w-8 */
        font-size: 0.875rem; /* text-sm */
        color: var(--echo-text-2); /* text-muted */
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

    .track-name {
        font-size: 0.9375rem; /* text-[15px] */
        font-weight: 500;
        color: var(--echo-text-1);
        transition: color 0.2s ease;
        font-family: var(--echo-font-body);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        max-width: 220px;
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

    .playing-visualizer .bar:nth-child(1) {
        height: 100%;
        animation: eq-bar-1 1.2s ease-in-out infinite;
    }
    .playing-visualizer .bar:nth-child(2) {
        height: 100%;
        animation: eq-bar-2 1.5s ease-in-out infinite;
    }
    .playing-visualizer .bar:nth-child(3) {
        height: 100%;
        animation: eq-bar-3 1.1s ease-in-out infinite;
    }
    .playing-visualizer .bar:nth-child(4) {
        height: 100%;
        animation: eq-bar-4 1.4s ease-in-out infinite;
    }

    @keyframes eq-bar-1 {
        0%,
        100% {
            transform: scaleY(0.3);
        }
        25% {
            transform: scaleY(0.9);
        }
        50% {
            transform: scaleY(0.5);
        }
        75% {
            transform: scaleY(1);
        }
    }

    @keyframes eq-bar-2 {
        0%,
        100% {
            transform: scaleY(0.6);
        }
        25% {
            transform: scaleY(0.2);
        }
        50% {
            transform: scaleY(1);
        }
        75% {
            transform: scaleY(0.4);
        }
    }

    @keyframes eq-bar-3 {
        0%,
        100% {
            transform: scaleY(0.8);
        }
        25% {
            transform: scaleY(0.4);
        }
        50% {
            transform: scaleY(0.9);
        }
        75% {
            transform: scaleY(0.3);
        }
    }

    @keyframes eq-bar-4 {
        0%,
        100% {
            transform: scaleY(0.4);
        }
        25% {
            transform: scaleY(1);
        }
        50% {
            transform: scaleY(0.3);
        }
        75% {
            transform: scaleY(0.8);
        }
    }

    .track-row.active .track-status,
    .track-row.active .track-name {
        color: var(--echo-primary-dark); /* text-[#B58E62] */
    }

    .track-right {
        display: flex;
        align-items: center;
        opacity: 0;
        transition: opacity 0.2s ease;
    }

    .track-row:hover .track-right {
        opacity: 1;
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

    /* ── Dropdown menu ── */
    .dropdown {
        position: absolute;
        right: 0.5rem;
        top: 2.25rem;
        width: 210px;
        border-radius: 10px;
        z-index: 20;
        overflow: hidden;
        display: flex;
        flex-direction: column;
        background: rgba(22, 22, 28, 0.95) !important;
    }

    .dropdown-section-label {
        padding: 0.5rem 0.875rem 0.25rem;
        font-size: 0.68rem;
        font-weight: 600;
        letter-spacing: 0.08em;
        text-transform: uppercase;
        color: var(--echo-text-3);
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

    .new-row {
        color: var(--echo-text-2);
    }
    .new-row:hover {
        color: var(--echo-text-1);
    }

    .dropdown-divider {
        height: 1px;
        background: var(--echo-border);
        margin: 0.25rem 0;
    }

    .new-playlist-form {
        display: flex;
        gap: 0.4rem;
        padding: 0.5rem 0.875rem;
        align-items: center;
    }

    .new-playlist-form input {
        font-size: 0.775rem;
        padding: 0.3rem 0.5rem;
        border-radius: 6px;
        flex: 1;
        background: rgba(255, 255, 255, 0.05);
        border: 1px solid rgba(255, 255, 255, 0.1);
        color: white;
    }

    .artist-clickable-link {
        background: none;
        border: none;
        color: rgba(255, 255, 255, 0.7);
        font-family: inherit;
        font-size: inherit;
        cursor: pointer;
        padding: 0;
        text-align: left;
        transition: color 0.15s ease;
    }
    .artist-clickable-link:hover {
        color: #B58E62;
        text-decoration: underline;
    }
</style>
