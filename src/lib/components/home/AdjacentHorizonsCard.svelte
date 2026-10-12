<script lang="ts">
    import { homeStore } from "$lib/stores/home.svelte";
    import { exploreStore } from "$lib/stores/explore.svelte";
    import TrackRow from "$lib/components/TrackRow.svelte";
    import { audioStore } from "$lib/stores/audio.svelte";
    import { Compass, Play } from "phosphor-svelte";
    import CarouselControls from "$lib/components/common/CarouselControls.svelte";

    let payload = $derived(homeStore.adjacentHorizon);
    let totalHorizons = $derived(homeStore.adjacentHorizons.length);
    let currentIndex = $derived(homeStore.activeHorizonIndex);
</script>

{#if payload}
    <section class="adjacent-horizons-section">
        <!-- Section Header & Deck Stepper Controls -->
        <div class="section-title-row">
            <div class="title-group">
                <Compass size={20} weight="bold" class="horizon-header-icon" />
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

        <!-- Double-Bezel (Doppelrand) Stage Card -->
        <div class="horizon-bezel-outer">
            <div class="horizon-card-core">
                <div class="ambient-glow"></div>
                
                <!-- Vinyl Groove Watermark -->
                <div class="vinyl-watermark">
                    <svg viewBox="0 0 240 240" xmlns="http://www.w3.org/2000/svg">
                        <circle cx="120" cy="120" r="100" stroke="rgba(255,255,255,0.03)" stroke-width="1.5" fill="none" />
                        <circle cx="120" cy="120" r="75" stroke="rgba(255,255,255,0.035)" stroke-width="1.5" fill="none" />
                        <circle cx="120" cy="120" r="50" stroke="rgba(181,142,98,0.08)" stroke-width="2" fill="none" />
                    </svg>
                </div>

                <!-- Left Half: Editorial & Button-in-Button Action -->
                <div class="horizon-left">
                    <div class="horizon-eyebrow">
                        <Compass size={11} weight="bold" class="pill-icon" />
                        <span>ADJACENT HORIZON • MARKOV BRIDGE</span>
                    </div>

                    <h3 class="horizon-tagline">{payload.tagline}</h3>
                    <p class="horizon-desc">{payload.description}</p>

                    <div class="horizon-actions">
                        <button 
                            class="explore-btn group"
                            onclick={() => {
                                if (payload) {
                                    exploreStore.openHorizon(payload);
                                }
                            }}
                        >
                            <span>Explore Horizon</span>
                            <div class="btn-icon-bubble">
                                <Play size={13} weight="fill" />
                            </div>
                        </button>
                    </div>
                </div>

                <!-- Right Half: Sound Preview Ledger -->
                {#if payload.preview_tracks && payload.preview_tracks.length > 0}
                    <div class="horizon-right">
                        <div class="preview-ledger-header">
                            <span class="ledger-label">PREVIEW SOUNDSCAPE</span>
                        </div>
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
        </div>
    </section>
{/if}

<style>
    .adjacent-horizons-section {
        display: flex;
        flex-direction: column;
        gap: 1.25rem;
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
        color: #b58e62;
    }

    .section-title-row h2 {
        font-family: var(--echo-font-heading, sans-serif);
        font-size: 1.35rem;
        font-weight: 600;
        margin: 0;
        letter-spacing: -0.02em;
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
        background: var(--echo-primary-dark);
    }

    /* ─── Double-Bezel (Doppelrand) Architecture ─── */
    .horizon-bezel-outer {
        position: relative;
        padding: 6px;
        border-radius: 20px;
        background: rgba(255, 255, 255, 0.025);
        border: 1px solid rgba(255, 255, 255, 0.07);
        box-shadow: 0 16px 40px -12px rgba(0, 0, 0, 0.7);
        transition: border-color 0.25s ease;
    }

    .horizon-bezel-outer:hover {
        border-color: rgba(181, 142, 98, 0.25);
    }

    .horizon-card-core {
        position: relative;
        background: var(--echo-surface-card);
        border-radius: 14px;
        border: 1px solid rgba(255, 255, 255, 0.06);
        padding: 2.25rem;
        display: grid;
        grid-template-columns: minmax(0, 1.15fr) minmax(0, 0.85fr);
        gap: 2.5rem;
        align-items: center;
        overflow: hidden;
        box-shadow: inset 0 1px 0 0 rgba(255, 255, 255, 0.1);
    }

    @media (max-width: 1024px) {
        .horizon-card-core {
            grid-template-columns: minmax(0, 1fr);
            gap: 1.75rem;
            padding: 1.5rem;
        }
    }

    .ambient-glow {
        position: absolute;
        top: -40%;
        left: -15%;
        width: 65%;
        height: 180%;
        background: radial-gradient(circle, rgba(181, 142, 98, 0.1) 0%, transparent 65%);
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
        opacity: 0.55;
    }

    .horizon-left {
        display: flex;
        flex-direction: column;
        align-items: flex-start;
        gap: 0.9rem;
        z-index: 1;
    }

    .horizon-eyebrow {
        display: inline-flex;
        align-items: center;
        gap: 0.45rem;
        background: rgba(181, 142, 98, 0.1);
        border: 1px solid rgba(181, 142, 98, 0.22);
        padding: 0.22rem 0.65rem;
        border-radius: 9999px;
        width: fit-content;
    }

    :global(.pill-icon) {
        color: var(--echo-primary);
    }

    .horizon-eyebrow span {
        font-family: var(--lyria-font-mono, monospace);
        font-size: 0.62rem;
        font-weight: 700;
        letter-spacing: 0.14em;
        color: var(--echo-primary);
    }

    .horizon-tagline {
        font-family: var(--echo-font-heading, sans-serif);
        font-size: 1.55rem;
        font-weight: 600;
        line-height: 1.25;
        margin: 0;
        color: #fff;
        letter-spacing: -0.025em;
    }

    .horizon-desc {
        font-size: 0.88rem;
        line-height: 1.55;
        color: var(--echo-text-2, rgba(255, 255, 255, 0.65));
        margin: 0;
        max-width: 95%;
    }

    .horizon-actions {
        margin-top: 0.5rem;
    }

    /* ─── Button-in-Button Architecture ─── */
    .explore-btn {
        display: inline-flex;
        align-items: center;
        gap: 0.75rem;
        background: var(--echo-primary-dark);
        color: #0e0e10;
        border: none;
        padding: 0.45rem 0.55rem 0.45rem 1.25rem;
        border-radius: 9999px;
        font-size: 0.84rem;
        font-weight: 600;
        letter-spacing: 0.01em;
        cursor: pointer;
        box-shadow: 0 4px 14px rgba(181, 142, 98, 0.25);
        transition: background 0.2s ease, transform 0.15s cubic-bezier(0.16, 1, 0.3, 1), box-shadow 0.2s ease;
    }

    .explore-btn:hover {
        background: var(--echo-primary);
        transform: translateY(-1px);
        box-shadow: 0 6px 20px rgba(181, 142, 98, 0.35);
    }

    .explore-btn:active {
        transform: scale(0.98);
    }

    .btn-icon-bubble {
        width: 28px;
        height: 28px;
        border-radius: 50%;
        background: rgba(0, 0, 0, 0.22);
        color: #0e0e10;
        display: flex;
        align-items: center;
        justify-content: center;
        transition: transform 0.2s cubic-bezier(0.16, 1, 0.3, 1), background 0.2s ease;
    }

    .explore-btn:hover .btn-icon-bubble {
        transform: scale(1.08) translateX(1px);
        background: rgba(0, 0, 0, 0.32);
    }

    .horizon-right {
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
        background: rgba(14, 14, 18, 0.75);
        padding: 1.25rem;
        border-radius: 12px;
        border: 1px solid rgba(255, 255, 255, 0.06);
        z-index: 1;
        box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.05);
    }

    .preview-ledger-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding-bottom: 0.35rem;
        border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    }

    .ledger-label {
        font-family: var(--lyria-font-mono, monospace);
        font-size: 0.65rem;
        font-weight: 600;
        letter-spacing: 0.12em;
        color: var(--echo-text-3, rgba(255, 255, 255, 0.45));
    }

    .preview-ledger {
        display: flex;
        flex-direction: column;
        gap: 0.4rem;
    }
</style>
