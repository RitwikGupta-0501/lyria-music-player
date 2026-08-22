<script lang="ts">
    import { homeStore, type FederatedTrack } from "$lib/stores/home.svelte";
    import { audioStore } from "$lib/stores/audio.svelte";
    import { Play, Pause, Heart } from "phosphor-svelte";

    function isCurrentTrack(song: FederatedTrack): boolean {
        const cur = audioStore.currentQueueTrack;
        if (!cur) return false;
        return cur.title.toLowerCase() === song.title.toLowerCase()
            && (cur.artist || "").toLowerCase() === song.artist.toLowerCase();
    }

    function isPlaying(song: FederatedTrack): boolean {
        return isCurrentTrack(song) && audioStore.playbackState === "Playing";
    }
</script>

<section class="quick-picks-section">
    <div class="section-title-row">
        <div class="title-group">
            <h2>Quick Picks</h2>
            <span class="section-tag">High Rotation</span>
        </div>
    </div>

    <div class="quick-picks-grid">
        {#each homeStore.quickPicks.slice(0, 8) as song}
            <div 
                class="quick-pick-pill"
                class:active-track={isCurrentTrack(song)}
                role="button"
                tabindex="0"
                onclick={() => homeStore.playFederatedTrack(song)}
                onkeydown={(e) => { if (e.key === 'Enter') homeStore.playFederatedTrack(song); }}
            >
                <div class="pill-art">
                    {#if song.cover_art_url}
                        <img 
                            src={song.cover_art_url.startsWith('/') ? `asset://localhost/${encodeURIComponent(song.cover_art_url)}` : song.cover_art_url} 
                            alt={song.title} 
                            loading="lazy"
                        />
                    {:else}
                        <div class="placeholder-art"></div>
                    {/if}

                    <div class="pill-play-overlay">
                        {#if isPlaying(song)}
                            <div class="equalizer-indicator">
                                <span class="bar bar-1"></span>
                                <span class="bar bar-2"></span>
                                <span class="bar bar-3"></span>
                            </div>
                        {:else}
                            <Play size={18} weight="fill" />
                        {/if}
                    </div>
                </div>

                <div class="pill-info">
                    <span class="pill-title">{song.title}</span>
                    <span class="pill-artist">{song.artist}</span>
                </div>

                <button 
                    class="pill-like-btn" 
                    class:liked={song.liked}
                    onclick={(e) => { e.stopPropagation(); homeStore.toggleLike(song); }}
                    title={song.liked ? "Liked" : "Like track"}
                >
                    <Heart size={18} weight={song.liked ? "fill" : "regular"} />
                </button>
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

    .section-title-row h2 {
        font-size: 1.35rem;
        font-weight: 700;
        margin: 0;
        letter-spacing: -0.02em;
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

    .quick-picks-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
        gap: 0.85rem;
    }

    .quick-pick-pill {
        display: flex;
        align-items: center;
        gap: 0.85rem;
        background: var(--surface-1, rgba(255, 255, 255, 0.035));
        border: 1px solid rgba(255, 255, 255, 0.06);
        border-radius: 10px;
        padding: 0.5rem 0.75rem 0.5rem 0.5rem;
        cursor: pointer;
        transition: transform 0.2s cubic-bezier(0.16, 1, 0.3, 1), background 0.2s ease, border-color 0.2s ease;
    }

    .quick-pick-pill:hover {
        transform: translateY(-2px);
        background: var(--surface-2, rgba(255, 255, 255, 0.08));
        border-color: rgba(255, 255, 255, 0.15);
    }

    .quick-pick-pill.active-track {
        background: rgba(255, 255, 255, 0.1);
        border-color: rgba(255, 255, 255, 0.25);
    }

    .pill-art {
        width: 50px;
        height: 50px;
        border-radius: 7px;
        overflow: hidden;
        position: relative;
        flex-shrink: 0;
        background: rgba(255, 255, 255, 0.05);
    }

    .pill-art img {
        width: 100%;
        height: 100%;
        object-fit: cover;
    }

    .placeholder-art {
        width: 100%;
        height: 100%;
        background: linear-gradient(135deg, rgba(255, 255, 255, 0.08), rgba(255, 255, 255, 0.02));
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

    /* Equalizer Bar Animation */
    .equalizer-indicator {
        display: flex;
        align-items: flex-end;
        gap: 2px;
        height: 14px;
    }

    .equalizer-indicator .bar {
        width: 3px;
        background: var(--accent-primary, #ffd166);
        border-radius: 1px;
        animation: eqBounce 0.8s ease-in-out infinite alternate;
    }

    .bar-1 { height: 60%; animation-delay: 0.1s; }
    .bar-2 { height: 100%; animation-delay: 0.3s; }
    .bar-3 { height: 40%; animation-delay: 0.2s; }

    @keyframes eqBounce {
        0% { height: 20%; }
        100% { height: 100%; }
    }

    .pill-info {
        flex: 1;
        min-width: 0;
        display: flex;
        flex-direction: column;
        gap: 0.15rem;
    }

    .pill-title {
        font-size: 0.92rem;
        font-weight: 600;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .pill-artist {
        font-size: 0.8rem;
        color: var(--text-muted, rgba(255, 255, 255, 0.6));
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .pill-like-btn {
        background: transparent;
        border: none;
        color: var(--text-muted, rgba(255, 255, 255, 0.35));
        cursor: pointer;
        padding: 0.4rem;
        border-radius: 50%;
        display: flex;
        align-items: center;
        justify-content: center;
        transition: color 0.2s ease, transform 0.2s ease;
    }

    .pill-like-btn:hover {
        color: #ff6b6b;
        transform: scale(1.15);
    }

    .pill-like-btn.liked {
        color: #ff6b6b;
    }
</style>
