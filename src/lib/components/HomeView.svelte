<script lang="ts">
    import { onMount } from "svelte";
    import { homeStore } from "$lib/stores/home.svelte";
    import { settingsStore } from "$lib/stores/settings.svelte";
    import { ArrowClockwise, Sparkle, Compass, FolderOpen, Play } from "phosphor-svelte";

    import QuickPicksGrid from "./home/QuickPicksGrid.svelte";
    import DailyDiscoverCarousel from "./home/DailyDiscoverCarousel.svelte";
    import RadioMixCarousel from "./home/RadioMixCarousel.svelte";
    import AdjacentHorizonsCard from "./home/AdjacentHorizonsCard.svelte";
    import JumpBackInShelf from "./home/JumpBackInShelf.svelte";
    import HeavyRotationShelf from "./home/HeavyRotationShelf.svelte";
    import ForgottenFavoritesShelf from "./home/ForgottenFavoritesShelf.svelte";

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
</script>

<div class="home-view">
    <!-- Header & Quick-Filter Mood Bar -->
    <header class="home-header">
        <div class="header-top">
            <div class="header-left">
                <h1>{getGreeting()}</h1>
                
            </div>
            <div class="header-actions">
                <button 
                    class="refresh-btn" 
                    class:is-glass={settingsStore.glassyPlayerBar}
                    class:spinning={homeStore.isLoadingRemote}
                    onclick={() => homeStore.loadHome(true)}
                    title="Refresh recommendations"
                >
                    <ArrowClockwise size={18} weight="bold" />
                </button>
            </div>
        </div>

        <!-- Tactile Quick-Filter Mood Bar -->
        <div class="mood-filter-bar">
            {#each ["All", "Deep Focus", "Relax & Chill", "Energy & Drive", "Commute", "Late Night Drift"] as mood}
                <button 
                    class="mood-pill" 
                    class:is-glass={settingsStore.glassyPlayerBar}
                    class:active={homeStore.currentMood === mood}
                    onclick={() => homeStore.selectMood(mood)}
                >
                    <span>{mood}</span>
                </button>
            {/each}
        </div>
    </header>

    <!-- Cold-Start View: When total telemetry play count is 0 -->
    {#if homeStore.phase1Loaded && !homeStore.hasTelemetry}
        <section class="welcome-hero-card">
            <div class="welcome-badge">
                <Sparkle size={16} weight="fill" />
                <span>Algorithmic Cockpit</span>
            </div>
            <h2>Welcome to Echo</h2>
            <p>Start playing music from your local library or explore global releases. Echo learns from your unique listening telemetry and generates dynamic daily mixes automatically.</p>
            
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

        <!-- Cold-Start Local Library Sampler -->
        {#if homeStore.coldStartSeeds.length > 0}
            <section class="cold-start-shelf">
                <div class="section-title-row">
                    <div class="title-group">
                        <FolderOpen size={20} weight="bold" class="cold-icon" />
                        <h2>Discover from Your Library</h2>
                    </div>
                    
                </div>

                <div class="cold-seeds-grid">
                    {#each homeStore.coldStartSeeds as seed}
                        <div 
                            class="cold-seed-card"
                            role="button"
                            tabindex="0"
                            onclick={() => homeStore.playColdStartSeed(seed)}
                            onkeydown={(e) => { if (e.key === 'Enter') homeStore.playColdStartSeed(seed); }}
                        >
                            <div class="cold-art-wrapper">
                                {#if seed.cover_art_url}
                                    <img src={seed.cover_art_url.startsWith('/') ? `asset://localhost/${encodeURIComponent(seed.cover_art_url)}` : seed.cover_art_url} alt={seed.track_title} loading="lazy" />
                                {:else}
                                    <div class="placeholder-art"></div>
                                {/if}
                                <div class="cold-overlay">
                                    <div class="play-bubble">
                                        <Play size={20} weight="fill" />
                                    </div>
                                </div>
                            </div>
                            <div class="cold-info">
                                <span class="cold-title">{seed.track_title}</span>
                                <span class="cold-artist">{seed.artist}</span>
                                {#if seed.album_title}
                                    <span class="cold-album">{seed.album_title}</span>
                                {/if}
                            </div>
                        </div>
                    {/each}
                </div>
            </section>
        {/if}
    {:else}
        <!-- Active Cockpit: 7 Ego Shelves -->

        <!-- 1. ⚡ Quick Picks Grid -->
        {#if homeStore.quickPicks.length > 0}
            <QuickPicksGrid />
        {/if}

        <!-- 2. 🔄 Jump Back In -->
        {#if homeStore.jumpBackIn.length > 0}
            <JumpBackInShelf />
        {/if}

        <!-- 3. ✨ Daily Discover -->
        {#if homeStore.isLoadingRemote || homeStore.dailyDiscover.length > 0}
            <DailyDiscoverCarousel />
        {/if}

        <!-- 4. 📻 Algorithmic Radios -->
        {#if homeStore.radioMixes.length > 0}
            <RadioMixCarousel />
        {/if}

        <!-- 5. 🎲 Adjacent Horizons -->
        {#if homeStore.adjacentHorizon}
            <AdjacentHorizonsCard />
        {/if}

        <!-- 6. ☕ Heavy Rotation (7-Day Top Artists & Albums) -->
        {#if homeStore.heavyRotation.artists.length > 0 || homeStore.heavyRotation.albums.length > 0}
            <HeavyRotationShelf />
        {/if}

        <!-- 7. 📦 Forgotten Favorites -->
        {#if homeStore.forgottenFavorites.length > 0}
            <ForgottenFavoritesShelf />
        {/if}
    {/if}
</div>

<style>
    .home-view {
        padding: 2.5rem 2.5rem var(--player-clearance, 10rem) 2.5rem;
        height: 100%;
        overflow-y: auto;
        scroll-padding-bottom: var(--player-scroll-padding, 10rem);
        display: flex;
        flex-direction: column;
        gap: 3rem;
        width: 100%;
        box-sizing: border-box;
    }

    @media (max-width: 900px) {
        .home-view {
            padding: 1.5rem 1.25rem 10rem 1.25rem;
            gap: 2rem;
        }
    }

    .home-header {
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
    }

    .header-top {
        display: flex;
        align-items: center;
        justify-content: space-between;
    }

    .mood-filter-bar {
        display: flex;
        align-items: center;
        gap: 0.6rem;
        overflow-x: auto;
        padding-bottom: 0.25rem;
        scrollbar-width: none;
    }

    .mood-filter-bar::-webkit-scrollbar {
        display: none;
    }

    .mood-pill {
        font-family: var(--echo-font-body);
        font-size: 0.82rem;
        font-weight: 500;
        color: var(--echo-text-2, rgba(255, 255, 255, 0.6));
        background: #141416;
        border: 1px solid rgba(255, 255, 255, 0.08);
        padding: 0.45rem 1rem;
        border-radius: 20px;
        cursor: pointer;
        white-space: nowrap;
        transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
    }

    .mood-pill.is-glass {
        background: rgba(25, 25, 32, 0.35);
        backdrop-filter: blur(12px) saturate(1.4);
        -webkit-backdrop-filter: blur(12px) saturate(1.4);
        border: 1px solid rgba(255, 255, 255, 0.12);
        box-shadow: 
            0 4px 14px rgba(0, 0, 0, 0.25),
            inset 0 1px 1px rgba(255, 255, 255, 0.18),
            inset 0 -1px 1px rgba(0, 0, 0, 0.2);
        color: var(--echo-text-2, rgba(255, 255, 255, 0.7));
    }

    .mood-pill:hover:not(.active) {
        color: var(--echo-text-1, #eae8e3);
        border-color: rgba(255, 255, 255, 0.18);
        background: #1c1c22;
    }

    .mood-pill.is-glass:hover:not(.active) {
        background: rgba(35, 35, 45, 0.55);
        border-color: rgba(255, 255, 255, 0.22);
        color: var(--echo-text-1, #ffffff);
        box-shadow: 
            0 6px 18px rgba(0, 0, 0, 0.35),
            inset 0 1px 1px rgba(255, 255, 255, 0.28),
            inset 0 -1px 1px rgba(0, 0, 0, 0.2);
    }

    .mood-pill.active {
        color: var(--echo-primary, #e2a973);
        background: rgba(226, 169, 115, 0.12);
        border-color: rgba(226, 169, 115, 0.35);
        font-weight: 600;
    }

    .mood-pill.is-glass.active {
        color: var(--echo-primary, #e2a973);
        background: rgba(226, 169, 115, 0.18);
        border-color: rgba(226, 169, 115, 0.45);
        font-weight: 600;
        box-shadow: 
            0 6px 20px rgba(0, 0, 0, 0.35),
            inset 0 1px 1.5px rgba(226, 169, 115, 0.35),
            inset 0 -1px 1.5px rgba(0, 0, 0, 0.25);
    }

    .home-header h1 {
        font-family: var(--echo-font-heading, "Playfair Display", serif);
        font-size: 2.5rem;
        font-weight: 600;
        letter-spacing: -0.02em;
        margin: 0 0 0.25rem 0;
        color: #fff;
    }

    

    .refresh-btn {
        background: rgba(255, 255, 255, 0.08);
        border: 1px solid rgba(255, 255, 255, 0.12);
        border-radius: 50%;
        width: 36px;
        height: 36px;
        display: flex;
        align-items: center;
        justify-content: center;
        color: #FFFFFF;
        cursor: pointer;
        padding: 0;
        box-shadow: 0 4px 16px rgba(0, 0, 0, 0.35);
        transition: color 0.15s ease, background-color 0.15s ease, border-color 0.15s ease, transform 0.1s ease;
    }

    .refresh-btn :global(svg) {
        display: block;
        flex-shrink: 0;
        fill: currentColor;
    }

    .refresh-btn.is-glass {
        background: rgba(25, 25, 32, 0.45);
        backdrop-filter: blur(12px) saturate(1.4);
        -webkit-backdrop-filter: blur(12px) saturate(1.4);
        border: 1px solid rgba(255, 255, 255, 0.14);
        box-shadow: 
            0 4px 14px rgba(0, 0, 0, 0.25),
            inset 0 1px 1px rgba(255, 255, 255, 0.18),
            inset 0 -1px 1px rgba(0, 0, 0, 0.2);
    }

    .refresh-btn:hover {
        background: rgba(255, 255, 255, 0.16);
        color: #B58E62;
        border-color: rgba(181, 142, 98, 0.4);
    }

    .refresh-btn.is-glass:hover {
        background: rgba(45, 45, 60, 0.65);
        border-color: rgba(181, 142, 98, 0.4);
        color: #B58E62;
        box-shadow: 
            0 6px 18px rgba(0, 0, 0, 0.35),
            inset 0 1px 1px rgba(255, 255, 255, 0.28),
            inset 0 -1px 1px rgba(0, 0, 0, 0.2);
    }

    .refresh-btn:active {
        transform: scale(0.92);
    }

    .refresh-btn.spinning {
        animation: spin 1s linear infinite;
    }

    @keyframes spin {
        from { transform: rotate(0deg); }
        to { transform: rotate(360deg); }
    }

    /* Welcome Hero Card */
    .welcome-hero-card {
        background: linear-gradient(135deg, rgba(255, 255, 255, 0.08) 0%, rgba(255, 255, 255, 0.02) 100%);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 16px;
        padding: 2.5rem 2.5rem 10rem 2.5rem;
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

    .welcome-hero-card h2 {
        font-size: 1.75rem;
        margin: 0;
    }

    .welcome-hero-card p {
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

    /* Cold Start Local Sampler */
    .cold-start-shelf {
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

    :global(.cold-icon) {
        color: #48cae4;
    }

    

    .cold-seeds-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
        gap: 1rem;
    }

    .cold-seed-card {
        display: flex;
        flex-direction: column;
        gap: 0.55rem;
        cursor: pointer;
        transition: transform 0.2s cubic-bezier(0.16, 1, 0.3, 1);
    }

    .cold-seed-card:hover {
        transform: translateY(-4px);
    }

    .cold-art-wrapper {
        width: 100%;
        aspect-ratio: 1;
        border-radius: 10px;
        overflow: hidden;
        position: relative;
        background: var(--surface-1, rgba(255, 255, 255, 0.04));
        border: 1px solid rgba(255, 255, 255, 0.06);
    }

    .cold-art-wrapper img {
        width: 100%;
        height: 100%;
        object-fit: cover;
    }

    .placeholder-art {
        width: 100%;
        height: 100%;
        background: linear-gradient(135deg, rgba(255, 255, 255, 0.08), rgba(255, 255, 255, 0.02));
    }

    .cold-overlay {
        position: absolute;
        inset: 0;
        background: rgba(0, 0, 0, 0.4);
        display: flex;
        align-items: center;
        justify-content: center;
        opacity: 0;
        transition: opacity 0.2s ease;
    }

    .cold-seed-card:hover .cold-overlay {
        opacity: 1;
    }

    .play-bubble {
        width: 40px;
        height: 40px;
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

    .cold-seed-card:hover .play-bubble {
        transform: scale(1);
    }

    .cold-info {
        display: flex;
        flex-direction: column;
        gap: 0.15rem;
    }

    .cold-title {
        font-size: 0.9rem;
        font-weight: 600;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .cold-artist {
        font-size: 0.78rem;
        color: var(--text-muted, rgba(255, 255, 255, 0.6));
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .cold-album {
        font-size: 0.7rem;
        color: var(--text-muted, rgba(255, 255, 255, 0.45));
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }
</style>
