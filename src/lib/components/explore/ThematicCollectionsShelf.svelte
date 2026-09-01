<script lang="ts">
    import { exploreStore, type PlaylistItem } from "$lib/stores/explore.svelte";
    import { Headphones, CaretLeft, CaretRight, Play, Disc } from "phosphor-svelte";

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
        if (trackContainer && exploreStore.featuredPlaylists.length > 0) {
            updateScrollState();
        }
    });

    function scrollPrev() {
        if (!trackContainer) return;
        const cardSpan = 300 + 16;
        trackContainer.scrollBy({ left: -cardSpan * 2, behavior: "smooth" });
    }

    function scrollNext() {
        if (!trackContainer) return;
        const cardSpan = 300 + 16;
        trackContainer.scrollBy({ left: cardSpan * 2, behavior: "smooth" });
    }

    function openPlaylist(playlist: PlaylistItem) {
        exploreStore.openPlaylist({
            id: playlist.id,
            title: playlist.title,
            author: playlist.author || undefined,
            cover_art_url: playlist.cover_art_url,
            provider_id: playlist.provider_id,
        });
    }
</script>

<svelte:window onresize={updateScrollState} />

<section class="thematic-collections-section">
    <div class="section-title-row">
        <div class="title-group">
            <Headphones size={20} weight="bold" class="section-icon" />
            <h2>Curated Thematic Collections</h2>
        </div>

        {#if hasOverflow}
            <div class="chevron-controls">
                <button 
                    class="chevron-btn" 
                    onclick={scrollPrev} 
                    disabled={!canScrollLeft}
                    title="Previous Collections"
                    aria-label="Previous Collections"
                >
                    <CaretLeft size={16} weight="bold" />
                </button>
                <button 
                    class="chevron-btn" 
                    onclick={scrollNext} 
                    disabled={!canScrollRight}
                    title="Next Collections"
                    aria-label="Next Collections"
                >
                    <CaretRight size={16} weight="bold" />
                </button>
            </div>
        {/if}
    </div>

    <div 
        class="thematic-carousel-track" 
        bind:this={trackContainer} 
        onscroll={updateScrollState}
    >
        {#each exploreStore.featuredPlaylists as playlist}
            <div 
                class="thematic-card"
                role="button"
                tabindex="0"
                onclick={() => openPlaylist(playlist)}
                onkeydown={(e) => { if (e.key === 'Enter') openPlaylist(playlist); }}
            >
                <div class="card-art-backdrop">
                    {#if playlist.cover_art_url}
                        <img src={playlist.cover_art_url} alt={playlist.title} class="backdrop-img" loading="lazy" />
                    {/if}
                    <div class="backdrop-gradient"></div>
                </div>

                <div class="card-content">
                    <div class="card-top-bar">
                        <div class="curated-pill">
                            <Disc size={12} weight="bold" color="#D4A86E" />
                            <span>COLLECTION</span>
                        </div>
                        {#if playlist.item_count}
                            <span class="track-count-badge">{playlist.item_count} Tracks</span>
                        {/if}
                    </div>

                    <div class="card-bottom-bar">
                        <div class="card-meta">
                            <h3 class="collection-title" title={playlist.title}>{playlist.title}</h3>
                            <p class="collection-subtitle" title={playlist.author}>{playlist.author || "Curated Functional Audio"}</p>
                        </div>

                        <button 
                            type="button" 
                            class="play-pill-btn" 
                            onclick={(e) => { e.stopPropagation(); openPlaylist(playlist); }}
                            title="Play Collection"
                        >
                            <Play size={13} weight="fill" />
                            <span>Explore</span>
                        </button>
                    </div>
                </div>
            </div>
        {/each}
    </div>
</section>

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
    }

    .title-group {
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

    .chevron-controls {
        display: flex;
        align-items: center;
        gap: 0.35rem;
        background: rgba(18, 18, 22, 0.75);
        border: 1px solid rgba(255, 255, 255, 0.08);
        border-radius: 20px;
        padding: 0.2rem 0.3rem;
    }

    .chevron-btn {
        width: 28px;
        height: 28px;
        border-radius: 50%;
        background: rgba(255, 255, 255, 0.08);
        border: 1px solid rgba(255, 255, 255, 0.12);
        color: #FFFFFF;
        display: flex;
        align-items: center;
        justify-content: center;
        cursor: pointer;
        padding: 0;
        transition: all 0.15s ease;
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

    .thematic-carousel-track {
        display: flex;
        gap: 1.25rem;
        overflow-x: auto;
        scroll-snap-type: x mandatory;
        scroll-behavior: smooth;
        padding-bottom: 0.5rem;
        scrollbar-width: none;
    }

    .thematic-carousel-track::-webkit-scrollbar {
        display: none;
    }

    .thematic-card {
        flex: 0 0 310px;
        height: 190px;
        border-radius: 12px;
        position: relative;
        overflow: hidden;
        cursor: pointer;
        scroll-snap-align: start;
        background: #141416;
        border: 1px solid rgba(255, 255, 255, 0.08);
        transition: transform 0.25s cubic-bezier(0.16, 1, 0.3, 1), border-color 0.25s ease, box-shadow 0.25s ease;
    }

    .thematic-card:hover {
        transform: translateY(-3px);
        border-color: rgba(181, 142, 98, 0.45);
        box-shadow: 0 12px 30px -6px rgba(0, 0, 0, 0.65);
    }

    .card-art-backdrop {
        position: absolute;
        inset: 0;
    }

    .backdrop-img {
        width: 100%;
        height: 100%;
        object-fit: cover;
        opacity: 0.45;
        transition: opacity 0.3s ease, transform 0.4s ease;
    }

    .thematic-card:hover .backdrop-img {
        opacity: 0.65;
        transform: scale(1.05);
    }

    .backdrop-gradient {
        position: absolute;
        inset: 0;
        background: linear-gradient(180deg, rgba(14, 14, 16, 0.4) 0%, rgba(10, 10, 12, 0.95) 100%);
    }

    .card-content {
        position: relative;
        z-index: 2;
        padding: 1.15rem;
        height: 100%;
        box-sizing: border-box;
        display: flex;
        flex-direction: column;
        justify-content: space-between;
    }

    .card-top-bar {
        display: flex;
        align-items: center;
        justify-content: space-between;
    }

    .curated-pill {
        display: inline-flex;
        align-items: center;
        gap: 0.35rem;
        background: rgba(181, 142, 98, 0.15);
        border: 1px solid rgba(181, 142, 98, 0.3);
        padding: 0.18rem 0.5rem;
        border-radius: 4px;
    }

    .curated-pill span {
        font-family: var(--echo-font-mono, monospace);
        font-size: 0.58rem;
        font-weight: 700;
        letter-spacing: 0.1em;
        color: #D4A86E;
    }

    .track-count-badge {
        font-family: var(--echo-font-mono, monospace);
        font-size: 0.65rem;
        color: rgba(255, 255, 255, 0.5);
    }

    .card-bottom-bar {
        display: flex;
        align-items: flex-end;
        justify-content: space-between;
        gap: 0.75rem;
    }

    .card-meta {
        flex: 1;
        min-width: 0;
        display: flex;
        flex-direction: column;
        gap: 0.2rem;
    }

    .collection-title {
        font-family: var(--echo-font-heading, "Playfair Display", serif);
        font-size: 1.1rem;
        font-weight: 600;
        margin: 0;
        color: #fff;
        line-height: 1.25;
        display: -webkit-box;
        -webkit-line-clamp: 1;
        line-clamp: 1;
        -webkit-box-orient: vertical;
        overflow: hidden;
    }

    .collection-subtitle {
        font-size: 0.74rem;
        margin: 0;
        color: rgba(255, 255, 255, 0.5);
        display: -webkit-box;
        -webkit-line-clamp: 1;
        line-clamp: 1;
        -webkit-box-orient: vertical;
        overflow: hidden;
    }

    .play-pill-btn {
        display: inline-flex;
        align-items: center;
        gap: 0.4rem;
        font-size: 0.72rem;
        font-weight: 600;
        color: #D4A86E;
        background: rgba(181, 142, 98, 0.14);
        border: 1px solid rgba(181, 142, 98, 0.3);
        padding: 0.35rem 0.65rem;
        border-radius: 6px;
        flex-shrink: 0;
        cursor: pointer;
        transition: all 0.2s ease;
    }

    .thematic-card:hover .play-pill-btn {
        background: #D4A86E;
        color: #0E0E10;
        border-color: #D4A86E;
    }
</style>
