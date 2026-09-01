<script lang="ts">
    import { homeStore, type FederatedTrack } from "$lib/stores/home.svelte";
    import { settingsStore } from "$lib/stores/settings.svelte";
    import { Sparkle, Play, CaretLeft, CaretRight, Heart } from "phosphor-svelte";
    import { resolveCoverArt } from "$lib/utils/media";

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
        const pageStep = Math.max(cardSpan, trackContainer.clientWidth - cardSpan);
        trackContainer.scrollBy({ left: -pageStep, behavior: "smooth" });
    }

    function scrollNext() {
        if (!trackContainer) return;
        const cardSpan = 176 + 20;
        const pageStep = Math.max(cardSpan, trackContainer.clientWidth - cardSpan);
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


            <div class="chevron-controls">
                <button 
                    class="chevron-btn" 
                    onclick={scrollPrev} 
                    disabled={!canScrollLeft}
                    title="Scroll Left"
                    aria-label="Previous tracks"
                >
                    <CaretLeft size={16} weight="bold" />
                </button>
                <button 
                    class="chevron-btn" 
                    onclick={scrollNext} 
                    disabled={!canScrollRight}
                    title="Scroll Right"
                    aria-label="Next tracks"
                >
                    <CaretRight size={16} weight="bold" />
                </button>
            </div>
        </div>
    </div>

    {#if homeStore.isLoadingRemote && homeStore.dailyDiscover.length === 0}
        <div class="discover-carousel" bind:this={trackContainer} onscroll={updateScrollState}>
            {#each Array(6) as _}
                <div class="discover-card skeleton">
                    <div class="card-art-wrapper skeleton-box"></div>
                    <div class="skeleton-line title"></div>
                    <div class="skeleton-line artist"></div>
                </div>
            {/each}
        </div>
    {:else if homeStore.dailyDiscover.length > 0}
        <div class="discover-carousel" bind:this={trackContainer} onscroll={updateScrollState}>
            {#each homeStore.dailyDiscover as item}
                <div 
                    class="discover-card"
                    role="button"
                    tabindex="0"
                    onclick={() => homeStore.playFederatedTrack(item)}
                    onkeydown={(e) => { if (e.key === "Enter") homeStore.playFederatedTrack(item); }}
                >
                    <div class="card-art-wrapper">
                        {#if resolveCoverArt(item.cover_art_url)}
                            <img src={resolveCoverArt(item.cover_art_url)} alt={item.title} loading="lazy" />
                        {:else}
                            <div class="placeholder-art"></div>
                        {/if}
                        <div class="card-overlay">
                            <div class="play-bubble">
                                <Play size={18} weight="fill" />
                            </div>
                        </div>
                        <button 
                            class="liquid-like-btn" 
                            class:is-glass={settingsStore.glassyPlayerBar}
                            class:liked={item.liked}
                            onclick={(e) => { e.stopPropagation(); homeStore.toggleLike(item); }}
                            title={item.liked ? "Liked" : "Like track"}
                            aria-label={item.liked ? "Unlike track" : "Like track"}
                        >
                            <Heart size={16} weight={item.liked ? "fill" : "bold"} color={item.liked ? "#ffd285" : "#FFFFFF"} />
                        </button>
                    </div>
                    <div class="card-info">
                        <span class="card-title" title={item.title}>{item.title}</span>
                        <span class="card-artist" title={item.artist}>{item.artist}</span>

                    </div>
                </div>
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
        font-family: var(--echo-font-heading, "Playfair Display", serif);
        font-size: 1.35rem;
        font-weight: 600;
        margin: 0;
        letter-spacing: -0.01em;
        color: #fff;
    }

    :global(.sparkle-icon) {
        color: #B58E62;
    }

    .header-right-controls {
        display: flex;
        align-items: center;
        gap: 0.75rem;
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

    .chevron-btn :global(svg) {
        display: block;
        flex-shrink: 0;
    }

    .chevron-btn:hover:not(:disabled) {
        color: #B58E62;
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

    .discover-carousel {
        display: flex;
        gap: 1.25rem;
        overflow-x: auto;
        scroll-snap-type: x mandatory;
        scroll-behavior: smooth;
        padding-bottom: 0.5rem;
        scrollbar-width: none;
        will-change: scroll-position;
    }

    .discover-carousel::-webkit-scrollbar {
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
        transform: scale(0.85);
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

    .liquid-like-btn:hover {
        transform: scale(1.12) !important;
        background: rgba(30, 32, 40, 0.95) !important;
        border-color: rgba(226, 169, 115, 0.35) !important;
        box-shadow: 0 6px 16px rgba(0, 0, 0, 0.55) !important;
    }

    .liquid-like-btn.is-glass:hover {
        background: rgba(255, 255, 255, 0.08) !important;
        border-color: rgba(226, 169, 115, 0.35) !important;
        box-shadow: 
            inset 0 1px 1px rgba(255, 255, 255, 0.25),
            0 6px 16px rgba(0, 0, 0, 0.45) !important;
    }

    .liquid-like-btn:active {
        transform: scale(0.92) !important;
    }

    .liquid-like-btn.liked {
        opacity: 1;
        transform: scale(1);
        pointer-events: auto;
        color: #ffd285;
        background: rgba(45, 35, 25, 0.9);
        border-color: rgba(224, 184, 143, 0.35);
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
    }

    .liquid-like-btn.is-glass.liked {
        background: linear-gradient(180deg, rgba(200, 157, 110, 0.22) 0%, rgba(150, 107, 61, 0.15) 100%);
        border-color: rgba(224, 184, 143, 0.35);
        box-shadow: 
            inset 0 1px 1px rgba(255, 255, 255, 0.25),
            0 4px 12px rgba(0, 0, 0, 0.35);
    }

    .liquid-like-btn :global(svg) {
        display: block;
        flex-shrink: 0;
        width: 16px;
        height: 16px;
        pointer-events: none;
    }

    .play-bubble {
        width: 42px;
        height: 42px;
        border-radius: 50%;
        background: #B58E62;
        color: #0E0E10;
        display: flex;
        align-items: center;
        justify-content: center;
        box-shadow: 0 4px 16px rgba(0, 0, 0, 0.5);
        transform: scale(0.9);
        transition: transform 0.2s ease, background 0.2s ease;
    }

    .discover-card:hover .play-bubble {
        transform: scale(1);
    }

    .play-bubble:hover {
        background: #D4A86E;
    }

    .card-info {
        display: flex;
        flex-direction: column;
        gap: 0.2rem;
    }

    .card-title {
        font-size: 0.88rem;
        font-weight: 600;
        color: #fff;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .card-artist {
        font-size: 0.78rem;
        color: rgba(255, 255, 255, 0.55);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    

    /* Skeleton */
    .skeleton-box {
        background: linear-gradient(90deg, rgba(255, 255, 255, 0.03) 25%, rgba(255, 255, 255, 0.07) 50%, rgba(255, 255, 255, 0.03) 75%);
        background-size: 200% 100%;
        animation: shimmer 1.5s infinite;
    }

    .skeleton-line {
        height: 11px;
        border-radius: 4px;
        background: linear-gradient(90deg, rgba(255, 255, 255, 0.03) 25%, rgba(255, 255, 255, 0.07) 50%, rgba(255, 255, 255, 0.03) 75%);
        background-size: 200% 100%;
        animation: shimmer 1.5s infinite;
    }

    .skeleton-line.title { width: 80%; }
    .skeleton-line.artist { width: 50%; }

    @keyframes shimmer {
        0% { background-position: 200% 0; }
        100% { background-position: -200% 0; }
    }
</style>
