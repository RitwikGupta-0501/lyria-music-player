<script lang="ts">
    import { homeStore, type FederatedTrack } from "$lib/stores/home.svelte";
    import { Sparkle, Play, CaretLeft, CaretRight } from "phosphor-svelte";

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
            {#if homeStore.failedProviders.length > 0}
                <span class="section-tag warning" title="Degraded: {homeStore.failedProviders.join(', ')}">Partial Feed</span>
            {:else}
                <span class="section-tag">Curated Provenance</span>
            {/if}

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
                        {#if item.cover_art_url}
                            <img src={item.cover_art_url.startsWith("/") ? `asset://localhost/${encodeURIComponent(item.cover_art_url)}` : item.cover_art_url} alt={item.title} loading="lazy" />
                        {:else}
                            <div class="placeholder-art"></div>
                        {/if}
                        <div class="card-overlay">
                            <div class="play-bubble">
                                <Play size={18} weight="fill" />
                            </div>
                        </div>
                    </div>
                    <div class="card-info">
                        <span class="card-title" title={item.title}>{item.title}</span>
                        <span class="card-artist" title={item.artist}>{item.artist}</span>
                        {#if item.seed_provenance}
                            <span class="card-provenance" title={item.seed_provenance}>{item.seed_provenance}</span>
                        {/if}
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

    .section-tag {
        font-family: var(--echo-font-mono, monospace);
        font-size: 0.65rem;
        font-weight: 600;
        text-transform: uppercase;
        letter-spacing: 0.08em;
        color: #B58E62;
        background: rgba(181, 142, 98, 0.08);
        border: 1px solid rgba(181, 142, 98, 0.18);
        padding: 0.2rem 0.55rem;
        border-radius: 4px;
    }

    .section-tag.warning {
        color: #E09F55;
        background: rgba(224, 159, 85, 0.1);
        border-color: rgba(224, 159, 85, 0.2);
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
        transition: opacity 0.2s ease;
    }

    .discover-card:hover .card-overlay {
        opacity: 1;
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

    .card-provenance {
        display: inline-block;
        font-family: var(--echo-font-mono, monospace);
        font-size: 0.65rem;
        font-weight: 600;
        letter-spacing: 0.05em;
        color: rgba(181, 142, 98, 0.85);
        background: rgba(181, 142, 98, 0.1);
        border: 1px solid rgba(181, 142, 98, 0.2);
        padding: 0.15rem 0.45rem;
        border-radius: 4px;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        max-width: 100%;
        margin-top: 0.15rem;
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
