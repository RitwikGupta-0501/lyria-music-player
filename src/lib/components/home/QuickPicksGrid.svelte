<script lang="ts">
    import { homeStore, type FederatedTrack } from "$lib/stores/home.svelte";
    import { audioStore } from "$lib/stores/audio.svelte";
    import { Play, Heart, Lightning, MusicNotes } from "phosphor-svelte";
    import { resolveCoverArt } from "$lib/utils/media";
    import EqualizerWave from "$lib/components/common/EqualizerWave.svelte";
    import { isCurrentTrack, formatDuration } from "$lib/utils/format";

    let { limit = 9 } = $props<{ limit?: number }>();

    function isPlaying(song: FederatedTrack): boolean {
        return isCurrentTrack(song, audioStore.currentQueueTrack) && audioStore.playbackState === "Playing";
    }
</script>

<section class="quick-picks-section">
    <div class="section-title-row">
        <div class="title-group">
            <Lightning size={20} weight="fill" class="quick-icon" />
            <h2>Quick Picks</h2>
        </div>
    </div>

    <div class="quick-picks-grid">
        {#if !homeStore.phase1Loaded && homeStore.quickPicks.length === 0}
            {#each Array(limit) as _}
                <div class="quick-pick-pill skeleton-pill-card">
                    <div class="pill-art skeleton-box"></div>
                    <div class="pill-info">
                        <div class="skeleton-line skeleton-box title-placeholder"></div>
                        <div class="skeleton-line skeleton-box artist-placeholder"></div>
                    </div>
                    <div class="pill-trailing">
                        <div class="skeleton-circle skeleton-box like-placeholder"></div>
                    </div>
                </div>
            {/each}
        {:else}
            {#each homeStore.quickPicks.slice(0, limit) as song}
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div 
                class="quick-pick-pill"
                class:active-track={isCurrentTrack(song, audioStore.currentQueueTrack)}
                class:is-playing={isPlaying(song)}
                onclick={() => homeStore.playFederatedTrack(song)}
            >
                <div class="pill-art">
                    {#if resolveCoverArt(song.cover_art_url)}
                        <img 
                            src={resolveCoverArt(song.cover_art_url)} 
                            alt={song.title} 
                            loading="lazy"
                        />
                    {:else}
                        <div class="placeholder-art">
                            <MusicNotes size={20} weight="thin" color="rgba(255, 255, 255, 0.35)" />
                        </div>
                    {/if}

                    <div class="pill-play-overlay">
                        {#if isPlaying(song)}
                            <EqualizerWave />
                        {:else}
                            <Play size={15} weight="fill" class="play-icon-glyph" />
                        {/if}
                    </div>
                </div>

                <div class="pill-info">
                    <span class="pill-title" title={song.title}>{song.title}</span>
                    <div class="pill-sub-row">
                        <span class="pill-artist" title={song.artist}>{song.artist}</span>
                        {#if song.duration_ms && song.duration_ms > 0}
                            <span class="sub-sep">•</span>
                            <span class="pill-duration">{formatDuration(song.duration_ms)}</span>
                        {/if}
                    </div>
                </div>

                <div class="pill-trailing">
                    <button 
                        type="button"
                        class="pill-like-btn" 
                        class:liked={song.liked}
                        onclick={(e) => { e.stopPropagation(); homeStore.toggleLike(song); }}
                        title={song.liked ? "Liked" : "Like track"}
                        aria-label={song.liked ? "Unlike track" : "Like track"}
                    >
                        <Heart size={15} weight={song.liked ? "fill" : "regular"} />
                    </button>
                </div>
            </div>
        {/each}
        {/if}
    </div>
</section>

<style>
    .quick-picks-section {
        display: flex;
        flex-direction: column;
        gap: 1.25rem;
        width: 100%;
        min-width: 0;
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

    :global(.quick-icon) {
        color: var(--echo-primary-dark, #b58e62);
    }

    .section-title-row h2 {
        font-family: var(--echo-font-heading, sans-serif);
        font-size: 1.35rem;
        font-weight: 600;
        margin: 0;
        letter-spacing: -0.02em;
        color: #fff;
    }

    .quick-picks-grid {
        display: grid;
        grid-template-columns: repeat(3, minmax(0, 1fr));
        gap: 0.65rem;
        width: 100%;
    }

    @media (max-width: 900px) {
        .quick-picks-grid {
            grid-template-columns: repeat(auto-fill, minmax(190px, 1fr));
        }
    }

    /* ─── Studio Channel Strip Pill ─── */
    .quick-pick-pill {
        display: flex;
        align-items: center;
        gap: 0.65rem;
        background: var(--echo-surface-pill, #141416);
        border: 1px solid var(--echo-border-glass);
        border-radius: 10px;
        padding: 0.45rem 0.65rem 0.45rem 0.5rem;
        cursor: pointer;
        position: relative;
        overflow: hidden;
        min-height: 58px;
        box-sizing: border-box;
        transform: translateZ(0);
        backface-visibility: hidden;
        transition: transform 0.2s cubic-bezier(0.16, 1, 0.3, 1), background-color 0.2s ease, border-color 0.2s ease, box-shadow 0.2s ease;
    }

    .quick-pick-pill:hover {
        transform: translateY(-2px);
        background: var(--echo-surface-pill-hover, #1c1c22);
        border-color: rgba(181, 142, 98, 0.35);
        box-shadow: 0 6px 18px -4px rgba(0, 0, 0, 0.55), inset 0 1px 0 rgba(255, 255, 255, 0.06);
    }

    /* Active Track Channel State */
    .quick-pick-pill.active-track {
        background: linear-gradient(90deg, rgba(181, 142, 98, 0.12) 0%, rgba(20, 20, 24, 0.85) 100%);
        border-color: rgba(181, 142, 98, 0.42);
        box-shadow: inset 0 1px 0 rgba(255, 235, 205, 0.12), 0 4px 14px rgba(0, 0, 0, 0.4);
    }

    .quick-pick-pill.active-track::before {
        content: "";
        position: absolute;
        left: 0;
        top: 0;
        bottom: 0;
        width: 3.5px;
        background: var(--echo-primary, #e2a973);
        box-shadow: 0 0 10px rgba(226, 169, 115, 0.8);
        border-radius: 4px 0 0 4px;
    }

    /* Precision Specular Art Wrapper */
    .pill-art {
        width: 46px;
        height: 46px;
        border-radius: 8px;
        overflow: hidden;
        position: relative;
        flex-shrink: 0;
        background: #18181b;
        border: 1px solid var(--echo-border-glass);
        box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.12), 0 2px 6px rgba(0, 0, 0, 0.4);
        transition: border-color 0.2s ease;
    }

    .quick-pick-pill:hover .pill-art {
        border-color: rgba(181, 142, 98, 0.4);
    }

    .pill-art img {
        width: 100%;
        height: 100%;
        object-fit: cover;
        display: block;
    }

    .placeholder-art {
        width: 100%;
        height: 100%;
        display: flex;
        align-items: center;
        justify-content: center;
        background: radial-gradient(circle at center, #222228 0%, #101014 100%);
    }

    .pill-play-overlay {
        position: absolute;
        inset: 0;
        background: rgba(10, 10, 14, 0.6);
        backdrop-filter: blur(2px);
        -webkit-backdrop-filter: blur(2px);
        display: flex;
        align-items: center;
        justify-content: center;
        opacity: 0;
        transition: opacity 0.2s cubic-bezier(0.16, 1, 0.3, 1);
        color: #ffffff;
    }

    :global(.play-icon-glyph) {
        color: var(--echo-primary, #e2a973);
    }

    .quick-pick-pill:hover .pill-play-overlay,
    .quick-pick-pill.active-track .pill-play-overlay {
        opacity: 1;
    }

    /* Track Details & Telemetry */
    .pill-info {
        flex: 1;
        min-width: 0;
        display: flex;
        flex-direction: column;
        gap: 0.15rem;
    }

    .pill-title {
        font-family: var(--echo-font-heading, sans-serif);
        font-size: 0.86rem;
        font-weight: 600;
        color: #ffffff;
        letter-spacing: -0.01em;
        line-height: 1.25;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .quick-pick-pill.active-track .pill-title {
        color: var(--echo-primary, #e2a973);
    }

    .pill-sub-row {
        display: flex;
        align-items: center;
        gap: 0.35rem;
        min-width: 0;
        line-height: 1.3;
    }

    .pill-artist {
        font-family: var(--echo-font-body, system-ui, sans-serif);
        font-size: 0.74rem;
        color: var(--echo-text-2, rgba(255, 255, 255, 0.55));
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .sub-sep {
        font-size: 0.6rem;
        color: rgba(255, 255, 255, 0.25);
        flex-shrink: 0;
    }

    .pill-duration {
        font-family: var(--lyria-font-mono, monospace);
        font-size: 0.65rem;
        font-weight: 500;
        color: var(--echo-text-3, rgba(255, 255, 255, 0.4));
        flex-shrink: 0;
    }

    /* Trailing Actions Tray */
    .pill-trailing {
        display: flex;
        align-items: center;
        gap: 0.35rem;
        flex-shrink: 0;
    }

    .pill-like-btn {
        background: transparent;
        border: none;
        color: rgba(255, 255, 255, 0.25);
        cursor: pointer;
        padding: 0.35rem;
        border-radius: 50%;
        display: flex;
        align-items: center;
        justify-content: center;
        transition: color 0.15s ease, transform 0.15s cubic-bezier(0.16, 1, 0.3, 1), background-color 0.15s ease;
    }

    .pill-like-btn:hover {
        color: #b58e62;
        background: rgba(255, 255, 255, 0.06);
        transform: scale(1.15);
    }

    .pill-like-btn.liked {
        color: #ffd285;
        filter: drop-shadow(0 0 6px rgba(255, 210, 133, 0.35));
    }

    /* ─── Skeleton Loading State ─── */
    .skeleton-pill-card {
        cursor: default;
        pointer-events: none;
    }

    .skeleton-pill-card .title-placeholder {
        width: 75%;
        height: 13px;
        margin-bottom: 2px;
    }

    .skeleton-pill-card .artist-placeholder {
        width: 45%;
        height: 10px;
    }

    .skeleton-pill-card .like-placeholder {
        width: 22px;
        height: 22px;
    }
</style>
