<script lang="ts">
    import { 
        exploreStore, 
        AVAILABLE_SOURCES,
        type TrackResult, 
        type AlbumItem, 
        type ArtistItem, 
        type PlaylistItem, 
        type TopResultItem 
    } from "$lib/stores/explore.svelte";
    import { audioStore } from "$lib/stores/audio.svelte";
    import { settingsStore } from "$lib/stores/settings.svelte";
    import { formatDuration, getInitial, isCurrentTrack } from "$lib/utils/format";
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
                                        <div 
                                            class="ledger-row sub-track-row"
                                            class:active-track={isCurrentTrack(item.data, audioStore.currentQueueTrack)}
                                            role="button"
                                            tabindex="0"
                                            ondblclick={() => exploreStore.playTrack(item.data, item.data.provider_id)}
                                            onkeydown={(e) => { if (e.key === 'Enter') exploreStore.playTrack(item.data, item.data.provider_id); }}
                                        >
                                            <div class="track-art-wrapper">
                                                {#if item.data.cover_art_url}
                                                    <img src={item.data.cover_art_url} alt={item.data.title} class="track-squircle" />
                                                {:else}
                                                    <div class="typographic-art-squircle">
                                                        <span>{getInitial(item.data.artist)}</span>
                                                    </div>
                                                {/if}
                                                <button class="play-overlay-btn" onclick={() => exploreStore.playTrack(item.data, item.data.provider_id)}>
                                                    {#if isCurrentTrack(item.data, audioStore.currentQueueTrack) && audioStore.playbackState === "Playing"}
                                                        <Pause size={14} weight="fill" />
                                                    {:else}
                                                        <Play size={14} weight="fill" />
                                                    {/if}
                                                </button>
                                            </div>

                                            <div class="track-meta">
                                                <span class="track-title">{item.data.title}</span>
                                                <div class="track-subline">
                                                    <span class="track-artist">{item.data.artist}</span>
                                                    {#if item.data.album}
                                                        <span class="track-dot">•</span>
                                                        <span class="track-album">{item.data.album}</span>
                                                    {/if}
                                                </div>
                                            </div>

                                            <span class="track-duration">{formatDuration(item.data.duration_ms)}</span>
                                        </div>
                                    {/if}
                                {/each}
                            </div>

                        <!-- B. Songs / Videos Shelf -->
                        {:else if section.category === "Songs" || section.category === "Videos"}
                            <div class="search-tracks-ledger">
                                {#each (exploreStore.activeSearchFilter === "all" ? section.items.slice(0, 8) : section.items) as item, index}
                                    {#if item.type === "Track"}
                                        <div 
                                            class="ledger-row"
                                            class:active-track={isCurrentTrack(item.data, audioStore.currentQueueTrack)}
                                            role="button"
                                            tabindex="0"
                                            ondblclick={() => exploreStore.playTrack(item.data, item.data.provider_id)}
                                            onkeydown={(e) => { if (e.key === 'Enter') exploreStore.playTrack(item.data, item.data.provider_id); }}
                                        >
                                            <span class="ledger-num">{(index + 1).toString().padStart(2, '0')}</span>
                                            
                                            <div class="track-art-wrapper">
                                                {#if item.data.cover_art_url}
                                                    <img src={item.data.cover_art_url} alt={item.data.title} class="track-squircle" />
                                                {:else}
                                                    <div class="typographic-art-squircle">
                                                        <span>{getInitial(item.data.artist)}</span>
                                                    </div>
                                                {/if}
                                                <button class="play-overlay-btn" onclick={() => exploreStore.playTrack(item.data, item.data.provider_id)}>
                                                    {#if isCurrentTrack(item.data, audioStore.currentQueueTrack) && audioStore.playbackState === "Playing"}
                                                        <Pause size={14} weight="fill" />
                                                    {:else}
                                                        <Play size={14} weight="fill" />
                                                    {/if}
                                                </button>
                                            </div>

                                            <div class="track-meta">
                                                <span class="track-title">{item.data.title}</span>
                                                <div class="track-subline">
                                                    <span class="track-artist">{item.data.artist}</span>
                                                    {#if item.data.album}
                                                        <span class="track-dot">•</span>
                                                        <span class="track-album">{item.data.album}</span>
                                                    {/if}
                                                </div>
                                            </div>

                                            {#if item.data.provider_name}
                                                <span class="provider-tag">{item.data.provider_name}</span>
                                            {/if}

                                            <span class="track-duration">{formatDuration(item.data.duration_ms)}</span>
                                        </div>
                                    {/if}
                                {/each}
                            </div>

                        <!-- C. Albums Shelf -->
                        {:else if section.category === "Albums"}
                            <div class="albums-search-grid">
                                {#each (exploreStore.activeSearchFilter === "all" ? section.items.slice(0, 5) : section.items) as item}
                                    {#if item.type === "Album"}
                                        <div 
                                            class="echo-album-card"
                                            role="button"
                                            tabindex="0"
                                            onclick={() => exploreStore.openAlbum(item.data)}
                                            onkeydown={(e) => { if (e.key === 'Enter') exploreStore.openAlbum(item.data); }}
                                        >
                                            <div class="echo-album-art-container">
                                                {#if item.data.cover_art_url}
                                                    <img src={item.data.cover_art_url} alt={item.data.title} class="echo-album-img" />
                                                {:else}
                                                    <div class="typographic-art-squircle large">
                                                        <span>{getInitial(item.data.artist)}</span>
                                                    </div>
                                                {/if}
                                                <div class="echo-album-overlay">
                                                    <button 
                                                        class="echo-play-btn"
                                                        onclick={(e) => {
                                                            e.stopPropagation();
                                                            exploreStore.playAlbum(item.data);
                                                        }}
                                                    >
                                                        <Play size={24} weight="fill" />
                                                    </button>
                                                </div>
                                            </div>
                                            <div class="echo-album-meta">
                                                <span class="echo-album-title" title={item.data.title}>{item.data.title}</span>
                                                <span class="echo-album-artist" title={item.data.artist}>
                                                    <button 
                                                        class="artist-inline-btn"
                                                        onclick={(e) => {
                                                            e.stopPropagation();
                                                            exploreStore.openArtist({
                                                                id: item.data.artist,
                                                                name: item.data.artist,
                                                            });
                                                        }}
                                                    >
                                                        {item.data.artist}
                                                    </button>
                                                    {#if item.data.year} • {item.data.year}{/if}
                                                </span>
                                            </div>
                                        </div>
                                    {/if}
                                {/each}
                            </div>

                        <!-- D. Artists Shelf -->
                        {:else if section.category === "Artists"}
                            <div class="artists-search-grid">
                                {#each (exploreStore.activeSearchFilter === "all" ? section.items.slice(0, 6) : section.items) as item}
                                    {#if item.type === "Artist"}
                                        <div 
                                            class="artist-card interactive-artist-card"
                                            role="button"
                                            tabindex="0"
                                            onclick={() => exploreStore.openArtist(item.data)}
                                            onkeydown={(e) => { if (e.key === 'Enter') exploreStore.openArtist(item.data); }}
                                        >
                                            <div class="artist-avatar-wrapper">
                                                {#if item.data.avatar_url}
                                                    <img src={item.data.avatar_url} alt={item.data.name} class="artist-avatar" />
                                                {:else}
                                                    <div class="artist-avatar-fallback">
                                                        <span>{getInitial(item.data.name)}</span>
                                                    </div>
                                                {/if}
                                            </div>
                                            <span class="artist-name">{item.data.name}</span>
                                            <span class="artist-sub">{item.data.subscribers || "Artist"}</span>
                                        </div>
                                    {/if}
                                {/each}
                            </div>

                        <!-- E. Playlists Shelf -->
                        {:else if section.category === "Playlists"}
                            <div class="albums-search-grid">
                                {#each section.items as item}
                                    {#if item.type === "Playlist"}
                                        <div class="echo-album-card">
                                            <div class="echo-album-art-container">
                                                {#if item.data.cover_art_url}
                                                    <img src={item.data.cover_art_url} alt={item.data.title} class="echo-album-img" />
                                                {:else}
                                                    <div class="typographic-art-squircle large">
                                                        <Playlist size={32} />
                                                    </div>
                                                {/if}
                                            </div>
                                            <div class="echo-album-meta">
                                                <span class="echo-album-title" title={item.data.title}>{item.data.title}</span>
                                                <span class="echo-album-artist" title={item.data.author || "Curated Playlist"}>{item.data.author || "Curated Playlist"}</span>
                                            </div>
                                        </div>
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
    .ledger-row {
        display: flex;
        align-items: center;
        gap: 0.9rem;
        padding: 0.6rem 0.75rem;
        border-radius: 8px;
        background: rgba(255, 255, 255, 0.02);
        border: 1px solid transparent;
        cursor: pointer;
        transition: background-color 0.15s ease, border-color 0.15s ease;
    }
    .ledger-row:hover {
        background: rgba(255, 255, 255, 0.06);
        border-color: rgba(255, 255, 255, 0.08);
    }
    .ledger-row.active-track {
        background: rgba(181, 142, 98, 0.12);
        border-color: rgba(181, 142, 98, 0.3);
    }
    .ledger-num {
        font-family: ui-monospace, monospace;
        font-size: 0.9rem;
        font-weight: 700;
        color: #B58E62;
        min-width: 22px;
    }
    .track-art-wrapper {
        position: relative;
        width: 40px;
        height: 40px;
        flex-shrink: 0;
        border-radius: 6px;
        overflow: hidden;
    }
    .track-squircle {
        width: 100%;
        height: 100%;
        object-fit: cover;
    }
    .typographic-art-squircle {
        width: 100%;
        height: 100%;
        background: #232328;
        display: flex;
        align-items: center;
        justify-content: center;
        color: #B58E62;
        font-family: ui-serif, Georgia, serif;
        font-weight: 700;
    }
    .typographic-art-squircle.large {
        font-size: 2.2rem;
    }
    .play-overlay-btn {
        position: absolute;
        inset: 0;
        background: rgba(0, 0, 0, 0.6);
        border: none;
        color: #fff;
        display: flex;
        align-items: center;
        justify-content: center;
        opacity: 0;
        cursor: pointer;
        transition: opacity 0.15s ease;
    }
    .ledger-row:hover .play-overlay-btn, .ledger-row.active-track .play-overlay-btn {
        opacity: 1;
    }
    .track-meta {
        flex: 1;
        min-width: 0;
        display: flex;
        flex-direction: column;
        gap: 0.15rem;
    }
    .track-title {
        font-size: 0.92rem;
        font-weight: 600;
        color: #FFFFFF;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }
    .track-subline {
        display: flex;
        align-items: center;
        gap: 0.35rem;
        font-size: 0.8rem;
        color: rgba(255, 255, 255, 0.55);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }
    .track-duration {
        font-family: ui-monospace, monospace;
        font-size: 0.78rem;
        color: rgba(255, 255, 255, 0.45);
    }
    .albums-search-grid {
        display: grid;
        grid-template-columns: repeat(5, 1fr);
        gap: 1.2rem;
    }
    @media (max-width: 1100px) {
        .albums-search-grid {
            grid-template-columns: repeat(3, 1fr);
        }
    }
    .echo-album-card {
        display: flex;
        flex-direction: column;
        gap: 0.6rem;
        cursor: pointer;
    }
    .echo-album-art-container {
        position: relative;
        width: 100%;
        aspect-ratio: 1/1;
        border-radius: 10px;
        overflow: hidden;
        background: #1E1E22;
    }
    .echo-album-img {
        width: 100%;
        height: 100%;
        object-fit: cover;
    }
    .echo-album-overlay {
        position: absolute;
        inset: 0;
        background: rgba(0, 0, 0, 0.4);
        display: flex;
        align-items: center;
        justify-content: center;
        opacity: 0;
        transition: opacity 0.2s ease;
    }
    .echo-album-card:hover .echo-album-overlay {
        opacity: 1;
    }
    .echo-album-meta {
        display: flex;
        flex-direction: column;
        gap: 0.15rem;
    }
    .echo-album-title {
        font-size: 0.9rem;
        font-weight: 600;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }
    .echo-album-artist {
        font-size: 0.8rem;
        color: rgba(255, 255, 255, 0.55);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }
    .artists-search-grid {
        display: grid;
        grid-template-columns: repeat(6, 1fr);
        gap: 1.2rem;
    }
    .artist-card {
        display: flex;
        flex-direction: column;
        align-items: center;
        text-align: center;
        gap: 0.6rem;
        cursor: pointer;
        transition: transform 0.18s ease;
    }
    .artist-card:hover {
        transform: translateY(-3px);
    }
    .artist-avatar-wrapper {
        width: 90px;
        height: 90px;
        border-radius: 50%;
        overflow: hidden;
        background: #232328;
    }
    .artist-avatar {
        width: 100%;
        height: 100%;
        object-fit: cover;
    }
    .artist-avatar-fallback {
        width: 100%;
        height: 100%;
        display: flex;
        align-items: center;
        justify-content: center;
        font-family: ui-serif, Georgia, serif;
        font-weight: 700;
        font-size: 1.8rem;
        color: #B58E62;
    }
    .artist-name {
        font-size: 0.88rem;
        font-weight: 600;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        max-width: 100%;
    }
    .artist-sub {
        font-size: 0.75rem;
        color: rgba(255, 255, 255, 0.5);
    }
    .search-empty-state {
        padding: 3rem 0;
        text-align: center;
        color: rgba(255, 255, 255, 0.5);
        font-size: 0.95rem;
    }
</style>
