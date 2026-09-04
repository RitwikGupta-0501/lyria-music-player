<script lang="ts">
    import { exploreStore } from "$lib/stores/explore.svelte";
    import { audioStore } from "$lib/stores/audio.svelte";
    import { formatDuration, getInitial, isCurrentTrack } from "$lib/utils/format";
    import { ArrowLeft, Play, Pause, Disc, MusicNotes } from "phosphor-svelte";
    import CategoryShelfCarousel from "./CategoryShelfCarousel.svelte";
    import AlbumCard from "$lib/components/AlbumCard.svelte";
    import SectionHeaderSkeleton from "$lib/components/common/SectionHeaderSkeleton.svelte";
</script>

{#if exploreStore.activeCategory}
    <div class="category-hub-canvas">
        <!-- Hub Header -->
        <header class="hub-header">
            <button class="hub-back-btn" onclick={() => exploreStore.closeCategory()} title="Back to Explore (Esc)">
                <ArrowLeft size={16} weight="bold" />
                <span>Back to Explore</span>
            </button>
            <div class="hub-title-row">
                {#if exploreStore.activeCategory.color_hex}
                    <div class="hub-color-badge" style="background: {exploreStore.activeCategory.color_hex}"></div>
                {/if}
                <h1>{exploreStore.activeCategory.title}</h1>
            </div>
            <p class="hub-subtitle">Ranked charts & curated editorial mixes</p>
        </header>

        {#if exploreStore.isCategoryLoading}
            <div class="hub-loading-state">
                <!-- Skeleton Shelf 1 -->
                <section class="hub-shelf-section">
                    <SectionHeaderSkeleton hasControls={true} titleWidth="180px" />
                    <div class="hub-carousel-track">
                        {#each Array(6) as _}
                            <div class="skeleton-card skeleton">
                                <div class="card-art-wrapper skeleton-box"></div>
                                <div class="title-skeleton skeleton-box"></div>
                                <div class="subtitle-skeleton skeleton-box"></div>
                            </div>
                        {/each}
                    </div>
                </section>

                <!-- Skeleton Shelf 2 -->
                <section class="hub-shelf-section">
                    <SectionHeaderSkeleton hasControls={true} titleWidth="150px" />
                    <div class="hub-carousel-track">
                        {#each Array(6) as _}
                            <div class="skeleton-card skeleton">
                                <div class="card-art-wrapper skeleton-box"></div>
                                <div class="title-skeleton skeleton-box"></div>
                                <div class="subtitle-skeleton skeleton-box"></div>
                            </div>
                        {/each}
                    </div>
                </section>

                <!-- Skeleton Top Songs Ledger -->
                <section class="hub-shelf-section">
                    <SectionHeaderSkeleton hasControls={false} titleWidth="160px" />
                    <div class="hub-tracks-grid">
                        {#each Array(6) as _}
                            <div class="ledger-row skeleton">
                                <div class="skeleton-num skeleton-box"></div>
                                <div class="track-squircle skeleton-box"></div>
                                <div class="track-meta">
                                    <div class="track-title-skeleton skeleton-box"></div>
                                    <div class="track-artist-skeleton skeleton-box"></div>
                                </div>
                            </div>
                        {/each}
                    </div>
                </section>
            </div>
        {:else}
            <!-- 1. Thematic Editorial Shelves (e.g. Flow State, Classical Focus, Deep Grooves) -->
            {#if exploreStore.categoryShelves.length > 0}
                {#each exploreStore.categoryShelves as shelf (shelf.title)}
                    <CategoryShelfCarousel title={shelf.title} items={shelf.items} />
                {/each}
            {:else if exploreStore.categoryPlaylists.length > 0}
                <!-- Fallback single shelf if provider returned flat playlists -->
                <CategoryShelfCarousel title={`Featured ${exploreStore.activeCategory.title} Mixes`} items={exploreStore.categoryPlaylists} />
            {/if}

            <!-- 2. Top Ranked Songs Section (if returned) -->
            {#if exploreStore.categoryTracks.length > 0}
                <section class="hub-shelf-section">
                    <div class="shelf-header-row">
                        <div class="shelf-title-group">
                            <MusicNotes size={18} weight="bold" class="shelf-icon" />
                            <h2>Top {exploreStore.activeCategory.title} Songs</h2>
                        </div>
                    </div>

                    <div class="hub-tracks-grid">
                        {#each exploreStore.categoryTracks as track, index}
                            {@const isPlaying = isCurrentTrack(track, audioStore.currentQueueTrack) && audioStore.playbackState === "Playing"}
                            {@const isActive = isCurrentTrack(track, audioStore.currentQueueTrack)}
                            <div 
                                class="ledger-row"
                                class:active-track={isActive}
                                role="button"
                                tabindex="0"
                                ondblclick={() => exploreStore.playTrack(track, track.provider_id)}
                                onkeydown={(e) => { if (e.key === "Enter") exploreStore.playTrack(track, track.provider_id); }}
                            >
                                <span class="ledger-num">{(index + 1).toString().padStart(2, "0")}</span>
                                
                                <div class="track-art-wrapper">
                                    {#if track.cover_art_url}
                                        <img src={track.cover_art_url} alt={track.title} class="track-squircle" loading="lazy" />
                                    {:else}
                                        <div class="typographic-art-squircle">
                                            <span>{getInitial(track.artist)}</span>
                                        </div>
                                    {/if}
                                    <button 
                                        class="play-overlay-btn" 
                                        class:always-visible={isActive}
                                        onclick={(e) => { e.stopPropagation(); exploreStore.playTrack(track, track.provider_id); }}
                                        title={isPlaying ? "Pause" : "Play"}
                                    >
                                        {#if isPlaying}
                                            <Pause size={14} weight="fill" />
                                        {:else}
                                            <Play size={14} weight="fill" />
                                        {/if}
                                    </button>
                                </div>

                                <div class="track-meta">
                                    <span class="track-title">{track.title}</span>
                                    <div class="track-subline">
                                        <span class="track-artist">{track.artist}</span>
                                        {#if track.album}
                                            <span class="track-dot">•</span>
                                            <span class="track-album">{track.album}</span>
                                        {/if}
                                    </div>
                                </div>

                                <span class="track-duration">{formatDuration(track.duration_ms)}</span>
                            </div>
                        {/each}
                    </div>
                </section>
            {/if}

            <!-- 3. Genre Albums Section (if returned) -->
            {#if exploreStore.categoryAlbums.length > 0}
                <section class="hub-shelf-section">
                    <div class="shelf-header-row">
                        <div class="shelf-title-group">
                            <Disc size={18} weight="bold" class="shelf-icon" />
                            <h2>Albums & EPs</h2>
                        </div>
                    </div>

                    <div class="hub-carousel-track">
                        {#each exploreStore.categoryAlbums as album}
                            <div class="carousel-card-wrap">
                                <AlbumCard 
                                    album={album}
                                    onclick={() => exploreStore.openAlbum(album)}
                                />
                            </div>
                        {/each}
                    </div>
                </section>
            {/if}

            {#if exploreStore.categoryShelves.length === 0 && exploreStore.categoryPlaylists.length === 0 && exploreStore.categoryTracks.length === 0 && exploreStore.categoryAlbums.length === 0}
                <div class="empty-state">
                    <p>No curated releases found for this category.</p>
                    <button class="return-btn" onclick={() => exploreStore.closeCategory()}>Return to Explore</button>
                </div>
            {/if}
        {/if}
    </div>
{/if}

<style>
    .category-hub-canvas {
        display: flex;
        flex-direction: column;
        gap: 2.5rem;
        padding: 0 0 4rem 0;
        animation: fadeIn 0.3s cubic-bezier(0.16, 1, 0.3, 1);
    }

    @keyframes fadeIn {
        from { opacity: 0; transform: translateY(8px); }
        to { opacity: 1; transform: translateY(0); }
    }

    .hub-header {
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
        padding-bottom: 1.5rem;
        border-bottom: 1px solid rgba(255, 255, 255, 0.06);
    }

    .hub-back-btn {
        display: inline-flex;
        align-items: center;
        gap: 0.5rem;
        background: transparent;
        border: none;
        color: rgba(255, 255, 255, 0.6);
        font-size: 0.85rem;
        font-weight: 500;
        cursor: pointer;
        padding: 0.4rem 0.6rem;
        margin-left: -0.6rem;
        border-radius: 6px;
        width: fit-content;
        transition: color 0.15s ease, background-color 0.15s ease;
    }

    .hub-back-btn:hover {
        color: #fff;
        background: rgba(255, 255, 255, 0.06);
    }

    .hub-title-row {
        display: flex;
        align-items: center;
        gap: 0.85rem;
    }

    .hub-color-badge {
        width: 14px;
        height: 14px;
        border-radius: 50%;
        box-shadow: 0 0 12px currentColor;
    }

    .hub-header h1 {
        font-family: var(--echo-font-heading, "Playfair Display", serif);
        font-size: 2.25rem;
        font-weight: 700;
        margin: 0;
        letter-spacing: -0.02em;
        color: #fff;
    }

    .hub-subtitle {
        font-size: 0.95rem;
        color: rgba(255, 255, 255, 0.5);
        margin: 0;
    }

    .hub-loading-state {
        display: flex;
        flex-direction: column;
        gap: 2.5rem;
    }

    .hub-shelf-section {
        display: flex;
        flex-direction: column;
        gap: 1.1rem;
        width: 100%;
    }

    .shelf-header-row {
        display: flex;
        align-items: center;
        justify-content: space-between;
    }

    .shelf-title-group {
        display: flex;
        align-items: center;
        gap: 0.6rem;
    }

    :global(.shelf-icon) {
        color: #B58E62;
    }

    .shelf-title-group h2 {
        font-family: var(--echo-font-heading, "Playfair Display", serif);
        font-size: 1.25rem;
        font-weight: 600;
        margin: 0;
        letter-spacing: -0.01em;
        color: #fff;
    }

    .hub-carousel-track {
        display: flex;
        gap: 1.25rem;
        overflow-x: auto;
        scrollbar-width: none;
        padding-bottom: 0.35rem;
    }

    .hub-carousel-track::-webkit-scrollbar {
        display: none;
    }

    .carousel-card-wrap {
        flex: 0 0 176px;
        min-width: 0;
    }

    .skeleton-card {
        flex: 0 0 176px;
        display: flex;
        flex-direction: column;
        gap: 0.6rem;
    }

    .skeleton-card .card-art-wrapper {
        width: 176px;
        height: 176px;
        border-radius: 1.5rem;
    }

    .skeleton-box {
        background: linear-gradient(90deg, rgba(255,255,255,0.03) 0%, rgba(255,255,255,0.07) 50%, rgba(255,255,255,0.03) 100%);
        background-size: 200% 100%;
        animation: shimmer 1.8s infinite;
    }

    @keyframes shimmer {
        0% { background-position: -200% 0; }
        100% { background-position: 200% 0; }
    }

    .title-skeleton {
        height: 14px;
        width: 75%;
        border-radius: 4px;
    }

    .subtitle-skeleton {
        height: 12px;
        width: 50%;
        border-radius: 4px;
    }

    .hub-tracks-grid {
        display: grid;
        grid-template-columns: repeat(2, 1fr);
        gap: 0.5rem 1rem;
        background: rgba(18, 18, 22, 0.5);
        border: 1px solid rgba(255, 255, 255, 0.05);
        border-radius: 16px;
        padding: 1rem;
    }

    @media (max-width: 900px) {
        .hub-tracks-grid {
            grid-template-columns: 1fr;
        }
    }

    .ledger-row {
        display: flex;
        align-items: center;
        gap: 0.85rem;
        padding: 0.55rem 0.75rem;
        border-radius: 10px;
        cursor: pointer;
        user-select: none;
        transition: background 0.15s ease;
    }

    .ledger-row:hover {
        background: rgba(255, 255, 255, 0.05);
    }

    .ledger-row.active-track {
        background: rgba(212, 168, 110, 0.1);
    }

    .ledger-row.active-track .track-title {
        color: #D4A86E;
    }

    .ledger-num {
        font-size: 0.85rem;
        font-family: var(--echo-font-mono, monospace);
        font-weight: 500;
        color: rgba(255, 255, 255, 0.35);
        width: 1.5rem;
        text-align: right;
        flex-shrink: 0;
    }

    .track-art-wrapper {
        position: relative;
        width: 40px;
        height: 40px;
        flex-shrink: 0;
    }

    .track-squircle {
        width: 100%;
        height: 100%;
        border-radius: 8px;
        object-fit: cover;
    }

    .typographic-art-squircle {
        width: 100%;
        height: 100%;
        border-radius: 8px;
        background: linear-gradient(135deg, #27272a 0%, #18181b 100%);
        border: 1px solid rgba(255, 255, 255, 0.08);
        display: flex;
        align-items: center;
        justify-content: center;
        font-family: var(--echo-font-heading, serif);
        font-weight: 600;
        color: #D4A86E;
        font-size: 1rem;
    }

    .play-overlay-btn {
        position: absolute;
        inset: 0;
        display: flex;
        align-items: center;
        justify-content: center;
        background: rgba(0, 0, 0, 0.6);
        border: none;
        border-radius: 8px;
        color: #fff;
        cursor: pointer;
        opacity: 0;
        transition: opacity 0.15s ease;
        padding: 0;
    }

    .ledger-row:hover .play-overlay-btn,
    .play-overlay-btn.always-visible {
        opacity: 1;
    }

    .track-meta {
        display: flex;
        flex-direction: column;
        gap: 0.2rem;
        flex: 1;
        min-width: 0;
    }

    .track-title {
        font-size: 0.9rem;
        font-weight: 500;
        color: #fff;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .track-subline {
        display: flex;
        align-items: center;
        gap: 0.4rem;
        font-size: 0.78rem;
        color: rgba(255, 255, 255, 0.5);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .track-artist {
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .track-dot {
        opacity: 0.4;
    }

    .track-album {
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        opacity: 0.75;
    }

    .track-duration {
        font-size: 0.8rem;
        color: rgba(255, 255, 255, 0.35);
        font-family: var(--echo-font-mono, monospace);
        margin-left: 0.5rem;
        flex-shrink: 0;
    }

    .ledger-row.skeleton {
        cursor: default;
    }

    .skeleton-num {
        width: 1.5rem;
        height: 12px;
        border-radius: 3px;
        flex-shrink: 0;
    }

    .track-meta .track-title-skeleton {
        height: 12px;
        width: 65%;
        border-radius: 3px;
    }

    .track-meta .track-artist-skeleton {
        height: 10px;
        width: 40%;
        border-radius: 3px;
    }

    .empty-state {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        padding: 4rem 1rem;
        color: rgba(255, 255, 255, 0.4);
        text-align: center;
        gap: 1rem;
    }

    .return-btn {
        background: rgba(255, 255, 255, 0.08);
        border: 1px solid rgba(255, 255, 255, 0.12);
        color: #fff;
        padding: 0.5rem 1.25rem;
        border-radius: 8px;
        cursor: pointer;
        font-size: 0.9rem;
        transition: background 0.15s ease;
    }

    .return-btn:hover {
        background: rgba(255, 255, 255, 0.15);
    }
</style>
