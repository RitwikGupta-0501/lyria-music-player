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
        Shuffle as ShuffleIcon,
        TrashIcon,
        PencilIcon,
        Heart as HeartIcon,
        DotsSixVertical as GripIcon,
    } from "phosphor-svelte";
    import { createVirtualizer } from "@tanstack/svelte-virtual";
    import PillButton from "$lib/components/common/PillButton.svelte";
    import EqualizerWave from "$lib/components/common/EqualizerWave.svelte";
    import { exploreStore, type DrawerCollection } from "$lib/stores/explore.svelte";
    import { convertFileSrc } from "@tauri-apps/api/core";
    import { isCurrentTrack } from "$lib/utils/format";

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

    let displayTracks = $derived(
        isFavorites 
            ? favoriteTracks 
            : isRemote 
                ? (collection.tracks && collection.tracks.length > 0 ? collection.tracks : tracks) 
                : tracks
    );

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

    let draggedIndex = $state(-1);
    let dragoverIndex = $state(-1);
    let dropPosition = $state<"top" | "bottom" | null>(null);
    let justReorderedIndex = $state(-1);

    let scrollContainer = $state<HTMLElement | null>(null);
    let virtStore = $derived.by(() => {
        const container = scrollContainer;
        return createVirtualizer({
            count: displayTracks.length,
            getScrollElement: () => container,
            estimateSize: () => 44,
            overscan: 10,
            initialRect: { width: 400, height: 800 },
        });
    });

    function handleDragStart(e: DragEvent, index: number) {
        if (!isCustomLocalPlaylist) return;
        draggedIndex = index;
        if (e.dataTransfer) {
            e.dataTransfer.effectAllowed = "move";
            e.dataTransfer.setData("text/plain", index.toString());
        }
    }

    function handleDragOver(e: DragEvent, index: number) {
        if (!isCustomLocalPlaylist || draggedIndex === -1) return;
        e.preventDefault();
        if (draggedIndex === index) {
            dragoverIndex = -1;
            dropPosition = null;
            return;
        }

        const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
        const relY = e.clientY - rect.top;
        dropPosition = relY < rect.height / 2 ? "top" : "bottom";
        dragoverIndex = index;

        if (e.dataTransfer) {
            e.dataTransfer.dropEffect = "move";
        }
    }

    async function handleDrop(e: DragEvent, targetIndex: number) {
        if (!isCustomLocalPlaylist) return;
        e.preventDefault();
        const fromIdx = draggedIndex;
        const pos = dropPosition;

        dragoverIndex = -1;
        dropPosition = null;
        draggedIndex = -1;

        if (fromIdx !== -1 && fromIdx !== targetIndex) {
            let toIdx = targetIndex;
            if (pos === "bottom" && fromIdx < targetIndex) {
                toIdx = targetIndex;
            } else if (pos === "top" && fromIdx > targetIndex) {
                toIdx = targetIndex;
            } else if (pos === "bottom" && fromIdx > targetIndex) {
                toIdx = targetIndex + 1;
            } else if (pos === "top" && fromIdx < targetIndex) {
                toIdx = targetIndex - 1;
            }

            toIdx = Math.max(0, Math.min(toIdx, tracks.length - 1));

            if (fromIdx !== toIdx) {
                const newTracks = [...tracks];
                const [moved] = newTracks.splice(fromIdx, 1);
                newTracks.splice(toIdx, 0, moved);
                tracks = newTracks;

                justReorderedIndex = toIdx;
                setTimeout(() => {
                    justReorderedIndex = -1;
                }, 400);

                try {
                    await libraryStore.reorderPlaylistTrack(Number(collection.id), fromIdx + 1, toIdx + 1);
                } catch (err) {
                    console.error("Failed to reorder playlist track:", err);
                    toastStore.show("Failed to reorder track", "error");
                    await loadData();
                }
            }
        }
    }

    async function loadData() {
        if (collection.id === "favorites" || collection.source === "remote") {
            return;
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

            const queueTracks = displayTracks.map((t: any) => ({
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

        const queueTracks = displayTracks.map((t: any) => ({
            id: t.id,
            title: t.title,
            artist: t.artist,
            album: collection.title,
            file_path: t.file_path,
            track_number: t.track_number,
        }));

        await audioStore.setQueue(queueTracks, index);
    }

    let isCollectionPlaying = $derived.by(() => {
        if (audioStore.playbackState !== "Playing" || !audioStore.currentQueueTrack) return false;
        const cur = audioStore.currentQueueTrack;
        return displayTracks.some((t: any) => {
            if (cur.source.type === "Remote") {
                const remoteUri = `remote://${cur.source.provider_id}/${cur.source.remote_track_id}`;
                return (t.id !== undefined && String(t.id) === String(cur.source.remote_track_id || cur.source.track_id)) ||
                       (t.file_path && t.file_path === remoteUri) ||
                       (t.title === cur.title && t.artist === cur.artist);
            } else {
                return (t.file_path && t.file_path === cur.source.file_path) ||
                       (t.id !== undefined && t.id === cur.source.track_id);
            }
        });
    });

    async function togglePlayCollection() {
        if (displayTracks.length === 0) return;
        if (isCollectionPlaying) {
            await audioStore.pause();
        } else if (
            audioStore.playbackState === "Paused" &&
            audioStore.currentQueueTrack &&
            displayTracks.some((t: any) => {
                const cur = audioStore.currentQueueTrack!;
                if (cur.source.type === "Remote") {
                    return (t.id !== undefined && String(t.id) === String(cur.source.remote_track_id || cur.source.track_id)) ||
                           (t.title === cur.title && t.artist === cur.artist);
                } else {
                    return (t.file_path && t.file_path === cur.source.file_path) ||
                           (t.id !== undefined && t.id === cur.source.track_id);
                }
            })
        ) {
            await audioStore.play();
        } else {
            if (isRemote) {
                const pId = collection.provider_id || "youtube-wasm";
                const queueTracks = displayTracks.map((t: any) => ({
                    id: t.id,
                    title: t.title,
                    artist: t.artist,
                    album: collection.title,
                    remote_track_id: t.id,
                    provider_id: t.provider_id || pId,
                    cover_art_url: t.cover_art_url || collection.cover_art_url,
                    duration_ms: t.duration_ms,
                }));
                await audioStore.setQueue(queueTracks, 0);
            } else {
                const queueTracks = displayTracks.map((t: any) => ({
                    id: t.id,
                    title: t.title,
                    artist: t.artist,
                    album: collection.title,
                    file_path: t.file_path,
                    track_number: t.track_number,
                }));
                await audioStore.setQueue(queueTracks, 0);
            }
        }
    }

    async function shuffleCollection() {
        if (displayTracks.length === 0) return;
        const shuffled = [...displayTracks].sort(() => Math.random() - 0.5);
        if (isRemote) {
            const pId = collection.provider_id || "youtube-wasm";
            const queueTracks = shuffled.map((t: any) => ({
                id: t.id,
                title: t.title,
                artist: t.artist,
                album: collection.title,
                remote_track_id: t.id,
                provider_id: t.provider_id || pId,
                cover_art_url: t.cover_art_url || collection.cover_art_url,
                duration_ms: t.duration_ms,
            }));
            await audioStore.setQueue(queueTracks, 0);
        } else {
            const queueTracks = shuffled.map((t: any) => ({
                id: t.id,
                title: t.title,
                artist: t.artist,
                album: collection.title,
                file_path: t.file_path,
                track_number: t.track_number,
            }));
            await audioStore.setQueue(queueTracks, 0);
        }
    }
</script>

<svelte:window onclick={() => activeDropdown = null} />

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

            <div class="playlist-actions-row">
                <PillButton 
                    variant="primary"
                    size="sm"
                    icon={isCollectionPlaying ? PauseIcon : PlayIcon}
                    label={isCollectionPlaying ? "Pause" : "Play"}
                    disabled={displayTracks.length === 0}
                    onclick={togglePlayCollection}
                    title={isCollectionPlaying ? "Pause collection" : "Play collection"}
                />

                {#if displayTracks.length > 1}
                    <PillButton 
                        variant="secondary"
                        size="sm"
                        icon={ShuffleIcon}
                        label="Shuffle"
                        onclick={shuffleCollection}
                        title="Shuffle collection"
                    />
                {/if}

                {#if isCustomLocalPlaylist}
                    <PillButton 
                        variant="secondary"
                        size="sm"
                        icon={PencilIcon}
                        label="Edit"
                        onclick={() => isEditingName = true}
                        title="Rename playlist"
                    />
                    <PillButton 
                        variant="danger"
                        size="sm"
                        icon={TrashIcon}
                        label="Delete"
                        onclick={deletePlaylist}
                        title="Delete playlist"
                    />
                {:else if isRemote && !isFavorites}
                    {@const isSaved = isPlaylist ? libraryStore.isPlaylistSaved(String(collection.id)) : libraryStore.isAlbumSaved(String(collection.id), collectionTitle, collectionSubtitle)}
                    <PillButton 
                        variant="secondary"
                        size="sm"
                        icon={HeartIcon}
                        label={isSaved ? "Saved" : "Save"}
                        onclick={() => {
                            if (isPlaylist) {
                                libraryStore.toggleSavePlaylist({
                                    id: String(collection.id),
                                    title: collectionTitle,
                                    author: collectionSubtitle || null,
                                    cover_art_url: collection.cover_art_url || null,
                                    provider_id: collection.provider_id || "youtube-wasm",
                                });
                            } else {
                                libraryStore.toggleSaveAlbum({
                                    id: String(collection.id),
                                    title: collectionTitle,
                                    artist: collectionSubtitle || null,
                                    cover_art_url: collection.cover_art_url || null,
                                    provider_id: collection.provider_id || "youtube-wasm",
                                });
                            }
                        }} 
                        title={isSaved ? (isPlaylist ? "Unsave playlist" : "Unsave album") : (isPlaylist ? "Save playlist" : "Save album")}
                    />
                {/if}
            </div>
        </div>
    </div>

    <div class="track-list" class:is-reordering={draggedIndex !== -1} bind:this={scrollContainer}>
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
                {@const isCurrent = isCurrentTrack(track, audioStore.currentQueueTrack)}
                <!-- svelte-ignore a11y_click_events_have_key_events -->
                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <div
                    class="track-row"
                    class:active={isCurrent}
                    class:menu-open={activeDropdown === i}
                    class:is-dragging={draggedIndex === i}
                    class:drag-over-top={dragoverIndex === i && dropPosition === "top" && draggedIndex !== i}
                    class:drag-over-bottom={dragoverIndex === i && dropPosition === "bottom" && draggedIndex !== i}
                    class:just-reordered={justReorderedIndex === i}
                    draggable={isCustomLocalPlaylist}
                    style="position: absolute; top: 0; left: 0; width: 100%; transform: translateY({row.start}px);"
                    ondragstart={(e) => handleDragStart(e, i)}
                    ondragover={(e) => handleDragOver(e, i)}
                    ondrop={(e) => handleDrop(e, i)}
                    ondragend={() => {
                        dragoverIndex = -1;
                        dropPosition = null;
                        draggedIndex = -1;
                    }}
                    onclick={() => playTrack(i)}
                >
                    <div class="track-left">
                        <div class="track-status">
                            {#if isCurrent}
                                {#if audioStore.playbackState === "Playing"}
                                    <EqualizerWave />
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
                        {#if isCustomLocalPlaylist}
                            <span class="drag-handle" title="Drag to reorder">
                                <GripIcon size={15} weight="bold" />
                            </span>
                        {/if}
                        <button 
                            class="row-like-btn"
                            class:liked={isTrackLiked}
                            onclick={(e) => {
                                e.stopPropagation();
                                libraryStore.toggleLike(track, collection.subtitle);
                            }}
                            title={isTrackLiked ? "Unlike track" : "Like track"}
                        >
                            <HeartIcon size={15} weight={isTrackLiked ? "fill" : "regular"} color={isTrackLiked ? "#ffd285" : "rgba(255,255,255,0.3)"} />
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
        padding: 1.5rem 1.5rem var(--drawer-scroll-padding, 8rem) 1.5rem;
        scroll-padding-bottom: var(--drawer-scroll-padding, 8rem);
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

    .track-list.is-reordering .track-row * {
        pointer-events: none;
    }

    .track-row {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 0.375rem 0.75rem;
        border-radius: 0.5rem;
        cursor: pointer;
        transition: all 0.15s ease;
        position: relative;
        border: 1px solid transparent;
    }

    .track-row.is-dragging {
        opacity: 0.25;
        background: rgba(255, 255, 255, 0.02);
    }

    .track-row.drag-over-top::before {
        content: '';
        position: absolute;
        top: -1px;
        left: 0.75rem;
        right: 0.75rem;
        height: 2px;
        background: var(--echo-primary, #B58E62);
        box-shadow: 0 0 10px var(--echo-primary, #B58E62);
        border-radius: 2px;
        pointer-events: none;
        z-index: 20;
    }

    .track-row.drag-over-top::after {
        content: '';
        position: absolute;
        top: -3px;
        left: 0.65rem;
        width: 6px;
        height: 6px;
        border-radius: 50%;
        background: var(--echo-primary, #B58E62);
        box-shadow: 0 0 8px var(--echo-primary, #B58E62);
        pointer-events: none;
        z-index: 21;
    }

    .track-row.drag-over-bottom::before {
        content: '';
        position: absolute;
        bottom: -1px;
        left: 0.75rem;
        right: 0.75rem;
        height: 2px;
        background: var(--echo-primary, #B58E62);
        box-shadow: 0 0 10px var(--echo-primary, #B58E62);
        border-radius: 2px;
        pointer-events: none;
        z-index: 20;
    }

    .track-row.drag-over-bottom::after {
        content: '';
        position: absolute;
        bottom: -3px;
        left: 0.65rem;
        width: 6px;
        height: 6px;
        border-radius: 50%;
        background: var(--echo-primary, #B58E62);
        box-shadow: 0 0 8px var(--echo-primary, #B58E62);
        pointer-events: none;
        z-index: 21;
    }

    .drag-handle {
        display: flex;
        align-items: center;
        justify-content: center;
        color: var(--echo-text-3);
        opacity: 0;
        padding: 4px;
        margin-right: 4px;
        border-radius: 4px;
        cursor: grab;
        transition: opacity 0.12s ease, color 0.12s ease;
    }

    .track-row:hover .drag-handle {
        opacity: 0.6;
    }

    .drag-handle:hover {
        opacity: 1 !important;
        color: var(--echo-text-1);
        background: rgba(255, 255, 255, 0.08);
    }

    .drag-handle:active {
        cursor: grabbing;
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
        gap: 0.5rem;
        min-width: 0;
        flex: 1;
    }

    .track-status {
        width: 1.35rem;
        font-size: 0.8rem;
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
        flex-wrap: wrap;
        gap: 0.5rem;
        margin-top: 0.4rem;
    }


</style>
