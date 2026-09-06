<script lang="ts">
    import { exploreStore } from "$lib/stores/explore.svelte";
    import { libraryStore, getCanonicalKey } from "$lib/stores/library.svelte";
    import { settingsStore } from "$lib/stores/settings.svelte";
    import { getInitial } from "$lib/utils/format";
    import { Sparkle, Play, Heart, CaretLeft, CaretRight } from "phosphor-svelte";
    import SectionHeaderSkeleton from "$lib/components/common/SectionHeaderSkeleton.svelte";

    let currentPage = $state(0);
    const PAGE_SIZE = 4;
    const totalPages = $derived(Math.ceil(exploreStore.filteredNewReleases.length / PAGE_SIZE) || 1);
    const displayedAlbums = $derived(
        exploreStore.filteredNewReleases.slice(currentPage * PAGE_SIZE, (currentPage + 1) * PAGE_SIZE)
    );

    function prevPage() {
        if (currentPage > 0) currentPage--;
    }

    function nextPage() {
        if (currentPage < totalPages - 1) currentPage++;
    }
</script>

{#if exploreStore.isLoading && exploreStore.filteredNewReleases.length === 0}
    <div class="new-release-radar">
        <SectionHeaderSkeleton hasControls={true} titleWidth="140px" />
        <div class="albums-2x2-grid">
            {#each Array(4) as _}
                <div class="discover-card skeleton">
                    <div class="card-art-wrapper skeleton-box"></div>
                    <div class="skeleton-line title"></div>
                    <div class="skeleton-line artist"></div>
                </div>
            {/each}
        </div>
    </div>
{:else if exploreStore.filteredNewReleases.length > 0}
    <div class="new-release-radar">
        <div class="section-header">
            <div class="header-title-group">
                <Sparkle size={20} weight="bold" class="section-icon" />
                <h2>New Releases</h2>
            </div>
            {#if totalPages > 1}
                <div class="chevron-controls">
                    <button 
                        type="button"
                        class="chevron-btn" 
                        onclick={prevPage} 
                        disabled={currentPage === 0}
                        title="Previous releases"
                        aria-label="Previous releases"
                    >
                        <CaretLeft size={16} weight="bold" />
                    </button>
                    <button 
                        type="button"
                        class="chevron-btn" 
                        onclick={nextPage} 
                        disabled={currentPage >= totalPages - 1}
                        title="Next releases"
                        aria-label="Next releases"
                    >
                        <CaretRight size={16} weight="bold" />
                    </button>
                </div>
            {/if}
        </div>

        <div class="albums-2x2-grid">
            {#each displayedAlbums as album}
                {@const isSaved = libraryStore.isAlbumSaved(album.id, album.title, album.artist)}
                <div 
                    class="discover-card"
                    role="button"
                    tabindex="0"
                    onclick={() => exploreStore.openAlbum(album)}
                    onkeydown={(e) => { if (e.key === "Enter") exploreStore.openAlbum(album); }}
                >
                    <div class="card-art-wrapper">
                        {#if album.cover_art_url}
                            <img src={album.cover_art_url} alt={album.title} loading="lazy" />
                        {:else}
                            <div class="placeholder-art">
                                <span>{getInitial(album.artist)}</span>
                            </div>
                        {/if}

                        <div class="card-overlay">
                            <button 
                                type="button"
                                class="play-bubble"
                                onclick={(e) => {
                                    e.stopPropagation();
                                    exploreStore.playAlbum(album);
                                }}
                                title="Play release"
                                aria-label="Play release"
                            >
                                <Play size={18} weight="fill" />
                            </button>
                        </div>

                        <button 
                            type="button"
                            class="liquid-like-btn" 
                            class:is-glass={settingsStore.glassyPlayerBar}
                            class:liked={isSaved}
                            onclick={(e) => { 
                                e.stopPropagation(); 
                                libraryStore.toggleSaveAlbum({
                                    id: album.id,
                                    title: album.title,
                                    artist: album.artist,
                                    cover_art_url: album.cover_art_url,
                                    provider_id: album.provider_id || "youtube-wasm",
                                });
                            }}
                            title={isSaved ? "Saved" : "Save release"}
                            aria-label={isSaved ? "Unsave release" : "Save release"}
                        >
                            <Heart size={16} weight={isSaved ? "fill" : "bold"} color={isSaved ? "#ffd285" : "#FFFFFF"} />
                        </button>
                    </div>

                    <div class="card-info">
                        <span class="card-title" title={album.title}>{album.title}</span>
                        <span class="card-artist" title={album.artist}>{album.artist}</span>
                    </div>
                </div>
            {/each}
        </div>
    </div>
{/if}

<style>
    .new-release-radar {
        display: flex;
        flex-direction: column;
        gap: 1.25rem;
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
        gap: 0.65rem;
    }

    :global(.section-icon) {
        color: #b58e62;
    }

    h2 {
        font-family: var(--echo-font-heading, serif);
        font-size: 1.35rem;
        font-weight: 600;
        margin: 0;
        letter-spacing: -0.01em;
        color: #fff;
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

    .albums-2x2-grid {
        display: grid;
        grid-template-columns: repeat(2, 1fr);
        gap: 1.25rem;
    }

    .discover-card {
        display: flex;
        flex-direction: column;
        gap: 0.6rem;
        cursor: pointer;
        transition: transform 0.2s cubic-bezier(0.16, 1, 0.3, 1);
        min-width: 0;
    }

    .discover-card:hover {
        transform: translateY(-3px);
    }

    .card-art-wrapper {
        width: 100%;
        aspect-ratio: 1;
        border-radius: 12px;
        overflow: hidden;
        position: relative;
        background: #141416;
        border: 1px solid rgba(255, 255, 255, 0.08);
        contain: layout paint;
        isolation: isolate;
        transition: border-color 0.2s ease, box-shadow 0.2s ease;
    }

    .discover-card:hover .card-art-wrapper {
        border-color: rgba(181, 142, 98, 0.35);
        box-shadow: 0 10px 24px -6px rgba(0, 0, 0, 0.6);
    }

    .card-art-wrapper img {
        width: 100%;
        height: 100%;
        object-fit: cover;
    }

    .placeholder-art {
        width: 100%;
        height: 100%;
        background: #18181B;
        display: flex;
        align-items: center;
        justify-content: center;
        font-family: var(--echo-font-heading, serif);
        font-size: 2rem;
        font-weight: 700;
        color: #D4A86E;
    }

    .card-overlay {
        position: absolute;
        inset: 0;
        background: rgba(0, 0, 0, 0.35);
        display: flex;
        align-items: center;
        justify-content: center;
        opacity: 0;
        pointer-events: none;
        transform: translateZ(0);
        backface-visibility: hidden;
        will-change: opacity;
        transition: opacity 0.2s cubic-bezier(0.16, 1, 0.3, 1);
    }

    .discover-card:hover .card-overlay {
        opacity: 1;
        pointer-events: auto;
    }

    .play-bubble {
        width: 44px;
        height: 44px;
        border-radius: 50%;
        background: #B58E62;
        color: #0E0E10;
        display: flex;
        align-items: center;
        justify-content: center;
        box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
        transform: scale(0.9);
        transition: transform 0.2s cubic-bezier(0.16, 1, 0.3, 1), background 0.15s ease;
    }

    .discover-card:hover .play-bubble {
        transform: scale(1);
    }

    .play-bubble:hover {
        transform: scale(1.08) !important;
        background: #C9A070;
    }

    .liquid-like-btn {
        position: absolute;
        top: 8px;
        right: 8px;
        width: 32px !important;
        height: 32px !important;
        min-width: 32px !important;
        max-width: 32px !important;
        min-height: 32px !important;
        max-height: 32px !important;
        border-radius: 50% !important;
        padding: 0 !important;
        margin: 0 !important;
        background: rgba(18, 20, 26, 0.85);
        border: 1px solid rgba(255, 255, 255, 0.12);
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.45);
        color: #ffffff;
        display: flex !important;
        align-items: center !important;
        justify-content: center !important;
        cursor: pointer;
        opacity: 0;
        transform: scale(0.85) translateZ(0);
        backface-visibility: hidden;
        pointer-events: none;
        transition: opacity 0.2s cubic-bezier(0.16, 1, 0.3, 1), transform 0.2s cubic-bezier(0.16, 1, 0.3, 1), background 0.2s ease, border-color 0.2s ease, box-shadow 0.2s ease;
        z-index: 5;
    }

    .liquid-like-btn.is-glass {
        background: linear-gradient(180deg, rgba(255, 255, 255, 0.08) 0%, rgba(255, 255, 255, 0.02) 100%);
        border: 1.5px solid rgba(255, 255, 255, 0.14);
        box-shadow: 
            inset 0 1.5px 0 0 rgba(255, 255, 255, 0.30),
            0 4px 12px rgba(0, 0, 0, 0.35);
    }

    .discover-card:hover .liquid-like-btn {
        opacity: 1;
        transform: scale(1);
        pointer-events: auto;
    }

    .liquid-like-btn.liked {
        opacity: 1 !important;
        transform: scale(1) !important;
        pointer-events: auto !important;
        background: rgba(45, 35, 25, 0.9) !important;
        border-color: rgba(224, 184, 143, 0.45) !important;
    }

    .card-info {
        display: flex;
        flex-direction: column;
        gap: 0.2rem;
        min-width: 0;
    }

    .card-title {
        font-family: var(--echo-font-body, system-ui, sans-serif);
        font-size: 0.88rem;
        font-weight: 600;
        color: #fff;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .card-artist {
        font-size: 0.76rem;
        color: rgba(255, 255, 255, 0.5);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    /* Skeleton Loading Cards */
    .discover-card.skeleton {
        pointer-events: none;
    }

    .skeleton-box {
        background: linear-gradient(90deg, rgba(255, 255, 255, 0.04) 0%, rgba(255, 255, 255, 0.08) 50%, rgba(255, 255, 255, 0.04) 100%);
        background-size: 200% 100%;
        animation: skeleton-pulse 1.5s infinite;
    }

    .skeleton-line {
        height: 10px;
        border-radius: 4px;
        background: linear-gradient(90deg, rgba(255, 255, 255, 0.04) 0%, rgba(255, 255, 255, 0.08) 50%, rgba(255, 255, 255, 0.04) 100%);
        background-size: 200% 100%;
        animation: skeleton-pulse 1.5s infinite;
    }

    .skeleton-line.title {
        width: 75%;
        margin-top: 4px;
    }

    .skeleton-line.artist {
        width: 50%;
        margin-top: 2px;
    }

    @keyframes skeleton-pulse {
        0% { background-position: 200% 0; }
        100% { background-position: -200% 0; }
    }
</style>
