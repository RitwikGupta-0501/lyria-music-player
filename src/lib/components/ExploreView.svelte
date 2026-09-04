<script lang="ts">
    import { onMount } from "svelte";
    import { exploreStore } from "$lib/stores/explore.svelte";
    import { MagnifyingGlass, ArrowClockwise, X } from "phosphor-svelte";

    import SpotlightCarousel from "./explore/SpotlightCarousel.svelte";
    import TopChartsLedger from "./explore/TopChartsLedger.svelte";
    import NewReleaseRadar from "./explore/NewReleaseRadar.svelte";
    import CategoryGrid from "./explore/CategoryGrid.svelte";
    import ThematicCollectionsShelf from "./explore/ThematicCollectionsShelf.svelte";
    import CategoryHubView from "./explore/CategoryHubView.svelte";
    import SearchResultsFeed from "./explore/SearchResultsFeed.svelte";

    let { activeView = $bindable("explore") } = $props<{ activeView?: string }>();
    let radarHeight = $state<number>(0);

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
    <!-- Top Discovery Search Bar -->
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
        <!-- 4.1. Featured Editorial Spotlight -->
        <SpotlightCarousel />

        <!-- 4.3 & 4.4. Discovery Split: Top Charts (58%) & New Release Radar (42%) -->
        <section class="discovery-split-section">
            {#if exploreStore.isLoadingChartTab || exploreStore.isLoading || exploreStore.currentChartTracks.length > 0}
                <div 
                    class="split-column left-ledger-column"
                    style={(exploreStore.isLoading || exploreStore.filteredNewReleases.length > 0) && radarHeight > 0 ? `max-height: ${radarHeight}px; height: ${radarHeight}px;` : ""}
                >
                    <TopChartsLedger />
                </div>
            {/if}

            {#if exploreStore.isLoading || exploreStore.filteredNewReleases.length > 0}
                <div 
                    class="split-column right-albums-column"
                    bind:clientHeight={radarHeight}
                >
                    <NewReleaseRadar />
                </div>
            {/if}
        </section>

        <!-- 4.2. Mood & Genre Matrix -->
        <CategoryGrid />

        <!-- 4.6. Curated Thematic Collections Carousel -->
        <ThematicCollectionsShelf />
    {/if}
</div>

<style>
    .explore-canvas {
        padding: 1.5rem 2rem var(--player-clearance, 10rem) 2rem;
        scroll-padding-bottom: var(--player-scroll-padding, 10rem);
        width: 100%;
        display: flex;
        flex-direction: column;
        gap: 2.5rem;
        color: #fff;
        box-sizing: border-box;
    }

    @media (max-width: 900px) {
        .explore-canvas {
            padding: 1.25rem 1.25rem 9rem 1.25rem;
            gap: 1.75rem;
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

    .discovery-split-section {
        display: flex;
        flex-wrap: wrap;
        gap: 1.5rem;
        align-items: flex-start;
        width: 100%;
    }

    .split-column {
        display: flex;
        flex-direction: column;
        min-width: 0;
        min-height: 0;
    }

    .left-ledger-column {
        flex: 58 1 440px;
    }

    .right-albums-column {
        flex: 42 1 340px;
    }
</style>
