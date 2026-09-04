<script lang="ts">
    import { exploreStore, type PlaylistItem } from "$lib/stores/explore.svelte";
    import PlaylistCard from "$lib/components/PlaylistCard.svelte";
    import { Playlist, CaretLeft, CaretRight } from "phosphor-svelte";

    let { title, items = [] } = $props<{
        title: string;
        items: PlaylistItem[];
    }>();

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
        if (trackContainer && items.length > 0) {
            updateScrollState();
        }
    });

    function scrollPrev() {
        if (!trackContainer) return;
        const cardSpan = 176 + 20;
        trackContainer.scrollBy({ left: -cardSpan * 2, behavior: "smooth" });
    }

    function scrollNext() {
        if (!trackContainer) return;
        const cardSpan = 176 + 20;
        trackContainer.scrollBy({ left: cardSpan * 2, behavior: "smooth" });
    }
</script>

<svelte:window onresize={updateScrollState} />

{#if items.length > 0}
    <section class="hub-shelf-section">
        <div class="shelf-header-row">
            <div class="shelf-title-group">
                <Playlist size={18} weight="bold" class="shelf-icon" />
                <h2>{title}</h2>
            </div>
            {#if hasOverflow}
                <div class="chevron-controls">
                    <button class="chevron-btn" onclick={scrollPrev} disabled={!canScrollLeft} title="Scroll Left">
                        <CaretLeft size={13} weight="bold" />
                    </button>
                    <button class="chevron-btn" onclick={scrollNext} disabled={!canScrollRight} title="Scroll Right">
                        <CaretRight size={13} weight="bold" />
                    </button>
                </div>
            {/if}
        </div>

        <div 
            class="hub-carousel-track" 
            bind:this={trackContainer}
            onscroll={updateScrollState}
        >
            {#each items as playlist}
                <div class="carousel-card-wrap">
                    <PlaylistCard 
                        playlist={playlist}
                        onclick={() => exploreStore.openPlaylist({ id: playlist.id, title: playlist.title, author: playlist.author || undefined, cover_art_url: playlist.cover_art_url, provider_id: playlist.provider_id })}
                    />
                </div>
            {/each}
        </div>
    </section>
{/if}

<style>
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
        font-family: var(--echo-font-heading, "Playfair Display", serif);
        font-size: 1.25rem;
        font-weight: 600;
        margin: 0;
        letter-spacing: -0.01em;
        color: #fff;
    }

    .chevron-controls {
        display: flex;
        align-items: center;
        gap: 0.35rem;
        background: rgba(18, 18, 22, 0.75);
        border: 1px solid rgba(255, 255, 255, 0.08);
        border-radius: 20px;
        padding: 0.2rem 0.3rem;
        box-shadow: 0 4px 16px rgba(0, 0, 0, 0.35);
    }

    .chevron-btn {
        width: 26px;
        height: 26px;
        border-radius: 50%;
        background: rgba(255, 255, 255, 0.08);
        border: 1px solid rgba(255, 255, 255, 0.12);
        color: #FFFFFF;
        display: flex;
        align-items: center;
        justify-content: center;
        cursor: pointer;
        padding: 0;
        transition: color 0.15s ease, background-color 0.15s ease, border-color 0.15s ease, transform 0.1s ease;
    }

    .chevron-btn:hover:not(:disabled) {
        color: #B58E62;
        background: rgba(255, 255, 255, 0.16);
        border-color: rgba(181, 142, 98, 0.4);
    }

    .chevron-btn:disabled {
        opacity: 0.25;
        pointer-events: none;
        cursor: default;
    }

    .hub-carousel-track {
        display: flex;
        gap: 1.25rem;
        overflow-x: auto;
        scrollbar-width: none;
        padding-top: 8px;
        margin-top: -8px;
        padding-bottom: 0.5rem;
        margin-bottom: -0.15rem;
    }

    .hub-carousel-track::-webkit-scrollbar {
        display: none;
    }

    .carousel-card-wrap {
        flex: 0 0 176px;
        min-width: 0;
    }
</style>
