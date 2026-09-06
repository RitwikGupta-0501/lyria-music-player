<script lang="ts">
    import { libraryStore, getCanonicalKey } from "$lib/stores/library.svelte";
    import { homeStore, type FederatedTrack } from "$lib/stores/home.svelte";
    import { settingsStore } from "$lib/stores/settings.svelte";
    import CardPlayButton from "$lib/components/common/CardPlayButton.svelte";
    import { resolveCoverArt } from "$lib/utils/media";
    import { Play, Heart, MusicNotes } from "phosphor-svelte";

    let {
        track,
        selected = false,
        onclick,
        onplay
    } = $props<{
        track: FederatedTrack | any;
        selected?: boolean;
        onclick: () => void;
        onplay?: () => void;
    }>();

    let title = $derived(track.title || "Unknown Track");
    let artist = $derived(track.artist || "Unknown Artist");
    let artUrl = $derived(resolveCoverArt(track.cover_art_url || track.cover_art_path));
    let isLiked = $derived(
        track.liked !== undefined 
            ? track.liked 
            : libraryStore.isLikedSong(track.canonical_key || getCanonicalKey(track))
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
        if (typeof homeStore.toggleLike === "function" && track.canonical_key) {
            homeStore.toggleLike(track);
        } else {
            libraryStore.toggleLike({
                canonical_key: track.canonical_key || getCanonicalKey(track),
                title: track.title,
                artist: track.artist || undefined,
                album: track.album || undefined,
                cover_art_url: track.cover_art_url || undefined,
                provider_id: track.last_provider_id || track.provider_id || "youtube-wasm",
                id: track.last_source_id || track.id || undefined,
                duration_ms: track.duration_ms || undefined,
            });
        }
    }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div 
    class="track-card group {selected ? 'selected' : ''}" 
    {onclick}
>
    <div class="art-container">
        {#if artUrl}
            <img 
                src={artUrl} 
                alt={title} 
                class="art-img" 
                loading="lazy" 
            />
        {:else}
            <div class="art-placeholder">
                <MusicNotes size={56} weight="thin" color="rgba(255, 255, 255, 0.35)" />
            </div>
        {/if}

        <div class="play-overlay">
            <CardPlayButton 
                onclick={handlePlayClick}
                title="Play track"
            />
        </div>

        <button 
            type="button"
            class="liquid-like-btn" 
            class:is-glass={settingsStore.glassyPlayerBar}
            class:liked={isLiked}
            onclick={handleLikeClick}
            title={isLiked ? "Liked" : "Like track"}
            aria-label={isLiked ? "Unlike track" : "Like track"}
        >
            <Heart size={16} weight={isLiked ? "fill" : "bold"} color={isLiked ? "#ffd285" : "#FFFFFF"} />
        </button>

        {#if track.last_provider_id && track.last_provider_id !== "local"}
            <span class="provider-badge">YouTube</span>
        {/if}
    </div>

    <div class="card-info">
        <span class="card-title" title={title}>{title}</span>
        <span class="card-artist" title={artist}>{artist}</span>
    </div>
</div>

<style>
    .track-card {
        cursor: pointer;
        display: flex;
        flex-direction: column;
        align-items: flex-start;
        justify-content: flex-start;
        align-self: flex-start;
        width: 176px;
        min-width: 176px;
        flex: 0 0 176px;
        gap: 0.6rem;
        text-align: left;
        margin: 0;
        padding: 0;
    }

    .track-card.selected .art-container {
        border-color: var(--echo-primary);
        box-shadow: 0 0 0 2px var(--echo-primary), 0 10px 15px -3px rgba(0, 0, 0, 0.5);
    }

    .art-container {
        width: 176px;
        height: 176px;
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

    .track-card:hover .art-container {
        border-color: rgba(181, 142, 98, 0.35);
        box-shadow: 0 10px 24px -6px rgba(0, 0, 0, 0.6);
    }

    .art-img {
        width: 100%;
        height: 100%;
        object-fit: cover;
        display: block;
        transform: translateZ(0);
        backface-visibility: hidden;
        will-change: transform;
        transition: transform 0.55s cubic-bezier(0.05, 0.75, 0.15, 1);
    }

    .track-card:hover .art-img {
        transform: scale(1.05);
    }

    .art-placeholder {
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

    .track-card:hover .play-overlay {
        opacity: 1;
        pointer-events: auto;
    }

    .liquid-like-btn {
        position: absolute;
        top: 8px;
        right: 8px;
        width: 32px !important;
        height: 32px !important;
        min-width: 32px !important;
        max-width: 32px !important;
        min-height: 32px !important;
        max-height: 32px !important;
        border-radius: 50% !important;
        padding: 0 !important;
        margin: 0 !important;
        background: rgba(18, 20, 26, 0.85);
        border: 1px solid rgba(255, 255, 255, 0.12);
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.45);
        color: #ffffff;
        display: flex !important;
        align-items: center !important;
        justify-content: center !important;
        cursor: pointer;
        opacity: 0;
        transform: scale(0.85) translateZ(0);
        backface-visibility: hidden;
        pointer-events: none;
        transition: opacity 0.2s cubic-bezier(0.16, 1, 0.3, 1), transform 0.2s cubic-bezier(0.16, 1, 0.3, 1), background 0.2s ease, border-color 0.2s ease, box-shadow 0.2s ease;
        z-index: 20;
    }

    .liquid-like-btn.is-glass {
        background: linear-gradient(180deg, rgba(255, 255, 255, 0.08) 0%, rgba(255, 255, 255, 0.02) 100%);
        border: 1.5px solid rgba(255, 255, 255, 0.14);
        box-shadow: 
            inset 0 1.5px 0 0 rgba(255, 255, 255, 0.30),
            0 4px 12px rgba(0, 0, 0, 0.35);
    }

    .track-card:hover .liquid-like-btn,
    .liquid-like-btn.liked {
        opacity: 1;
        transform: scale(1);
        pointer-events: auto;
    }

    .liquid-like-btn:hover {
        transform: scale(1.12) !important;
        background: rgba(30, 32, 40, 0.95) !important;
        border-color: rgba(226, 169, 115, 0.35) !important;
        box-shadow: 0 6px 16px rgba(0, 0, 0, 0.55) !important;
    }

    .liquid-like-btn.is-glass:hover {
        background: rgba(255, 255, 255, 0.08) !important;
        border-color: rgba(226, 169, 115, 0.35) !important;
        box-shadow: 
            inset 0 1px 1px rgba(255, 255, 255, 0.25),
            0 6px 16px rgba(0, 0, 0, 0.45) !important;
    }

    .liquid-like-btn:active {
        transform: scale(0.92) !important;
    }

    .liquid-like-btn.liked {
        color: #ffd285;
        background: rgba(45, 35, 25, 0.9);
        border-color: rgba(224, 184, 143, 0.35);
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
    }

    .liquid-like-btn.is-glass.liked {
        background: linear-gradient(180deg, rgba(200, 157, 110, 0.22) 0%, rgba(150, 107, 61, 0.15) 100%);
        border-color: rgba(224, 184, 143, 0.35);
        box-shadow: 
            inset 0 1px 1px rgba(255, 255, 255, 0.25),
            0 4px 12px rgba(0, 0, 0, 0.35);
    }

    .liquid-like-btn :global(svg) {
        display: block;
        flex-shrink: 0;
        width: 16px;
        height: 16px;
        pointer-events: none;
    }

    .provider-badge {
        position: absolute;
        bottom: 8px;
        left: 8px;
        font-size: 0.68rem;
        font-weight: 600;
        letter-spacing: 0.05em;
        text-transform: uppercase;
        background: rgba(18, 18, 20, 0.75);
        backdrop-filter: blur(8px);
        border: 1px solid rgba(255, 255, 255, 0.1);
        color: #D4A86E;
        padding: 0.2rem 0.45rem;
        border-radius: 6px;
        z-index: 10;
    }

    .card-info {
        display: flex;
        flex-direction: column;
        align-items: flex-start;
        gap: 0.2rem;
        width: 100%;
        min-width: 0;
        text-align: left;
    }

    .card-title {
        font-family: var(--echo-font-body, system-ui, sans-serif);
        font-size: 0.88rem;
        font-weight: 600;
        color: #fff;
        display: block;
        width: 100%;
        text-align: left;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .card-artist {
        font-family: var(--echo-font-body, system-ui, sans-serif);
        font-size: 0.78rem;
        color: rgba(255, 255, 255, 0.55);
        display: block;
        width: 100%;
        text-align: left;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }
</style>
