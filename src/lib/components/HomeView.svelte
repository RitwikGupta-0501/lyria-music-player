<script lang="ts">
    import { onMount } from "svelte";
    import { homeStore, type CanonicalSong } from "$lib/stores/home.svelte";
    import { audioStore } from "$lib/stores/audio.svelte";
    import { Play, Pause, Heart, ArrowClockwise, Sparkle, ClockCounterClockwise, ArrowRight, Compass, FolderOpen } from "phosphor-svelte";
    import { libraryStore } from "$lib/stores/library.svelte";

    let { activeView = $bindable("home") } = $props<{ activeView?: string }>();

    onMount(() => {
        homeStore.init();
    });

    function getGreeting(): string {
        const hour = new Date().getHours();
        if (hour < 12) return "Good morning";
        if (hour < 18) return "Good afternoon";
        return "Good evening";
    }

    function isCurrentTrack(song: CanonicalSong): boolean {
        const cur = audioStore.currentQueueTrack;
        if (!cur) return false;
        return cur.title.toLowerCase() === song.title.toLowerCase()
            && (cur.artist || "").toLowerCase() === song.artist.toLowerCase();
    }

    function playRemoteItem(item: any, providerId: string) {
        const track = {
            id: `remote-${providerId}-${item.id || item.video_id || item.title}`,
            title: item.title,
            artist: item.artist,
            album: item.album || '',
            file_path: item.stream_url || '',
            provider_id: providerId,
            stream_url: item.stream_url,
            cover_art_url: item.cover_art_url,
            duration_ms: item.duration_ms,
        };
        audioStore.setQueue([track], 0);
        homeStore.recordPlay({
            id: 0,
            canonical_key: '',
            title: item.title,
            artist: item.artist,
            album: item.album || null,
            cover_art_url: item.cover_art_url || null,
            play_count: 1,
            last_played_at: null,
            liked: false,
            local_track_id: null,
            local_file_path: null,
            last_provider_id: providerId,
            last_source_id: item.id || item.video_id || '',
            duration_ms: item.duration_ms || null,
        });
    }
</script>

<div class="home-view">
    <!-- Header -->
    <header class="home-header">
        <div class="header-left">
            <h1>{getGreeting()}</h1>
            <p class="subtitle">Your personalized music stream & discovery</p>
        </div>
        <div class="header-actions">
            <button 
                class="refresh-btn" 
                class:spinning={homeStore.isRefreshing}
                onclick={() => homeStore.loadHome(true)}
                title="Refresh recommendations"
            >
                <ArrowClockwise size={20} weight="bold" />
            </button>
        </div>
    </header>

    <!-- Fresh / Empty State Banner (When 0 plays recorded) -->
    {#if homeStore.phase1Loaded && homeStore.quickPicks.length === 0 && homeStore.keepListening.length === 0}
        <section class="welcome-card">
            <div class="welcome-badge">
                <Sparkle size={16} weight="fill" />
                <span>Algorithmic Dashboard</span>
            </div>
            <h2>Welcome to Echo</h2>
            <p>Start playing music from your local library or explore global charts. Echo will adapt and curate your daily recommendations automatically.</p>
            <div class="welcome-actions">
                <button class="primary-btn" onclick={() => activeView = "explore"}>
                    <Compass size={18} weight="bold" />
                    <span>Explore Charts & Releases</span>
                </button>
                <button class="secondary-btn" onclick={() => activeView = "albums"}>
                    <FolderOpen size={18} weight="bold" />
                    <span>Browse Local Library</span>
                </button>
            </div>
        </section>
    {/if}

    <!-- 1. Speed Dial / Quick Picks Grid -->
    {#if homeStore.quickPicks.length > 0}
        <section class="home-section quick-picks-section">
            <div class="section-title-row">
                <h2>Quick Picks</h2>
                <span class="section-tag">High Rotation</span>
            </div>
            <div class="quick-picks-grid">
                {#each homeStore.quickPicks.slice(0, 8) as song}
                    <div 
                        class="quick-pick-pill"
                        class:playing={isCurrentTrack(song)}
                        role="button"
                        tabindex="0"
                        onclick={() => homeStore.playCanonicalSong(song)}
                        onkeydown={(e) => { if (e.key === 'Enter') homeStore.playCanonicalSong(song); }}
                    >
                        <div class="pill-art">
                            {#if song.cover_art_url}
                                <img src={song.cover_art_url.startsWith('/') ? `asset://localhost/${encodeURIComponent(song.cover_art_url)}` : song.cover_art_url} alt={song.title} />
                            {:else}
                                <div class="placeholder-art"></div>
                            {/if}
                            <div class="pill-play-overlay">
                                {#if isCurrentTrack(song) && audioStore.playbackState === "Playing"}
                                    <Pause size={18} weight="fill" />
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
    {/if}

    <!-- 2. Keep Listening (Past 14 Days) -->
    {#if homeStore.keepListening.length > 0}
        <section class="home-section">
            <div class="section-title-row">
                <h2>
                    <ClockCounterClockwise size={20} weight="bold" />
                    <span>Keep Listening</span>
                </h2>
                <span class="section-tag">Recent 14 Days</span>
            </div>
            <div class="carousel-track">
                {#each homeStore.keepListening as song}
                    <div 
                        class="carousel-card"
                        role="button"
                        tabindex="0"
                        onclick={() => homeStore.playCanonicalSong(song)}
                        onkeydown={(e) => { if (e.key === 'Enter') homeStore.playCanonicalSong(song); }}
                    >
                        <div class="card-art-wrapper">
                            {#if song.cover_art_url}
                                <img src={song.cover_art_url.startsWith('/') ? `asset://localhost/${encodeURIComponent(song.cover_art_url)}` : song.cover_art_url} alt={song.title} />
                            {:else}
                                <div class="placeholder-art"></div>
                            {/if}
                            <div class="card-overlay">
                                <div class="play-bubble">
                                    <Play size={22} weight="fill" />
                                </div>
                            </div>
                        </div>
                        <div class="card-info">
                            <span class="card-title">{song.title}</span>
                            <span class="card-artist">{song.artist}</span>
                        </div>
                    </div>
                {/each}
            </div>
        </section>
    {/if}

    <!-- 3. Daily Discover Shelves (Contextual Discovery) -->
    {#each homeStore.discoverShelves as shelf}
        <section class="home-section">
            <div class="section-title-row">
                <h2>
                    <Sparkle size={20} weight="fill" class="sparkle-icon" />
                    <span>{shelf.contextTag}</span>
                </h2>
            </div>

            {#if shelf.loading}
                <div class="carousel-track">
                    {#each Array(6) as _}
                        <div class="carousel-card skeleton">
                            <div class="card-art-wrapper skeleton-box"></div>
                            <div class="skeleton-line title"></div>
                            <div class="skeleton-line artist"></div>
                        </div>
                    {/each}
                </div>
            {:else if shelf.tracks.length > 0}
                <div class="carousel-track">
                    {#each shelf.tracks as item}
                        <div 
                            class="carousel-card"
                            role="button"
                            tabindex="0"
                            onclick={() => playRemoteItem(item, item.provider_id || shelf.seed.last_provider_id)}
                            onkeydown={(e) => { if (e.key === 'Enter') playRemoteItem(item, item.provider_id || shelf.seed.last_provider_id); }}
                        >
                            <div class="card-art-wrapper">
                                {#if item.cover_art_url}
                                    <img src={item.cover_art_url} alt={item.title} />
                                {:else}
                                    <div class="placeholder-art"></div>
                                {/if}
                                <div class="card-overlay">
                                    <div class="play-bubble">
                                        <Play size={22} weight="fill" />
                                    </div>
                                </div>
                            </div>
                            <div class="card-info">
                                <span class="card-title">{item.title}</span>
                                <span class="card-artist">{item.artist}</span>
                            </div>
                        </div>
                    {/each}
                </div>
            {/if}
        </section>
    {/each}

    <!-- 4. Forgotten Favorites -->
    {#if homeStore.forgottenFavorites.length > 0}
        <section class="home-section">
            <div class="section-title-row">
                <h2>Forgotten Favorites</h2>
                <span class="section-tag">Rediscover</span>
            </div>
            <div class="carousel-track">
                {#each homeStore.forgottenFavorites as song}
                    <div 
                        class="carousel-card"
                        role="button"
                        tabindex="0"
                        onclick={() => homeStore.playCanonicalSong(song)}
                        onkeydown={(e) => { if (e.key === 'Enter') homeStore.playCanonicalSong(song); }}
                    >
                        <div class="card-art-wrapper">
                            {#if song.cover_art_url}
                                <img src={song.cover_art_url.startsWith('/') ? `asset://localhost/${encodeURIComponent(song.cover_art_url)}` : song.cover_art_url} alt={song.title} />
                            {:else}
                                <div class="placeholder-art"></div>
                            {/if}
                            <div class="card-overlay">
                                <div class="play-bubble">
                                    <Play size={22} weight="fill" />
                                </div>
                            </div>
                        </div>
                        <div class="card-info">
                            <span class="card-title">{song.title}</span>
                            <span class="card-artist">{song.artist}</span>
                        </div>
                    </div>
                {/each}
            </div>
        </section>
    {/if}
</div>

<style>
    .home-view {
        padding: 2.5rem;
        height: 100%;
        overflow-y: auto;
        display: flex;
        flex-direction: column;
        gap: 3rem;
    }

    .home-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
    }

    .home-header h1 {
        font-size: 2.25rem;
        font-weight: 700;
        letter-spacing: -0.03em;
        margin: 0 0 0.25rem 0;
    }

    .subtitle {
        margin: 0;
        font-size: 0.95rem;
        color: var(--text-muted, rgba(255, 255, 255, 0.6));
    }

    .refresh-btn {
        background: var(--surface-1, rgba(255, 255, 255, 0.05));
        border: 1px solid rgba(255, 255, 255, 0.08);
        border-radius: 50%;
        width: 40px;
        height: 40px;
        display: flex;
        align-items: center;
        justify-content: center;
        color: var(--text-muted, rgba(255, 255, 255, 0.7));
        cursor: pointer;
        transition: all 0.2s ease;
    }

    .refresh-btn:hover {
        background: var(--surface-2, rgba(255, 255, 255, 0.1));
        color: #fff;
    }

    .refresh-btn.spinning {
        animation: spin 1s linear infinite;
    }

    @keyframes spin {
        from { transform: rotate(0deg); }
        to { transform: rotate(360deg); }
    }

    /* Welcome Card */
    .welcome-card {
        background: linear-gradient(135deg, rgba(255, 255, 255, 0.08) 0%, rgba(255, 255, 255, 0.02) 100%);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 16px;
        padding: 2.5rem;
        display: flex;
        flex-direction: column;
        align-items: flex-start;
        gap: 1rem;
    }

    .welcome-badge {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        font-size: 0.8rem;
        font-weight: 600;
        color: #ffd166;
        background: rgba(255, 209, 102, 0.1);
        padding: 0.25rem 0.75rem;
        border-radius: 20px;
    }

    .welcome-card h2 {
        font-size: 1.75rem;
        margin: 0;
    }

    .welcome-card p {
        margin: 0;
        max-width: 600px;
        color: var(--text-muted, rgba(255, 255, 255, 0.7));
        line-height: 1.5;
    }

    .welcome-actions {
        display: flex;
        gap: 1rem;
        margin-top: 0.5rem;
    }

    .primary-btn, .secondary-btn {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        padding: 0.75rem 1.25rem;
        border-radius: 10px;
        font-size: 0.95rem;
        font-weight: 600;
        cursor: pointer;
        transition: transform 0.15s ease, background 0.2s ease;
    }

    .primary-btn {
        background: #fff;
        color: #000;
        border: none;
    }

    .primary-btn:hover {
        transform: translateY(-2px);
        background: #f0f0f0;
    }

    .secondary-btn {
        background: rgba(255, 255, 255, 0.08);
        color: #fff;
        border: 1px solid rgba(255, 255, 255, 0.12);
    }

    .secondary-btn:hover {
        transform: translateY(-2px);
        background: rgba(255, 255, 255, 0.14);
    }

    /* Section Title */
    .home-section {
        display: flex;
        flex-direction: column;
        gap: 1.25rem;
    }

    .section-title-row {
        display: flex;
        align-items: center;
        justify-content: space-between;
    }

    .section-title-row h2 {
        font-size: 1.4rem;
        font-weight: 700;
        margin: 0;
        display: flex;
        align-items: center;
        gap: 0.6rem;
    }

    .section-tag {
        font-size: 0.75rem;
        font-weight: 600;
        text-transform: uppercase;
        color: var(--text-muted, rgba(255, 255, 255, 0.5));
        background: var(--surface-1, rgba(255, 255, 255, 0.05));
        padding: 0.2rem 0.6rem;
        border-radius: 6px;
    }

    :global(.sparkle-icon) {
        color: #ffd166;
    }

    /* Quick Picks Pill Grid */
    .quick-picks-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
        gap: 1rem;
    }

    .quick-pick-pill {
        display: flex;
        align-items: center;
        gap: 1rem;
        background: var(--surface-1, rgba(255, 255, 255, 0.04));
        border: 1px solid rgba(255, 255, 255, 0.06);
        border-radius: 10px;
        padding: 0.5rem 0.75rem 0.5rem 0.5rem;
        cursor: pointer;
        transition: transform 0.2s ease, background 0.2s ease, border-color 0.2s ease;
    }

    .quick-pick-pill:hover {
        transform: translateY(-2px);
        background: var(--surface-2, rgba(255, 255, 255, 0.08));
        border-color: rgba(255, 255, 255, 0.15);
    }

    .quick-pick-pill.playing {
        background: rgba(255, 255, 255, 0.12);
        border-color: rgba(255, 255, 255, 0.3);
    }

    .pill-art {
        width: 52px;
        height: 52px;
        border-radius: 6px;
        overflow: hidden;
        position: relative;
        flex-shrink: 0;
        background: rgba(255, 255, 255, 0.05);
    }

    .pill-art img, .card-art-wrapper img {
        width: 100%;
        height: 100%;
        object-fit: cover;
    }

    .placeholder-art {
        width: 100%;
        height: 100%;
        background: linear-gradient(135deg, rgba(255, 255, 255, 0.1), rgba(255, 255, 255, 0.02));
    }

    .pill-play-overlay {
        position: absolute;
        inset: 0;
        background: rgba(0, 0, 0, 0.4);
        display: flex;
        align-items: center;
        justify-content: center;
        opacity: 0;
        transition: opacity 0.2s ease;
        color: #fff;
    }

    .quick-pick-pill:hover .pill-play-overlay,
    .quick-pick-pill.playing .pill-play-overlay {
        opacity: 1;
    }

    .pill-info {
        flex: 1;
        min-width: 0;
        display: flex;
        flex-direction: column;
        gap: 0.2rem;
    }

    .pill-title {
        font-size: 0.95rem;
        font-weight: 600;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .pill-artist {
        font-size: 0.82rem;
        color: var(--text-muted, rgba(255, 255, 255, 0.6));
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .pill-like-btn {
        background: transparent;
        border: none;
        color: var(--text-muted, rgba(255, 255, 255, 0.4));
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

    /* Horizontal Carousels */
    .carousel-track {
        display: flex;
        gap: 1.5rem;
        overflow-x: auto;
        scroll-snap-type: x mandatory;
        padding-bottom: 0.75rem;
        scrollbar-width: none;
    }

    .carousel-track::-webkit-scrollbar {
        display: none;
    }

    .carousel-card {
        flex: 0 0 160px;
        scroll-snap-align: start;
        display: flex;
        flex-direction: column;
        gap: 0.6rem;
        cursor: pointer;
        transition: transform 0.2s ease;
    }

    .carousel-card:hover {
        transform: translateY(-4px);
    }

    .card-art-wrapper {
        width: 160px;
        height: 160px;
        border-radius: 10px;
        overflow: hidden;
        position: relative;
        background: var(--surface-1, rgba(255, 255, 255, 0.05));
    }

    .card-overlay {
        position: absolute;
        inset: 0;
        background: rgba(0, 0, 0, 0.4);
        display: flex;
        align-items: center;
        justify-content: center;
        opacity: 0;
        transition: opacity 0.2s ease;
    }

    .carousel-card:hover .card-overlay {
        opacity: 1;
    }

    .play-bubble {
        width: 44px;
        height: 44px;
        border-radius: 50%;
        background: #fff;
        color: #000;
        display: flex;
        align-items: center;
        justify-content: center;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
        transform: scale(0.9);
        transition: transform 0.2s ease;
    }

    .carousel-card:hover .play-bubble {
        transform: scale(1);
    }

    .card-info {
        display: flex;
        flex-direction: column;
        gap: 0.2rem;
    }

    .card-title {
        font-size: 0.92rem;
        font-weight: 600;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .card-artist {
        font-size: 0.8rem;
        color: var(--text-muted, rgba(255, 255, 255, 0.6));
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    /* Skeleton Shimmer */
    .skeleton-box {
        background: linear-gradient(90deg, rgba(255, 255, 255, 0.03) 25%, rgba(255, 255, 255, 0.08) 50%, rgba(255, 255, 255, 0.03) 75%);
        background-size: 200% 100%;
        animation: shimmer 1.5s infinite;
    }

    .skeleton-line {
        height: 12px;
        border-radius: 4px;
        background: linear-gradient(90deg, rgba(255, 255, 255, 0.03) 25%, rgba(255, 255, 255, 0.08) 50%, rgba(255, 255, 255, 0.03) 75%);
        background-size: 200% 100%;
        animation: shimmer 1.5s infinite;
    }

    .skeleton-line.title {
        width: 80%;
    }

    .skeleton-line.artist {
        width: 50%;
    }

    @keyframes shimmer {
        0% { background-position: 200% 0; }
        100% { background-position: -200% 0; }
    }
</style>
