<script lang="ts">
    import { homeStore, type IncompleteSessionItem } from "$lib/stores/home.svelte";
    import CarouselControls from "$lib/components/common/CarouselControls.svelte";
    import CardPlayButton from "$lib/components/common/CardPlayButton.svelte";
    import { ArrowCounterClockwise, MusicNotes } from "phosphor-svelte";
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
        if (trackContainer && homeStore.jumpBackIn.length > 0) {
            updateScrollState();
        }
    });

    function scrollPrev() {
        if (!trackContainer) return;
        const cardSpan = 176 + 20;
        const pageStep = Math.max(
            cardSpan,
            trackContainer.clientWidth - cardSpan,
        );
        trackContainer.scrollBy({ left: -pageStep, behavior: "smooth" });
    }

    function scrollNext() {
        if (!trackContainer) return;
        const cardSpan = 176 + 20;
        const pageStep = Math.max(
            cardSpan,
            trackContainer.clientWidth - cardSpan,
        );
        trackContainer.scrollBy({ left: pageStep, behavior: "smooth" });
    }
</script>

<svelte:window onresize={updateScrollState} />

{#if homeStore.jumpBackIn.length > 0}
    <section class="jump-back-in-section">
        <div class="section-title-row">
            <div class="title-group">
                <ArrowCounterClockwise size={20} weight="bold" class="jump-icon" />
                <h2>Jump Back In</h2>
            </div>

            <div class="header-right-controls">
                <CarouselControls
                    container={trackContainer}
                    canPrev={canScrollLeft}
                    canNext={canScrollRight}
                    onPrev={scrollPrev}
                    onNext={scrollNext}
                    prevLabel="Previous sessions"
                    nextLabel="Next sessions"
                />
            </div>
        </div>

        <div
            class="session-carousel-track"
            bind:this={trackContainer}
            onscroll={updateScrollState}
        >
            {#each homeStore.jumpBackIn as session}
                <div 
                    class="session-card"
                    role="button"
                    tabindex="0"
                    onclick={() => homeStore.resumeSession(session)}
                    onkeydown={(e) => { if (e.key === 'Enter') homeStore.resumeSession(session); }}
                >
                    <div class="session-art-wrapper">
                        {#if resolveCoverArt(session.cover_art_url)}
                            <img src={resolveCoverArt(session.cover_art_url)} alt={session.title} loading="lazy" />
                        {:else}
                            <div class="placeholder-art">
                                <MusicNotes size={48} weight="thin" color="rgba(255, 255, 255, 0.35)" />
                            </div>
                        {/if}
                        <div class="session-overlay">
                            <CardPlayButton 
                                onclick={(e) => { e.stopPropagation(); homeStore.resumeSession(session); }}
                                title="Resume session"
                            />
                        </div>

                        <!-- Progress Bar Overlay at bottom of cover -->
                        <div class="progress-track">
                            <div class="progress-fill" style="width: {Math.round(session.progress_percent * 100)}%;"></div>
                        </div>
                    </div>

                    <div class="session-info">
                        <span class="session-title" title={session.title}>{session.title}</span>
                        <span class="session-subtitle" title={session.subtitle}>{session.subtitle}</span>
                        <span class="session-stat">Track {session.current_track_index + 1} of {session.total_tracks} • {Math.round(session.progress_percent * 100)}%</span>
                    </div>
                </div>
            {/each}
        </div>
    </section>
{/if}

<style>
    .jump-back-in-section {
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
        font-family: var(--echo-font-heading, sans-serif);
        font-size: 1.35rem;
        font-weight: 600;
        margin: 0;
        letter-spacing: -0.02em;
        color: #fff;
    }

    :global(.jump-icon) {
        color: #b58e62;
    }

    .header-right-controls {
        display: flex;
        align-items: center;
        gap: 0.75rem;
    }

    .session-carousel-track {
        display: flex;
        align-items: flex-start;
        gap: 1.25rem;
        overflow-x: auto;
        scroll-snap-type: x mandatory;
        scroll-behavior: smooth;
        padding-bottom: 0.5rem;
        scrollbar-width: none;
        will-change: scroll-position;
        min-width: 0;
        max-width: 100%;
    }

    .session-carousel-track::-webkit-scrollbar {
        display: none;
    }

    .session-card {
        flex: 0 0 176px;
        width: 176px;
        min-width: 176px;
        scroll-snap-align: start;
        display: flex;
        flex-direction: column;
        gap: 0.6rem;
        cursor: pointer;
        text-align: left;
    }

    .session-art-wrapper {
        width: 176px;
        height: 176px;
        border-radius: 12px;
        overflow: hidden;
        position: relative;
        background: var(--echo-surface-pill);
        border: 1px solid var(--echo-border-glass);
        transition: border-color 0.2s ease, box-shadow 0.2s ease;
    }

    .session-card:hover .session-art-wrapper {
        border-color: rgba(181, 142, 98, 0.35);
        box-shadow: 0 10px 24px -6px rgba(0, 0, 0, 0.6);
    }

    .session-art-wrapper img {
        width: 100%;
        height: 100%;
        object-fit: cover;
        display: block;
        transition: transform 0.55s cubic-bezier(0.05, 0.75, 0.15, 1);
    }

    .session-card:hover .session-art-wrapper img {
        transform: scale(1.05);
    }

    .placeholder-art {
        width: 100%;
        height: 100%;
        display: flex;
        align-items: center;
        justify-content: center;
        background: radial-gradient(circle at center, #18181b 0%, #09090b 100%);
    }

    .session-overlay {
        position: absolute;
        inset: 0;
        background: rgba(0, 0, 0, 0.35);
        backdrop-filter: blur(2px);
        display: flex;
        align-items: center;
        justify-content: center;
        opacity: 0;
        pointer-events: none;
        transition: opacity 0.2s cubic-bezier(0.16, 1, 0.3, 1);
    }

    .session-card:hover .session-overlay {
        opacity: 1;
        pointer-events: auto;
    }

    .progress-track {
        position: absolute;
        bottom: 0;
        left: 0;
        right: 0;
        height: 4px;
        background: rgba(0, 0, 0, 0.65);
        z-index: 5;
    }

    .progress-fill {
        height: 100%;
        background: var(--echo-primary, #e2a973);
        border-radius: 0 2px 2px 0;
    }

    .session-info {
        display: flex;
        flex-direction: column;
        gap: 0.2rem;
        width: 100%;
        min-width: 0;
    }

    .session-title {
        font-family: var(--echo-font-body, system-ui, sans-serif);
        font-size: 0.88rem;
        font-weight: 600;
        color: #fff;
        display: block;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .session-subtitle {
        font-family: var(--echo-font-body, system-ui, sans-serif);
        font-size: 0.78rem;
        color: var(--echo-text-2, rgba(255, 255, 255, 0.55));
        display: block;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .session-stat {
        font-family: var(--echo-font-mono, monospace);
        font-size: 0.7rem;
        color: var(--echo-primary, #e2a973);
        font-weight: 500;
        letter-spacing: -0.01em;
    }
</style>
