<script lang="ts">
    import { homeStore, type RadioMixCard } from "$lib/stores/home.svelte";
    import { Broadcast, Play } from "phosphor-svelte";
</script>

<section class="radio-mix-section">
    <div class="section-title-row">
        <div class="title-group">
            <Broadcast size={20} weight="bold" class="radio-icon" />
            <h2>Algorithmic Radios</h2>
        </div>
        <span class="section-tag">Infinite Dynamic Mixes</span>
    </div>

    <div class="radio-carousel-track">
        {#each homeStore.radioMixes as card}
            <div 
                class="radio-card"
                style="--grad-start: {card.gradient_start}; --grad-end: {card.gradient_end};"
                role="button"
                tabindex="0"
                onclick={() => homeStore.playRadioMix(card)}
                onkeydown={(e) => { if (e.key === 'Enter') homeStore.playRadioMix(card); }}
            >
                <div class="radio-backdrop"></div>
                <div class="radio-card-content">
                    <div class="radio-badge">
                        <span>{card.category === 'artist' ? 'ARTIST MIX' : 'TEMPORAL MOOD'}</span>
                    </div>

                    <div class="radio-meta">
                        <h3 class="radio-title">{card.title}</h3>
                        <p class="radio-sub">{card.subtitle}</p>
                    </div>

                    <div class="radio-play-btn">
                        <Play size={20} weight="fill" />
                        <span>Play Radio</span>
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
        font-size: 1.35rem;
        font-weight: 700;
        margin: 0;
        letter-spacing: -0.02em;
    }

    :global(.radio-icon) {
        color: #06d6a0;
    }

    .section-tag {
        font-size: 0.72rem;
        font-weight: 600;
        text-transform: uppercase;
        letter-spacing: 0.05em;
        color: var(--text-muted, rgba(255, 255, 255, 0.5));
        background: var(--surface-1, rgba(255, 255, 255, 0.05));
        padding: 0.2rem 0.55rem;
        border-radius: 6px;
        border: 1px solid rgba(255, 255, 255, 0.05);
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
        flex: 0 0 240px;
        height: 200px;
        border-radius: 14px;
        position: relative;
        overflow: hidden;
        cursor: pointer;
        scroll-snap-align: start;
        border: 1px solid rgba(255, 255, 255, 0.1);
        transition: transform 0.25s cubic-bezier(0.16, 1, 0.3, 1), box-shadow 0.25s ease;
    }

    .radio-card:hover {
        transform: translateY(-4px);
        box-shadow: 0 12px 24px -6px rgba(0, 0, 0, 0.5);
    }

    .radio-backdrop {
        position: absolute;
        inset: 0;
        background: linear-gradient(145deg, var(--grad-start) 0%, var(--grad-end) 100%);
        opacity: 0.85;
        transition: opacity 0.2s ease;
    }

    .radio-card:hover .radio-backdrop {
        opacity: 1;
    }

    .radio-card-content {
        position: absolute;
        inset: 0;
        padding: 1.25rem;
        display: flex;
        flex-direction: column;
        justify-content: space-between;
        z-index: 1;
    }

    .radio-badge span {
        font-size: 0.68rem;
        font-weight: 700;
        letter-spacing: 0.08em;
        color: #fff;
        background: rgba(255, 255, 255, 0.15);
        backdrop-filter: blur(8px);
        padding: 0.2rem 0.5rem;
        border-radius: 20px;
        display: inline-block;
    }

    .radio-meta {
        display: flex;
        flex-direction: column;
        gap: 0.3rem;
    }

    .radio-title {
        font-size: 1.15rem;
        font-weight: 700;
        margin: 0;
        color: #fff;
        line-height: 1.25;
    }

    .radio-sub {
        font-size: 0.78rem;
        margin: 0;
        color: rgba(255, 255, 255, 0.7);
        line-height: 1.35;
        display: -webkit-box;
        line-clamp: 2;
        -webkit-line-clamp: 2;
        -webkit-box-orient: vertical;
        overflow: hidden;
    }

    .radio-play-btn {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        font-size: 0.82rem;
        font-weight: 600;
        color: #fff;
        background: rgba(255, 255, 255, 0.18);
        padding: 0.45rem 0.85rem;
        border-radius: 8px;
        backdrop-filter: blur(8px);
        width: fit-content;
        transition: background 0.2s ease, transform 0.2s ease;
    }

    .radio-card:hover .radio-play-btn {
        background: #fff;
        color: #000;
        transform: translateY(-1px);
    }
</style>
