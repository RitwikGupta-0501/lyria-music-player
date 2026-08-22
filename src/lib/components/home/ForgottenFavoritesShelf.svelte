<script lang="ts">
    import { homeStore } from "$lib/stores/home.svelte";
    import { ClockCounterClockwise, Play } from "phosphor-svelte";
</script>

{#if homeStore.forgottenFavorites.length > 0}
    <section class="forgotten-favorites-section">
        <div class="section-title-row">
            <div class="title-group">
                <ClockCounterClockwise size={20} weight="bold" class="fav-icon" />
                <h2>Forgotten Favorites</h2>
            </div>
            <span class="section-tag">Rediscover Past Loves</span>
        </div>

        <div class="favs-carousel-track">
            {#each homeStore.forgottenFavorites as song}
                <div 
                    class="fav-card"
                    role="button"
                    tabindex="0"
                    onclick={() => homeStore.playFederatedTrack(song)}
                    onkeydown={(e) => { if (e.key === 'Enter') homeStore.playFederatedTrack(song); }}
                >
                    <div class="fav-art-wrapper">
                        {#if song.cover_art_url}
                            <img src={song.cover_art_url.startsWith('/') ? `asset://localhost/${encodeURIComponent(song.cover_art_url)}` : song.cover_art_url} alt={song.title} loading="lazy" />
                        {:else}
                            <div class="placeholder-art"></div>
                        {/if}
                        <div class="fav-overlay">
                            <div class="play-bubble">
                                <Play size={20} weight="fill" />
                            </div>
                        </div>
                    </div>
                    <div class="fav-info">
                        <span class="fav-title">{song.title}</span>
                        <span class="fav-artist">{song.artist}</span>
                    </div>
                </div>
            {/each}
        </div>
    </section>
{/if}

<style>
    .forgotten-favorites-section {
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

    :global(.fav-icon) {
        color: #a8dadc;
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

    .favs-carousel-track {
        display: flex;
        gap: 1.25rem;
        overflow-x: auto;
        scroll-snap-type: x mandatory;
        padding-bottom: 0.5rem;
        scrollbar-width: none;
    }

    .favs-carousel-track::-webkit-scrollbar {
        display: none;
    }

    .fav-card {
        flex: 0 0 145px;
        scroll-snap-align: start;
        display: flex;
        flex-direction: column;
        gap: 0.55rem;
        cursor: pointer;
        transition: transform 0.2s cubic-bezier(0.16, 1, 0.3, 1);
    }

    .fav-card:hover {
        transform: translateY(-4px);
    }

    .fav-art-wrapper {
        width: 145px;
        height: 145px;
        border-radius: 10px;
        overflow: hidden;
        position: relative;
        background: var(--surface-1, rgba(255, 255, 255, 0.04));
        border: 1px solid rgba(255, 255, 255, 0.06);
    }

    .fav-art-wrapper img {
        width: 100%;
        height: 100%;
        object-fit: cover;
    }

    .placeholder-art {
        width: 100%;
        height: 100%;
        background: linear-gradient(135deg, rgba(255, 255, 255, 0.08), rgba(255, 255, 255, 0.02));
    }

    .fav-overlay {
        position: absolute;
        inset: 0;
        background: rgba(0, 0, 0, 0.4);
        display: flex;
        align-items: center;
        justify-content: center;
        opacity: 0;
        transition: opacity 0.2s ease;
    }

    .fav-card:hover .fav-overlay {
        opacity: 1;
    }

    .play-bubble {
        width: 38px;
        height: 38px;
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

    .fav-card:hover .play-bubble {
        transform: scale(1);
    }

    .fav-info {
        display: flex;
        flex-direction: column;
        gap: 0.15rem;
    }

    .fav-title {
        font-size: 0.88rem;
        font-weight: 600;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .fav-artist {
        font-size: 0.76rem;
        color: var(--text-muted, rgba(255, 255, 255, 0.6));
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }
</style>
