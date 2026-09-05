<script lang="ts">
    import { libraryStore, getCanonicalKey } from "$lib/stores/library.svelte";
    import { settingsStore } from "$lib/stores/settings.svelte";
    import { resolveCoverArt } from "$lib/utils/media";
    import { formatDuration } from "$lib/utils/format";
    import CardPlayButton from "$lib/components/common/CardPlayButton.svelte";
    import { Play, Heart, Video } from "phosphor-svelte";

    let {
        video,
        selected = false,
        onclick,
        onplay
    } = $props<{
        video: any;
        selected?: boolean;
        onclick: () => void;
        onplay?: () => void;
    }>();

    let title = $derived(video.title || "Untitled Video");
    let artist = $derived(video.artist || "Official Music Video");
    let artUrl = $derived(resolveCoverArt(video.cover_art_url || video.cover_art_path));
    
    let isLiked = $derived(
        video.liked !== undefined
            ? video.liked
            : libraryStore.isLikedSong(video.canonical_key || getCanonicalKey({ title: video.title, artist: video.artist }))
    );

    function handlePlayClick(e: MouseEvent) {
        e.stopPropagation();
        if (onplay) {
            onplay();
        } else {
            onclick();
        }
    }

    function handleLikeClick(e: MouseEvent) {
        e.stopPropagation();
        libraryStore.toggleLike({
            canonical_key: video.canonical_key || getCanonicalKey({ title: video.title, artist: video.artist }),
            title: video.title,
            artist: video.artist || undefined,
            album: video.album || undefined,
            cover_art_url: video.cover_art_url || undefined,
            provider_id: video.last_provider_id || video.provider_id || "youtube-wasm",
            id: video.last_source_id || video.id || undefined,
            duration_ms: video.duration_ms || undefined,
        });
    }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div 
    class="video-card group {selected ? 'selected' : ''}" 
    {onclick}
>
    <div class="thumbnail-container">
        {#if artUrl}
            <img 
                src={artUrl} 
                alt={title} 
                class="thumbnail-img" 
                loading="lazy" 
            />
        {:else}
            <div class="thumbnail-placeholder">
                <Video size={40} weight="thin" color="rgba(255, 255, 255, 0.35)" />
            </div>
        {/if}

        <div class="play-overlay">
            <CardPlayButton 
                onclick={handlePlayClick}
                title="Play video audio"
            />
        </div>

        <button 
            type="button"
            class="liquid-like-btn" 
            class:is-glass={settingsStore.glassyPlayerBar}
            class:liked={isLiked}
            onclick={handleLikeClick}
            title={isLiked ? "Liked" : "Like video"}
            aria-label={isLiked ? "Unlike video" : "Like video"}
        >
            <Heart size={16} weight={isLiked ? "fill" : "bold"} color={isLiked ? "#ffd285" : "#FFFFFF"} />
        </button>

        {#if video.duration_ms}
            <span class="duration-chip">{formatDuration(video.duration_ms)}</span>
        {/if}

        {#if video.provider_id && video.provider_id !== "local"}
            <span class="provider-badge">YouTube</span>
        {/if}
    </div>

    <div class="card-info">
        <span class="card-title" title={title}>{title}</span>
        <span class="card-subtitle" title={artist}>{artist}</span>
    </div>
</div>

<style>
    .video-card {
        cursor: pointer;
        display: flex;
        flex-direction: column;
        align-items: flex-start;
        justify-content: flex-start;
        width: 100%;
        min-width: 0;
        gap: 0.6rem;
        text-align: left;
        margin: 0;
        padding: 0;
    }

    .video-card.selected .thumbnail-container {
        border-color: var(--echo-primary);
        box-shadow:
            0 0 0 2px var(--echo-primary),
            0 10px 15px -3px rgba(0, 0, 0, 0.5);
    }

    .thumbnail-container {
        width: 100%;
        aspect-ratio: 16 / 9;
        border-radius: 12px;
        background-color: #141416;
        border: 1px solid rgba(255, 255, 255, 0.08);
        overflow: hidden;
        position: relative;
        contain: layout paint;
        isolation: isolate;
        transform: translateZ(0);
        backface-visibility: hidden;
        transition: border-color 0.2s ease, box-shadow 0.2s ease;
    }

    .video-card:hover .thumbnail-container {
        border-color: rgba(181, 142, 98, 0.35);
        box-shadow: 0 10px 24px -6px rgba(0, 0, 0, 0.6);
    }

    .thumbnail-img {
        width: 100%;
        height: 100%;
        object-fit: cover;
        display: block;
        transform: translateZ(0);
        backface-visibility: hidden;
        will-change: transform;
        transition: transform 0.55s cubic-bezier(0.05, 0.75, 0.15, 1);
    }

    .video-card:hover .thumbnail-img {
        transform: scale(1.05);
    }

    .thumbnail-placeholder {
        width: 100%;
        height: 100%;
        display: flex;
        align-items: center;
        justify-content: center;
        background: radial-gradient(circle at center, #18181b 0%, #09090b 100%);
    }

    .play-overlay {
        position: absolute;
        inset: 0;
        display: flex;
        align-items: center;
        justify-content: center;
        background: rgba(0, 0, 0, 0.35);
        backdrop-filter: blur(2px);
        opacity: 0;
        pointer-events: none;
        transform: translateZ(0);
        backface-visibility: hidden;
        will-change: opacity;
        transition: opacity 0.2s cubic-bezier(0.16, 1, 0.3, 1);
    }

    .video-card:hover .play-overlay {
        opacity: 1;
    }

    .duration-chip {
        position: absolute;
        bottom: 8px;
        right: 8px;
        background: rgba(0, 0, 0, 0.75);
        backdrop-filter: blur(4px);
        color: rgba(255, 255, 255, 0.9);
        font-family: var(--echo-font-mono, monospace);
        font-size: 0.72rem;
        font-weight: 600;
        padding: 0.2rem 0.45rem;
        border-radius: 4px;
        border: 1px solid rgba(255, 255, 255, 0.1);
        pointer-events: none;
        z-index: 2;
    }

    .provider-badge {
        position: absolute;
        top: 8px;
        left: 8px;
        background: rgba(0, 0, 0, 0.6);
        backdrop-filter: blur(4px);
        color: rgba(255, 255, 255, 0.75);
        font-size: 0.62rem;
        font-weight: 600;
        letter-spacing: 0.05em;
        text-transform: uppercase;
        padding: 0.15rem 0.4rem;
        border-radius: 4px;
        border: 1px solid rgba(255, 255, 255, 0.08);
        pointer-events: none;
        z-index: 2;
    }

    .video-card:hover .liquid-like-btn {
        opacity: 1;
        transform: scale(1);
        pointer-events: auto;
    }

    .card-info {
        display: flex;
        flex-direction: column;
        gap: 0.18rem;
        width: 100%;
        overflow: hidden;
    }

    .card-title {
        font-size: 0.88rem;
        font-weight: 600;
        color: var(--echo-text-1);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        line-height: 1.25;
        width: 100%;
    }

    .video-card:hover .card-title {
        color: var(--echo-primary);
    }

    .card-subtitle {
        font-size: 0.78rem;
        color: var(--echo-text-2);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        line-height: 1.2;
        width: 100%;
    }
</style>
