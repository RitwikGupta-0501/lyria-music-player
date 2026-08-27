<script lang="ts">
    import { homeStore, type RadioMixCard } from "$lib/stores/home.svelte";
    import { exploreStore } from "$lib/stores/explore.svelte";
    import { Broadcast, Play, Waves } from "phosphor-svelte";
</script>

<section class="radio-mix-section">
    <div class="section-title-row">
        <div class="title-group">
            <Broadcast size={20} weight="bold" class="radio-icon" />
            <h2>Algorithmic Radios</h2>
        </div>
        
    </div>

    <div class="radio-carousel-track">
        {#each homeStore.radioMixes as card}
            <div 
                class="radio-card"
                role="button"
                tabindex="0"
                onclick={() => exploreStore.openRadioMix(card)}
                onkeydown={(e) => { if (e.key === "Enter") exploreStore.openRadioMix(card); }}
            >
                <div class="radio-backdrop"></div>
                <!-- Tactile Vinyl Groove & Waveform Watermark -->
                <div class="radio-wave-watermark">
                    <svg viewBox="0 0 200 200" xmlns="http://www.w3.org/2000/svg" class="vinyl-rings">
                        <circle cx="100" cy="100" r="80" stroke="rgba(255,255,255,0.03)" stroke-width="1.5" fill="none" />
                        <circle cx="100" cy="100" r="60" stroke="rgba(255,255,255,0.03)" stroke-width="1.5" fill="none" />
                        <circle cx="100" cy="100" r="40" stroke="rgba(255,255,255,0.035)" stroke-width="1.5" fill="none" />
                        <circle cx="100" cy="100" r="20" stroke="rgba(181,142,98,0.06)" stroke-width="2" fill="none" />
                    </svg>
                </div>
                <div class="radio-ambient-highlight"></div>

                <div class="radio-card-content">
                    <div class="radio-top-bar">
                        <div class="radio-badge">
                            <span>{card.category === "artist" ? "ARTIST MIX" : "TEMPORAL MOOD"}</span>
                        </div>
                        <div class="radio-groove-indicator">
                            <Waves size={16} weight="bold" color="rgba(181,142,98,0.6)" />
                        </div>
                    </div>

                    <div class="radio-meta">
                        <h3 class="radio-title">{card.title}</h3>
                        <p class="radio-sub">{card.subtitle}</p>
                    </div>

                    <div class="radio-bottom-bar">
                        <button 
                            type="button" 
                            class="radio-play-btn" 
                            onclick={(e) => { e.stopPropagation(); homeStore.playRadioMix(card); }}
                        >
                            <Play size={14} weight="fill" />
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

    

    .radio-carousel-track {
        display: flex;
        gap: 1.25rem;
        overflow-x: auto;
        scroll-snap-type: x mandatory;
        padding-bottom: 0.5rem;
        scrollbar-width: none;
    }

    .radio-carousel-track::-webkit-scrollbar {
        display: none;
    }

    .radio-card {
        flex: 0 0 260px;
        height: 200px;
        border-radius: 12px;
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
        border-color: rgba(181, 142, 98, 0.4);
        box-shadow: 0 12px 28px -6px rgba(0, 0, 0, 0.6);
    }

    .radio-backdrop {
        position: absolute;
        inset: 0;
        background: linear-gradient(180deg, #18181C 0%, #121214 100%);
    }

    .radio-wave-watermark {
        position: absolute;
        right: -30px;
        bottom: -30px;
        width: 170px;
        height: 170px;
        opacity: 0.7;
        pointer-events: none;
        transition: transform 0.4s ease, opacity 0.3s ease;
    }

    .radio-card:hover .radio-wave-watermark {
        transform: scale(1.08) rotate(15deg);
        opacity: 1;
    }

    .vinyl-rings {
        width: 100%;
        height: 100%;
    }

    .radio-ambient-highlight {
        position: absolute;
        inset: 0;
        background: radial-gradient(circle at 15% 15%, rgba(181, 142, 98, 0.08) 0%, transparent 65%);
        pointer-events: none;
    }

    .radio-card-content {
        position: absolute;
        inset: 0;
        padding: 1.35rem;
        display: flex;
        flex-direction: column;
        justify-content: space-between;
        z-index: 1;
    }

    .radio-top-bar {
        display: flex;
        align-items: center;
        justify-content: space-between;
    }

    .radio-badge span {
        font-family: var(--echo-font-mono, monospace);
        font-size: 0.62rem;
        font-weight: 700;
        letter-spacing: 0.09em;
        color: #D4A86E;
        background: rgba(181, 142, 98, 0.12);
        border: 1px solid rgba(181, 142, 98, 0.22);
        padding: 0.2rem 0.5rem;
        border-radius: 4px;
        display: inline-block;
    }

    .radio-meta {
        display: flex;
        flex-direction: column;
        gap: 0.35rem;
    }

    .radio-title {
        font-family: var(--echo-font-heading, "Playfair Display", serif);
        font-size: 1.25rem;
        font-weight: 600;
        margin: 0;
        color: #fff;
        line-height: 1.25;
    }

    .radio-sub {
        font-size: 0.78rem;
        margin: 0;
        color: rgba(255, 255, 255, 0.6);
        line-height: 1.4;
        display: -webkit-box;
        line-clamp: 2;
        -webkit-line-clamp: 2;
        -webkit-box-orient: vertical;
        overflow: hidden;
    }

    .radio-play-btn {
        display: flex;
        align-items: center;
        gap: 0.45rem;
        font-size: 0.76rem;
        font-weight: 600;
        color: #D4A86E;
        background: rgba(181, 142, 98, 0.12);
        border: 1px solid rgba(181, 142, 98, 0.25);
        padding: 0.38rem 0.75rem;
        border-radius: 6px;
        width: fit-content;
        transition: all 0.2s ease;
    }

    .radio-card:hover .radio-play-btn {
        background: #B58E62;
        color: #0E0E10;
        border-color: #B58E62;
        transform: translateY(-1px);
    }
</style>
