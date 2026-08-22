<script lang="ts">
    import {
        getCanonicalKey,
        libraryStore,
        type Album,
        type Playlist,
        type LocalTrack,
    } from "$lib/stores/library.svelte";
    import { audioStore } from "$lib/stores/audio.svelte";
    import { toastStore } from "$lib/stores/toast.svelte";
    import {
        Playlist as PlaylistIcon,
        Disc as DiscIcon,
        DotsThreeIcon,
        PlayIcon,
        PauseIcon,
        TrashIcon,
        PencilIcon,
        Heart as HeartIcon,
    } from "phosphor-svelte";
    import { createVirtualizer } from "@tanstack/svelte-virtual";
    import { exploreStore, type DrawerCollection } from "$lib/stores/explore.svelte";
    import { convertFileSrc } from "@tauri-apps/api/core";

    let { 
        collection, 
        onBack, 
        onDeleted 
    } = $props<{ 
        collection: DrawerCollection; 
        onBack: () => void; 
        onDeleted?: () => void; 
    }>();

    let tracks = $state<any[]>([]);
    let localArtUrl = $state<string | null>(null);
    let mosaicUrls = $state<string[]>([]);
    let isEditingName = $state(false);
    let editName = $state("");
    let activeDropdown = $state<number | null>(null);

    let isRemote = $derived(collection.source === "remote");
    let isPlaylist = $derived(collection.kind === "playlist");
    let isFavorites = $derived(collection.id === "favorites");
    let isCustomLocalPlaylist = $derived(!isRemote && isPlaylist && !isFavorites);

    let likedKeys = $derived(new Set(libraryStore.likedSongs.map(s => s.canonical_key.toLowerCase().trim())));

    let favoriteTracks = $derived(
        libraryStore.likedSongs.map((s, idx) => ({
            id: s.last_source_id || s.local_track_id || idx,
            title: s.title,
            artist: s.artist,
            album: s.album,
            file_path: s.local_file_path || s.file_path,
            cover_art_url: s.cover_art_url,
            provider_id: s.last_provider_id || (s.local_file_path || s.file_path ? undefined : "youtube-wasm"),
            duration_ms: s.duration_ms,
            canonical_key: s.canonical_key,
            liked: true,
        }))
    );

    let displayTracks = $derived(isFavorites ? favoriteTracks : tracks);

    let artUrl = $derived(
        collection.cover_art_url ||
        (collection.cover_art_path ? convertFileSrc(collection.cover_art_path) : localArtUrl) ||
        (mosaicUrls.length > 0 ? mosaicUrls[0] : null)
    );

    let collectionTitle = $derived(collection.title || (isPlaylist ? "Playlist" : "Album"));
    let collectionSubtitle = $derived(
        collection.subtitle || 
        (isPlaylist ? "Playlist" : "Unknown Artist")
    );

    let scrollContainer = $state<HTMLElement | null>(null);
    let virtStore = $derived.by(() => {
        const container = scrollContainer;
        return createVirtualizer({
            count: displayTracks.length,
            getScrollElement: () => container,
            estimateSize: () => 52,
            overscan: 10,
        });
    });

    async function loadData() {
        if (collection.id === "favorites") {
            tracks = collection.tracks || [];
        } else if (collection.source === "remote") {
            tracks = collection.tracks || [];
        } else if (collection.kind === "album") {
            const albumId = Number(collection.id);
            const localTracks = await libraryStore.getAlbumTracks(albumId);
            tracks = localTracks;
            if (localTracks.length > 0 && !collection.cover_art_path) {
                localArtUrl = await libraryStore.getArtworkUrl(localTracks[0].id, localTracks[0].file_path);
            }
        } else if (collection.kind === "playlist") {
            const playlistId = Number(collection.id);
            tracks = await libraryStore.getPlaylistTracks(playlistId);
            mosaicUrls = await libraryStore.getPlaylistArtworkMosaic(playlistId);
            if (!isEditingName) {
                editName = collection.title;
            }
        }
    }

    $effect(() => {
        // Reload whenever collection ID or tracks change
        if (collection) {
            loadData();
        }

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
        if (isPlaylist) {
            await libraryStore.removeFromPlaylist(Number(collection.id), trackId);
            await loadData();
        }
        activeDropdown = null;
    }

    async function deletePlaylist() {
        if (isCustomLocalPlaylist && confirm("Are you sure you want to delete this playlist?")) {
            await libraryStore.deletePlaylist(Number(collection.id));
            if (onDeleted) onDeleted();
            exploreStore.closeDrawerCollection();
        }
    }

    async function saveName() {
        if (isCustomLocalPlaylist && editName.trim() && editName !== collection.title) {
            await libraryStore.renamePlaylist(Number(collection.id), editName.trim());
            collection.title = editName.trim();
        }
        isEditingName = false;
    }

    async function playTrack(index: number) {
        if (displayTracks.length === 0) return;

        const current = displayTracks[index];

        if (isRemote) {
            const pId = current.provider_id || collection.provider_id || "youtube-wasm";
            const trackPayload = {
                id: current.id,
                title: current.title,
                artist: current.artist,
                album: collection.title,
                remote_track_id: current.id,
                provider_id: pId,
                cover_art_url: current.cover_art_url || collection.cover_art_url,
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

            const queueTracks = displayTracks.map((t) => ({
                id: t.id,
                title: t.title,
                artist: t.artist,
                album: collection.title,
                remote_track_id: t.id,
                provider_id: t.provider_id || pId,
                cover_art_url: t.cover_art_url || collection.cover_art_url,
                duration_ms: t.duration_ms,
            }));
            await audioStore.setQueue(queueTracks, index);
            return;
        }

        // Local Playback
        if (audioStore.queue.length > 0) {
            const trackPayload = {
                id: current.id,
                title: current.title,
                artist: current.artist,
                album: collection.title,
                file_path: current.file_path,
                track_number: current.track_number,
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

        const queueTracks = displayTracks.map((t) => ({
            id: t.id,
            title: t.title,
            artist: t.artist,
            album: collection.title,
            file_path: t.file_path,
            track_number: t.track_number,
        }));

        await audioStore.setQueue(queueTracks, index);
    }
</script>

<div class="view-album">
    <div class="album-header">
        <div class="art-container" class:favorites-art-container={collection.id === "favorites"}>
            {#if collection.id === "favorites"}
                <div class="favorites-art-gradient">
                    <HeartIcon size={54} weight="fill" color="#D4A86E" class="favorites-heart-icon" />
                </div>
            {:else if collection.cover_art_url}
                <img src={collection.cover_art_url} alt={collectionTitle} class="art-img" loading="eager" />
            {:else if collection.cover_art_path}
                <img src={convertFileSrc(collection.cover_art_path)} alt={collectionTitle} class="art-img" loading="eager" />
            {:else if localArtUrl}
                <img src={localArtUrl} alt={collectionTitle} class="art-img" loading="eager" />
            {:else if mosaicUrls.length >= 4}
                <div class="mosaic-grid">
                    <img src={mosaicUrls[0]} alt="Cover" class="mosaic-img" />
                    <img src={mosaicUrls[1]} alt="Cover" class="mosaic-img" />
                    <img src={mosaicUrls[2]} alt="Cover" class="mosaic-img" />
                    <img src={mosaicUrls[3]} alt="Cover" class="mosaic-img" />
                </div>
            {:else if mosaicUrls.length > 0}
                <img src={mosaicUrls[0]} alt={collectionTitle} class="art-img" loading="eager" />
            {:else if isPlaylist}
                <div class="art-placeholder">
                    <PlaylistIcon size={40} weight="thin" color="rgba(255, 255, 255, 0.35)" />
                </div>
            {:else}
                <div class="art-placeholder">
                    <DiscIcon size={40} weight="thin" color="rgba(255, 255, 255, 0.35)" />
                </div>
            {/if}
        </div>
        <div class="album-info">
            {#if isEditingName && isCustomLocalPlaylist}
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
                <h3 class="album-title font-headline-lg" title={collectionTitle}>
                    <span class="title-text">{collectionTitle}</span>
                </h3>
            {/if}

            <p class="album-artist" title={isFavorites ? `${displayTracks.length} Favorite Songs` : (isCustomLocalPlaylist ? `${displayTracks.length} tracks` : collectionSubtitle)}>
                {#if isFavorites}
                    {displayTracks.length} {displayTracks.length === 1 ? "Favorite Song" : "Favorite Songs"}
                {:else if isCustomLocalPlaylist}
                    {displayTracks.length} {displayTracks.length === 1 ? "track" : "tracks"}
                {:else if isRemote && !isPlaylist && collection.subtitle}
                    <button 
                        class="artist-clickable-link" 
                        onclick={() => exploreStore.openArtist({ id: collection.subtitle!, name: collection.subtitle, provider_id: collection.provider_id })}
                    >
                        {collectionSubtitle}
                    </button>
                    {#if displayTracks.length > 0}
                        • {displayTracks.length} {displayTracks.length === 1 ? "track" : "tracks"}
                    {/if}
                {:else}
                    {collectionSubtitle}
                    {#if displayTracks.length > 0}
                        • {displayTracks.length} {displayTracks.length === 1 ? "track" : "tracks"}
                    {/if}
                {/if}
            </p>

            {#if isCustomLocalPlaylist}
                <div class="playlist-actions-row">
                    <button class="action-btn" onclick={() => isEditingName = true} title="Rename playlist">
                        <PencilIcon size={16} />
                        <span>Edit</span>
                    </button>
                    <button class="action-btn text-danger" onclick={deletePlaylist} title="Delete playlist">
                        <TrashIcon size={16} />
                        <span>Delete</span>
                    </button>
                </div>
            {/if}
        </div>
    </div>

    <div class="track-list" bind:this={scrollContainer}>
        {#if exploreStore.isLoadingCollection && displayTracks.length === 0}
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
                {@const track = displayTracks[i]}
                {@const trackKey = getCanonicalKey(track, collection.subtitle)}
                {@const isTrackLiked = isFavorites ? true : likedKeys.has(trackKey)}
                <!-- svelte-ignore a11y_click_events_have_key_events -->
                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <div
                    class="track-row"
                    class:active={audioStore.currentTrack === track.title ||
                        audioStore.currentTrack === track.file_path}
                    class:menu-open={activeDropdown === i}
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
                                <span class="track-number">{track.track_number || i + 1}</span>
                                <PlayIcon
                                    size={18}
                                    weight="bold"
                                    class="track-play-icon"
                                />
                            {/if}
                        </div>

                        <div class="track-details">
                            <span class="track-name" title={track.title}>{track.title}</span>
                            <span class="track-artist-sub" title={track.artist || "Unknown Artist"}>{track.artist || "Unknown Artist"}</span>
                        </div>
                    </div>

                    <div class="track-right">
                        <button 
                            class="row-like-btn"
                            class:liked={isTrackLiked}
                            onclick={(e) => {
                                e.stopPropagation();
                                libraryStore.toggleLike(track, collection.subtitle);
                            }}
                            title={isTrackLiked ? "Unlike track" : "Like track"}
                        >
                            <HeartIcon size={15} weight={isTrackLiked ? "fill" : "regular"} color={isTrackLiked ? "var(--echo-primary, #B58E62)" : "rgba(255,255,255,0.3)"} />
                        </button>
                        {#if isCustomLocalPlaylist}
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
        gap: 1.25rem;
        margin-bottom: 2rem;
        width: 100%;
        min-width: 0;
    }

    .art-container {
        width: 6.5rem;
        height: 6.5rem;
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
        flex: 1;
        min-width: 0;
        overflow: hidden;
    }

    .album-title {
        font-size: 1.35rem;
        color: var(--echo-text-1);
        margin: 0;
        line-height: 1.25;
        display: flex;
        align-items: center;
        gap: 0.5rem;
        width: 100%;
        min-width: 0;
    }

    .title-text {
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        min-width: 0;
        flex: 1;
        display: block;
    }

    .artist-clickable-link {
        background: transparent;
        border: none;
        padding: 0;
        margin: 0;
        color: var(--echo-text-2);
        font-family: inherit;
        font-size: inherit;
        font-weight: 600;
        cursor: pointer;
        transition: color 0.15s ease, text-decoration 0.15s ease;
    }

    .artist-clickable-link:hover {
        color: #B58E62;
        text-decoration: underline;
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
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
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
        border: 1px solid transparent;
    }

    .track-row:hover {
        background-color: rgba(255, 255, 255, 0.03);
    }

    .track-row.menu-open {
        background-color: rgba(255, 255, 255, 0.06);
        border-color: rgba(181, 142, 98, 0.35);
        box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
        z-index: 25;
    }

    .track-row.menu-open .more-btn {
        color: var(--echo-primary, #B58E62);
        background-color: rgba(255, 255, 255, 0.08);
    }

    .track-left {
        display: flex;
        align-items: center;
        gap: 0.8rem;
        min-width: 0;
        flex: 1;
    }

    .track-status {
        width: 2rem;
        font-size: 0.875rem;
        color: var(--echo-text-2);
        display: flex;
        align-items: center;
        flex-shrink: 0;
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
        min-width: 0;
        flex: 1;
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
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
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

    .row-like-btn {
        background: transparent;
        border: none;
        padding: 0.3rem;
        margin-right: 0.4rem;
        cursor: pointer;
        display: inline-flex;
        align-items: center;
        justify-content: center;
        border-radius: 4px;
        transition: transform 0.15s ease;
    }

    .row-like-btn:hover {
        transform: scale(1.15);
    }


    .art-container.favorites-art-container {
        border: 1px solid rgba(181, 142, 98, 0.22);
        background: #121215;
        box-shadow: 0 10px 18px -3px rgba(0, 0, 0, 0.6), 0 4px 6px -4px rgba(0, 0, 0, 0.4);
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
    }

    :global(.favorites-heart-icon) {
        filter: drop-shadow(0 4px 16px rgba(212, 168, 110, 0.35));
    }

    .mosaic-grid {
        display: grid;
        grid-template-columns: 1fr 1fr;
        grid-template-rows: 1fr 1fr;
        width: 100%;
        height: 100%;
    }

    .mosaic-img {
        width: 100%;
        height: 100%;
        object-fit: cover;
    }


    .playlist-actions-row {
        display: flex;
        align-items: center;
        gap: 0.35rem;
        margin-top: 0.25rem;
    }

    .action-btn {
        background: transparent;
        border: none;
        color: var(--echo-text-2, rgba(255, 255, 255, 0.5));
        cursor: pointer;
        padding: 0.3rem 0.55rem;
        display: inline-flex;
        align-items: center;
        gap: 0.35rem;
        border-radius: 6px;
        font-size: 0.8rem;
        font-weight: 500;
        transition: color 0.15s ease, background 0.15s ease;
    }

    .action-btn:hover {
        color: var(--echo-text-1, #FFFFFF);
        background: rgba(255, 255, 255, 0.08);
    }

    .action-btn.text-danger {
        color: var(--echo-text-2, rgba(255, 255, 255, 0.5));
        background: transparent;
    }

    .action-btn.text-danger:hover {
        color: rgb(239, 68, 68);
        background: rgba(239, 68, 68, 0.12);
    }

</style>
