<script lang="ts">
    import { onMount } from "svelte";
    import { exploreStore } from "$lib/stores/explore.svelte";
    import TrackRow from "$lib/components/TrackRow.svelte";
    import { audioStore } from "$lib/stores/audio.svelte";
    import { formatDuration, getInitial, isCurrentTrack } from "$lib/utils/format";
    import { ArrowLeft, Play, Pause, Disc, MusicNotes } from "phosphor-svelte";
    import CategoryShelfCarousel from "./CategoryShelfCarousel.svelte";
    import AlbumCard from "$lib/components/AlbumCard.svelte";
    import SectionHeaderSkeleton from "$lib/components/common/SectionHeaderSkeleton.svelte";
    import BackButton from "$lib/components/common/BackButton.svelte";
    onMount(() => {
        const mainContent = document.querySelector('.main-content');
        if (mainContent) {
            mainContent.scrollTop = 0;
        }
    });
</script>

{#if exploreStore.activeCategory}
    <div class="category-hub-canvas">
        <!-- Hub Header -->
        <header class="hub-header">
            <BackButton label="Back to Explore" title="Back to Explore (Esc)" onclick={() => exploreStore.closeCategory()} />
            <div class="hub-title-row">
                {#if exploreStore.activeCategory.color_hex}
                    <div class="hub-color-badge" style="background: {exploreStore.activeCategory.color_hex}"></div>
                {/if}
                <h1>{exploreStore.activeCategory.title}</h1>
            </div>
            <p class="hub-subtitle">Ranked charts & curated editorial mixes</p>
        </header>

        {#if exploreStore.isCategoryLoading}
            <div class="hub-loading-state">
                <!-- Skeleton Shelf 1 -->
                <section class="hub-shelf-section">
                    <SectionHeaderSkeleton hasControls={true} titleWidth="180px" />
                    <div class="hub-carousel-track">
                        {#each Array(6) as _}
                            <div class="skeleton-card skeleton">
                                <div class="card-art-wrapper skeleton-box"></div>
                                <div class="title-skeleton skeleton-box"></div>
                                <div class="subtitle-skeleton skeleton-box"></div>
                            </div>
                        {/each}
                    </div>
                </section>

                <!-- Skeleton Shelf 2 -->
                <section class="hub-shelf-section">
                    <SectionHeaderSkeleton hasControls={true} titleWidth="150px" />
                    <div class="hub-carousel-track">
                        {#each Array(6) as _}
                            <div class="skeleton-card skeleton">
                                <div class="card-art-wrapper skeleton-box"></div>
                                <div class="title-skeleton skeleton-box"></div>
                                <div class="subtitle-skeleton skeleton-box"></div>
                            </div>
                        {/each}
                    </div>
                </section>

                <!-- Skeleton Top Songs Ledger -->
                <section class="hub-shelf-section">
                    <SectionHeaderSkeleton hasControls={false} titleWidth="160px" />
                    <div class="hub-tracks-grid">
                        {#each Array(6) as _}
                            <div class="ledger-row skeleton">
                                <div class="skeleton-num skeleton-box"></div>
                                <div class="track-squircle skeleton-box"></div>
                                <div class="track-meta">
                                    <div class="track-title-skeleton skeleton-box"></div>
                                    <div class="track-artist-skeleton skeleton-box"></div>
                                </div>
                            </div>
                        {/each}
                    </div>
                </section>
            </div>
        {:else}
            <!-- 1. Thematic Editorial Shelves (e.g. Flow State, Classical Focus, Deep Grooves) -->
            {#if exploreStore.categoryShelves.length > 0}
                {#each exploreStore.categoryShelves as shelf (shelf.title)}
                    <CategoryShelfCarousel title={shelf.title} items={shelf.items} />
                {/each}
            {:else if exploreStore.categoryPlaylists.length > 0}
                <!-- Fallback single shelf if provider returned flat playlists -->
                <CategoryShelfCarousel title={`Featured ${exploreStore.activeCategory.title} Mixes`} items={exploreStore.categoryPlaylists} />
            {/if}

            <!-- 2. Top Ranked Songs Section (if returned) -->
            {#if exploreStore.categoryTracks.length > 0}
                <section class="hub-shelf-section">
                    <div class="shelf-header-row">
                        <div class="shelf-title-group">
                            <MusicNotes size={18} weight="bold" class="shelf-icon" />
                            <h2>Top {exploreStore.activeCategory.title} Songs</h2>
                        </div>
                    </div>

                    <div class="hub-tracks-grid">
                        {#each exploreStore.categoryTracks as track, index}
                            <TrackRow 
                                track={track}
                                index={index + 1}
                            />
                        {/each}
                    </div>
                </section>
            {/if}

            <!-- 3. Genre Albums Section (if returned) -->
            {#if exploreStore.categoryAlbums.length > 0}
                <section class="hub-shelf-section">
                    <div class="shelf-header-row">
                        <div class="shelf-title-group">
                            <Disc size={18} weight="bold" class="shelf-icon" />
                            <h2>Albums & EPs</h2>
                        </div>
                    </div>

                    <div class="hub-carousel-track">
                        {#each exploreStore.categoryAlbums as album}
                            <div class="carousel-card-wrap">
                                <AlbumCard 
                                    album={album}
                                    onclick={() => exploreStore.openAlbum(album)}
                                />
                            </div>
                        {/each}
                    </div>
                </section>
            {/if}

            {#if exploreStore.categoryShelves.length === 0 && exploreStore.categoryPlaylists.length === 0 && exploreStore.categoryTracks.length === 0 && exploreStore.categoryAlbums.length === 0}
                <div class="empty-state">
                    <p>No curated releases found for this category.</p>
                    <button class="return-btn" onclick={() => exploreStore.closeCategory()}>Return to Explore</button>
                </div>
            {/if}
        {/if}
    </div>
{/if}

<style>
    .category-hub-canvas {
        display: flex;
        flex-direction: column;
        gap: 2.5rem;
        padding: 0 0 4rem 0;
        animation: fadeIn 0.3s cubic-bezier(0.16, 1, 0.3, 1);
    }

    @keyframes fadeIn {
        from { opacity: 0; transform: translateY(8px); }
        to { opacity: 1; transform: translateY(0); }
    }

    .hub-header {
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
        padding-bottom: 1.5rem;
        border-bottom: 1px solid rgba(255, 255, 255, 0.06);
    }


    .hub-title-row {
        display: flex;
        align-items: center;
        gap: 0.85rem;
    }

    .hub-color-badge {
        width: 14px;
        height: 14px;
        border-radius: 50%;
        box-shadow: 0 0 12px currentColor;
    }

    .hub-header h1 {
        font-family: var(--echo-font-heading, 'Newsreader', serif);
        font-size: 2.25rem;
        font-weight: 700;
        margin: 0;
        letter-spacing: -0.02em;
        color: #fff;
    }

    .hub-subtitle {
        font-size: 0.95rem;
        color: rgba(255, 255, 255, 0.5);
        margin: 0;
    }

    .hub-loading-state {
        display: flex;
        flex-direction: column;
        gap: 2.5rem;
    }

    .hub-shelf-section {
        display: flex;
        flex-direction: column;
        gap: 1.1rem;
        width: 100%;
    }

    .shelf-header-row {
        display: flex;
        align-items: center;
        justify-content: space-between;
    }

    .shelf-title-group {
        display: flex;
        align-items: center;
        gap: 0.6rem;
    }

    :global(.shelf-icon) {
        color: #B58E62;
    }

    .shelf-title-group h2 {
        font-family: var(--echo-font-heading, 'Newsreader', serif);
        font-size: 1.25rem;
        font-weight: 600;
        margin: 0;
        letter-spacing: -0.01em;
        color: #fff;
    }

    .hub-carousel-track {
        display: flex;
        gap: 1.25rem;
        overflow-x: auto;
        scrollbar-width: none;
        padding-bottom: 0.35rem;
    }

    .hub-carousel-track::-webkit-scrollbar {
        display: none;
    }

    .carousel-card-wrap {
        flex: 0 0 176px;
        min-width: 0;
    }

    .skeleton-card {
        flex: 0 0 176px;
        display: flex;
        flex-direction: column;
        gap: 0.6rem;
    }

    .skeleton-card .card-art-wrapper {
        width: 176px;
        height: 176px;
        border-radius: 1.5rem;
    }

    .skeleton-box {
        background: linear-gradient(90deg, rgba(255,255,255,0.03) 0%, rgba(255,255,255,0.07) 50%, rgba(255,255,255,0.03) 100%);
        background-size: 200% 100%;
        animation: shimmer 1.8s infinite;
    }

    @keyframes shimmer {
        0% { background-position: -200% 0; }
        100% { background-position: 200% 0; }
    }

    .title-skeleton {
        height: 14px;
        width: 75%;
        border-radius: 4px;
    }

    .subtitle-skeleton {
        height: 12px;
        width: 50%;
        border-radius: 4px;
    }

    .hub-tracks-grid {
        display: grid;
        grid-template-columns: repeat(2, 1fr);
        gap: 0.5rem 1rem;
        background: rgba(18, 18, 22, 0.5);
        border: 1px solid rgba(255, 255, 255, 0.05);
        border-radius: 16px;
        padding: 1rem;
    }

    @media (max-width: 900px) {
        .hub-tracks-grid {
            grid-template-columns: 1fr;
        }
    }


</style>
