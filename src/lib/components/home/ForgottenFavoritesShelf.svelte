<script lang="ts">
    import TrackCard from "$lib/components/TrackCard.svelte";
    import CarouselControls from "$lib/components/common/CarouselControls.svelte";
    import { homeStore } from "$lib/stores/home.svelte";
    import { ClockCounterClockwise } from "phosphor-svelte";

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
        if (trackContainer && homeStore.forgottenFavorites.length > 0) {
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

<svelte:window onresize={updateScrollState} />

{#if homeStore.forgottenFavorites.length > 0}
    <section class="forgotten-favorites-section">
        <div class="section-title-row">
            <div class="title-group">
                <ClockCounterClockwise size={20} weight="bold" class="fav-icon" />
                <h2>Forgotten Favorites</h2>
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

        <div
            class="favs-carousel-track"
            bind:this={trackContainer}
            onscroll={updateScrollState}
        >
            {#each homeStore.forgottenFavorites as song}
                <TrackCard
                    track={song}
                    onclick={() => homeStore.playFederatedTrack(song)}
                />
            {/each}
        </div>
    </section>
{/if}

<style>
    .forgotten-favorites-section {
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
        font-family: var(--echo-font-heading, sans-serif);
        font-size: 1.35rem;
        font-weight: 600;
        margin: 0;
        letter-spacing: -0.02em;
        color: #fff;
    }

    :global(.fav-icon) {
        color: #b58e62;
    }

    .header-right-controls {
        display: flex;
        align-items: center;
        gap: 0.75rem;
    }

    .favs-carousel-track {
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

    .favs-carousel-track::-webkit-scrollbar {
        display: none;
    }
</style>
