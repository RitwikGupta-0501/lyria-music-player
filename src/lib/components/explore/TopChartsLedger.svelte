<script lang="ts">
    import { exploreStore } from "$lib/stores/explore.svelte";
    import { audioStore } from "$lib/stores/audio.svelte";
    import { formatDuration, getInitial, isCurrentTrack } from "$lib/utils/format";
    import { Play, Pause, ChartLineUp, Flame, GlobeHemisphereWest } from "phosphor-svelte";
</script>

<div class="top-charts-ledger">
    <div class="section-header-row">
        <div class="header-title-group">
            <ChartLineUp size={20} weight="bold" class="section-icon" />
            <h2>Top Charts</h2>
        </div>

        <div class="chart-tab-pills">
            <button 
                class="chart-pill" 
                class:active={exploreStore.activeChartTab === 'global'}
                onclick={() => exploreStore.setChartTab('global')}
            >
                <GlobeHemisphereWest size={13} weight="bold" />
                <span>Global Top 50</span>
            </button>
            <button 
                class="chart-pill" 
                class:active={exploreStore.activeChartTab === 'viral'}
                onclick={() => exploreStore.setChartTab('viral')}
            >
                <Flame size={13} weight="bold" />
                <span>Trending Viral</span>
            </button>
            <button 
                class="chart-pill" 
                class:active={exploreStore.activeChartTab === 'regional'}
                onclick={() => exploreStore.setChartTab('regional')}
            >
                <span>Regional ({exploreStore.userRegion.countryName})</span>
            </button>
        </div>
    </div>

    <div class="ledger-container">
        {#if exploreStore.isLoadingChartTab || (exploreStore.isLoading && exploreStore.currentChartTracks.length === 0)}
            <div class="ledger-loading-skeleton">
                {#each Array(6) as _}
                    <div class="skeleton-row"></div>
                {/each}
            </div>
        {:else if exploreStore.currentChartTracks.length > 0}
            <div class="ledger-rows-stack">
                {#each exploreStore.currentChartTracks.slice(0, 10) as track, i (track.id || i)}
                    {@const isPlaying = isCurrentTrack(track, audioStore.currentQueueTrack) && audioStore.playbackState === "Playing"}
                    {@const isActive = isCurrentTrack(track, audioStore.currentQueueTrack)}
                    <div 
                        class="ledger-row"
                        class:active-track={isActive}
                        role="button"
                        tabindex="0"
                        onclick={() => exploreStore.playTrack(track, track.provider_id)}
                        onkeydown={(e) => { if (e.key === 'Enter') exploreStore.playTrack(track, track.provider_id); }}
                    >
                        <span class="ledger-num" class:top-rank={i < 3}>{(i + 1).toString().padStart(2, '0')}</span>
                        
                        <div class="track-art-wrapper">
                            {#if track.cover_art_url}
                                <img src={track.cover_art_url} alt={track.title} class="track-squircle" loading="lazy" />
                            {:else}
                                <div class="typographic-art-squircle">
                                    <span>{getInitial(track.artist)}</span>
                                </div>
                            {/if}
                            <button 
                                class="play-overlay-btn" 
                                class:always-visible={isActive}
                                onclick={(e) => { e.stopPropagation(); exploreStore.playTrack(track, track.provider_id); }}
                                title={isPlaying ? "Pause" : "Play"}
                            >
                                {#if isPlaying}
                                    <Pause size={13} weight="fill" />
                                {:else}
                                    <Play size={13} weight="fill" />
                                {/if}
                            </button>
                        </div>

                        <div class="track-meta">
                            <span class="track-title" title={track.title}>{track.title}</span>
                            <span class="track-artist" title={track.artist}>{track.artist}</span>
                        </div>

                        {#if isPlaying}
                            <div class="playing-visualizer">
                                <div class="bar"></div>
                                <div class="bar"></div>
                                <div class="bar"></div>
                            </div>
                        {/if}

                        <span class="track-duration">{formatDuration(track.duration_ms)}</span>
                    </div>
                {/each}
            </div>
        {:else}
            <div class="empty-ledger">No top chart tracks available.</div>
        {/if}
    </div>
</div>

<style>
    .top-charts-ledger {
        display: flex;
        flex-direction: column;
        gap: 1rem;
        height: 100%;
    }

    .section-header-row {
        display: flex;
        align-items: center;
        justify-content: space-between;
        flex-wrap: wrap;
        gap: 0.75rem;
        min-height: 36px;
    }

    .header-title-group {
        display: flex;
        align-items: center;
        gap: 0.55rem;
    }

    :global(.section-icon) {
        color: #B58E62;
    }

    h2 {
        font-family: var(--echo-font-heading, "Playfair Display", serif);
        font-size: 1.35rem;
        font-weight: 600;
        margin: 0;
        color: #fff;
    }

    .chart-tab-pills {
        display: flex;
        align-items: center;
        gap: 0.35rem;
        background: rgba(18, 18, 22, 0.75);
        border: 1px solid rgba(255, 255, 255, 0.08);
        border-radius: 20px;
        padding: 0.2rem 0.3rem;
    }

    .chart-pill {
        display: inline-flex;
        align-items: center;
        gap: 0.35rem;
        font-size: 0.72rem;
        font-weight: 600;
        color: rgba(255, 255, 255, 0.55);
        background: transparent;
        border: none;
        padding: 0.25rem 0.6rem;
        border-radius: 14px;
        cursor: pointer;
        transition: all 0.15s ease;
    }

    .chart-pill:hover {
        color: #fff;
        background: rgba(255, 255, 255, 0.06);
    }

    .chart-pill.active {
        color: #0E0E10;
        background: #D4A86E;
    }

    .ledger-container {
        height: 380px;
        max-height: 380px;
        overflow-y: auto;
        background: rgba(18, 18, 22, 0.55);
        border: 1px solid rgba(255, 255, 255, 0.07);
        border-radius: 12px;
        padding: 0.6rem;
        box-sizing: border-box;
    }

    .ledger-container::-webkit-scrollbar {
        width: 4px;
    }

    .ledger-container::-webkit-scrollbar-track {
        background: transparent;
    }

    .ledger-container::-webkit-scrollbar-thumb {
        background: rgba(255, 255, 255, 0.15);
        border-radius: 4px;
    }

    .ledger-container::-webkit-scrollbar-thumb:hover {
        background: rgba(212, 168, 110, 0.5);
    }

    .ledger-rows-stack {
        display: flex;
        flex-direction: column;
        gap: 0.35rem;
    }

    .ledger-loading-skeleton {
        display: flex;
        flex-direction: column;
        gap: 0.45rem;
    }

    .skeleton-row {
        height: 48px;
        border-radius: 8px;
        background: rgba(255, 255, 255, 0.04);
        animation: pulse 1.5s infinite ease-in-out;
    }

    @keyframes pulse {
        0%, 100% { opacity: 0.3; }
        50% { opacity: 0.7; }
    }

    .ledger-row {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        padding: 0.45rem 0.75rem;
        border-radius: 8px;
        transition: background 0.15s ease, transform 0.15s ease;
        cursor: pointer;
        position: relative;
        min-height: 48px;
        box-sizing: border-box;
    }

    .ledger-row:hover {
        background: rgba(255, 255, 255, 0.06);
        transform: translateX(2px);
    }

    .ledger-row.active-track {
        background: rgba(181, 142, 98, 0.14);
        border: 1px solid rgba(181, 142, 98, 0.3);
    }

    .ledger-num {
        font-family: var(--echo-font-mono, monospace);
        font-size: 0.8rem;
        font-weight: 700;
        color: rgba(255, 255, 255, 0.35);
        width: 1.5rem;
        text-align: center;
        flex-shrink: 0;
    }

    .ledger-num.top-rank {
        color: #D4A86E;
    }

    .track-art-wrapper {
        width: 40px;
        height: 40px;
        position: relative;
        flex-shrink: 0;
        border-radius: 6px;
        overflow: hidden;
        background: #141416;
    }

    .track-squircle {
        width: 100%;
        height: 100%;
        object-fit: cover;
    }

    .typographic-art-squircle {
        width: 100%;
        height: 100%;
        background: #202026;
        display: flex;
        align-items: center;
        justify-content: center;
        font-family: var(--echo-font-heading, serif);
        font-size: 0.95rem;
        font-weight: 700;
        color: #D4A86E;
    }

    .play-overlay-btn {
        position: absolute;
        inset: 0;
        background: rgba(0, 0, 0, 0.65);
        border: none;
        color: #fff;
        display: flex;
        align-items: center;
        justify-content: center;
        opacity: 0;
        cursor: pointer;
        transition: opacity 0.15s ease;
    }

    .ledger-row:hover .play-overlay-btn,
    .play-overlay-btn.always-visible {
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
        font-size: 0.86rem;
        font-weight: 600;
        color: #fff;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .track-artist {
        font-size: 0.74rem;
        color: rgba(255, 255, 255, 0.45);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .playing-visualizer {
        display: flex;
        align-items: flex-end;
        gap: 2px;
        height: 11px;
        margin-right: 0.4rem;
    }

    .playing-visualizer .bar {
        width: 2.2px;
        background-color: #D4A86E;
        border-radius: 1px;
    }

    .playing-visualizer .bar:nth-child(1) { height: 100%; animation: eq 1s ease-in-out infinite; }
    .playing-visualizer .bar:nth-child(2) { height: 60%; animation: eq 1.3s ease-in-out infinite; }
    .playing-visualizer .bar:nth-child(3) { height: 80%; animation: eq 0.8s ease-in-out infinite; }

    @keyframes eq {
        0%, 100% { transform: scaleY(0.3); }
        50% { transform: scaleY(1); }
    }

    .track-duration {
        font-family: var(--echo-font-mono, monospace);
        font-size: 0.74rem;
        color: rgba(255, 255, 255, 0.4);
        flex-shrink: 0;
    }

    .empty-ledger {
        padding: 2rem;
        text-align: center;
        color: rgba(255, 255, 255, 0.4);
        font-size: 0.85rem;
    }
</style>
