<script lang="ts">
    import { onMount } from "svelte";
    import { exploreStore } from "$lib/stores/explore.svelte";
    import { audioStore } from "$lib/stores/audio.svelte";
    import { formatDuration, getInitial, isCurrentTrack } from "$lib/utils/format";
    import { MagnifyingGlass, Play, Pause, ArrowClockwise, X } from "phosphor-svelte";

    import SpotlightCarousel from "./explore/SpotlightCarousel.svelte";
    import CategoryGrid from "./explore/CategoryGrid.svelte";
    import CategoryHubView from "./explore/CategoryHubView.svelte";
    import SearchResultsFeed from "./explore/SearchResultsFeed.svelte";

    let { activeView = $bindable("explore") } = $props<{ activeView?: string }>();

    onMount(() => {
        exploreStore.init();

        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === "Escape") {
                if (exploreStore.searchQuery) {
                    exploreStore.clearSearch();
                } else if (exploreStore.activeCategory) {
                    exploreStore.closeCategory();
                }
            }
        };

        window.addEventListener("keydown", handleKeyDown);
        return () => {
            window.removeEventListener("keydown", handleKeyDown);
        };
    });
</script>

<div class="explore-canvas">
    <!-- Top Discovery Bar -->
    <div class="discovery-bar-wrapper">
        <div class="discovery-bar">
            <MagnifyingGlass size={18} weight="bold" color="#B58E62" />
            <input 
                type="text" 
                placeholder="Search songs, albums, artists, or providers..." 
                value={exploreStore.searchQuery}
                oninput={(e) => exploreStore.setSearchQuery((e.target as HTMLInputElement).value)}
                class="discovery-input"
            />
            {#if exploreStore.searchQuery}
                <button class="clear-search-btn" onclick={() => exploreStore.clearSearch()} title="Clear search (Esc)">
                    <X size={16} weight="bold" />
                </button>
            {/if}
            <button 
                class="refresh-btn" 
                class:spinning={exploreStore.isLoading || exploreStore.isSearching} 
                onclick={() => {
                    if (exploreStore.searchQuery) {
                        exploreStore.performSearch(exploreStore.searchQuery);
                    } else {
                        exploreStore.loadExplore(true);
                    }
                }}
                title="Refresh"
            >
                <ArrowClockwise size={16} weight="bold" />
            </button>
        </div>
    </div>

    <!-- 1. Dedicated Search Results Feed -->
    {#if exploreStore.searchQuery.trim().length > 0}
        <SearchResultsFeed />

    <!-- 2. Category Hub View -->
    {:else if exploreStore.activeCategory}
        <CategoryHubView />

    <!-- 3. General Curated Explore Canvas -->
    {:else}
        <!-- Featured Spotlight Carousel -->
        <SpotlightCarousel />

        <!-- Browse by Category Grid -->
        <CategoryGrid />

        <!-- Discovery Split: Top Global Tracks (60%) & New Releases (40%) -->
        <section class="discovery-split-section">
            <div class="split-column left-ledger-column">
                <div class="section-header">
                    <h2>Top Global Tracks</h2>
                </div>
                <div class="ledger-container">
                    {#if exploreStore.filteredRankedTracks.length > 0}
                        {#each exploreStore.filteredRankedTracks as track, i}
                            <div 
                                class="ledger-row"
                                class:active-track={isCurrentTrack(track, audioStore.currentQueueTrack)}
                                role="button"
                                tabindex="0"
                                ondblclick={() => exploreStore.playTrack(track, track.provider_id)}
                                onkeydown={(e) => { if (e.key === 'Enter') exploreStore.playTrack(track, track.provider_id); }}
                            >
                                <span class="ledger-num">{(i + 1).toString().padStart(2, '0')}</span>
                                
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
                                    <span class="track-artist">{track.artist}</span>
                                </div>

                                <span class="track-duration">{formatDuration(track.duration_ms)}</span>
                            </div>
                        {/each}
                    {:else}
                        <div class="empty-ledger">No top tracks found.</div>
                    {/if}
                </div>
            </div>

            <div class="split-column right-albums-column">
                <div class="section-header">
                    <h2>New Releases</h2>
                </div>
                <div class="albums-2x2-grid">
                    {#each exploreStore.filteredNewReleases as album}
                        <div 
                            class="album-card"
                            role="button"
                            tabindex="0"
                            onclick={() => exploreStore.playAlbum(album)}
                            onkeydown={(e) => { if (e.key === 'Enter') exploreStore.playAlbum(album); }}
                        >
                            <div class="album-art-wrapper">
                                {#if album.cover_art_url}
                                    <img src={album.cover_art_url} alt={album.title} class="album-img" />
                                {:else}
                                    <div class="typographic-art-squircle large">
                                        <span>{getInitial(album.artist)}</span>
                                    </div>
                                {/if}
                                <div class="album-overlay">
                                    <div class="play-bubble">
                                        <Play size={18} weight="fill" />
                                    </div>
                                </div>
                            </div>
                            <div class="album-meta">
                                <span class="album-title">{album.title}</span>
                                <span class="album-artist">{album.artist}</span>
                            </div>
                        </div>
                    {/each}
                </div>
            </div>
        </section>
    {/if}
</div>

<style>
    .explore-canvas {
        padding: 1.5rem 2rem var(--player-clearance, 10rem) 2rem;
        scroll-padding-bottom: var(--player-scroll-padding, 10rem);
        width: 100%;
        display: flex;
        flex-direction: column;
        gap: 2.2rem;
        color: #fff;
        box-sizing: border-box;
    }
    @media (max-width: 900px) {
        .explore-canvas {
            padding: 1.25rem 1.25rem 9rem 1.25rem;
            gap: 1.5rem;
        }
    }
    .discovery-bar-wrapper {
        width: 100%;
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
    }
    .discovery-bar {
        height: 44px;
        background: #161618;
        border: 1px solid rgba(255, 255, 255, 0.08);
        border-radius: 8px;
        display: flex;
        align-items: center;
        padding: 0 0.85rem;
        gap: 0.75rem;
        transition: border-color 0.2s ease, box-shadow 0.2s ease, background-color 0.2s ease;
    }
    .discovery-bar:focus-within {
        background: #19191C;
        border-color: rgba(181, 142, 98, 0.45);
        box-shadow: 0 0 0 1px rgba(181, 142, 98, 0.25), 0 4px 16px rgba(0, 0, 0, 0.25);
    }
    .discovery-input {
        flex: 1;
        height: 100%;
        background: transparent !important;
        border: none !important;
        outline: none !important;
        box-shadow: none !important;
        padding: 0 0.35rem !important;
        margin: 0 !important;
        color: #fff;
        font-family: ui-monospace, SFMono-Regular, monospace;
        font-size: 0.88rem;
    }
    .discovery-input::placeholder {
        color: rgba(255, 255, 255, 0.35);
    }
    .clear-search-btn, .refresh-btn {
        background: transparent;
        border: none;
        color: rgba(255, 255, 255, 0.75);
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 0.3rem;
        border-radius: 4px;
        transition: color 0.15s ease;
    }
    .clear-search-btn:hover, .refresh-btn:hover {
        color: #B58E62;
    }
    .refresh-btn.spinning :global(svg) {
        animation: spin 1s linear infinite;
    }
    @keyframes spin {
        from { transform: rotate(0deg); }
        to { transform: rotate(360deg); }
    }
    .section-header h2 {
        font-size: 1.15rem;
        font-weight: 700;
        letter-spacing: -0.01em;
        margin: 0;
        color: #EAEAEA;
    }
    .discovery-split-section {
        display: grid;
        grid-template-columns: 60% calc(40% - 1.5rem);
        gap: 1.5rem;
        align-items: start;
    }
    @media (max-width: 1024px) {
        .discovery-split-section {
            grid-template-columns: 1fr;
        }
    }
    .split-column {
        display: flex;
        flex-direction: column;
        gap: 0.9rem;
    }
    .ledger-container {
        display: flex;
        flex-direction: column;
        gap: 0.35rem;
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
    .typographic-art-squircle.large {
        font-size: 2.2rem;
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
    .track-artist {
        font-size: 0.8rem;
        color: rgba(255, 255, 255, 0.55);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }
    .track-duration {
        font-family: ui-monospace, SFMono-Regular, monospace;
        font-size: 0.78rem;
        color: rgba(255, 255, 255, 0.45);
    }
    .albums-2x2-grid {
        display: grid;
        grid-template-columns: repeat(2, minmax(0, 1fr));
        gap: 0.85rem;
        width: 100%;
    }
    .album-card {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
        cursor: pointer;
        padding: 0.6rem;
        border-radius: 10px;
        background: rgba(255, 255, 255, 0.025);
        border: 1px solid rgba(255, 255, 255, 0.06);
        transition: all 0.15s ease;
        min-width: 0;
    }
    .album-card:hover {
        background: rgba(255, 255, 255, 0.05);
        transform: translateY(-2px);
    }
    .album-art-wrapper {
        position: relative;
        width: 100%;
        aspect-ratio: 1 / 1;
        border-radius: 8px;
        overflow: hidden;
        background: #232328;
    }
    .album-img {
        width: 100%;
        height: 100%;
        object-fit: cover;
    }
    .album-overlay {
        position: absolute;
        inset: 0;
        background: rgba(0, 0, 0, 0.4);
        display: flex;
        align-items: center;
        justify-content: center;
        opacity: 0;
        transition: opacity 0.15s ease;
    }
    .album-card:hover .album-overlay {
        opacity: 1;
    }
    .play-bubble {
        width: 36px;
        height: 36px;
        border-radius: 50%;
        background: #FFFFFF;
        color: #000;
        display: flex;
        align-items: center;
        justify-content: center;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
    }
    .album-meta {
        display: flex;
        flex-direction: column;
        gap: 0.15rem;
        min-width: 0;
    }
    .album-title {
        font-size: 0.92rem;
        font-weight: 700;
        color: #FFFFFF;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }
    .album-artist {
        font-size: 0.78rem;
        color: rgba(255, 255, 255, 0.5);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }
    .empty-ledger {
        font-size: 0.88rem;
        color: rgba(255, 255, 255, 0.4);
        padding: 1rem 0;
    }
</style>
