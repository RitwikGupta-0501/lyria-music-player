<script lang="ts">
    import type { Snippet } from "svelte";
    import EqualizerWave from "$lib/components/common/EqualizerWave.svelte";
    import { Play, Pause } from "phosphor-svelte";
    import { audioStore } from "$lib/stores/audio.svelte";
    import { exploreStore } from "$lib/stores/explore.svelte";
    import { resolveCoverArt } from "$lib/utils/media";
    import { getInitial, isCurrentTrack, getTrackDisplayMetric, sanitizeAlbumName } from "$lib/utils/format";

    interface Props {
        track: {
            id?: string;
            title?: string;
            artist?: string;
            album?: string | null;
            cover_art_url?: string | null;
            duration_ms?: number | null;
            plays?: string | number | null;
            play_count?: string | number | null;
            views?: string | number | null;
            provider_id?: string;
            provider_name?: string;
        } | any;
        index?: number;
        showArtist?: boolean;
        showAlbum?: boolean;
        showCoverArt?: boolean;
        showProviderTag?: boolean;
        variant?: "standard" | "compact" | "chart";
        providerId?: string;
        onclick?: () => void;
        ondblclick?: () => void;
        rightSnippet?: Snippet;
    }

    let {
        track,
        index,
        showArtist = true,
        showAlbum = true,
        showCoverArt = true,
        showProviderTag = false,
        variant = "standard",
        providerId,
        onclick,
        ondblclick,
        rightSnippet,
    }: Props = $props();

    let resolvedProviderId = $derived(providerId || track?.provider_id);
    let isPlaying = $derived(isCurrentTrack(track, audioStore.currentQueueTrack) && audioStore.playbackState === "Playing");
    let isActive = $derived(isCurrentTrack(track, audioStore.currentQueueTrack));

    let coverSrc = $derived(resolveCoverArt(track?.cover_art_url));
    let initial = $derived(getInitial(showArtist ? track?.artist : track?.title));
    let metric = $derived(getTrackDisplayMetric(track));
    let albumName = $derived(showAlbum ? sanitizeAlbumName(track?.album) : null);

    let displayRank = $derived(typeof index === "number" ? index.toString().padStart(2, "0") : null);
    let isTopRank = $derived(variant === "chart" && typeof index === "number" && index <= 3);

    function handleClick() {
        if (onclick) {
            onclick();
        } else if (variant === "chart") {
            exploreStore.playTrack(track, resolvedProviderId);
        }
    }

    function handleDblClick() {
        if (ondblclick) {
            ondblclick();
        } else if (variant !== "chart") {
            exploreStore.playTrack(track, resolvedProviderId);
        }
    }

    function handlePlayOverlayClick(e: MouseEvent) {
        e.stopPropagation();
        exploreStore.playTrack(track, resolvedProviderId);
    }
</script>

<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<div 
    class="track-row variant-{variant}"
    class:active-track={isActive}
    class:is-playing={isPlaying}
    role="button"
    tabindex="0"
    onclick={handleClick}
    ondblclick={handleDblClick}
    onkeydown={(e) => { 
        if (e.key === "Enter") {
            if (onclick) onclick();
            else exploreStore.playTrack(track, resolvedProviderId);
        }
    }}
>
    <!-- 1. Rank / Index Number -->
    {#if displayRank}
        <span class="row-num" class:top-rank={isTopRank}>{displayRank}</span>
    {/if}

    <!-- 2. Track Cover Art Thumbnail -->
    {#if showCoverArt}
        <div class="row-art-wrapper">
            {#if coverSrc}
                <img src={coverSrc} alt={track?.title || "Track art"} class="row-squircle" loading="lazy" />
            {:else}
                <div class="typographic-art-squircle">
                    <span>{initial}</span>
                </div>
            {/if}

            <button 
                type="button"
                class="row-play-btn" 
                class:always-visible={isActive}
                onclick={handlePlayOverlayClick}
                title={isPlaying ? "Pause" : "Play"}
                aria-label={isPlaying ? "Pause track" : "Play track"}
            >
                {#if isPlaying}
                    <Pause size={13} weight="fill" />
                {:else}
                    <span class="icon-optical-wrap">
                        <Play size={13} weight="fill" />
                    </span>
                {/if}
            </button>
        </div>
    {/if}

    <!-- 3. Track Metadata -->
    <div class="row-meta">
        <span class="row-title" title={track?.title}>{track?.title || "Unknown Title"}</span>
        
        {#if (showArtist && track?.artist) || albumName}
            <div class="row-subline">
                {#if showArtist && track?.artist}
                    <span class="row-artist" title={track.artist}>{track.artist}</span>
                {/if}
                {#if showArtist && track?.artist && albumName}
                    <span class="row-dot">•</span>
                {/if}
                {#if albumName}
                    <span class="row-album" title={albumName}>{albumName}</span>
                {/if}
            </div>
        {/if}
    </div>

    <!-- 4. Playing Visualizer (Equalizer Wave) -->
    {#if isPlaying}
        <EqualizerWave />
    {/if}

    <!-- 5. Provider Tag (Search results) -->
    {#if showProviderTag && track?.provider_name}
        <span class="provider-tag">{track.provider_name}</span>
    {/if}

    <!-- 6. Custom Slot or Track Metric (Duration / Plays / Views) -->
    {#if rightSnippet}
        {@render rightSnippet()}
    {:else if metric}
        <span class="row-metric">{metric}</span>
    {/if}
</div>

<style>
    .track-row {
        display: flex;
        align-items: center;
        gap: 0.9rem;
        padding: 0.55rem 0.8rem;
        border-radius: 12px;
        background: transparent;
        border: 1px solid transparent;
        cursor: pointer;
        user-select: none;
        transition: background-color 0.16s ease, border-color 0.16s ease, transform 0.12s ease;
        position: relative;
        box-sizing: border-box;
        width: 100%;
        text-align: left;
    }

    .track-row:hover {
        background: rgba(255, 255, 255, 0.05);
        border-color: rgba(255, 255, 255, 0.08);
    }

    .track-row:active {
        transform: scale(0.995);
    }

    .track-row.active-track {
        background: rgba(181, 142, 98, 0.12);
        border-color: rgba(181, 142, 98, 0.3);
    }

    .track-row.active-track .row-title {
        color: var(--echo-primary, #B58E62);
    }

    /* Compact Variant */
    .track-row.variant-compact {
        padding: 0.4rem 0.6rem;
        gap: 0.7rem;
        border-radius: 10px;
    }

    .track-row.variant-compact .row-art-wrapper {
        width: 36px;
        height: 36px;
    }

    .track-row.variant-compact .row-title {
        font-size: 0.82rem;
    }

    /* Rank Number */
    .row-num {
        font-family: var(--echo-font-mono, monospace);
        font-size: 0.82rem;
        font-weight: 600;
        color: rgba(255, 255, 255, 0.3);
        width: 22px;
        text-align: center;
        flex-shrink: 0;
    }

    .row-num.top-rank {
        color: var(--echo-primary, #B58E62);
        font-weight: 700;
    }

    /* Thumbnail Art */
    .row-art-wrapper {
        position: relative;
        width: 42px;
        height: 42px;
        border-radius: 8px;
        overflow: hidden;
        flex-shrink: 0;
        background: #18181c;
        border: 1px solid rgba(255, 255, 255, 0.08);
        box-shadow: 0 4px 10px rgba(0, 0, 0, 0.3);
    }

    .row-squircle {
        width: 100%;
        height: 100%;
        object-fit: cover;
        display: block;
    }

    .typographic-art-squircle {
        width: 100%;
        height: 100%;
        display: flex;
        align-items: center;
        justify-content: center;
        background: linear-gradient(135deg, rgba(181, 142, 98, 0.2), rgba(181, 142, 98, 0.05));
        font-family: var(--echo-font-heading, "Playfair Display", serif);
        font-weight: 700;
        font-size: 1.1rem;
        color: var(--echo-primary, #B58E62);
    }

    /* Mini Play Overlay */
    .row-play-btn {
        position: absolute;
        inset: 0;
        background: rgba(0, 0, 0, 0.55);
        backdrop-filter: blur(2px);
        border: none;
        color: #fff;
        display: flex;
        align-items: center;
        justify-content: center;
        opacity: 0;
        cursor: pointer;
        transition: opacity 0.15s ease, background 0.15s ease;
        padding: 0;
    }

    .icon-optical-wrap {
        display: flex;
        align-items: center;
        justify-content: center;
        transform: translateX(0.5px);
    }

    .track-row:hover .row-play-btn,
    .row-play-btn.always-visible {
        opacity: 1;
    }

    .row-play-btn:hover {
        background: rgba(181, 142, 98, 0.85);
        color: #0E0E10;
    }

    /* Metadata */
    .row-meta {
        display: flex;
        flex-direction: column;
        justify-content: center;
        gap: 0.18rem;
        min-width: 0;
        flex: 1 1 0;
    }

    .row-title {
        font-size: 0.86rem;
        font-weight: 600;
        color: #ffffff;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        line-height: 1.25;
        transition: color 0.15s ease;
    }

    .row-subline {
        display: flex;
        align-items: center;
        gap: 0.35rem;
        font-size: 0.74rem;
        color: rgba(255, 255, 255, 0.45);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .row-artist {
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .row-dot {
        opacity: 0.5;
        font-size: 0.65rem;
        flex-shrink: 0;
    }

    .row-album {
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        opacity: 0.85;
    }

    /* Provider Tag */
    .provider-tag {
        font-size: 0.65rem;
        font-weight: 600;
        text-transform: uppercase;
        letter-spacing: 0.04em;
        padding: 0.18rem 0.45rem;
        border-radius: 4px;
        background: rgba(255, 255, 255, 0.06);
        color: rgba(255, 255, 255, 0.5);
        border: 1px solid rgba(255, 255, 255, 0.08);
        flex-shrink: 0;
    }

    /* Metric / Duration */
    .row-metric {
        font-size: 0.78rem;
        color: rgba(255, 255, 255, 0.38);
        font-family: var(--echo-font-mono, monospace);
        flex-shrink: 0;
        margin-left: 0.3rem;
    }
</style>
