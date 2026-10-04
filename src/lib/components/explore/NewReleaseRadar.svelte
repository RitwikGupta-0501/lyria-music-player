<script lang="ts">
    import { exploreStore } from "$lib/stores/explore.svelte";
    import AlbumCard from "$lib/components/AlbumCard.svelte";
    import { Sparkle, CaretLeft, CaretRight } from "phosphor-svelte";
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
                <div class="skeleton-card">
                    <div class="skeleton-art skeleton-box"></div>
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
                <AlbumCard 
                    album={album}
                    onclick={() => exploreStore.openAlbum(album)}
                />
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

    /* Skeleton Loading Cards */
    .skeleton-card {
        display: flex;
        flex-direction: column;
        gap: 0.6rem;
        min-width: 0;
        pointer-events: none;
    }

    .skeleton-box {
        width: 100%;
        aspect-ratio: 1;
        border-radius: 12px;
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
