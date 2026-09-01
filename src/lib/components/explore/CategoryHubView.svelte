<script lang="ts">
    import { exploreStore } from "$lib/stores/explore.svelte";
    import { audioStore } from "$lib/stores/audio.svelte";
    import { formatDuration, getInitial, isCurrentTrack } from "$lib/utils/format";
    import { ArrowLeft, Play, Pause } from "phosphor-svelte";
</script>

{#if exploreStore.activeCategory}
    <div class="category-hub-canvas">
        <header class="hub-header">
            <button class="hub-back-btn" onclick={() => exploreStore.closeCategory()}>
                <ArrowLeft size={16} weight="bold" />
                <span>Back to Explore</span>
            </button>
            <div class="hub-title-row">
                {#if exploreStore.activeCategory.color_hex}
                    <div class="hub-color-badge" style="background: {exploreStore.activeCategory.color_hex}"></div>
                {/if}
                <h1>{exploreStore.activeCategory.title}</h1>
            </div>
            <p class="hub-subtitle">Ranked charts & curated releases</p>
        </header>

        {#if exploreStore.isCategoryLoading}
            <div class="hub-tracks-grid">
                {#each Array(8) as _}
                    <div class="ledger-row skeleton">
                        <div class="skeleton-num skeleton-box"></div>
                        <div class="track-squircle skeleton-box"></div>
                        <div class="track-meta">
                            <div class="skeleton-line title"></div>
                            <div class="skeleton-line artist"></div>
                        </div>
                    </div>
                {/each}
            </div>
        {:else if exploreStore.categoryTracks.length > 0}
            <div class="hub-tracks-grid">
                {#each exploreStore.categoryTracks as track, index}
                    <div 
                        class="ledger-row"
                        class:active-track={isCurrentTrack(track, audioStore.currentQueueTrack)}
                        role="button"
                        tabindex="0"
                        ondblclick={() => exploreStore.playTrack(track, track.provider_id)}
                        onkeydown={(e) => { if (e.key === 'Enter') exploreStore.playTrack(track, track.provider_id); }}
                    >
                        <span class="ledger-num">{(index + 1).toString().padStart(2, '0')}</span>
                        
                        <div class="track-art-wrapper">
                            {#if track.cover_art_url}
                                <img src={track.cover_art_url} alt={track.title} class="track-squircle" />
                            {:else}
                                <div class="typographic-art-squircle">
                                    <span>{getInitial(track.artist)}</span>
                                </div>
                            {/if}
                            <button class="play-overlay-btn" onclick={() => exploreStore.playTrack(track, track.provider_id)}>
                                {#if isCurrentTrack(track, audioStore.currentQueueTrack) && audioStore.playbackState === "Playing"}
                                    <Pause size={14} weight="fill" />
                                {:else}
                                    <Play size={14} weight="fill" />
                                {/if}
                            </button>
                        </div>

                        <div class="track-meta">
                            <span class="track-title">{track.title}</span>
                            <div class="track-subline">
                                <span class="track-artist">{track.artist}</span>
                                {#if track.album}
                                    <span class="track-dot">•</span>
                                    <span class="track-album">{track.album}</span>
                                {/if}
                            </div>
                        </div>

                        <span class="track-duration">{formatDuration(track.duration_ms)}</span>
                    </div>
                {/each}
            </div>
        {:else}
            <div class="empty-state">No releases found for this category.</div>
        {/if}
    </div>
{/if}

<style>
    .category-hub-canvas {
        display: flex;
        flex-direction: column;
        gap: 1.8rem;
    }
    .hub-header {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }
    .hub-back-btn {
        align-self: flex-start;
        background: transparent;
        border: none;
        color: #B58E62;
        font-size: 0.88rem;
        font-weight: 600;
        cursor: pointer;
        display: flex;
        align-items: center;
        gap: 0.4rem;
        padding: 0;
        transition: color 0.15s ease;
    }
    .hub-back-btn:hover {
        color: #FFFFFF;
    }
    .hub-title-row {
        display: flex;
        align-items: center;
        gap: 0.75rem;
    }
    .hub-color-badge {
        width: 14px;
        height: 14px;
        border-radius: 50%;
    }
    .hub-title-row h1 {
        font-size: 2rem;
        font-weight: 700;
        margin: 0;
    }
    .hub-subtitle {
        font-size: 0.9rem;
        color: rgba(255, 255, 255, 0.6);
        margin: 0;
    }
    .hub-tracks-grid {
        display: flex;
        flex-direction: column;
        gap: 0.4rem;
    }
    .empty-state {
        font-size: 0.88rem;
        color: rgba(255, 255, 255, 0.4);
        padding: 1.5rem 0;
    }
    .ledger-row {
        display: flex;
        align-items: center;
        gap: 0.9rem;
        padding: 0.6rem 0.75rem;
        border-radius: 8px;
        background: rgba(255, 255, 255, 0.02);
        border: 1px solid transparent;
        cursor: pointer;
        transition: background-color 0.15s ease, border-color 0.15s ease;
    }
    .ledger-row:hover {
        background: rgba(255, 255, 255, 0.06);
        border-color: rgba(255, 255, 255, 0.08);
    }
    .ledger-row.active-track {
        background: rgba(181, 142, 98, 0.12);
        border-color: rgba(181, 142, 98, 0.3);
    }
    .ledger-num {
        font-family: ui-monospace, SFMono-Regular, monospace;
        font-size: 0.9rem;
        font-weight: 700;
        color: #B58E62;
        min-width: 22px;
    }
    .track-art-wrapper {
        position: relative;
        width: 40px;
        height: 40px;
        flex-shrink: 0;
        border-radius: 6px;
        overflow: hidden;
    }
    .track-squircle {
        width: 100%;
        height: 100%;
        object-fit: cover;
    }
    .typographic-art-squircle {
        width: 100%;
        height: 100%;
        background: #232328;
        display: flex;
        align-items: center;
        justify-content: center;
        color: #B58E62;
        font-family: ui-serif, Georgia, serif;
        font-weight: 700;
        font-size: 1.1rem;
    }
    .play-overlay-btn {
        position: absolute;
        inset: 0;
        background: rgba(0, 0, 0, 0.6);
        border: none;
        color: #fff;
        display: flex;
        align-items: center;
        justify-content: center;
        opacity: 0;
        cursor: pointer;
        transition: opacity 0.15s ease;
    }
    .ledger-row:hover .play-overlay-btn, .ledger-row.active-track .play-overlay-btn {
        opacity: 1;
    }
    .track-meta {
        flex: 1;
        min-width: 0;
        display: flex;
        flex-direction: column;
        gap: 0.15rem;
    }
    .track-title {
        font-size: 0.92rem;
        font-weight: 600;
        color: #FFFFFF;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }
    .track-subline {
        display: flex;
        align-items: center;
        gap: 0.35rem;
        font-size: 0.8rem;
        color: rgba(255, 255, 255, 0.55);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }
    .track-artist, .track-album {
        overflow: hidden;
        text-overflow: ellipsis;
    }
    .track-dot {
        opacity: 0.4;
    }
    .track-duration {
        font-family: ui-monospace, SFMono-Regular, monospace;
        font-size: 0.78rem;
        color: rgba(255, 255, 255, 0.45);
    }
    .skeleton-box {
        background: rgba(255, 255, 255, 0.05);
        border-radius: 4px;
        animation: pulse 1.5s infinite ease-in-out;
    }
    .skeleton-line {
        height: 12px;
        background: rgba(255, 255, 255, 0.05);
        border-radius: 4px;
        animation: pulse 1.5s infinite ease-in-out;
    }
    .skeleton-line.title {
        width: 60%;
        margin-bottom: 4px;
    }
    .skeleton-line.artist {
        width: 35%;
    }
    @keyframes pulse {
        0%, 100% { opacity: 0.4; }
        50% { opacity: 0.8; }
    }
</style>
