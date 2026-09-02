<script lang="ts">
    import { exploreStore, type AlbumItem } from "$lib/stores/explore.svelte";
    import { libraryStore, getCanonicalKey } from "$lib/stores/library.svelte";
    import { settingsStore } from "$lib/stores/settings.svelte";
    import { getInitial } from "$lib/utils/format";
    import { Disc, CaretLeft, CaretRight, Play, Heart } from "phosphor-svelte";

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

{#if exploreStore.trendingAlbums.length > 0}
    <section class="thematic-collections-section">
        <div class="section-title-row">
            <div class="title-group">
                <Disc size={20} weight="bold" class="section-icon" />
                <h2>Trending Albums</h2>
            </div>

            {#if hasOverflow}
                <div class="chevron-controls">
                    <button 
                        class="chevron-btn" 
                        onclick={scrollPrev} 
                        disabled={!canScrollLeft}
                        title="Previous Albums"
                        aria-label="Previous Albums"
                    >
                        <CaretLeft size={16} weight="bold" />
                    </button>
                    <button 
                        class="chevron-btn" 
                        onclick={scrollNext} 
                        disabled={!canScrollRight}
                        title="Next Albums"
                        aria-label="Next Albums"
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
            {#each exploreStore.trendingAlbums as album}
                {@const albumKey = getCanonicalKey({ title: album.title, artist: album.artist })}
                {@const isLiked = libraryStore.likedSongs.some(s => s.canonical_key.toLowerCase().trim() === albumKey.toLowerCase().trim())}
                <div 
                    class="discover-card"
                    role="button"
                    tabindex="0"
                    onclick={() => openAlbum(album)}
                    onkeydown={(e) => { if (e.key === "Enter") openAlbum(album); }}
                >
                    <div class="card-art-wrapper">
                        {#if album.cover_art_url}
                            <img src={album.cover_art_url} alt={album.title} loading="lazy" />
                        {:else}
                            <div class="placeholder-art">
                                <span>{getInitial(album.artist)}</span>
                            </div>
                        {/if}

                        <div class="card-overlay">
                            <div class="play-bubble">
                                <Play size={18} weight="fill" />
                            </div>
                        </div>

                        <button 
                            type="button"
                            class="liquid-like-btn" 
                            class:is-glass={settingsStore.glassyPlayerBar}
                            class:liked={isLiked}
                            onclick={(e) => { 
                                e.stopPropagation(); 
                                libraryStore.toggleLike({
                                    title: album.title,
                                    artist: album.artist,
                                    canonical_key: albumKey,
                                    cover_art_url: album.cover_art_url || undefined,
                                });
                            }}
                            title={isLiked ? "Liked" : "Like album"}
                            aria-label={isLiked ? "Unlike album" : "Like album"}
                        >
                            <Heart size={16} weight={isLiked ? "fill" : "bold"} color={isLiked ? "#ffd285" : "#FFFFFF"} />
                        </button>
                    </div>

                    <div class="card-info">
                        <span class="card-title" title={album.title}>{album.title}</span>
                        <span class="card-artist" title={album.artist}>
                            {album.artist}{#if album.year} • {album.year}{/if}
                        </span>
                    </div>
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
        transition: color 0.15s ease, background-color 0.15s ease, border-color 0.15s ease, transform 0.1s ease, opacity 0.2s ease;
    }

    .chevron-btn:hover:not(:disabled) {
        color: var(--echo-primary, #B58E62);
        background: rgba(255, 255, 255, 0.16);
        border-color: rgba(181, 142, 98, 0.4);
    }

    .chevron-btn:active:not(:disabled) {
        transform: scale(0.92);
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
        scrollbar-width: none;
        padding-bottom: 0.5rem;
    }

    .thematic-carousel-track::-webkit-scrollbar {
        display: none;
    }

    .discover-card {
        flex: 0 0 176px;
        width: 176px;
        min-width: 176px;
        scroll-snap-align: start;
        display: flex;
        flex-direction: column;
        gap: 0.6rem;
        cursor: pointer;
        transition: transform 0.2s cubic-bezier(0.16, 1, 0.3, 1);
    }

    .discover-card:hover {
        transform: translateY(-3px);
    }

    .card-art-wrapper {
        width: 176px;
        height: 176px;
        border-radius: 12px;
        overflow: hidden;
        position: relative;
        background: #141416;
        border: 1px solid rgba(255, 255, 255, 0.08);
        contain: layout paint;
        isolation: isolate;
        transition: border-color 0.2s ease, box-shadow 0.2s ease;
    }

    .discover-card:hover .card-art-wrapper {
        border-color: rgba(181, 142, 98, 0.35);
        box-shadow: 0 10px 24px -6px rgba(0, 0, 0, 0.6);
    }

    .card-art-wrapper img {
        width: 100%;
        height: 100%;
        object-fit: cover;
    }

    .placeholder-art {
        width: 100%;
        height: 100%;
        background: #18181B;
        display: flex;
        align-items: center;
        justify-content: center;
        font-family: var(--echo-font-heading, serif);
        font-size: 2rem;
        font-weight: 700;
        color: #D4A86E;
    }

    .card-overlay {
        position: absolute;
        inset: 0;
        background: rgba(0, 0, 0, 0.35);
        display: flex;
        align-items: center;
        justify-content: center;
        opacity: 0;
        pointer-events: none;
        transform: translateZ(0);
        backface-visibility: hidden;
        will-change: opacity;
        transition: opacity 0.2s cubic-bezier(0.16, 1, 0.3, 1);
    }

    .discover-card:hover .card-overlay {
        opacity: 1;
        pointer-events: auto;
    }

    .play-bubble {
        width: 44px;
        height: 44px;
        border-radius: 50%;
        background: #B58E62;
        color: #0E0E10;
        display: flex;
        align-items: center;
        justify-content: center;
        box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
        transform: scale(0.9);
        transition: transform 0.2s cubic-bezier(0.16, 1, 0.3, 1), background 0.15s ease;
    }

    .discover-card:hover .play-bubble {
        transform: scale(1);
    }

    .play-bubble:hover {
        transform: scale(1.08) !important;
        background: #C9A070;
    }

    .liquid-like-btn {
        position: absolute;
        top: 8px;
        right: 8px;
        width: 32px !important;
        height: 32px !important;
        min-width: 32px !important;
        max-width: 32px !important;
        min-height: 32px !important;
        max-height: 32px !important;
        border-radius: 50% !important;
        padding: 0 !important;
        margin: 0 !important;
        background: rgba(18, 20, 26, 0.85);
        border: 1px solid rgba(255, 255, 255, 0.12);
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.45);
        color: #ffffff;
        display: flex !important;
        align-items: center !important;
        justify-content: center !important;
        cursor: pointer;
        opacity: 0;
        transform: scale(0.85) translateZ(0);
        backface-visibility: hidden;
        pointer-events: none;
        transition: opacity 0.2s cubic-bezier(0.16, 1, 0.3, 1), transform 0.2s cubic-bezier(0.16, 1, 0.3, 1), background 0.2s ease, border-color 0.2s ease, box-shadow 0.2s ease;
        z-index: 5;
    }

    .liquid-like-btn.is-glass {
        background: rgba(255, 255, 255, 0.028);
        backdrop-filter: blur(8px) saturate(1.35) contrast(1.08) brightness(1.02);
        -webkit-backdrop-filter: blur(8px) saturate(1.35) contrast(1.08) brightness(1.02);
        border: 1px solid rgba(255, 255, 255, 0.10);
        box-shadow: 
            inset 0 1px 1px rgba(255, 255, 255, 0.18),
            inset 0 -1px 1px rgba(0, 0, 0, 0.18),
            0 4px 12px rgba(0, 0, 0, 0.35);
    }

    .discover-card:hover .liquid-like-btn {
        opacity: 1;
        transform: scale(1);
        pointer-events: auto;
    }

    .liquid-like-btn.liked {
        opacity: 1 !important;
        transform: scale(1) !important;
        pointer-events: auto !important;
        background: rgba(45, 35, 25, 0.9) !important;
        border-color: rgba(224, 184, 143, 0.45) !important;
    }

    .card-info {
        display: flex;
        flex-direction: column;
        gap: 0.2rem;
        min-width: 0;
    }

    .card-title {
        font-family: var(--echo-font-body, system-ui, sans-serif);
        font-size: 0.88rem;
        font-weight: 600;
        color: #fff;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .card-artist {
        font-size: 0.76rem;
        color: rgba(255, 255, 255, 0.5);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }
</style>
