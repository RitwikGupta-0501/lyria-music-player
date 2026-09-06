<script lang="ts">
    import { homeStore, type FederatedTrack } from "$lib/stores/home.svelte";
    import { audioStore } from "$lib/stores/audio.svelte";
    import { Play, Pause, Heart, Lightning } from "phosphor-svelte";
    import { resolveCoverArt } from "$lib/utils/media";
    import EqualizerWave from "$lib/components/common/EqualizerWave.svelte";
    import { isCurrentTrack } from "$lib/utils/format";

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
        {#each homeStore.quickPicks.slice(0, 12) as song}
            <div 
                class="quick-pick-pill"
                class:active-track={isCurrentTrack(song, audioStore.currentQueueTrack)}
                role="button"
                tabindex="0"
                onclick={() => homeStore.playFederatedTrack(song)}
                onkeydown={(e) => { if (e.key === "Enter") homeStore.playFederatedTrack(song); }}
            >
                <div class="pill-art">
                    {#if resolveCoverArt(song.cover_art_url)}
                        <img 
                            src={resolveCoverArt(song.cover_art_url)} 
                            alt={song.title} 
                            loading="lazy"
                        />
                    {:else}
                        <div class="placeholder-art"></div>
                    {/if}

                    <div class="pill-play-overlay">
                        {#if isPlaying(song)}
                            <EqualizerWave />
                        {:else}
                            <Play size={16} weight="fill" />
                        {/if}
                    </div>
                </div>

                <div class="pill-info">
                    <span class="pill-title" title={song.title}>{song.title}</span>
                    <span class="pill-artist" title={song.artist}>{song.artist}</span>
                </div>

                <div class="pill-trailing">
                    <button 
                        class="pill-like-btn" 
                        class:liked={song.liked}
                        onclick={(e) => { e.stopPropagation(); homeStore.toggleLike(song); }}
                        title={song.liked ? "Liked" : "Like track"}
                    >
                        <Heart size={15} weight={song.liked ? "fill" : "regular"} />
                    </button>
                </div>
            </div>
        {/each}
    </div>
</section>

<style>
    .quick-picks-section {
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
        gap: 0.75rem;
    }

    :global(.quick-icon) {
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

    

    .quick-picks-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
        gap: 0.85rem;
    }

    .quick-pick-pill {
        display: flex;
        align-items: center;
        gap: 0.85rem;
        background: #141416;
        border: 1px solid rgba(255, 255, 255, 0.08);
        border-radius: 8px;
        padding: 0.45rem 0.75rem 0.45rem 0.45rem;
        cursor: pointer;
        position: relative;
        overflow: hidden;
        min-height: 60px;
        transform: translateZ(0);
        backface-visibility: hidden;
        will-change: transform;
        transition: transform 0.2s cubic-bezier(0.16, 1, 0.3, 1), background 0.2s ease, border-color 0.2s ease, box-shadow 0.2s ease;
    }

    .quick-pick-pill:hover {
        transform: translateY(-2px);
        background: #18181C;
        border-color: rgba(181, 142, 98, 0.35);
        box-shadow: 0 8px 20px -4px rgba(0, 0, 0, 0.5);
    }

    .quick-pick-pill.active-track {
        background: rgba(181, 142, 98, 0.12);
        border-color: rgba(181, 142, 98, 0.4);
    }

    .pill-art {
        width: 48px;
        height: 48px;
        border-radius: 8px;
        overflow: hidden;
        position: relative;
        flex-shrink: 0;
        background: #18181B;
        border: 1px solid rgba(255, 255, 255, 0.08);
    }

    .pill-art img {
        width: 100%;
        height: 100%;
        object-fit: cover;
    }

    .placeholder-art {
        width: 100%;
        height: 100%;
        background: #202024;
    }

    .pill-play-overlay {
        position: absolute;
        inset: 0;
        background: rgba(0, 0, 0, 0.45);
        display: flex;
        align-items: center;
        justify-content: center;
        opacity: 0;
        transition: opacity 0.2s ease;
        color: #fff;
    }

    .quick-pick-pill:hover .pill-play-overlay,
    .quick-pick-pill.active-track .pill-play-overlay {
        opacity: 1;
    }



    .pill-info {
        flex: 1;
        min-width: 0;
        display: flex;
        flex-direction: column;
        gap: 0.15rem;
    }

    .pill-title {
        font-size: 0.88rem;
        font-weight: 500;
        color: #EDEDED;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .pill-artist {
        font-size: 0.76rem;
        color: rgba(255, 255, 255, 0.48);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .pill-trailing {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        flex-shrink: 0;
    }



    .pill-like-btn {
        background: transparent;
        border: none;
        color: rgba(255, 255, 255, 0.3);
        cursor: pointer;
        padding: 0.35rem;
        border-radius: 50%;
        display: flex;
        align-items: center;
        justify-content: center;
        transition: color 0.2s ease, transform 0.2s ease;
    }

    .pill-like-btn:hover {
        color: #B58E62;
        transform: scale(1.15);
    }

    .pill-like-btn.liked {
        color: #ffd285;
    }
</style>
