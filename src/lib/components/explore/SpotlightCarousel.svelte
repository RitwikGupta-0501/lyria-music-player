<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import { exploreStore } from "$lib/stores/explore.svelte";
    import { settingsStore } from "$lib/stores/settings.svelte";
    import { Play, Sparkle, Disc } from "phosphor-svelte";
    import CarouselControls from "$lib/components/common/CarouselControls.svelte";
    import PillButton from "$lib/components/common/PillButton.svelte";

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

{#if exploreStore.isLoading && exploreStore.spotlights.length === 0}
    <section class="hero-spotlight-card skeleton-hero" aria-hidden="true">
        <div class="hero-content">
            <div class="hero-left">
                <div class="skeleton-pill eyebrow-skeleton"></div>
                <div class="skeleton-line xl title-skeleton"></div>
                <div class="skeleton-line md artist-skeleton"></div>
                <div class="skeleton-line sm desc-skeleton"></div>
                <div class="hero-actions-skeleton">
                    <div class="skeleton-pill action-pill"></div>
                    <div class="skeleton-pill action-pill-sec"></div>
                </div>
            </div>
            <div class="hero-right">
                <div class="skeleton-box hero-art-skeleton"></div>
            </div>
        </div>
    </section>
{:else if exploreStore.spotlights.length > 0}
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
                        <PillButton 
                            variant="primary"
                            icon={Play}
                            label="Play Album"
                            onclick={() => exploreStore.playSpotlight()}
                        />
                        <PillButton 
                            variant="secondary"
                            icon={Disc}
                            label="Explore Release"
                            onclick={() => exploreStore.openAlbum({
                                id: spot.id,
                                title: spot.title,
                                artist: spot.artist,
                                cover_art_url: spot.cover_art_url,
                                provider_id: spot.provider_id || settingsStore.getEffectiveRemoteProvider(),
                            })}
                        />
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
                <div class="hero-controls-wrap">
                    <CarouselControls
                        canPrev={true}
                        canNext={true}
                        isGlass={settingsStore.glassyPlayerBar}
                        onPrev={() => exploreStore.prevSpotlight()}
                        onNext={() => exploreStore.nextSpotlight()}
                        prevLabel="Previous spotlight"
                        nextLabel="Next spotlight"
                    >
                        {#snippet children()}
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
                        {/snippet}
                    </CarouselControls>
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
        font-family: var(--echo-font-heading, 'Newsreader', serif);
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
    .hero-controls-wrap {
        position: absolute;
        bottom: 1.2rem;
        right: 1.5rem;
        z-index: 3;
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

    /* Hero Skeleton Loader */
    .skeleton-hero {
        background: #141417;
        border: 1px solid rgba(255, 255, 255, 0.06);
    }

    .eyebrow-skeleton {
        width: 140px;
        height: 22px;
        margin-bottom: 0.5rem;
    }

    .title-skeleton {
        width: 80%;
        margin-bottom: 0.5rem;
    }

    .artist-skeleton {
        width: 50%;
        margin-bottom: 0.75rem;
    }

    .desc-skeleton {
        width: 65%;
        margin-bottom: 1.25rem;
    }

    .hero-actions-skeleton {
        display: flex;
        gap: 0.75rem;
    }

    .action-pill {
        width: 130px;
        height: 38px;
    }

    .action-pill-sec {
        width: 150px;
        height: 38px;
    }

    .hero-art-skeleton {
        width: 170px;
        height: 170px;
        border-radius: 12px;
    }

</style>
