<script lang="ts">
    import TrackCard from "$lib/components/TrackCard.svelte";
    import CarouselControls from "$lib/components/common/CarouselControls.svelte";
    import { homeStore } from "$lib/stores/home.svelte";
    import { Sparkle } from "phosphor-svelte";

    let trackContainer = $state<HTMLElement | null>(null);
    let canScrollLeft = $state(false);
    let canScrollRight = $state(true);

    function updateScrollState() {
        if (!trackContainer) return;
        const { scrollLeft, scrollWidth, clientWidth } = trackContainer;
        canScrollLeft = scrollLeft > 6;
        canScrollRight = scrollLeft + clientWidth < scrollWidth - 6;
    }

    $effect(() => {
        if (trackContainer && homeStore.dailyDiscover.length > 0) {
            updateScrollState();
        }
    });

    function scrollPrev() {
        if (!trackContainer) return;
        const cardSpan = 176 + 20; // 176px card width + 20px gap
        const pageStep = Math.max(
            cardSpan,
            trackContainer.clientWidth - cardSpan,
        );
        trackContainer.scrollBy({ left: -pageStep, behavior: "smooth" });
    }

    function scrollNext() {
        if (!trackContainer) return;
        const cardSpan = 176 + 20;
        const pageStep = Math.max(
            cardSpan,
            trackContainer.clientWidth - cardSpan,
        );
        trackContainer.scrollBy({ left: pageStep, behavior: "smooth" });
    }
</script>

<section class="daily-discover-section">
    <div class="section-title-row">
        <div class="title-group">
            <Sparkle size={20} weight="fill" class="sparkle-icon" />
            <h2>Daily Discover</h2>
        </div>

        <div class="header-right-controls">
            <CarouselControls
                container={trackContainer}
                canPrev={canScrollLeft}
                canNext={canScrollRight}
                onPrev={scrollPrev}
                onNext={scrollNext}
                prevLabel="Previous tracks"
                nextLabel="Next tracks"
            />
        </div>
    </div>

    {#if homeStore.isLoadingRemote && homeStore.dailyDiscover.length === 0}
        <div
            class="discover-carousel"
            bind:this={trackContainer}
            onscroll={updateScrollState}
        >
            {#each Array(6) as _}
                <div class="discover-skeleton-card">
                    <div class="skeleton-art skeleton-box"></div>
                    <div class="skeleton-info">
                        <div class="skeleton-line title"></div>
                        <div class="skeleton-line artist"></div>
                    </div>
                </div>
            {/each}
        </div>
    {:else if homeStore.dailyDiscover.length > 0}
        <div
            class="discover-carousel"
            bind:this={trackContainer}
            onscroll={updateScrollState}
        >
            {#each homeStore.dailyDiscover as item}
                <TrackCard
                    track={item}
                    onclick={() => homeStore.playFederatedTrack(item)}
                />
            {/each}
        </div>
    {/if}
</section>

<style>
    .daily-discover-section {
        display: flex;
        flex-direction: column;
        gap: 1.25rem;
    }

    .section-title-row {
        display: flex;
        align-items: center;
        justify-content: space-between;
    }

    .title-group {
        display: flex;
        align-items: center;
        gap: 0.65rem;
    }

    .section-title-row h2 {
        font-family: var(--echo-font-heading, 'Newsreader', serif);
        font-size: 1.35rem;
        font-weight: 600;
        margin: 0;
        letter-spacing: -0.01em;
        color: #fff;
    }

    :global(.sparkle-icon) {
        color: #b58e62;
    }

    .header-right-controls {
        display: flex;
        align-items: center;
        gap: 0.75rem;
    }

    .discover-carousel {
        display: flex;
        align-items: flex-start;
        gap: 1.25rem;
        overflow-x: auto;
        scroll-snap-type: x mandatory;
        scroll-behavior: smooth;
        padding-bottom: 0.5rem;
        scrollbar-width: none;
        will-change: scroll-position;
        min-width: 0;
        max-width: 100%;
    }

    .discover-carousel::-webkit-scrollbar {
        display: none;
    }

    /* Skeleton */
    .discover-skeleton-card {
        flex: 0 0 176px;
        width: 176px;
        min-width: 176px;
        scroll-snap-align: start;
        display: flex;
        flex-direction: column;
        gap: 0.6rem;
    }

    .skeleton-art {
        width: 176px;
        height: 176px;
        border-radius: 12px;
        background: #141416;
        border: 1px solid rgba(255, 255, 255, 0.08);
    }

    .skeleton-box {
        background: linear-gradient(
            90deg,
            rgba(255, 255, 255, 0.03) 25%,
            rgba(255, 255, 255, 0.07) 50%,
            rgba(255, 255, 255, 0.03) 75%
        );
        background-size: 200% 100%;
        animation: shimmer 1.5s infinite;
    }

    .skeleton-info {
        display: flex;
        flex-direction: column;
        gap: 0.35rem;
    }

    .skeleton-line {
        height: 11px;
        border-radius: 4px;
        background: linear-gradient(
            90deg,
            rgba(255, 255, 255, 0.03) 25%,
            rgba(255, 255, 255, 0.07) 50%,
            rgba(255, 255, 255, 0.03) 75%
        );
        background-size: 200% 100%;
        animation: shimmer 1.5s infinite;
    }

    .skeleton-line.title {
        width: 80%;
    }
    .skeleton-line.artist {
        width: 50%;
    }

    @keyframes shimmer {
        0% {
            background-position: 200% 0;
        }
        100% {
            background-position: -200% 0;
        }
    }
</style>
