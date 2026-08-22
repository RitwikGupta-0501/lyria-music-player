<script lang="ts">
    import { homeStore, type IncompleteSessionItem } from "$lib/stores/home.svelte";
    import { ArrowCounterClockwise, Play } from "phosphor-svelte";
</script>

{#if homeStore.jumpBackIn.length > 0}
    <section class="jump-back-in-section">
        <div class="section-title-row">
            <div class="title-group">
                <ArrowCounterClockwise size={20} weight="bold" class="jump-icon" />
                <h2>Jump Back In</h2>
            </div>
            <span class="section-tag">Resume Playback</span>
        </div>

        <div class="session-carousel-track">
            {#each homeStore.jumpBackIn as session}
                <div 
                    class="session-card"
                    role="button"
                    tabindex="0"
                    onclick={() => homeStore.resumeSession(session)}
                    onkeydown={(e) => { if (e.key === 'Enter') homeStore.resumeSession(session); }}
                >
                    <div class="session-art-wrapper">
                        {#if session.cover_art_url}
                            <img src={session.cover_art_url.startsWith('/') ? `asset://localhost/${encodeURIComponent(session.cover_art_url)}` : session.cover_art_url} alt={session.title} loading="lazy" />
                        {:else}
                            <div class="placeholder-art"></div>
                        {/if}
                        <div class="session-overlay">
                            <div class="play-bubble">
                                <Play size={20} weight="fill" />
                            </div>
                        </div>

                        <!-- Progress Bar Overlay at bottom of cover -->
                        <div class="progress-track">
                            <div class="progress-fill" style="width: {Math.round(session.progress_percent * 100)}%;"></div>
                        </div>
                    </div>

                    <div class="session-info">
                        <span class="session-title">{session.title}</span>
                        <span class="session-subtitle">{session.subtitle}</span>
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
        font-size: 1.35rem;
        font-weight: 700;
        margin: 0;
        letter-spacing: -0.02em;
    }

    :global(.jump-icon) {
        color: #ff9f1c;
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

    .session-carousel-track {
        display: flex;
        gap: 1.25rem;
        overflow-x: auto;
        scroll-snap-type: x mandatory;
        padding-bottom: 0.5rem;
        scrollbar-width: none;
    }

    .session-carousel-track::-webkit-scrollbar {
        display: none;
    }

    .session-card {
        flex: 0 0 160px;
        scroll-snap-align: start;
        display: flex;
        flex-direction: column;
        gap: 0.55rem;
        cursor: pointer;
        transition: transform 0.2s cubic-bezier(0.16, 1, 0.3, 1);
    }

    .session-card:hover {
        transform: translateY(-4px);
    }

    .session-art-wrapper {
        width: 160px;
        height: 160px;
        border-radius: 10px;
        overflow: hidden;
        position: relative;
        background: var(--surface-1, rgba(255, 255, 255, 0.04));
        border: 1px solid rgba(255, 255, 255, 0.06);
    }

    .session-art-wrapper img {
        width: 100%;
        height: 100%;
        object-fit: cover;
    }

    .placeholder-art {
        width: 100%;
        height: 100%;
        background: linear-gradient(135deg, rgba(255, 255, 255, 0.08), rgba(255, 255, 255, 0.02));
    }

    .session-overlay {
        position: absolute;
        inset: 0;
        background: rgba(0, 0, 0, 0.4);
        display: flex;
        align-items: center;
        justify-content: center;
        opacity: 0;
        transition: opacity 0.2s ease;
    }

    .session-card:hover .session-overlay {
        opacity: 1;
    }

    .play-bubble {
        width: 40px;
        height: 40px;
        border-radius: 50%;
        background: #fff;
        color: #000;
        display: flex;
        align-items: center;
        justify-content: center;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.35);
        transform: scale(0.9);
        transition: transform 0.2s ease;
    }

    .session-card:hover .play-bubble {
        transform: scale(1);
    }

    .progress-track {
        position: absolute;
        bottom: 0;
        left: 0;
        right: 0;
        height: 4px;
        background: rgba(0, 0, 0, 0.5);
    }

    .progress-fill {
        height: 100%;
        background: #ff9f1c;
        border-radius: 0 2px 2px 0;
    }

    .session-info {
        display: flex;
        flex-direction: column;
        gap: 0.15rem;
    }

    .session-title {
        font-size: 0.9rem;
        font-weight: 600;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .session-subtitle {
        font-size: 0.78rem;
        color: var(--text-muted, rgba(255, 255, 255, 0.6));
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .session-stat {
        font-size: 0.7rem;
        color: #ff9f1c;
        font-weight: 500;
    }
</style>
