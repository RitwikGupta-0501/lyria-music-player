<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import { fade } from "svelte/transition";
    import { 
        exploreStore, 
        type TrackResult, 
        type AlbumItem, 
        type GenreItem, 
        type ArtistItem, 
        type PlaylistItem, 
        type TopResultItem 
    } from "$lib/stores/explore.svelte";
    import { audioStore } from "$lib/stores/audio.svelte";
    import { settingsStore } from "$lib/stores/settings.svelte";
    import { 
        MagnifyingGlass, 
        Play, 
        Pause, 
        ArrowLeft, 
        ArrowClockwise, 
        Sparkle, 
        CaretLeft, 
        CaretRight, 
        X,
        MusicNotes,
        Disc,
        User,
        Playlist
    } from "phosphor-svelte";

    let { activeView = $bindable("explore") } = $props<{ activeView?: string }>();

    let autoScrollInterval: ReturnType<typeof setInterval> | null = null;
    let isHoveringHero = $state(false);

    const FILTER_CHIPS = [
        { id: "all", label: "All" },
        { id: "songs", label: "Songs" },
        { id: "albums", label: "Albums" },
        { id: "artists", label: "Artists" },
        { id: "playlists", label: "Playlists" },
    ];

    function startAutoScroll() {
        stopAutoScroll();
        autoScrollInterval = setInterval(() => {
            if (!isHoveringHero && exploreStore.spotlights.length > 1 && !exploreStore.searchQuery) {
                exploreStore.nextSpotlight();
            }
        }, 7000);
    }

    function stopAutoScroll() {
        if (autoScrollInterval) {
            clearInterval(autoScrollInterval);
            autoScrollInterval = null;
        }
    }

    onMount(() => {
        exploreStore.init();
        startAutoScroll();

        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === "Escape") {
                if (exploreStore.searchQuery) {
                    exploreStore.clearSearch();
                } else if (exploreStore.activeCategory) {
                    exploreStore.closeCategory();
                }
            }
        };

        window.addEventListener("keydown", handleKeyDown);
        return () => {
            window.removeEventListener("keydown", handleKeyDown);
            stopAutoScroll();
        };
    });

    onDestroy(() => {
        stopAutoScroll();
    });

    function isCurrentTrack(track: TrackResult): boolean {
        const cur = audioStore.currentQueueTrack;
        if (!cur) return false;
        return cur.title.toLowerCase() === track.title.toLowerCase()
            && (cur.artist || "").toLowerCase() === track.artist.toLowerCase();
    }

    function formatDuration(ms: number | null | undefined): string {
        if (!ms) return "03:30";
        const totalSecs = Math.floor(ms / 1000);
        const mins = Math.floor(totalSecs / 60);
        const secs = totalSecs % 60;
        return `${mins}:${secs.toString().padStart(2, '0')}`;
    }

    function getInitial(artist: string | undefined): string {
        if (!artist || artist.trim().length === 0) return "E";
        return artist.trim().charAt(0).toUpperCase();
    }
</script>

<div class="explore-canvas">
    <!-- Top Discovery Bar -->
    <div class="discovery-bar-wrapper">
        <div class="discovery-bar">
            <MagnifyingGlass size={18} weight="bold" color="#B58E62" />
            <input 
                type="text" 
                placeholder="Search songs, albums, artists, or providers..." 
                value={exploreStore.searchQuery}
                oninput={(e) => exploreStore.setSearchQuery((e.target as HTMLInputElement).value)}
                class="discovery-input"
            />
            {#if exploreStore.searchQuery}
                <button class="clear-search-btn" onclick={() => exploreStore.clearSearch()} title="Clear search (Esc)">
                    <X size={16} weight="bold" />
                </button>
            {/if}
            <button 
                class="refresh-btn" 
                class:spinning={exploreStore.isLoading || exploreStore.isSearching} 
                onclick={() => {
                    if (exploreStore.searchQuery) {
                        exploreStore.performSearch(exploreStore.searchQuery);
                    } else {
                        exploreStore.loadExplore(true);
                    }
                }}
                title="Refresh"
            >
                <ArrowClockwise size={16} weight="bold" />
            </button>
        </div>

        <!-- Filter Chips (Visible when searching) -->
        {#if exploreStore.searchQuery.trim().length > 0}
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
        {/if}
    </div>

    <!-- 1. DEDICATED CATEGORIZED SEARCH RESULTS CANVAS -->
    {#if exploreStore.searchQuery.trim().length > 0}
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
                <!-- Multi-Shelf Categorized Feed -->
                <div class="shelves-feed">
                    {#each exploreStore.searchSections as section}
                        {#if section.items.length > 0}
                            <div class="search-shelf">
                                <div class="shelf-header">
                                    <h3>{section.category}</h3>
                                    <span class="shelf-count">{section.items.length}</span>
                                </div>

                                <!-- A. Top Result Hero Card -->
                                {#if section.category === "Top Result"}
                                    <div class="top-result-cluster">
                                        {#each section.items as item}
                                            {#if item.type === "TopResult"}
                                                <div class="top-result-card" class:is-glass={settingsStore.glassyPlayerBar}>
                                                    <div class="top-result-art-wrapper">
                                                        {#if item.data.cover_art_url}
                                                            <img src={item.data.cover_art_url} alt={item.data.title} class="top-result-art" />
                                                        {:else}
                                                            <div class="typographic-art-squircle large">
                                                                <span>{getInitial(item.data.title)}</span>
                                                            </div>
                                                        {/if}
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
                                                        <p class="top-result-sub">{item.data.subtitle}</p>
                                                    </div>
                                                </div>
                                            {:else if item.type === "Track"}
                                                <div 
                                                    class="ledger-row sub-track-row"
                                                    class:active-track={isCurrentTrack(item.data)}
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
                                                            {#if isCurrentTrack(item.data) && audioStore.playbackState === "Playing"}
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

                                <!-- B. Songs / Videos Shelf (Ranked Ledger) -->
                                {:else if section.category === "Songs" || section.category === "Videos"}
                                    <div class="search-tracks-ledger">
                                        {#each section.items as item, index}
                                            {#if item.type === "Track"}
                                                <div 
                                                    class="ledger-row"
                                                    class:active-track={isCurrentTrack(item.data)}
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
                                                            {#if isCurrentTrack(item.data) && audioStore.playbackState === "Playing"}
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

                                <!-- C. Albums Shelf (Uniform Symmetrical Grid) -->
                                {:else if section.category === "Albums"}
                                    <div class="albums-search-grid">
                                        {#each section.items as item}
                                            {#if item.type === "Album"}
                                                <div 
                                                    class="echo-album-card"
                                                    role="button"
                                                    tabindex="0"
                                                    onclick={() => exploreStore.playAlbum(item.data)}
                                                    onkeydown={(e) => { if (e.key === 'Enter') exploreStore.playAlbum(item.data); }}
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
                                                            <div class="echo-play-btn">
                                                                <Play size={24} weight="fill" />
                                                            </div>
                                                        </div>
                                                    </div>
                                                    <div class="echo-album-meta">
                                                        <span class="echo-album-title" title={item.data.title}>{item.data.title}</span>
                                                        <span class="echo-album-artist" title={item.data.artist}>{item.data.artist} {#if item.data.year}• {item.data.year}{/if}</span>
                                                    </div>
                                                </div>
                                            {/if}
                                        {/each}
                                    </div>

                                <!-- D. Artists Shelf (Circular Avatars) -->
                                {:else if section.category === "Artists"}
                                    <div class="artists-search-grid">
                                        {#each section.items as item}
                                            {#if item.type === "Artist"}
                                                <div class="artist-card">
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
                <div class="empty-search-state">
                    <p>No matches found for "{exploreStore.searchQuery}".</p>
                    <button class="clear-search-link" onclick={() => exploreStore.clearSearch()}>Clear search to return to Explore</button>
                </div>
            {/if}
        </section>

    <!-- 2. CATEGORY HUB SUB-VIEW -->
    {:else if exploreStore.activeCategory}
        <section class="category-hub-canvas">
            <header class="hub-header">
                <button class="hub-back-btn" onclick={() => exploreStore.closeCategory()} title="Back to Explore (Esc)">
                    <ArrowLeft size={18} weight="bold" />
                    <span>Back to Explore</span>
                </button>
                <div class="hub-title-row">
                    <div class="hub-color-badge" style:background={exploreStore.activeCategory.color_hex || '#B58E62'}></div>
                    <h1>{exploreStore.activeCategory.title}</h1>
                </div>
                <p class="hub-subtitle">Curated editorial tracks, featured mixes & genre channels</p>
            </header>

            {#if exploreStore.isCategoryLoading}
                <div class="hub-tracks-grid">
                    {#each Array(8) as _}
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
            {:else if exploreStore.categoryTracks.length > 0}
                <div class="hub-tracks-grid">
                    {#each exploreStore.categoryTracks as track, index}
                        <div 
                            class="ledger-row"
                            class:active-track={isCurrentTrack(track)}
                            role="button"
                            tabindex="0"
                            ondblclick={() => exploreStore.playTrack(track, track.provider_id)}
                            onkeydown={(e) => { if (e.key === 'Enter') exploreStore.playTrack(track, track.provider_id); }}
                        >
                            <span class="ledger-num">{(index + 1).toString().padStart(2, '0')}</span>
                            
                            <div class="track-art-wrapper">
                                {#if track.cover_art_url}
                                    <img src={track.cover_art_url} alt={track.title} class="track-squircle" />
                                {:else}
                                    <div class="typographic-art-squircle">
                                        <span>{getInitial(track.artist)}</span>
                                    </div>
                                {/if}
                                <button class="play-overlay-btn" onclick={() => exploreStore.playTrack(track, track.provider_id)}>
                                    {#if isCurrentTrack(track) && audioStore.playbackState === "Playing"}
                                        <Pause size={14} weight="fill" />
                                    {:else}
                                        <Play size={14} weight="fill" />
                                    {/if}
                                </button>
                            </div>

                            <div class="track-meta">
                                <span class="track-title">{track.title}</span>
                                <span class="track-artist">{track.artist}</span>
                            </div>

                            <span class="track-duration">{formatDuration(track.duration_ms)}</span>
                        </div>
                    {/each}
                </div>
            {:else}
                <div class="empty-state">No tracks found for {exploreStore.activeCategory.title}.</div>
            {/if}
        </section>

    <!-- 3. ROOT CURATED EXPLORE CANVAS (Editorial Spotlight, Categories, Discovery Split) -->
    {:else}
        <!-- FEATURED SPOTLIGHT (16:7 Rotating Editorial Banner with Crossfade) -->
        {#if exploreStore.activeSpotlight}
            <section class="spotlight-banner-wrapper">
                <div 
                    class="spotlight-banner"
                    class:is-glass={settingsStore.glassyPlayerBar}
                    onmouseenter={() => isHoveringHero = true}
                    onmouseleave={() => isHoveringHero = false}
                    role="region"
                    aria-label="Editorial Spotlight Banner"
                >
                    {#key exploreStore.activeSpotlightIndex}
                        <div class="spotlight-slide" in:fade={{ duration: 350 }}>
                            <div 
                                class="spotlight-backdrop"
                                style:--hero-art="url('{exploreStore.activeSpotlight.cover_art_url || ''}')"
                            ></div>
                            
                            <div class="spotlight-content">
                                <div class="spotlight-eyebrow">
                                    <Sparkle size={14} weight="fill" class="eyebrow-icon" />
                                    <span>FEATURED SPOTLIGHT</span>
                                    {#if exploreStore.activeSpotlight.provider_name}
                                        <span class="provider-pill">{exploreStore.activeSpotlight.provider_name.toUpperCase()}</span>
                                    {/if}
                                </div>

                                <h1 class="spotlight-title">{exploreStore.activeSpotlight.title}</h1>
                                
                                <div class="spotlight-meta">
                                    <span class="meta-artist">{exploreStore.activeSpotlight.artist}</span>
                                    <span class="meta-dot">•</span>
                                    <span class="meta-year">{exploreStore.activeSpotlight.release_year || '2026'}</span>
                                    <span class="meta-badge">LOSSLESS</span>
                                </div>

                                <p class="spotlight-desc">{exploreStore.activeSpotlight.description || 'Curated Editorial Highlight • Master Audio'}</p>
                                
                                <div class="spotlight-actions">
                                    <button 
                                        class="play-spotlight-btn" 
                                        class:glass-btn={settingsStore.glassyPlayerBar}
                                        onclick={() => exploreStore.playSpotlight()}
                                    >
                                        <Play size={18} weight="fill" />
                                        <span>Play Album</span>
                                    </button>
                                </div>
                            </div>
                        </div>
                    {/key}

                    <!-- Combined Carousel Controls: [ < ] [ • • • ] [ > ] -->
                    {#if exploreStore.spotlights.length > 1}
                        <div class="hero-carousel-controls" class:is-glass={settingsStore.glassyPlayerBar}>
                            <button class="hero-nav-arrow" onclick={() => exploreStore.prevSpotlight()} title="Previous slide" aria-label="Previous slide">
                                <CaretLeft size={16} weight="bold" />
                            </button>
                            
                            <div class="hero-pills">
                                {#each exploreStore.spotlights as _, i}
                                    <button 
                                        class="hero-pill"
                                        class:active={i === exploreStore.activeSpotlightIndex}
                                        onclick={() => exploreStore.setSpotlightIndex(i)}
                                        title={`Slide ${i + 1}`}
                                        aria-label={`Slide ${i + 1}`}
                                    ></button>
                                {/each}
                            </div>

                            <button class="hero-nav-arrow" onclick={() => exploreStore.nextSpotlight()} title="Next slide" aria-label="Next slide">
                                <CaretRight size={16} weight="bold" />
                            </button>
                        </div>
                    {/if}
                </div>
            </section>
        {/if}

        <!-- BROWSE BY CATEGORY (4-Column Symmetrical Grid) -->
        <section class="explore-section">
            <div class="section-header">
                <h2>Browse by Category</h2>
            </div>
            <div class="category-grid">
                {#each exploreStore.categoryGrid as cat}
                    <button 
                        class="category-tile"
                        onclick={() => exploreStore.selectCategory(cat)}
                        style:--cat-accent={cat.color_hex || '#B58E62'}
                    >
                        <div class="tile-gradient-overlay"></div>
                        <span class="tile-label">{cat.title}</span>
                        <div class="tile-accent-bar"></div>
                    </button>
                {/each}
            </div>
        </section>

        <!-- DISCOVERY SPLIT (60% / 40%) -->
        <section class="discovery-split-section">
            <!-- Left Column (60% Width) — Ranked Track Ledger -->
            <div class="split-column left-ledger-column">
                <div class="section-header">
                    <h2>Top Global Tracks</h2>
                </div>
                <div class="ledger-container">
                    {#if exploreStore.filteredRankedTracks.length > 0}
                        {#each exploreStore.filteredRankedTracks as track, i}
                            <div 
                                class="ledger-row"
                                class:active-track={isCurrentTrack(track)}
                                role="button"
                                tabindex="0"
                                ondblclick={() => exploreStore.playTrack(track, track.provider_id)}
                                onkeydown={(e) => { if (e.key === 'Enter') exploreStore.playTrack(track, track.provider_id); }}
                            >
                                <span class="ledger-num">{(i + 1).toString().padStart(2, '0')}</span>
                                
                                <div class="track-art-wrapper">
                                    {#if track.cover_art_url}
                                        <img src={track.cover_art_url} alt={track.title} class="track-squircle" />
                                    {:else}
                                        <div class="typographic-art-squircle">
                                            <span>{getInitial(track.artist)}</span>
                                        </div>
                                    {/if}
                                    <button class="play-overlay-btn" onclick={() => exploreStore.playTrack(track, track.provider_id)}>
                                        {#if isCurrentTrack(track) && audioStore.playbackState === "Playing"}
                                            <Pause size={14} weight="fill" />
                                        {:else}
                                            <Play size={14} weight="fill" />
                                        {/if}
                                    </button>
                                </div>

                                <div class="track-meta">
                                    <span class="track-title">{track.title}</span>
                                    <span class="track-artist">{track.artist}</span>
                                </div>

                                <span class="track-duration">{formatDuration(track.duration_ms)}</span>
                            </div>
                        {/each}
                    {:else}
                        <div class="empty-ledger">No top tracks found.</div>
                    {/if}
                </div>
            </div>

            <!-- Right Column (40% Width) — New Releases 2x2 Grid -->
            <div class="split-column right-albums-column">
                <div class="section-header">
                    <h2>New Releases</h2>
                </div>
                <div class="albums-2x2-grid">
                    {#each exploreStore.filteredNewReleases as album}
                        <div 
                            class="album-card"
                            role="button"
                            tabindex="0"
                            onclick={() => exploreStore.playAlbum(album)}
                            onkeydown={(e) => { if (e.key === 'Enter') exploreStore.playAlbum(album); }}
                        >
                            <div class="album-art-wrapper">
                                {#if album.cover_art_url}
                                    <img src={album.cover_art_url} alt={album.title} class="album-img" />
                                {:else}
                                    <div class="typographic-art-squircle large">
                                        <span>{getInitial(album.artist)}</span>
                                    </div>
                                {/if}
                                <div class="album-overlay">
                                    <div class="play-bubble">
                                        <Play size={18} weight="fill" />
                                    </div>
                                </div>
                            </div>
                            <div class="album-meta">
                                <span class="album-title">{album.title}</span>
                                <span class="album-artist">{album.artist}</span>
                            </div>
                        </div>
                    {/each}
                </div>
            </div>
        </section>
    {/if}
</div>

<style>
    .explore-canvas {
        padding: 1.5rem 2rem 9rem 2rem; /* pb-36 (144px) clearance for Player Island */
        max-width: 1360px;
        margin: 0 auto;
        display: flex;
        flex-direction: column;
        gap: 2.2rem;
        color: #fff;
    }

    /* ─── Top Discovery Bar ─── */
    .discovery-bar-wrapper {
        width: 100%;
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
    }
    .discovery-bar {
        height: 44px;
        background: #161618;
        border: 1px solid rgba(255, 255, 255, 0.08);
        border-radius: 8px;
        display: flex;
        align-items: center;
        padding: 0 0.85rem;
        gap: 0.75rem;
        transition: border-color 0.2s ease, box-shadow 0.2s ease, background-color 0.2s ease;
    }
    .discovery-bar:focus-within {
        background: #19191C;
        border-color: rgba(181, 142, 98, 0.45);
        box-shadow: 0 0 0 1px rgba(181, 142, 98, 0.25), 0 4px 16px rgba(0, 0, 0, 0.25);
    }
    .discovery-input,
    .discovery-input:focus,
    .discovery-input:focus-visible {
        flex: 1;
        height: 100%;
        background: transparent !important;
        border: none !important;
        outline: none !important;
        box-shadow: none !important;
        padding: 0 0.35rem !important;
        margin: 0 !important;
        color: #fff;
        font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
        font-size: 0.88rem;
        line-height: 44px;
    }
    .discovery-input::placeholder {
        color: rgba(255, 255, 255, 0.35);
        font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    }
    .clear-search-btn, .refresh-btn {
        background: transparent;
        border: none;
        color: rgba(255, 255, 255, 0.5);
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 0.25rem;
        border-radius: 4px;
        transition: color 0.15s ease, transform 0.15s ease;
    }
    .clear-search-btn:hover, .refresh-btn:hover {
        color: #fff;
    }
    .refresh-btn.spinning {
        animation: spin 1s linear infinite;
    }
    @keyframes spin {
        100% { transform: rotate(360deg); }
    }

    /* Search Filter Chips */
    .search-filter-chips {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        overflow-x: auto;
        padding: 0.1rem 0;
    }
    .filter-chip {
        background: rgba(255, 255, 255, 0.04);
        border: 1px solid rgba(255, 255, 255, 0.08);
        color: rgba(255, 255, 255, 0.7);
        padding: 0.35rem 0.85rem;
        border-radius: 20px;
        font-size: 0.82rem;
        font-weight: 600;
        cursor: pointer;
        transition: all 0.15s ease;
    }
    .filter-chip:hover {
        background: rgba(255, 255, 255, 0.08);
        color: #fff;
        border-color: rgba(255, 255, 255, 0.15);
    }
    .filter-chip.active {
        background: #B58E62;
        border-color: #B58E62;
        color: #000000;
        font-weight: 700;
    }

    /* ─── Dedicated Search Results Canvas ─── */
    .search-results-canvas {
        display: flex;
        flex-direction: column;
        gap: 1.6rem;
    }
    .search-results-header {
        display: flex;
        align-items: baseline;
        justify-content: space-between;
        border-bottom: 1px solid rgba(255, 255, 255, 0.06);
        padding-bottom: 0.8rem;
    }
    .search-title-row {
        display: flex;
        align-items: baseline;
        gap: 0.75rem;
    }
    .search-title-row h2 {
        font-size: 1.3rem;
        font-weight: 700;
        margin: 0;
        color: #EAEAEA;
    }
    .query-highlight {
        color: #B58E62;
    }
    .search-count-badge {
        font-size: 0.8rem;
        color: rgba(255, 255, 255, 0.45);
        font-family: ui-monospace, monospace;
    }

    .shelves-feed {
        display: flex;
        flex-direction: column;
        gap: 2.2rem;
    }
    .search-shelf {
        display: flex;
        flex-direction: column;
        gap: 0.9rem;
    }
    .shelf-header {
        display: flex;
        align-items: center;
        gap: 0.6rem;
    }
    .shelf-header h3 {
        font-size: 1.15rem;
        font-weight: 700;
        margin: 0;
        color: #EAEAEA;
    }
    .shelf-count {
        font-size: 0.75rem;
        font-family: ui-monospace, monospace;
        color: rgba(255, 255, 255, 0.4);
        background: rgba(255, 255, 255, 0.06);
        padding: 0.1rem 0.4rem;
        border-radius: 4px;
    }

    /* Top Result Hero Card */
    .top-result-cluster {
        display: flex;
        flex-direction: column;
        gap: 0.6rem;
    }
    .top-result-card {
        display: flex;
        align-items: center;
        gap: 1.5rem;
        padding: 1.4rem;
        border-radius: 12px;
        background: #19191D;
        border: 1px solid rgba(255, 255, 255, 0.08);
        box-shadow: 0 8px 28px rgba(0, 0, 0, 0.35);
    }
    .top-result-card.is-glass {
        backdrop-filter: blur(14px);
        background: rgba(25, 25, 30, 0.65);
    }
    .top-result-art-wrapper {
        width: 96px;
        height: 96px;
        border-radius: 10px;
        overflow: hidden;
        flex-shrink: 0;
        background: #232328;
    }
    .top-result-art {
        width: 100%;
        height: 100%;
        object-fit: cover;
    }
    .top-result-info {
        display: flex;
        flex-direction: column;
        gap: 0.35rem;
    }
    .top-result-eyebrow {
        display: flex;
        align-items: center;
        gap: 0.45rem;
        font-family: ui-monospace, monospace;
        font-size: 0.74rem;
        font-weight: 700;
        color: #B58E62;
        letter-spacing: 0.06em;
    }
    .top-result-title {
        font-size: 1.6rem;
        font-weight: 700;
        margin: 0;
        color: #FFFFFF;
    }
    .top-result-sub {
        font-size: 0.9rem;
        color: rgba(255, 255, 255, 0.65);
        margin: 0;
    }
    .sub-track-row {
        margin-left: 0.5rem;
    }

    /* Search Tracks Ledger */
    .search-tracks-ledger {
        display: flex;
        flex-direction: column;
        gap: 0.35rem;
    }
    .track-subline {
        display: flex;
        align-items: center;
        gap: 0.35rem;
        font-size: 0.8rem;
        color: rgba(255, 255, 255, 0.55);
    }
    .track-album {
        color: rgba(255, 255, 255, 0.4);
    }
    .track-dot {
        color: rgba(255, 255, 255, 0.25);
    }
    .provider-tag {
        font-size: 0.65rem;
        font-family: ui-monospace, monospace;
        background: rgba(255, 255, 255, 0.05);
        border: 1px solid rgba(255, 255, 255, 0.1);
        color: rgba(255, 255, 255, 0.5);
        padding: 0.1rem 0.4rem;
        border-radius: 4px;
        margin-right: 0.5rem;
    }

    /* Albums & Playlists Search Grid (Uniform Inspired by AlbumView) */
    .albums-search-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
        gap: 1.5rem;
        width: 100%;
    }

    .echo-album-card {
        cursor: pointer;
        display: flex;
        flex-direction: column;
        width: 100%;
        transition: transform 0.2s ease;
    }

    .echo-album-card:hover {
        transform: translateY(-4px);
    }

    .echo-album-art-container {
        width: 100%;
        aspect-ratio: 1 / 1;
        border-radius: 1.25rem; /* rounded-[1.25rem] uniform squircle */
        background-color: #1c1c1f;
        border: 1px solid rgba(255, 255, 255, 0.08);
        overflow: hidden;
        box-shadow: 0 8px 20px -3px rgba(0, 0, 0, 0.4), 0 4px 6px -4px rgba(0, 0, 0, 0.4);
        margin-bottom: 0.75rem;
        position: relative;
    }

    .echo-album-img {
        width: 100%;
        height: 100%;
        object-fit: cover;
        display: block;
        transition: transform 0.5s cubic-bezier(0.4, 0, 0.2, 1);
    }

    .echo-album-card:hover .echo-album-img {
        transform: scale(1.06);
    }

    .echo-album-overlay {
        position: absolute;
        inset: 0;
        background-color: rgba(0, 0, 0, 0.4);
        backdrop-filter: blur(4px);
        -webkit-backdrop-filter: blur(4px);
        opacity: 0;
        transition: opacity 0.25s ease;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .echo-album-card:hover .echo-album-overlay {
        opacity: 1;
    }

    .echo-play-btn {
        width: 46px;
        height: 46px;
        border-radius: 50%;
        background-color: #B58E62; /* Echo Primary Brass */
        color: #000000;
        display: flex;
        align-items: center;
        justify-content: center;
        box-shadow: 0 4px 14px rgba(0, 0, 0, 0.5);
        transition: transform 0.2s ease, background-color 0.2s ease;
    }

    .echo-album-card:hover .echo-play-btn {
        transform: scale(1.08);
    }

    .echo-album-meta {
        display: flex;
        flex-direction: column;
        gap: 0.2rem;
        min-width: 0;
    }

    .echo-album-title {
        font-size: 0.92rem;
        font-weight: 600;
        color: #FFFFFF;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
        line-height: 1.3;
    }

    .echo-album-artist {
        font-size: 0.8rem;
        color: rgba(255, 255, 255, 0.55);
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
        line-height: 1.2;
    }

    /* Artists Search Grid */
    .artists-search-grid {
        display: grid;
        grid-template-columns: repeat(6, 1fr);
        gap: 1rem;
    }
    @media (max-width: 1024px) {
        .artists-search-grid {
            grid-template-columns: repeat(3, 1fr);
        }
    }
    .artist-card {
        display: flex;
        flex-direction: column;
        align-items: center;
        text-align: center;
        gap: 0.5rem;
        padding: 0.8rem 0.5rem;
        border-radius: 8px;
        background: rgba(255, 255, 255, 0.02);
        cursor: pointer;
        transition: background 0.15s ease, transform 0.15s ease;
    }
    .artist-card:hover {
        background: rgba(255, 255, 255, 0.06);
        transform: translateY(-2px);
    }
    .artist-avatar-wrapper {
        width: 72px;
        height: 72px;
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
        background: #2A2A30;
        color: #B58E62;
        font-weight: 700;
        font-size: 1.4rem;
    }
    .artist-name {
        font-size: 0.88rem;
        font-weight: 600;
        color: #FFFFFF;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        max-width: 100%;
    }
    .artist-sub {
        font-size: 0.74rem;
        color: rgba(255, 255, 255, 0.45);
    }

    .empty-search-state {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        padding: 4rem 1rem;
        gap: 0.85rem;
        color: rgba(255, 255, 255, 0.5);
    }
    .clear-search-link {
        background: transparent;
        border: 1px solid rgba(181, 142, 98, 0.35);
        color: #B58E62;
        padding: 0.5rem 1rem;
        border-radius: 6px;
        cursor: pointer;
        font-size: 0.85rem;
        transition: background 0.15s ease, color 0.15s ease;
    }
    .clear-search-link:hover {
        background: rgba(181, 142, 98, 0.15);
        color: #fff;
    }

    /* ─── Featured Spotlight (16:7 Rotating Editorial Banner) ─── */
    .spotlight-banner-wrapper {
        width: 100%;
    }
    .spotlight-banner {
        position: relative;
        width: 100%;
        min-height: 280px;
        border-radius: 12px;
        overflow: hidden;
        background: #141416;
        border: 1px solid rgba(255, 255, 255, 0.08);
        box-shadow: 0 16px 40px rgba(0, 0, 0, 0.4);
    }
    .spotlight-banner.is-glass {
        backdrop-filter: blur(16px);
        -webkit-backdrop-filter: blur(16px);
    }
    .spotlight-slide {
        position: relative;
        width: 100%;
        height: 100%;
        min-height: 280px;
    }
    .spotlight-backdrop {
        position: absolute;
        inset: 0;
        background-image: var(--hero-art);
        background-size: cover;
        background-position: center right;
        filter: brightness(0.65) saturate(1.1);
        z-index: 1;
    }
    .spotlight-content {
        position: relative;
        z-index: 2;
        padding: 2.2rem 2.5rem;
        display: flex;
        flex-direction: column;
        justify-content: center;
        align-items: flex-start;
        gap: 0.65rem;
        background: linear-gradient(to right, rgba(18, 18, 22, 0.96) 35%, rgba(18, 18, 22, 0.6) 70%, transparent 100%);
        min-height: 280px;
    }
    .spotlight-eyebrow {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        font-family: ui-monospace, SFMono-Regular, monospace;
        font-size: 0.76rem;
        font-weight: 700;
        letter-spacing: 0.1em;
        color: #B58E62; /* Brass Accent */
    }
    .provider-pill {
        font-size: 0.65rem;
        background: rgba(255, 255, 255, 0.08);
        border: 1px solid rgba(255, 255, 255, 0.15);
        color: rgba(255, 255, 255, 0.75);
        padding: 0.1rem 0.35rem;
        border-radius: 4px;
        letter-spacing: 0.04em;
    }
    .spotlight-title {
        font-family: ui-serif, Georgia, Cambria, "Times New Roman", Times, serif;
        font-size: 2.2rem;
        font-weight: 700;
        line-height: 1.15;
        color: #FFFFFF;
        margin: 0;
        max-width: 600px;
    }
    .spotlight-meta {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        font-size: 0.92rem;
        color: rgba(255, 255, 255, 0.85);
    }
    .meta-artist {
        font-weight: 600;
    }
    .meta-dot {
        color: rgba(255, 255, 255, 0.4);
    }
    .meta-year {
        color: rgba(255, 255, 255, 0.65);
    }
    .meta-badge {
        font-size: 0.68rem;
        font-weight: 700;
        letter-spacing: 0.06em;
        background: rgba(181, 142, 98, 0.18);
        border: 1px solid rgba(181, 142, 98, 0.35);
        color: #B58E62;
        padding: 0.15rem 0.45rem;
        border-radius: 4px;
        margin-left: 0.3rem;
    }
    .spotlight-desc {
        font-size: 0.88rem;
        color: rgba(255, 255, 255, 0.6);
        margin: 0.1rem 0 0.5rem 0;
    }
    .play-spotlight-btn {
        background: #FFFFFF;
        color: #000000;
        border: none;
        padding: 0.65rem 1.4rem;
        border-radius: 8px;
        font-size: 0.92rem;
        font-weight: 700;
        cursor: pointer;
        display: flex;
        align-items: center;
        gap: 0.45rem;
        box-shadow: 0 4px 14px rgba(255, 255, 255, 0.15);
        transition: transform 0.15s ease, background-color 0.15s ease;
    }
    .play-spotlight-btn:hover {
        transform: translateY(-2px);
        background: #EAEAEA;
    }
    .play-spotlight-btn.glass-btn {
        background: rgba(255, 255, 255, 0.9);
        backdrop-filter: blur(8px);
        -webkit-backdrop-filter: blur(8px);
    }

    /* Combined Carousel Controls: [ < ] [ • • • ] [ > ] */
    .hero-carousel-controls {
        position: absolute;
        bottom: 1.2rem;
        right: 1.5rem;
        z-index: 3;
        display: flex;
        align-items: center;
        gap: 0.35rem;
        background: rgba(18, 18, 22, 0.75);
        border: 1px solid rgba(255, 255, 255, 0.08);
        border-radius: 20px;
        padding: 0.25rem 0.4rem;
        box-shadow: 0 4px 16px rgba(0, 0, 0, 0.35);
        transition: background-color 0.2s ease, border-color 0.2s ease;
    }
    .hero-carousel-controls.is-glass {
        backdrop-filter: blur(12px);
        -webkit-backdrop-filter: blur(12px);
        background: rgba(25, 25, 32, 0.45);
        border-color: rgba(255, 255, 255, 0.12);
    }
    .hero-nav-arrow {
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
        transition: color 0.15s ease, background-color 0.15s ease, border-color 0.15s ease, transform 0.1s ease;
    }
    .hero-nav-arrow :global(svg) {
        display: block;
        flex-shrink: 0;
    }
    .hero-nav-arrow:hover {
        color: #B58E62;
        background: rgba(255, 255, 255, 0.16);
        border-color: rgba(181, 142, 98, 0.4);
    }
    .hero-nav-arrow:active {
        transform: scale(0.92);
    }
    .hero-pills {
        display: flex;
        align-items: center;
        gap: 0.35rem;
        padding: 0 0.25rem;
    }
    .hero-pill {
        width: 6px;
        height: 6px;
        border-radius: 3px;
        background: rgba(255, 255, 255, 0.3);
        border: none;
        cursor: pointer;
        padding: 0;
        transition: all 0.25s ease;
    }
    .hero-pill:hover {
        background: rgba(255, 255, 255, 0.65);
    }
    .hero-pill.active {
        width: 18px;
        background: #B58E62; /* Active brass pill */
    }

    /* ─── Browse by Category (4-Column Symmetrical Grid) ─── */
    .explore-section {
        display: flex;
        flex-direction: column;
        gap: 0.9rem;
    }
    .section-header h2 {
        font-size: 1.15rem;
        font-weight: 700;
        letter-spacing: -0.01em;
        margin: 0;
        color: #EAEAEA;
    }
    .category-grid {
        display: grid;
        grid-template-columns: repeat(4, 1fr);
        gap: 1rem;
    }
    @media (max-width: 900px) {
        .category-grid {
            grid-template-columns: repeat(2, 1fr);
        }
    }
    .category-tile {
        position: relative;
        aspect-ratio: 16 / 9;
        background: #1E1E22;
        border: 1px solid rgba(255, 255, 255, 0.06);
        border-radius: 10px;
        overflow: hidden;
        cursor: pointer;
        display: flex;
        align-items: flex-start;
        padding: 1rem;
        text-align: left;
        transition: transform 0.2s ease, border-color 0.2s ease, box-shadow 0.2s ease;
    }
    .category-tile:hover {
        transform: scale(1.02);
        border-color: rgba(255, 255, 255, 0.2);
        box-shadow: 0 8px 24px rgba(0, 0, 0, 0.35);
    }
    .tile-gradient-overlay {
        position: absolute;
        inset: 0;
        background: radial-gradient(circle at 100% 100%, var(--cat-accent) 0%, transparent 65%);
        opacity: 0.18;
        transition: opacity 0.2s ease;
    }
    .category-tile:hover .tile-gradient-overlay {
        opacity: 0.32;
    }
    .tile-label {
        position: relative;
        z-index: 2;
        font-size: 1.05rem;
        font-weight: 700;
        color: #EAEAEA;
    }
    .tile-accent-bar {
        position: absolute;
        bottom: 0;
        left: 0;
        right: 0;
        height: 3px;
        background: var(--cat-accent);
        opacity: 0.7;
    }

    /* ─── Discovery Split Section (60% / 40%) ─── */
    .discovery-split-section {
        display: grid;
        grid-template-columns: 60% calc(40% - 1.5rem);
        gap: 1.5rem;
        align-items: start;
    }
    @media (max-width: 1024px) {
        .discovery-split-section {
            grid-template-columns: 1fr;
        }
    }

    /* Left Column — Ranked Track Ledger */
    .split-column {
        display: flex;
        flex-direction: column;
        gap: 0.9rem;
    }
    .ledger-container {
        display: flex;
        flex-direction: column;
        gap: 0.35rem;
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
        font-family: ui-monospace, SFMono-Regular, monospace;
        font-size: 0.9rem;
        font-weight: 700;
        color: #B58E62; /* Monospace Brass */
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
        font-size: 1.1rem;
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
    .track-artist {
        font-size: 0.8rem;
        color: rgba(255, 255, 255, 0.55);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }
    .track-duration {
        font-family: ui-monospace, SFMono-Regular, monospace;
        font-size: 0.78rem;
        color: rgba(255, 255, 255, 0.45);
    }

    /* Right Column — New Releases 2x2 Grid */
    .albums-2x2-grid {
        display: grid;
        grid-template-columns: repeat(2, 1fr);
        gap: 0.85rem;
    }
    .album-card {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
        cursor: pointer;
        padding: 0.5rem;
        border-radius: 8px;
        background: rgba(255, 255, 255, 0.02);
        transition: background 0.15s ease, transform 0.15s ease;
    }
    .album-card:hover {
        background: rgba(255, 255, 255, 0.05);
        transform: translateY(-2px);
    }
    .album-art-wrapper {
        position: relative;
        width: 100%;
        aspect-ratio: 1 / 1;
        border-radius: 8px;
        overflow: hidden;
        background: #232328;
    }
    .album-img {
        width: 100%;
        height: 100%;
        object-fit: cover;
    }
    .album-overlay {
        position: absolute;
        inset: 0;
        background: rgba(0, 0, 0, 0.4);
        display: flex;
        align-items: center;
        justify-content: center;
        opacity: 0;
        transition: opacity 0.15s ease;
    }
    .album-card:hover .album-overlay {
        opacity: 1;
    }
    .play-bubble {
        width: 36px;
        height: 36px;
        border-radius: 50%;
        background: #FFFFFF;
        color: #000;
        display: flex;
        align-items: center;
        justify-content: center;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
    }
    .album-meta {
        display: flex;
        flex-direction: column;
        gap: 0.15rem;
    }
    .album-title {
        font-family: ui-serif, Georgia, serif;
        font-size: 0.92rem;
        font-weight: 700;
        color: #FFFFFF;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }
    .album-artist {
        font-size: 0.78rem;
        color: rgba(255, 255, 255, 0.5);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    /* ─── Category Hub Sub-View ─── */
    .category-hub-canvas {
        display: flex;
        flex-direction: column;
        gap: 1.8rem;
    }
    .hub-header {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }
    .hub-back-btn {
        align-self: flex-start;
        background: transparent;
        border: none;
        color: #B58E62;
        font-size: 0.88rem;
        font-weight: 600;
        cursor: pointer;
        display: flex;
        align-items: center;
        gap: 0.4rem;
        padding: 0;
        transition: color 0.15s ease;
    }
    .hub-back-btn:hover {
        color: #FFFFFF;
    }
    .hub-title-row {
        display: flex;
        align-items: center;
        gap: 0.75rem;
    }
    .hub-color-badge {
        width: 14px;
        height: 14px;
        border-radius: 50%;
    }
    .hub-title-row h1 {
        font-size: 2rem;
        font-weight: 700;
        margin: 0;
    }
    .hub-subtitle {
        font-size: 0.9rem;
        color: rgba(255, 255, 255, 0.6);
        margin: 0;
    }
    .hub-tracks-grid {
        display: flex;
        flex-direction: column;
        gap: 0.4rem;
    }
    .empty-state, .empty-ledger {
        font-size: 0.88rem;
        color: rgba(255, 255, 255, 0.4);
        padding: 1.5rem 0;
    }
</style>
