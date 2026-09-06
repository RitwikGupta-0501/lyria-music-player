<script lang="ts">
    import { onMount } from "svelte";
    import { 
        exploreStore, 
        AVAILABLE_SOURCES,
        type TrackResult, 
        type AlbumItem, 
        type ArtistItem, 
        type PlaylistItem, 
        type TopResultItem 
    } from "$lib/stores/explore.svelte";
    import AlbumCard from "$lib/components/AlbumCard.svelte";
    import PlaylistCard from "$lib/components/PlaylistCard.svelte";
    import VideoCard from "$lib/components/VideoCard.svelte";
    import { audioStore } from "$lib/stores/audio.svelte";
    import { settingsStore } from "$lib/stores/settings.svelte";
    import TrackRow from "$lib/components/TrackRow.svelte";
    import ArtistCard from "$lib/components/ArtistCard.svelte";
    import { formatDuration, getInitial, isCurrentTrack, getTrackDisplayMetric, sanitizeAlbumName } from "$lib/utils/format";
    import { 
        Play, 
        Pause, 
        Sparkle, 
        CaretRight, 
        CaretDown,
        Globe,
        Check,
        Playlist
    } from "phosphor-svelte";

    let isSourceMenuOpen = $state(false);

    function getCategoryFilterId(categoryName: string): string | null {
        const cat = categoryName.toLowerCase();
        if (cat.includes("song")) return "songs";
        if (cat.includes("album")) return "albums";
        if (cat.includes("artist")) return "artists";
        if (cat.includes("playlist")) return "playlists";
        if (cat.includes("video")) return "videos";
        return null;
    }

    const FILTER_CHIPS = [
        { id: "all", label: "All" },
        { id: "songs", label: "Songs" },
        { id: "albums", label: "Albums" },
        { id: "artists", label: "Artists" },
        { id: "playlists", label: "Playlists" },
    ];
    onMount(() => {
        const mainContent = document.querySelector('.main-content');
        if (mainContent) {
            mainContent.scrollTop = 0;
        }
    });
</script>

<div class="search-controls-row">
    <div class="search-filter-chips">
        {#each FILTER_CHIPS as chip}
            <button 
                class="filter-chip"
                class:active={exploreStore.activeSearchFilter === chip.id}
                onclick={() => exploreStore.setSearchFilter(chip.id)}
            >
                {chip.label}
            </button>
        {/each}
    </div>

    <!-- Multi-Select Source Filter -->
    <div class="custom-select-container">
        <button 
            class="select-trigger" 
            onclick={() => { isSourceMenuOpen = !isSourceMenuOpen; }}
            aria-expanded={isSourceMenuOpen}
        >
            <div class="select-trigger-left">
                <Globe size={14} weight="bold" />
                <span>Sources ({exploreStore.activeSourceFilters.length})</span>
            </div>
            <CaretDown size={13} weight="bold" class={isSourceMenuOpen ? 'rotated' : ''} />
        </button>

        {#if isSourceMenuOpen}
            <div class="source-dropdown-backdrop" role="button" tabindex="-1" onclick={() => { isSourceMenuOpen = false; }} onkeydown={() => { isSourceMenuOpen = false; }}></div>
            <div class="custom-select-menu">
                <div class="source-menu-header">Filter Sources</div>
                {#each AVAILABLE_SOURCES as src}
                    <button 
                        class="source-checkbox-item" 
                        onclick={() => exploreStore.toggleSourceFilter(src.id)}
                        role="checkbox"
                        aria-checked={exploreStore.isSourceActive(src.id)}
                    >
                        <div class="echo-checkbox" class:checked={exploreStore.isSourceActive(src.id)}>
                            {#if exploreStore.isSourceActive(src.id)}
                                <Check size={12} weight="bold" />
                            {/if}
                        </div>
                        <span class="source-item-name">{src.name}</span>
                        <span class="source-badge">{src.badge}</span>
                    </button>
                {/each}
            </div>
        {/if}
    </div>
</div>

<section class="search-results-canvas">
    <header class="search-results-header">
        <div class="search-title-row">
            <h2>Search Results for <span class="query-highlight">"{exploreStore.searchQuery}"</span></h2>
            {#if !exploreStore.isSearching}
                <span class="search-count-badge">{exploreStore.totalSearchResultCount} items found</span>
            {/if}
        </div>
    </header>

    {#if exploreStore.isSearching}
        <div class="search-loading-skeletons">
            {#each Array(6) as _}
                <div class="ledger-row skeleton">
                    <div class="skeleton-num skeleton-box"></div>
                    <div class="track-squircle skeleton-box"></div>
                    <div class="track-meta">
                        <div class="skeleton-line title"></div>
                        <div class="skeleton-line artist"></div>
                    </div>
                </div>
            {/each}
        </div>
    {:else if exploreStore.searchSections.length > 0}
        <div class="shelves-feed">
            {#each exploreStore.searchSections as section}
                {#if section.items.length > 0}
                    <div class="search-shelf">
                        {#if section.category !== "Top Result"}
                            <div class="shelf-header">
                                <h3>{section.category}</h3>
                                {#if exploreStore.activeSearchFilter === "all" && getCategoryFilterId(section.category)}
                                    <button 
                                        class="shelf-see-more-btn" 
                                        onclick={() => exploreStore.setSearchFilter(getCategoryFilterId(section.category)!)}
                                    >
                                        <span>See more</span>
                                        <CaretRight size={13} weight="bold" />
                                    </button>
                                {/if}
                            </div>
                        {/if}

                        <!-- A. Top Result Hero Card -->
                        {#if section.category === "Top Result"}
                            <div class="top-result-cluster">
                                {#each section.items as item}
                                    {#if item.type === "TopResult"}
                                        <div 
                                            class="top-result-card interactive" 
                                            class:is-glass={settingsStore.glassyPlayerBar}
                                            role="button"
                                            tabindex="0"
                                            onclick={() => {
                                                if (item.data.item_type === "album") {
                                                    exploreStore.openAlbum({
                                                        id: item.data.id,
                                                        title: item.data.title,
                                                        artist: item.data.subtitle,
                                                        cover_art_url: item.data.cover_art_url,
                                                        provider_id: item.data.provider_id || "youtube-wasm",
                                                    });
                                                } else if (item.data.item_type === "playlist") {
                                                    exploreStore.openPlaylist({
                                                        id: item.data.id,
                                                        title: item.data.title,
                                                        author: item.data.subtitle,
                                                        cover_art_url: item.data.cover_art_url,
                                                        provider_id: item.data.provider_id || "youtube-wasm",
                                                    });
                                                } else if (item.data.item_type === "artist") {
                                                    exploreStore.openArtist({
                                                        id: item.data.id,
                                                        name: item.data.title,
                                                        provider_id: item.data.provider_id || "youtube-wasm",
                                                    });
                                                } else {
                                                    exploreStore.playTrack({
                                                        id: item.data.id,
                                                        title: item.data.title,
                                                        artist: item.data.subtitle,
                                                        cover_art_url: item.data.cover_art_url,
                                                        provider_id: item.data.provider_id || "youtube-wasm",
                                                    });
                                                }
                                            }}
                                            onkeydown={(e) => {
                                                if (e.key === "Enter") {
                                                    if (item.data.item_type === "album") {
                                                        exploreStore.openAlbum({
                                                            id: item.data.id,
                                                            title: item.data.title,
                                                            artist: item.data.subtitle,
                                                            cover_art_url: item.data.cover_art_url,
                                                            provider_id: item.data.provider_id || "youtube-wasm",
                                                        });
                                                    } else if (item.data.item_type === "artist") {
                                                        exploreStore.openArtist({
                                                            id: item.data.id,
                                                            name: item.data.title,
                                                            provider_id: item.data.provider_id || "youtube-wasm",
                                                        });
                                                    } else {
                                                        exploreStore.playTrack({
                                                            id: item.data.id,
                                                            title: item.data.title,
                                                            artist: item.data.subtitle,
                                                            cover_art_url: item.data.cover_art_url,
                                                            provider_id: item.data.provider_id || "youtube-wasm",
                                                        });
                                                    }
                                                }
                                            }}
                                        >
                                            <div class="top-result-art-wrapper">
                                                {#if item.data.cover_art_url}
                                                    <img src={item.data.cover_art_url} alt={item.data.title} class="top-result-art" />
                                                {:else}
                                                    <div class="typographic-art-squircle large">
                                                        <span>{getInitial(item.data.title)}</span>
                                                    </div>
                                                {/if}
                                                <div class="top-result-play-overlay">
                                                    <button 
                                                        class="echo-play-btn"
                                                        onclick={(e) => {
                                                            e.stopPropagation();
                                                            if (item.data.item_type === "album") {
                                                                exploreStore.playAlbum({
                                                                    id: item.data.id,
                                                                    title: item.data.title,
                                                                    artist: item.data.subtitle,
                                                                    cover_art_url: item.data.cover_art_url,
                                                                    provider_id: item.data.provider_id || "youtube-wasm",
                                                                });
                                                            } else if (item.data.item_type === "artist") {
                                                                exploreStore.openArtist({
                                                                    id: item.data.id,
                                                                    name: item.data.title,
                                                                    provider_id: item.data.provider_id || "youtube-wasm",
                                                                });
                                                            } else {
                                                                exploreStore.playTrack({
                                                                    id: item.data.id,
                                                                    title: item.data.title,
                                                                    artist: item.data.subtitle,
                                                                    cover_art_url: item.data.cover_art_url,
                                                                    provider_id: item.data.provider_id || "youtube-wasm",
                                                                });
                                                            }
                                                        }}
                                                    >
                                                        <Play size={22} weight="fill" />
                                                    </button>
                                                </div>
                                            </div>
                                            <div class="top-result-info">
                                                <div class="top-result-eyebrow">
                                                    <Sparkle size={13} weight="fill" />
                                                    <span>BEST MATCH • {item.data.item_type.toUpperCase()}</span>
                                                    {#if item.data.provider_name}
                                                        <span class="provider-tag">{item.data.provider_name}</span>
                                                    {/if}
                                                </div>
                                                <h2 class="top-result-title">{item.data.title}</h2>
                                                <p class="top-result-sub">
                                                    {#if item.data.item_type !== "artist" && item.data.subtitle}
                                                        <button 
                                                            class="artist-inline-btn"
                                                            onclick={(e) => {
                                                                e.stopPropagation();
                                                                exploreStore.openArtist({
                                                                    id: item.data.subtitle,
                                                                    name: item.data.subtitle,
                                                                });
                                                            }}
                                                        >
                                                            {item.data.subtitle}
                                                        </button>
                                                    {:else}
                                                        {item.data.subtitle}
                                                    {/if}
                                                </p>
                                            </div>
                                        </div>
                                    {:else if item.type === "Track"}
                                        <TrackRow 
                                            track={item.data}
                                            showProviderTag={true}
                                        />
                                    {/if}
                                {/each}
                            </div>

                        <!-- B. Videos Shelf (16:9 VideoCard grid) -->
                        {:else if section.category === "Videos"}
                            <div class="videos-search-grid">
                                {#each (exploreStore.activeSearchFilter === "all" ? section.items.slice(0, 4) : section.items) as item}
                                    {#if item.type === "Track"}
                                        <VideoCard 
                                            video={item.data}
                                            onclick={() => exploreStore.playTrack(item.data, item.data.provider_id)}
                                            onplay={() => exploreStore.playTrack(item.data, item.data.provider_id)}
                                        />
                                    {/if}
                                {/each}
                            </div>

                        <!-- C. Songs Shelf -->
                        {:else if section.category === "Songs"}
                            <div class="search-tracks-ledger">
                                {#each (exploreStore.activeSearchFilter === "all" ? section.items.slice(0, 8) : section.items) as item, index}
                                    {#if item.type === "Track"}
                                        <TrackRow 
                                            track={item.data}
                                            index={index + 1}
                                            showProviderTag={true}
                                        />
                                    {/if}
                                {/each}
                            </div>

                        <!-- C. Albums Shelf -->
                        {:else if section.category === "Albums"}
                            <div class="albums-search-grid">
                                {#each (exploreStore.activeSearchFilter === "all" ? section.items.slice(0, 6) : section.items) as item}
                                    {#if item.type === "Album"}
                                        <AlbumCard 
                                            album={item.data}
                                            onclick={() => exploreStore.openAlbum(item.data)}
                                        />
                                    {/if}
                                {/each}
                            </div>

                        <!-- D. Artists Shelf -->
                        {:else if section.category === "Artists"}
                            <div class="artists-search-grid">
                                {#each (exploreStore.activeSearchFilter === "all" ? section.items.slice(0, 6) : section.items) as item}
                                    {#if item.type === "Artist"}
                                        <ArtistCard 
                                            artist={item.data}
                                            onclick={() => exploreStore.openArtist(item.data)}
                                            fluid
                                        />
                                    {/if}
                                {/each}
                            </div>

                        <!-- E. Playlists Shelf -->
                        {:else if section.category === "Playlists"}
                            <div class="playlists-search-grid">
                                {#each section.items as item}
                                    {#if item.type === "Playlist"}
                                        <PlaylistCard 
                                            playlist={item.data} 
                                            onclick={() => exploreStore.openPlaylist({
                                                id: item.data.id,
                                                title: item.data.title,
                                                author: item.data.author || undefined,
                                                cover_art_url: item.data.cover_art_url,
                                                provider_id: item.data.provider_id,
                                            })}
                                        />
                                    {/if}
                                {/each}
                            </div>
                        {/if}
                    </div>
                {/if}
            {/each}
        </div>
    {:else}
        <div class="search-empty-state">
            <p>No results found matching "{exploreStore.searchQuery}"</p>
        </div>
    {/if}
</section>

<style>

    .albums-search-grid,
    .playlists-search-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(170px, 1fr));
        gap: 1.25rem;
        width: 100%;
    }

    .artists-search-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
        gap: 1.5rem;
        width: 100%;
    }

    .videos-search-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
        gap: 1.25rem;
        width: 100%;
    }

    .search-controls-row {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 1rem;
        width: 100%;
    }
    .search-filter-chips {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        overflow-x: auto;
    }
    .filter-chip {
        padding: 0.35rem 0.85rem;
        border-radius: 20px;
        background: rgba(255, 255, 255, 0.04);
        border: 1px solid rgba(255, 255, 255, 0.08);
        color: rgba(255, 255, 255, 0.7);
        font-size: 0.82rem;
        font-weight: 500;
        cursor: pointer;
        transition: all 0.15s ease;
    }
    .filter-chip:hover {
        background: rgba(255, 255, 255, 0.08);
        color: #FFFFFF;
    }
    .filter-chip.active {
        background: #B58E62;
        color: #000000;
        border-color: #B58E62;
        font-weight: 600;
    }
    .custom-select-container {
        position: relative;
        flex-shrink: 0;
    }
    .select-trigger {
        display: flex;
        align-items: center;
        gap: 0.65rem;
        background: var(--echo-surface, #161618);
        border: 1px solid var(--echo-border-medium, rgba(255, 255, 255, 0.1));
        color: var(--echo-text-1, #FFFFFF);
        padding: 0.42rem 0.85rem;
        border-radius: 8px;
        font-size: 0.82rem;
        font-weight: 500;
        cursor: pointer;
        transition: all 0.2s ease;
    }
    .select-trigger:hover {
        border-color: rgba(181, 142, 98, 0.4);
        background: rgba(255, 255, 255, 0.04);
    }
    .select-trigger-left {
        display: flex;
        align-items: center;
        gap: 0.45rem;
        color: #B58E62;
    }
    .select-trigger-left span {
        color: var(--echo-text-1, #FFFFFF);
    }
    :global(.select-trigger svg.rotated) {
        transform: rotate(180deg);
        color: #B58E62;
    }
    .source-dropdown-backdrop {
        position: fixed;
        inset: 0;
        z-index: 100;
    }
    .custom-select-menu {
        position: absolute;
        top: calc(100% + 0.5rem);
        right: 0;
        width: 220px;
        display: flex;
        flex-direction: column;
        background: var(--echo-surface, #161618);
        border: 1px solid var(--echo-border-medium, rgba(255, 255, 255, 0.12));
        border-radius: 8px;
        padding: 0.35rem;
        z-index: 101;
        box-shadow: 0 10px 30px -10px rgba(0, 0, 0, 0.7);
    }
    .source-menu-header {
        font-family: ui-monospace, monospace;
        font-size: 0.65rem;
        font-weight: 700;
        text-transform: uppercase;
        color: var(--echo-text-2, rgba(255, 255, 255, 0.4));
        padding: 0.3rem 0.5rem;
        letter-spacing: 0.06em;
    }
    .source-checkbox-item {
        display: flex;
        align-items: center;
        gap: 0.65rem;
        width: 100%;
        background: transparent;
        border: none;
        padding: 0.5rem 0.6rem;
        border-radius: 6px;
        color: var(--echo-text-2, rgba(255, 255, 255, 0.7));
        cursor: pointer;
        transition: background 0.12s ease;
        text-align: left;
    }
    .source-checkbox-item:hover {
        background: rgba(255, 255, 255, 0.05);
        color: var(--echo-text-1, #FFFFFF);
    }
    .echo-checkbox {
        width: 17px;
        height: 17px;
        border-radius: 4px;
        border: 1.5px solid rgba(255, 255, 255, 0.25);
        display: flex;
        align-items: center;
        justify-content: center;
        background: transparent;
        transition: all 0.15s ease;
        flex-shrink: 0;
    }
    .echo-checkbox.checked {
        background: #B58E62;
        border-color: #B58E62;
        color: #000000;
    }
    .source-item-name {
        flex: 1;
        font-size: 0.82rem;
        color: inherit;
        font-weight: 500;
    }
    .source-badge {
        font-size: 0.58rem;
        font-family: ui-monospace, monospace;
        font-weight: 700;
        padding: 0.08rem 0.32rem;
        border-radius: 3px;
        background: rgba(181, 142, 98, 0.15);
        color: #B58E62;
        border: 1px solid rgba(181, 142, 98, 0.3);
    }
    .search-results-canvas {
        display: flex;
        flex-direction: column;
        gap: 1.8rem;
    }
    .search-title-row {
        display: flex;
        align-items: center;
        justify-content: space-between;
    }
    .search-title-row h2 {
        font-size: 1.4rem;
        font-weight: 700;
        margin: 0;
    }
    .query-highlight {
        color: #B58E62;
    }
    .search-count-badge {
        font-size: 0.82rem;
        color: rgba(255, 255, 255, 0.5);
    }
    .shelves-feed {
        display: flex;
        flex-direction: column;
        gap: 2.2rem;
    }
    .shelf-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        margin-bottom: 0.9rem;
    }
    .shelf-header h3 {
        font-size: 1.15rem;
        font-weight: 700;
        margin: 0;
    }
    .shelf-see-more-btn {
        display: flex;
        align-items: center;
        gap: 0.3rem;
        background: transparent;
        border: none;
        color: #B58E62;
        font-size: 0.78rem;
        font-weight: 700;
        cursor: pointer;
        padding: 0.2rem 0.5rem;
        border-radius: 4px;
        transition: all 0.15s ease;
    }
    .shelf-see-more-btn:hover {
        background: rgba(181, 142, 98, 0.12);
        color: #fff;
    }
    .top-result-cluster {
        display: grid;
        grid-template-columns: 1fr 1.2fr;
        gap: 1.5rem;
    }
    @media (max-width: 1024px) {
        .top-result-cluster {
            grid-template-columns: 1fr;
        }
    }
    .top-result-card {
        display: flex;
        align-items: center;
        gap: 1.5rem;
        padding: 1.5rem;
        background: #151518;
        border: 1px solid rgba(255, 255, 255, 0.08);
        border-radius: 14px;
        position: relative;
    }
    .top-result-card.is-glass {
        background: rgba(25, 25, 30, 0.45);
        backdrop-filter: blur(16px);
    }
    .top-result-card.interactive {
        cursor: pointer;
        transition: transform 0.2s ease, border-color 0.2s ease;
    }
    .top-result-card.interactive:hover {
        transform: translateY(-2px);
        border-color: rgba(181, 142, 98, 0.4);
    }
    .top-result-art-wrapper {
        position: relative;
        width: 120px;
        height: 120px;
        border-radius: 12px;
        overflow: hidden;
        flex-shrink: 0;
    }
    .top-result-art {
        width: 100%;
        height: 100%;
        object-fit: cover;
    }
    .top-result-play-overlay {
        position: absolute;
        inset: 0;
        background: rgba(0, 0, 0, 0.35);
        opacity: 0;
        transition: opacity 0.2s ease;
        display: flex;
        align-items: center;
        justify-content: center;
    }
    .top-result-card:hover .top-result-play-overlay {
        opacity: 1;
    }
    .echo-play-btn {
        width: 48px;
        height: 48px;
        border-radius: 50%;
        background: #B58E62;
        color: #000;
        display: flex;
        align-items: center;
        justify-content: center;
        border: none;
        cursor: pointer;
        box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
    }
    .top-result-info {
        display: flex;
        flex-direction: column;
        gap: 0.4rem;
        min-width: 0;
    }
    .top-result-eyebrow {
        display: flex;
        align-items: center;
        gap: 0.4rem;
        font-family: ui-monospace, monospace;
        font-size: 0.68rem;
        font-weight: 700;
        color: #B58E62;
    }
    .provider-tag {
        font-size: 0.62rem;
        padding: 0.1rem 0.35rem;
        border-radius: 3px;
        background: rgba(255, 255, 255, 0.08);
        color: rgba(255, 255, 255, 0.6);
    }
    .top-result-title {
        font-size: 1.6rem;
        font-weight: 700;
        margin: 0;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }
    .top-result-sub {
        font-size: 0.95rem;
        color: rgba(255, 255, 255, 0.6);
        margin: 0;
    }
    .artist-inline-btn {
        background: none;
        border: none;
        color: inherit;
        font: inherit;
        padding: 0;
        cursor: pointer;
    }
    .artist-inline-btn:hover {
        color: #B58E62;
        text-decoration: underline;
    }
    .search-tracks-ledger {
        display: flex;
        flex-direction: column;
        gap: 0.4rem;
    }



    .search-empty-state {
        padding: 3rem 0;
        text-align: center;
        color: rgba(255, 255, 255, 0.5);
        font-size: 0.95rem;
    }
</style>
