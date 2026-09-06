<script lang="ts">
    import { homeStore } from "$lib/stores/home.svelte";
    import { exploreStore } from "$lib/stores/explore.svelte";
    import TrackRow from "$lib/components/TrackRow.svelte";
    import { audioStore } from "$lib/stores/audio.svelte";
    import { resolveCoverArt } from "$lib/utils/media";
    import { Compass, Play, Pause } from "phosphor-svelte";
    import CarouselControls from "$lib/components/common/CarouselControls.svelte";

    let payload = $derived(homeStore.adjacentHorizon);
    let totalHorizons = $derived(homeStore.adjacentHorizons.length);
    let currentIndex = $derived(homeStore.activeHorizonIndex);

    function isTrackPlaying(track: any): boolean {
        return (
            audioStore.playbackState === "Playing" &&
            audioStore.currentQueueTrack?.title?.toLowerCase() === track.title?.toLowerCase()
        );
    }

    function formatDuration(ms?: number | null): string {
        if (!ms) return "";
        const s = Math.floor(ms / 1000);
        const mins = Math.floor(s / 60);
        const secs = s % 60;
        return `${mins}:${secs < 10 ? "0" : ""}${secs}`;
    }
</script>

{#if payload}
    <section class="adjacent-horizons-section">
        <!-- Section Header & Deck Stepper Controls -->
        <div class="section-title-row">
            <div class="title-group">
                <Compass size={20} weight="bold" color="#B58E62" />
                <h2>Adjacent Horizons</h2>
            </div>

            {#if totalHorizons > 1}
                <div class="deck-nav-group">
                    <!-- Dot Indicators -->
                    <div class="deck-dots">
                        {#each homeStore.adjacentHorizons as _, idx}
                            <button 
                                class="deck-dot" 
                                class:active={idx === currentIndex}
                                onclick={() => homeStore.selectHorizon(idx)}
                                title={`Horizon ${idx + 1}`}
                                aria-label={`Go to Horizon ${idx + 1}`}
                            ></button>
                        {/each}
                    </div>

                    <!-- Prev/Next Chevrons -->
                    <CarouselControls 
                        canPrev={true} 
                        canNext={true} 
                        onPrev={() => homeStore.prevHorizon()} 
                        onNext={() => homeStore.nextHorizon()} 
                        prevLabel="Previous Horizon"
                        nextLabel="Next Horizon"
                    />
                </div>
            {/if}
        </div>

        <!-- Master Luxury Horizon Card -->
        <div class="horizon-card">
            <div class="ambient-glow"></div>
            
            <!-- Vinyl Groove Watermark -->
            <div class="vinyl-watermark">
                <svg viewBox="0 0 240 240" xmlns="http://www.w3.org/2000/svg">
                    <circle cx="120" cy="120" r="100" stroke="rgba(255,255,255,0.03)" stroke-width="1.5" fill="none" />
                    <circle cx="120" cy="120" r="75" stroke="rgba(255,255,255,0.035)" stroke-width="1.5" fill="none" />
                    <circle cx="120" cy="120" r="50" stroke="rgba(181,142,98,0.08)" stroke-width="2" fill="none" />
                </svg>
            </div>

            <!-- Left Half: Editorial & Cohesive Primary Action -->
            <div class="horizon-left">
                <div class="horizon-pill">
                    <Compass size={12} weight="bold" class="pill-icon" />
                    <span>ADJACENT HORIZON</span>
                </div>

                <h3 class="horizon-tagline">{payload.tagline}</h3>
                <p class="horizon-desc">{payload.description}</p>

                <div class="horizon-actions">
                    <button 
                        class="explore-btn"
                        onclick={() => {
                            if (payload) {
                                exploreStore.openHorizon(payload);
                            }
                        }}
                    >
                        <Play size={15} weight="fill" />
                        <span>Explore Horizon</span>
                    </button>
                </div>
            </div>

            <!-- Right Half: Sound Preview Ledger -->
            {#if payload.preview_tracks && payload.preview_tracks.length > 0}
                <div class="horizon-right">
                    <div class="preview-ledger">
                        {#each payload.preview_tracks.slice(0, 3) as track}
                            <TrackRow 
                                track={track}
                                variant="compact"
                                onclick={() => homeStore.playFederatedTrack(track)}
                            />
                        {/each}
                    </div>
                </div>
            {/if}
        </div>
    </section>
{/if}

<style>
    .adjacent-horizons-section {
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
        width: 100%;
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

    :global(.horizon-header-icon) {
        color: #B58E62;
    }

    .section-title-row h2 {
        font-family: var(--echo-font-heading, 'Newsreader', serif);
        font-size: 1.35rem;
        font-weight: 600;
        margin: 0;
        letter-spacing: -0.01em;
        color: #fff;
    }



    .deck-nav-group {
        display: flex;
        align-items: center;
        gap: 1rem;
    }

    .deck-dots {
        display: flex;
        align-items: center;
        gap: 0.35rem;
    }

    .deck-dot {
        width: 6px;
        height: 6px;
        border-radius: 50%;
        background: rgba(255, 255, 255, 0.2);
        border: none;
        padding: 0;
        cursor: pointer;
        transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
    }

    .deck-dot:hover {
        background: rgba(255, 255, 255, 0.5);
    }

    .deck-dot.active {
        width: 18px;
        border-radius: 3px;
        background: #B58E62;
    }



    .horizon-card {
        position: relative;
        margin-top: 0.25rem;
        background: #141416;
        border: 1px solid rgba(255, 255, 255, 0.08);
        border-radius: 14px;
        padding: 2.25rem;
        display: grid;
        grid-template-columns: 1.15fr 0.85fr;
        gap: 3rem;
        align-items: center;
        overflow: hidden;
        box-shadow: 0 12px 32px -8px rgba(0, 0, 0, 0.6);
        transition: border-color 0.25s ease, box-shadow 0.25s ease;
    }

    .horizon-card:hover {
        border-color: rgba(181, 142, 98, 0.35);
        box-shadow: 0 16px 40px -8px rgba(0, 0, 0, 0.75);
    }

    .ambient-glow {
        position: absolute;
        top: -40%;
        left: -15%;
        width: 65%;
        height: 180%;
        background: radial-gradient(circle, rgba(181, 142, 98, 0.08) 0%, transparent 65%);
        pointer-events: none;
        filter: blur(50px);
    }

    .vinyl-watermark {
        position: absolute;
        right: -30px;
        bottom: -30px;
        width: 240px;
        height: 240px;
        pointer-events: none;
        opacity: 0.6;
    }

    .horizon-left {
        display: flex;
        flex-direction: column;
        align-items: flex-start;
        gap: 0.9rem;
        z-index: 1;
    }

    .horizon-pill {
        display: inline-flex;
        align-items: center;
        gap: 0.45rem;
        background: rgba(181, 142, 98, 0.1);
        border: 1px solid rgba(181, 142, 98, 0.22);
        padding: 0.22rem 0.6rem;
        border-radius: 4px;
        width: fit-content;
    }

    :global(.pill-icon) {
        color: #D4A86E;
    }

    .horizon-pill span {
        font-family: var(--lyria-font-mono);
        font-size: 0.62rem;
        font-weight: 700;
        letter-spacing: 0.08em;
        color: #D4A86E;
    }

    .horizon-tagline {
        font-family: var(--echo-font-heading, 'Newsreader', serif);
        font-size: 1.55rem;
        font-weight: 600;
        line-height: 1.25;
        margin: 0;
        color: #fff;
        letter-spacing: -0.01em;
    }

    .horizon-desc {
        font-size: 0.88rem;
        line-height: 1.55;
        color: rgba(255, 255, 255, 0.65);
        margin: 0;
        max-width: 95%;
    }

    .horizon-actions {
        margin-top: 0.5rem;
    }

    .explore-btn {
        display: inline-flex;
        align-items: center;
        gap: 0.55rem;
        background: #B58E62;
        color: #0E0E10;
        border: none;
        padding: 0.65rem 1.25rem;
        border-radius: 6px;
        font-size: 0.84rem;
        font-weight: 600;
        letter-spacing: 0.02em;
        cursor: pointer;
        transition: background 0.2s ease, transform 0.15s ease, box-shadow 0.2s ease;
    }

    .explore-btn:hover {
        background: #D4A86E;
        transform: translateY(-1px);
        box-shadow: 0 4px 14px rgba(181, 142, 98, 0.3);
    }

    .explore-btn:active {
        transform: translateY(0);
    }

    .horizon-right {
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
        background: #18181B;
        padding: 1.25rem;
        border-radius: 10px;
        border: 1px solid rgba(255, 255, 255, 0.06);
        z-index: 1;
    }

    .preview-ledger {
        display: flex;
        flex-direction: column;
        gap: 0.4rem;
    }


</style>
