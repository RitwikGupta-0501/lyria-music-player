<script lang="ts">
    import { exploreStore, type AlbumItem } from "$lib/stores/explore.svelte";
    import AlbumCard from "$lib/components/AlbumCard.svelte";
    import CarouselControls from "$lib/components/common/CarouselControls.svelte";
    import { Disc } from "phosphor-svelte";
    import SectionHeaderSkeleton from "$lib/components/common/SectionHeaderSkeleton.svelte";

    let trackContainer = $state<HTMLElement | null>(null);
    let canScrollLeft = $state(false);
    let canScrollRight = $state(true);
    let hasOverflow = $state(false);

    function updateScrollState() {
        if (!trackContainer) return;
        const { scrollLeft, scrollWidth, clientWidth } = trackContainer;
        hasOverflow = scrollWidth > clientWidth + 6;
        canScrollLeft = scrollLeft > 6;
        canScrollRight = scrollLeft + clientWidth < scrollWidth - 6;
    }

    $effect(() => {
        if (trackContainer && exploreStore.trendingAlbums.length > 0) {
            updateScrollState();
        }
    });

    function openAlbum(album: AlbumItem) {
        exploreStore.openAlbum({
            id: album.id,
            title: album.title,
            artist: album.artist,
            year: album.year,
            cover_art_url: album.cover_art_url,
            provider_id: album.provider_id,
        });
    }
</script>

<svelte:window onresize={updateScrollState} />

{#if exploreStore.isLoading && exploreStore.trendingAlbums.length === 0}
    <section class="thematic-collections-section">
        <SectionHeaderSkeleton hasControls={true} titleWidth="150px" />
        <div class="thematic-carousel-track">
            {#each Array(6) as _}
                <div class="thematic-album-item skeleton">
                    <div class="skeleton-art skeleton-box"></div>
                    <div class="skeleton-line title-skeleton"></div>
                    <div class="skeleton-line artist-skeleton"></div>
                </div>
            {/each}
        </div>
    </section>
{:else if exploreStore.trendingAlbums.length > 0}
    <section class="thematic-collections-section">
        <div class="section-title-row">
            <div class="title-group">
                <Disc size={20} weight="bold" class="section-icon" />
                <h2>Trending Albums</h2>
            </div>

            {#if hasOverflow}
                <CarouselControls
                    container={trackContainer}
                    bind:canPrev={canScrollLeft}
                    bind:canNext={canScrollRight}
                    prevLabel="Previous albums"
                    nextLabel="Next albums"
                />
            {/if}
        </div>

        <div 
            class="thematic-carousel-track" 
            bind:this={trackContainer} 
            onscroll={updateScrollState}
        >
            {#each exploreStore.trendingAlbums as album}
                <div class="thematic-album-item">
                    <AlbumCard 
                        album={album}
                        onclick={() => openAlbum(album)}
                    />
                </div>
            {/each}
        </div>
    </section>
{/if}

<style>
    .thematic-collections-section {
        display: flex;
        flex-direction: column;
        gap: 1.25rem;
    }

    .section-title-row {
        display: flex;
        align-items: center;
        justify-content: space-between;
        min-height: 36px;
    }

    .title-group {
        display: flex;
        align-items: center;
        gap: 0.65rem;
    }

    :global(.section-icon) {
        color: #B58E62;
    }

    h2 {
        font-family: var(--echo-font-heading, "Newsreader", serif);
        font-size: 1.35rem;
        font-weight: 600;
        margin: 0;
        letter-spacing: -0.01em;
        color: #fff;
    }

    .thematic-carousel-track {
        display: flex;
        gap: 1.25rem;
        overflow-x: auto;
        scroll-snap-type: x mandatory;
        scrollbar-width: none;
        padding-top: 8px;
        margin-top: -8px;
        padding-bottom: 0.75rem;
        margin-bottom: -0.25rem;
    }

    .thematic-carousel-track::-webkit-scrollbar {
        display: none;
    }

    .thematic-album-item {
        flex: 0 0 176px;
        width: 176px;
        min-width: 176px;
        scroll-snap-align: start;
    }

    /* Skeleton Loading State */
    .skeleton-art {
        width: 176px;
        height: 176px;
        border-radius: 10px;
        background: #18181c;
    }

    .skeleton-line {
        height: 12px;
        border-radius: 4px;
        background: rgba(255, 255, 255, 0.05);
        margin-top: 0.5rem;
    }

    .title-skeleton {
        width: 75%;
    }

    .artist-skeleton {
        width: 50%;
    }

    .skeleton-box {
        animation: pulse 1.5s ease-in-out infinite;
    }

    @keyframes pulse {
        0%, 100% { opacity: 0.4; }
        50% { opacity: 0.8; }
    }
</style>
