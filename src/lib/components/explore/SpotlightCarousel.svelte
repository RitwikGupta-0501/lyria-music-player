<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import { exploreStore } from "$lib/stores/explore.svelte";
    import { settingsStore } from "$lib/stores/settings.svelte";
    import { Play, Sparkle, CaretLeft, CaretRight, Disc } from "phosphor-svelte";

    let autoScrollInterval: ReturnType<typeof setInterval> | null = null;
    let isHoveringHero = $state(false);

    function startAutoScroll() {
        stopAutoScroll();
        autoScrollInterval = setInterval(() => {
            if (!isHoveringHero && exploreStore.spotlights.length > 1 && !exploreStore.searchQuery) {
                exploreStore.nextSpotlight();
            }
        }, 7000);
    }

    function stopAutoScroll() {
        if (autoScrollInterval) {
            clearInterval(autoScrollInterval);
            autoScrollInterval = null;
        }
    }

    onMount(() => {
        startAutoScroll();
    });

    onDestroy(() => {
        stopAutoScroll();
    });
</script>

{#if exploreStore.spotlights.length > 0}
    {@const spot = exploreStore.spotlights[exploreStore.activeSpotlightIndex]}
    {#if spot}
        <section 
            class="hero-spotlight-card" 
            class:is-glass={settingsStore.glassyPlayerBar}
            aria-label="Editorial Spotlight"
            onmouseenter={() => isHoveringHero = true}
            onmouseleave={() => isHoveringHero = false}
        >
            {#if spot.cover_art_url}
                <div 
                    class="hero-ambient-backdrop" 
                    style="background-image: url('{spot.cover_art_url}');"
                ></div>
            {/if}

            <div class="hero-content">
                <div class="hero-left">
                    <div class="hero-eyebrow">
                        <Sparkle size={13} weight="fill" />
                        <span>EDITORIAL SPOTLIGHT</span>
                    </div>
                    <h1 class="hero-title">{spot.title}</h1>
                    <p class="hero-artist">{spot.artist}</p>
                    {#if spot.description}
                        <p class="hero-desc">{spot.description}</p>
                    {/if}
                    <div class="hero-actions">
                        <button class="echo-play-pill primary" onclick={() => exploreStore.playSpotlight()}>
                            <Play size={16} weight="fill" />
                            <span>Play Album</span>
                        </button>
                        <button class="echo-play-pill secondary" onclick={() => exploreStore.openAlbum({
                            id: spot.id,
                            title: spot.title,
                            artist: spot.artist,
                            cover_art_url: spot.cover_art_url,
                            provider_id: spot.provider_id || "youtube-wasm",
                        })}>
                            <Disc size={16} weight="bold" />
                            <span>Explore Release</span>
                        </button>
                    </div>
                </div>

                <div class="hero-right">
                    <div class="hero-art-wrapper">
                        {#if spot.cover_art_url}
                            <img src={spot.cover_art_url} alt={spot.title} class="hero-art-img" />
                        {:else}
                            <div class="hero-art-placeholder">
                                <Disc size={48} weight="thin" />
                            </div>
                        {/if}
                    </div>
                </div>
            </div>

            <!-- Carousel Controls -->
            {#if exploreStore.spotlights.length > 1}
                <div class="hero-carousel-controls" class:is-glass={settingsStore.glassyPlayerBar}>
                    <button class="hero-nav-arrow" onclick={() => exploreStore.prevSpotlight()} title="Previous spotlight">
                        <CaretLeft size={14} weight="bold" />
                    </button>
                    <div class="hero-pills">
                        {#each exploreStore.spotlights as _, idx}
                            <button 
                                class="hero-pill" 
                                class:active={exploreStore.activeSpotlightIndex === idx}
                                onclick={() => exploreStore.setSpotlightIndex(idx)}
                                title="Go to slide {idx + 1}"
                            ></button>
                        {/each}
                    </div>
                    <button class="hero-nav-arrow" onclick={() => exploreStore.nextSpotlight()} title="Next spotlight">
                        <CaretRight size={14} weight="bold" />
                    </button>
                </div>
            {/if}
        </section>
    {/if}
{/if}

<style>
    .hero-spotlight-card {
        position: relative;
        border-radius: 16px;
        overflow: hidden;
        border: 1px solid rgba(255, 255, 255, 0.08);
        background: #141417;
        min-height: 280px;
        box-shadow: 0 12px 32px rgba(0, 0, 0, 0.4);
    }
    .hero-spotlight-card.is-glass {
        background: rgba(22, 22, 28, 0.5);
        backdrop-filter: blur(24px);
        -webkit-backdrop-filter: blur(24px);
    }
    .hero-ambient-backdrop {
        position: absolute;
        inset: -20px;
        background-size: cover;
        background-position: center;
        filter: blur(40px) brightness(0.25) saturate(1.4);
        opacity: 0.6;
        z-index: 0;
        pointer-events: none;
    }
    .hero-content {
        position: relative;
        z-index: 1;
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 2.2rem 2.5rem;
        gap: 2rem;
    }
    .hero-left {
        display: flex;
        flex-direction: column;
        gap: 0.6rem;
        max-width: 580px;
    }
    .hero-eyebrow {
        display: flex;
        align-items: center;
        gap: 0.4rem;
        font-family: ui-monospace, SFMono-Regular, monospace;
        font-size: 0.72rem;
        font-weight: 700;
        color: #B58E62;
        letter-spacing: 0.08em;
    }
    .hero-title {
        font-size: 2.1rem;
        font-weight: 700;
        letter-spacing: -0.02em;
        line-height: 1.15;
        margin: 0;
        color: #FFFFFF;
    }
    .hero-artist {
        font-size: 1.05rem;
        color: rgba(255, 255, 255, 0.85);
        margin: 0;
        font-weight: 500;
    }
    .hero-desc {
        font-size: 0.88rem;
        color: rgba(255, 255, 255, 0.55);
        line-height: 1.45;
        margin: 0.2rem 0 0 0;
        display: -webkit-box;
        -webkit-line-clamp: 2;
        -webkit-box-orient: vertical;
        line-clamp: 2;
        overflow: hidden;
    }
    .hero-actions {
        display: flex;
        align-items: center;
        gap: 0.8rem;
        margin-top: 0.8rem;
    }
    .echo-play-pill {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        padding: 0.6rem 1.25rem;
        border-radius: 24px;
        font-size: 0.88rem;
        font-weight: 600;
        cursor: pointer;
        transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
        border: none;
    }
    .echo-play-pill.primary {
        background: #B58E62;
        color: #0E0E10;
    }
    .echo-play-pill.primary:hover {
        background: #C49E72;
        transform: scale(1.03);
    }
    .echo-play-pill.secondary {
        background: rgba(255, 255, 255, 0.08);
        border: 1px solid rgba(255, 255, 255, 0.15);
        color: #FFFFFF;
    }
    .echo-play-pill.secondary:hover {
        background: rgba(255, 255, 255, 0.15);
        border-color: rgba(255, 255, 255, 0.3);
    }
    .hero-right {
        flex-shrink: 0;
    }
    .hero-art-wrapper {
        width: 170px;
        height: 170px;
        border-radius: 12px;
        overflow: hidden;
        box-shadow: 0 16px 36px rgba(0, 0, 0, 0.6);
        border: 1px solid rgba(255, 255, 255, 0.1);
        background: #1B1B20;
    }
    .hero-art-img {
        width: 100%;
        height: 100%;
        object-fit: cover;
    }
    .hero-art-placeholder {
        width: 100%;
        height: 100%;
        display: flex;
        align-items: center;
        justify-content: center;
        color: rgba(255, 255, 255, 0.2);
    }
    .hero-carousel-controls {
        position: absolute;
        bottom: 1.2rem;
        right: 1.5rem;
        z-index: 3;
        display: flex;
        align-items: center;
        gap: 0.35rem;
        background: rgba(18, 18, 22, 0.75);
        border: 1px solid rgba(255, 255, 255, 0.08);
        border-radius: 20px;
        padding: 0.25rem 0.4rem;
        box-shadow: 0 4px 16px rgba(0, 0, 0, 0.35);
    }
    .hero-carousel-controls.is-glass {
        backdrop-filter: blur(12px);
        background: rgba(25, 25, 32, 0.45);
        border-color: rgba(255, 255, 255, 0.12);
    }
    .hero-nav-arrow {
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
    .hero-nav-arrow:hover {
        color: #B58E62;
        background: rgba(255, 255, 255, 0.16);
        border-color: rgba(181, 142, 98, 0.4);
    }
    .hero-pills {
        display: flex;
        align-items: center;
        gap: 0.35rem;
        padding: 0 0.25rem;
    }
    .hero-pill {
        width: 6px;
        height: 6px;
        border-radius: 3px;
        background: rgba(255, 255, 255, 0.3);
        border: none;
        cursor: pointer;
        padding: 0;
        transition: all 0.25s ease;
    }
    .hero-pill.active {
        width: 18px;
        background: #B58E62;
    }
</style>
