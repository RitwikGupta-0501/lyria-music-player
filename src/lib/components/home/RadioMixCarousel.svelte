<script lang="ts">
    import { homeStore, type RadioMixCard } from "$lib/stores/home.svelte";
    import { exploreStore } from "$lib/stores/explore.svelte";
    import { Broadcast, Play, Waves, CaretLeft, CaretRight } from "phosphor-svelte";

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
        if (trackContainer && homeStore.radioMixes.length > 0) {
            updateScrollState();
        }
    });

    function scrollPrev() {
        if (!trackContainer) return;
        const cardSpan = 270 + 20; // 270px card width + 20px gap
        const pageStep = Math.max(cardSpan, trackContainer.clientWidth - cardSpan);
        trackContainer.scrollBy({ left: -pageStep, behavior: "smooth" });
    }

    function scrollNext() {
        if (!trackContainer) return;
        const cardSpan = 270 + 20;
        const pageStep = Math.max(cardSpan, trackContainer.clientWidth - cardSpan);
        trackContainer.scrollBy({ left: pageStep, behavior: "smooth" });
    }
</script>

<svelte:window onresize={updateScrollState} />

<section class="radio-mix-section">
    <div class="section-title-row">
        <div class="title-group">
            <Broadcast size={20} weight="bold" class="radio-icon" />
            <h2>Radios</h2>
        </div>

        {#if hasOverflow}
            <div class="chevron-controls">
                <button 
                    class="chevron-btn" 
                    onclick={scrollPrev} 
                    disabled={!canScrollLeft}
                    title="Scroll Left"
                    aria-label="Previous radios"
                >
                    <CaretLeft size={16} weight="bold" />
                </button>
                <button 
                    class="chevron-btn" 
                    onclick={scrollNext} 
                    disabled={!canScrollRight}
                    title="Scroll Right"
                    aria-label="Next radios"
                >
                    <CaretRight size={16} weight="bold" />
                </button>
            </div>
        {/if}
    </div>

    <div 
        class="radio-carousel-track" 
        bind:this={trackContainer} 
        onscroll={updateScrollState}
    >
        {#each homeStore.radioMixes as card}
            <div 
                class="radio-card"
                role="button"
                tabindex="0"
                style="--card-grad-start: {card.gradient_start}; --card-grad-end: {card.gradient_end};"
                onclick={() => exploreStore.openRadioMix(card)}
                onkeydown={(e) => { if (e.key === "Enter") exploreStore.openRadioMix(card); }}
            >
                <div class="radio-backdrop"></div>
                
                <!-- Tactile Vinyl Groove & Waveform Watermark -->
                <div class="radio-wave-watermark">
                    <svg viewBox="0 0 200 200" xmlns="http://www.w3.org/2000/svg" class="vinyl-rings">
                        <circle cx="100" cy="100" r="80" stroke="rgba(255,255,255,0.04)" stroke-width="1.5" fill="none" />
                        <circle cx="100" cy="100" r="60" stroke="rgba(255,255,255,0.04)" stroke-width="1.5" fill="none" />
                        <circle cx="100" cy="100" r="40" stroke="rgba(255,255,255,0.045)" stroke-width="1.5" fill="none" />
                        <circle cx="100" cy="100" r="20" stroke="rgba(181,142,98,0.09)" stroke-width="2" fill="none" />
                    </svg>
                </div>
                <div class="radio-ambient-highlight"></div>

                <div class="radio-card-content">
                    <div class="radio-top-bar">
                        <div class="radio-station-pill">
                            <Waves size={13} weight="bold" color="rgba(212, 168, 110, 0.9)" />
                            <span>RADIO</span>
                        </div>
                    </div>

                    <div class="radio-meta">
                        <h3 class="radio-title" title={card.title}>{card.title}</h3>
                        <p class="radio-sub" title={card.subtitle}>{card.subtitle}</p>
                    </div>

                    <div class="radio-bottom-bar">
                        <button 
                            type="button" 
                            class="radio-play-btn" 
                            onclick={(e) => { e.stopPropagation(); homeStore.playRadioMix(card); }}
                        >
                            <Play size={13} weight="fill" />
                            <span>Play Radio</span>
                        </button>
                    </div>
                </div>
            </div>
        {/each}
    </div>
</section>

<style>
    .radio-mix-section {
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

    :global(.radio-icon) {
        color: #B58E62;
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
        fill: currentColor;
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

    .radio-carousel-track {
        display: flex;
        gap: 1.25rem;
        overflow-x: auto;
        scroll-snap-type: x mandatory;
        scroll-behavior: smooth;
        padding-top: 8px;
        margin-top: -8px;
        padding-bottom: 0.75rem;
        margin-bottom: -0.25rem;
        scrollbar-width: none;
        will-change: scroll-position;
    }

    .radio-carousel-track::-webkit-scrollbar {
        display: none;
    }

    .radio-card {
        flex: 0 0 270px;
        height: 215px;
        border-radius: 14px;
        position: relative;
        overflow: hidden;
        cursor: pointer;
        scroll-snap-align: start;
        background: #141416;
        border: 1px solid rgba(255, 255, 255, 0.08);
        transition: transform 0.25s cubic-bezier(0.16, 1, 0.3, 1), border-color 0.25s ease, box-shadow 0.25s ease;
    }

    .radio-card:hover {
        transform: translateY(-3px);
        border-color: rgba(181, 142, 98, 0.45);
        box-shadow: 0 12px 30px -6px rgba(0, 0, 0, 0.65);
    }

    .radio-backdrop {
        position: absolute;
        inset: 0;
        background: 
            radial-gradient(circle at 10% 0%, var(--card-grad-start, #2A1E5C) 0%, transparent 60%),
            linear-gradient(180deg, #18181C 0%, #101012 100%);
        opacity: 0.85;
        transition: opacity 0.3s ease;
    }

    .radio-card:hover .radio-backdrop {
        opacity: 1;
    }

    .radio-wave-watermark {
        position: absolute;
        right: -25px;
        bottom: -25px;
        width: 170px;
        height: 170px;
        opacity: 0.6;
        pointer-events: none;
        transition: transform 0.4s ease, opacity 0.3s ease;
    }

    .radio-card:hover .radio-wave-watermark {
        transform: scale(1.08) rotate(15deg);
        opacity: 0.9;
    }

    .radio-ambient-highlight {
        position: absolute;
        top: 0;
        left: 0;
        right: 0;
        height: 40%;
        background: linear-gradient(180deg, rgba(255, 255, 255, 0.06) 0%, transparent 100%);
        pointer-events: none;
    }

    .radio-card-content {
        position: relative;
        z-index: 2;
        padding: 1.25rem;
        height: 100%;
        box-sizing: border-box;
        display: flex;
        flex-direction: column;
        justify-content: space-between;
    }

    .radio-top-bar {
        display: flex;
        align-items: center;
        justify-content: space-between;
    }

    .radio-station-pill {
        display: inline-flex;
        align-items: center;
        gap: 0.4rem;
        background: rgba(181, 142, 98, 0.12);
        border: 1px solid rgba(181, 142, 98, 0.25);
        padding: 0.18rem 0.5rem;
        border-radius: 4px;
    }

    .radio-station-pill span {
        font-family: var(--echo-font-mono, monospace);
        font-size: 0.58rem;
        font-weight: 700;
        letter-spacing: 0.1em;
        color: #D4A86E;
    }

    .radio-meta {
        display: flex;
        flex-direction: column;
        gap: 0.3rem;
        margin-top: auto;
        margin-bottom: 0.75rem;
    }

    .radio-title {
        font-family: var(--echo-font-heading, "Playfair Display", serif);
        font-size: 1.15rem;
        font-weight: 600;
        margin: 0;
        color: #fff;
        line-height: 1.25;
        display: -webkit-box;
        -webkit-line-clamp: 2;
        line-clamp: 2;
        -webkit-box-orient: vertical;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .radio-sub {
        font-size: 0.76rem;
        margin: 0;
        color: rgba(255, 255, 255, 0.55);
        line-height: 1.35;
        display: -webkit-box;
        -webkit-line-clamp: 2;
        line-clamp: 2;
        -webkit-box-orient: vertical;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .radio-bottom-bar {
        display: flex;
        align-items: center;
        justify-content: flex-start;
    }

    .radio-play-btn {
        display: inline-flex;
        align-items: center;
        gap: 0.45rem;
        font-size: 0.75rem;
        font-weight: 600;
        color: #D4A86E;
        background: rgba(181, 142, 98, 0.14);
        border: 1px solid rgba(181, 142, 98, 0.3);
        padding: 0.35rem 0.75rem;
        border-radius: 6px;
        width: fit-content;
        transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
        cursor: pointer;
    }

    .radio-card:hover .radio-play-btn {
        background: #B58E62;
        color: #0E0E10;
        border-color: #B58E62;
        transform: translateY(-1px);
    }
</style>
