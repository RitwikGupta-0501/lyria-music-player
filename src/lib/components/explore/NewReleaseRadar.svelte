<script lang="ts">
    import { exploreStore } from "$lib/stores/explore.svelte";
    import { getInitial } from "$lib/utils/format";
    import { Sparkle, Play } from "phosphor-svelte";
</script>

<div class="new-release-radar">
    <div class="section-header">
        <div class="header-title-group">
            <Sparkle size={20} weight="bold" class="section-icon" />
            <h2>New Releases</h2>
        </div>
        <span class="header-badge">RADAR</span>
    </div>

    <div class="albums-2x2-grid">
        {#each exploreStore.filteredNewReleases as album}
            <div 
                class="album-card"
                role="button"
                tabindex="0"
                onclick={() => exploreStore.openAlbum(album)}
                onkeydown={(e) => { if (e.key === 'Enter') exploreStore.openAlbum(album); }}
            >
                <div class="album-art-wrapper">
                    {#if album.cover_art_url}
                        <img src={album.cover_art_url} alt={album.title} class="album-img" loading="lazy" />
                    {:else}
                        <div class="typographic-art-squircle large">
                            <span>{getInitial(album.artist)}</span>
                        </div>
                    {/if}

                    <div class="release-badge-pill">
                        <span>NEW DROP</span>
                    </div>

                    <div class="album-overlay">
                        <button 
                            type="button" 
                            class="play-bubble" 
                            onclick={(e) => { e.stopPropagation(); exploreStore.playAlbum(album); }}
                            title="Play Album"
                        >
                            <Play size={18} weight="fill" />
                        </button>
                    </div>
                </div>

                <div class="album-meta">
                    <div class="title-row">
                        <span class="album-title" title={album.title}>{album.title}</span>
                    </div>
                    <span class="album-artist" title={album.artist}>{album.artist}</span>
                </div>
            </div>
        {/each}
    </div>
</div>

<style>
    .new-release-radar {
        display: flex;
        flex-direction: column;
        gap: 1rem;
        height: 100%;
    }

    .section-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        min-height: 36px;
    }

    .header-title-group {
        display: flex;
        align-items: center;
        gap: 0.55rem;
    }

    :global(.section-icon) {
        color: #B58E62;
    }

    h2 {
        font-family: var(--echo-font-heading, "Playfair Display", serif);
        font-size: 1.35rem;
        font-weight: 600;
        margin: 0;
        color: #fff;
    }

    .header-badge {
        font-family: var(--echo-font-mono, monospace);
        font-size: 0.62rem;
        font-weight: 700;
        letter-spacing: 0.1em;
        color: #D4A86E;
        background: rgba(181, 142, 98, 0.12);
        border: 1px solid rgba(181, 142, 98, 0.25);
        padding: 0.2rem 0.5rem;
        border-radius: 4px;
    }

    .albums-2x2-grid {
        display: grid;
        grid-template-columns: repeat(2, 1fr);
        gap: 0.85rem;
    }

    .album-card {
        background: rgba(18, 18, 22, 0.5);
        border: 1px solid rgba(255, 255, 255, 0.06);
        border-radius: 10px;
        padding: 0.65rem;
        display: flex;
        flex-direction: column;
        gap: 0.65rem;
        cursor: pointer;
        transition: transform 0.2s ease, border-color 0.2s ease, background-color 0.2s ease;
    }

    .album-card:hover {
        transform: translateY(-2px);
        background: rgba(26, 26, 32, 0.7);
        border-color: rgba(181, 142, 98, 0.35);
    }

    .album-art-wrapper {
        aspect-ratio: 1 / 1;
        width: 100%;
        border-radius: 8px;
        overflow: hidden;
        position: relative;
        background: #141416;
    }

    .album-img {
        width: 100%;
        height: 100%;
        object-fit: cover;
        transition: transform 0.3s ease;
    }

    .album-card:hover .album-img {
        transform: scale(1.04);
    }

    .typographic-art-squircle.large {
        width: 100%;
        height: 100%;
        background: #202026;
        display: flex;
        align-items: center;
        justify-content: center;
        font-family: var(--echo-font-heading, serif);
        font-size: 2rem;
        font-weight: 700;
        color: #D4A86E;
    }

    .release-badge-pill {
        position: absolute;
        top: 6px;
        left: 6px;
        background: rgba(14, 14, 16, 0.85);
        backdrop-filter: blur(8px);
        border: 1px solid rgba(212, 168, 110, 0.4);
        padding: 0.15rem 0.4rem;
        border-radius: 4px;
        z-index: 2;
    }

    .release-badge-pill span {
        font-family: var(--echo-font-mono, monospace);
        font-size: 0.55rem;
        font-weight: 700;
        letter-spacing: 0.08em;
        color: #D4A86E;
    }

    .album-overlay {
        position: absolute;
        inset: 0;
        background: rgba(0, 0, 0, 0.35);
        display: flex;
        align-items: flex-end;
        justify-content: flex-end;
        padding: 0.6rem;
        opacity: 0;
        transition: opacity 0.2s ease;
    }

    .album-card:hover .album-overlay {
        opacity: 1;
    }

    .play-bubble {
        width: 34px;
        height: 34px;
        border-radius: 50%;
        background: #D4A86E;
        color: #0E0E10;
        border: none;
        display: flex;
        align-items: center;
        justify-content: center;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.5);
        cursor: pointer;
        transition: transform 0.15s ease, background-color 0.15s ease;
    }

    .play-bubble:hover {
        transform: scale(1.1);
        background: #E5B97F;
    }

    .album-meta {
        display: flex;
        flex-direction: column;
        gap: 0.15rem;
    }

    .title-row {
        display: flex;
        align-items: center;
        gap: 0.35rem;
    }

    .album-title {
        font-size: 0.85rem;
        font-weight: 600;
        color: #fff;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .album-artist {
        font-size: 0.74rem;
        color: rgba(255, 255, 255, 0.5);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }
</style>
