<script lang="ts">
    import { onMount } from "svelte";
    import { homeStore } from "$lib/stores/home.svelte";
    import { flagsStore } from "$lib/stores/flags.svelte";
    import { settingsStore } from "$lib/stores/settings.svelte";
    import { ArrowClockwise, Compass, FolderOpen, MusicNotes } from "phosphor-svelte";
    import CardPlayButton from "$lib/components/common/CardPlayButton.svelte";

    import FeaturedDailyHero from "./home/FeaturedDailyHero.svelte";
    import AdjacentHorizonsCard from "./home/AdjacentHorizonsCard.svelte";
    import QuickPicksGrid from "./home/QuickPicksGrid.svelte";
    import RadioTunerDock from "./home/RadioTunerDock.svelte";
    import DailyDiscoverCarousel from "./home/DailyDiscoverCarousel.svelte";
    import JumpBackInShelf from "./home/JumpBackInShelf.svelte";
    import HeavyRotationShelf from "./home/HeavyRotationShelf.svelte";
    import ForgottenFavoritesShelf from "./home/ForgottenFavoritesShelf.svelte";
    import SectionHeaderSkeleton from "$lib/components/common/SectionHeaderSkeleton.svelte";

    let { activeView = $bindable("home") } = $props<{ activeView?: string }>();
    
    onMount(() => {
        homeStore.init();
    });

    let hasJumpBackIn = $derived(
        flagsStore.isEnabled("home_jump_back_in") && homeStore.jumpBackIn.length > 0
    );

    let hasHeavyRotation = $derived(
        flagsStore.isEnabled("home_heavy_rotation") && 
        (homeStore.heavyRotation.artists.length > 0 || homeStore.heavyRotation.albums.length > 0)
    );

    let hasForgottenFavorites = $derived(
        flagsStore.isEnabled("home_forgotten_favorites") && 
        homeStore.forgottenFavorites.length > 0
    );
</script>

<div class="home-view">
    <!-- Header & Quick-Filter Mood Bar -->
    <header class="home-header">
        <div class="header-top">
            <div class="header-left">
                <h1>Listen Now</h1>
                <p class="header-subtitle">Music and mixes tailored to your listening taste.</p>
            </div>
            <div class="header-actions">
                <button 
                    class="refresh-btn" 
                    class:is-glass={settingsStore.glassyPlayerBar}
                    class:spinning={homeStore.isLoadingRemote}
                    onclick={() => homeStore.loadHome(true)}
                    title="Refresh recommendations"
                    aria-label="Refresh recommendations"
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
                <span>LOCAL & FEDERATED AUDIO</span>
            </div>
            <h2>Welcome to Lyria</h2>
            <p>Start playing music from your local library or explore global releases. Lyria learns from your unique listening telemetry and generates dynamic daily mixes automatically.</p>
            
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
                        <!-- svelte-ignore a11y_click_events_have_key_events -->
                        <!-- svelte-ignore a11y_no_static_element_interactions -->
                        <div 
                            class="cold-seed-card group"
                            onclick={() => homeStore.playColdStartSeed(seed)}
                        >
                            <div class="cold-art-wrapper">
                                {#if seed.cover_art_url}
                                    <img src={seed.cover_art_url.startsWith('/') ? `asset://localhost/${encodeURIComponent(seed.cover_art_url)}` : seed.cover_art_url} alt="Cover art for {seed.track_title}" loading="lazy" />
                                {:else}
                                    <div class="placeholder-art">
                                        <MusicNotes size={40} weight="thin" color="rgba(255, 255, 255, 0.35)" />
                                    </div>
                                {/if}
                                <div class="cold-overlay">
                                    <CardPlayButton 
                                        onclick={(e) => { e.stopPropagation(); homeStore.playColdStartSeed(seed); }}
                                        title="Play {seed.track_title}"
                                    />
                                </div>
                            </div>
                            <div class="cold-info">
                                <span class="cold-title" title={seed.track_title}>{seed.track_title}</span>
                                <span class="cold-artist" title={seed.artist}>{seed.artist}</span>
                                {#if seed.album_title}
                                    <span class="cold-album" title={seed.album_title}>{seed.album_title}</span>
                                {/if}
                            </div>
                        </div>
                    {/each}
                </div>
            </section>
        {/if}
    {:else}
        <!-- ═══════════════════════════════════════════════════════════════════
             ACTIVE 3-ZONE HIGH-END ALGORITHMIC COCKPIT
             ═══════════════════════════════════════════════════════════════════ -->

        <!-- Optional: Incomplete Session Resumption Banner -->
        {#if hasJumpBackIn}
            <div class="zone-resumption">
                <JumpBackInShelf />
            </div>
        {/if}

        <!-- ZONE 1: Quick Picks + Radio Stations -->
        <div class="cockpit-bento-grid">
            <div class="cockpit-main-column">
                <QuickPicksGrid limit={9} />
            </div>
            <div class="cockpit-side-column">
                <RadioTunerDock />
            </div>
        </div>

        <!-- ZONE 2: Spotlight Stage (Adjacent Horizons or Daily Hero) -->
        <div class="zone-stage">
            {#if flagsStore.isEnabled("home_adjacent_horizons") && homeStore.adjacentHorizon}
                <AdjacentHorizonsCard />
            {:else}
                <FeaturedDailyHero />
            {/if}
        </div>

        <!-- ZONE 3: Deep Shelves & Archival Exploration (Below the Fold) -->
        <div class="zone-deep-shelves">
            <!-- 1. ✨ Daily Discover (Federated Exploration) -->
            {#if flagsStore.isEnabled("home_daily_discover") && (homeStore.isLoadingRemote || homeStore.dailyDiscover.length > 0)}
                <DailyDiscoverCarousel />
            {/if}

            <!-- 2 & 3. ☕ Heavy Rotation + 📦 Forgotten Favorites (Library Telemetry 2-Column Bento) -->
            {#if hasHeavyRotation || hasForgottenFavorites}
                <div 
                    class="telemetry-bento-grid"
                    class:single-column={!hasHeavyRotation || !hasForgottenFavorites}
                >
                    {#if hasHeavyRotation}
                        <div class="telemetry-column">
                            <HeavyRotationShelf />
                        </div>
                    {/if}

                    {#if hasForgottenFavorites}
                        <div class="telemetry-column">
                            <ForgottenFavoritesShelf />
                        </div>
                    {/if}
                </div>
            {:else if !homeStore.phase1Loaded}
                <!-- Phase 1 Telemetry Bento Skeleton Loading State -->
                <div class="telemetry-bento-grid">
                    <div class="telemetry-column">
                        <section class="shelf-skeleton-section">
                            <SectionHeaderSkeleton titleWidth="150px" hasControls={true} />
                            <div class="shelf-skeleton-track">
                                {#each Array(3) as _}
                                    <div class="shelf-skeleton-card">
                                        <div class="skeleton-art-box skeleton-box"></div>
                                        <div class="skeleton-line skeleton-box shelf-card-title"></div>
                                        <div class="skeleton-line skeleton-box shelf-card-sub"></div>
                                    </div>
                                {/each}
                            </div>
                        </section>
                    </div>

                    <div class="telemetry-column">
                        <section class="shelf-skeleton-section">
                            <SectionHeaderSkeleton titleWidth="170px" hasControls={true} />
                            <div class="shelf-skeleton-track">
                                {#each Array(3) as _}
                                    <div class="shelf-skeleton-card">
                                        <div class="skeleton-art-box skeleton-box"></div>
                                        <div class="skeleton-line skeleton-box shelf-card-title"></div>
                                        <div class="skeleton-line skeleton-box shelf-card-sub"></div>
                                    </div>
                                {/each}
                            </div>
                        </section>
                    </div>
                </div>
            {/if}

        </div>
    {/if}
</div>

<style>
    .home-view {
        padding: 2.5rem 2.5rem var(--player-clearance, 7.5rem) 2.5rem;
        scroll-padding-bottom: var(--player-scroll-padding, 7.5rem);
        display: flex;
        flex-direction: column;
        gap: 3rem;
        width: 100%;
        max-width: 100%;
        min-width: 0;
        box-sizing: border-box;
    }

    @media (max-width: 900px) {
        .home-view {
            padding: 1.5rem 1.25rem 7.5rem 1.25rem;
            gap: 2.25rem;
        }
    }

    /* ─── Header & Mood Bar ─── */
    .home-header {
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
        position: relative;
        z-index: 2;
        transform: translateZ(0);
        backface-visibility: hidden;
        isolation: isolate;
    }

    .header-top {
        display: flex;
        align-items: flex-start;
        justify-content: space-between;
    }

    .header-left {
        display: flex;
        flex-direction: column;
        gap: 0.35rem;
        min-width: 0;
    }


    .home-header h1 {
        font-family: var(--lyria-font-heading, sans-serif);
        font-size: 2.35rem;
        font-weight: 600;
        letter-spacing: -0.03em;
        line-height: 1.2;
        margin: 0;
        color: var(--echo-text-1, #eae8e3);
        transform: translateZ(0);
        backface-visibility: hidden;
    }

    .header-subtitle {
        font-size: 0.88rem;
        color: var(--echo-text-2, rgba(255, 255, 255, 0.6));
        margin: 0;
        line-height: 1.45;
        max-width: 650px;
    }

    .refresh-btn {
        background: rgba(255, 255, 255, 0.08);
        border: 1px solid rgba(255, 255, 255, 0.12);
        border-radius: 50%;
        width: 36px;
        height: 36px;
        margin-top: 4px;
        display: flex;
        align-items: center;
        justify-content: center;
        color: #ffffff;
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
        background: linear-gradient(180deg, rgba(255, 255, 255, 0.08) 0%, rgba(255, 255, 255, 0.02) 100%);
        border: 1.5px solid rgba(255, 255, 255, 0.10);
        border-top-color: rgba(255, 255, 255, 0.28);
        border-bottom-color: rgba(255, 255, 255, 0.06);
        box-shadow: 
            inset 0 1px 2px rgba(255, 255, 255, 0.10),
            0 4px 14px rgba(0, 0, 0, 0.35);
    }

    .refresh-btn:hover {
        background: rgba(255, 255, 255, 0.16);
        color: #ffffff;
        border-color: rgba(255, 255, 255, 0.25);
    }

    .refresh-btn.is-glass:hover {
        background: linear-gradient(180deg, rgba(255, 255, 255, 0.15) 0%, rgba(255, 255, 255, 0.04) 100%);
        border-color: rgba(255, 255, 255, 0.20);
        border-top-color: rgba(255, 255, 255, 0.45);
        color: #ffffff;
        box-shadow: 
            inset 0 1px 2px rgba(255, 255, 255, 0.22),
            0 6px 18px rgba(0, 0, 0, 0.45);
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

    .mood-filter-bar {
        display: flex;
        align-items: center;
        gap: 0.6rem;
        overflow-x: auto;
        padding-bottom: 0.25rem;
        scrollbar-width: none;
        max-width: 100%;
        min-width: 0;
    }

    .mood-filter-bar::-webkit-scrollbar {
        display: none;
    }

    .mood-pill {
        font-family: var(--echo-font-body, system-ui, sans-serif);
        font-size: 0.82rem;
        font-weight: 500;
        color: var(--echo-text-2, rgba(255, 255, 255, 0.6));
        background: var(--echo-surface-pill);
        border: 1px solid var(--echo-border-glass);
        padding: 0.45rem 1rem;
        border-radius: 20px;
        cursor: pointer;
        white-space: nowrap;
        transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
    }

    .mood-pill.is-glass {
        background: var(--echo-glass-gradient-raised);
        border: 1.5px solid rgba(255, 255, 255, 0.10);
        border-top-color: rgba(255, 255, 255, 0.24);
        border-bottom-color: rgba(255, 255, 255, 0.06);
        box-shadow: 
            inset 0 1px 2px rgba(255, 255, 255, 0.08),
            0 4px 12px rgba(0, 0, 0, 0.35);
        color: var(--echo-text-2, rgba(255, 255, 255, 0.7));
    }

    .mood-pill:hover:not(.active) {
        color: var(--echo-text-1, #eae8e3);
        border-color: var(--echo-border-glass-hover);
        background: var(--echo-surface-pill-hover);
    }

    .mood-pill.is-glass:hover:not(.active) {
        background: var(--echo-glass-gradient-hover);
        border-color: var(--echo-border-glass-hover);
        border-top-color: rgba(255, 255, 255, 0.38);
        color: var(--echo-text-1, #ffffff);
        box-shadow: 
            inset 0 1px 2px rgba(255, 255, 255, 0.15),
            0 6px 18px rgba(0, 0, 0, 0.45);
    }

    .mood-pill.active {
        color: #ffffff;
        background: var(--echo-glass-gradient-primary);
        border-color: var(--echo-border-primary);
        font-weight: 600;
    }

    .mood-pill.is-glass.active {
        color: #ffffff;
        background: var(--echo-glass-gradient-primary-glass);
        border: 1.5px solid var(--echo-border-primary);
        border-top-color: rgba(255, 230, 195, 0.85);
        border-bottom-color: rgba(160, 105, 55, 0.30);
        backdrop-filter: blur(12px) saturate(130%) brightness(1.08);
        -webkit-backdrop-filter: blur(12px) saturate(130%) brightness(1.08);
        font-weight: 600;
        box-shadow: 
            inset 0 1px 2px 0 rgba(255, 235, 205, 0.35),
            inset 0 -1px 2px 0 rgba(140, 95, 50, 0.25),
            0 4px 16px rgba(226, 169, 115, 0.20),
            0 2px 6px rgba(0, 0, 0, 0.4);
        text-shadow: 0 1px 2px rgba(0, 0, 0, 0.4);
    }

    /* ─── Zone 1: Stage ─── */
    .zone-stage {
        width: 100%;
    }

    .zone-resumption {
        width: 100%;
    }

    /* ─── Zone 2: Cockpit Bento Grid ─── */
    .cockpit-bento-grid {
        display: grid;
        grid-template-columns: minmax(0, 1.4fr) minmax(0, 1fr);
        gap: 2.25rem;
        align-items: start;
        width: 100%;
    }

    .cockpit-main-column,
    .cockpit-side-column {
        min-width: 0;
        width: 100%;
    }

    @media (max-width: 1100px) {
        .cockpit-bento-grid {
            grid-template-columns: 1fr;
            gap: 2.5rem;
        }
    }

    /* ─── Zone 3: Deep Shelves ─── */
    .zone-deep-shelves {
        display: flex;
        flex-direction: column;
        gap: 3rem;
        width: 100%;
        min-width: 0;
    }

    /* ─── Telemetry Bento Grid (Heavy Rotation + Forgotten Favorites) ─── */
    .telemetry-bento-grid {
        display: grid;
        grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
        gap: 2.5rem;
        align-items: start;
        width: 100%;
    }

    .telemetry-bento-grid.single-column {
        grid-template-columns: minmax(0, 1fr);
    }

    .telemetry-column {
        min-width: 0;
        width: 100%;
    }

    @media (max-width: 1100px) {
        .telemetry-bento-grid {
            grid-template-columns: minmax(0, 1fr);
            gap: 3rem;
        }
    }

    /* ─── Shelf Skeletons ─── */
    .shelf-skeleton-section {
        display: flex;
        flex-direction: column;
        gap: 1.25rem;
        width: 100%;
        min-width: 0;
        pointer-events: none;
    }

    .shelf-skeleton-track {
        display: flex;
        gap: 1.25rem;
        overflow: hidden;
        width: 100%;
    }

    .shelf-skeleton-card {
        flex: 0 0 176px;
        width: 176px;
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .shelf-skeleton-card .skeleton-art-box {
        width: 176px;
        height: 176px;
        border-radius: 12px;
        border: 1px solid var(--echo-border-glass);
    }

    .shelf-skeleton-card .shelf-card-title {
        width: 75%;
        height: 13px;
    }

    .shelf-skeleton-card .shelf-card-sub {
        width: 45%;
        height: 10px;
    }

    /* ─── Welcome / Cold Start ─── */
    .welcome-hero-card {
        background: var(--echo-glass-gradient-base);
        border: 1px solid var(--echo-border-glass);
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
        color: #fff;
    }

    .welcome-hero-card p {
        margin: 0;
        max-width: 600px;
        color: var(--echo-text-2, rgba(255, 255, 255, 0.7));
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
        font-family: var(--echo-font-heading, sans-serif);
        font-size: 1.35rem;
        font-weight: 600;
        margin: 0;
        letter-spacing: -0.02em;
        color: #fff;
    }

    :global(.cold-icon) {
        color: #b58e62;
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
        border-radius: 12px;
        overflow: hidden;
        position: relative;
        background: var(--echo-surface-pill);
        border: 1px solid var(--echo-border-glass);
        transition: border-color 0.2s ease, box-shadow 0.2s ease;
    }

    .cold-seed-card:hover .cold-art-wrapper {
        border-color: rgba(181, 142, 98, 0.35);
        box-shadow: 0 10px 24px -6px rgba(0, 0, 0, 0.6);
    }

    .cold-art-wrapper img {
        width: 100%;
        height: 100%;
        object-fit: cover;
    }

    .placeholder-art {
        width: 100%;
        height: 100%;
        display: flex;
        align-items: center;
        justify-content: center;
        background: radial-gradient(circle at center, #18181b 0%, #09090b 100%);
    }

    .cold-overlay {
        position: absolute;
        inset: 0;
        background: rgba(0, 0, 0, 0.35);
        backdrop-filter: blur(2px);
        display: flex;
        align-items: center;
        justify-content: center;
        opacity: 0;
        pointer-events: none;
        transition: opacity 0.2s cubic-bezier(0.16, 1, 0.3, 1);
    }

    .cold-seed-card:hover .cold-overlay {
        opacity: 1;
        pointer-events: auto;
    }

    .cold-info {
        display: flex;
        flex-direction: column;
        gap: 0.2rem;
    }

    .cold-title {
        font-family: var(--echo-font-body, system-ui, sans-serif);
        font-size: 0.88rem;
        font-weight: 600;
        color: #fff;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .cold-artist {
        font-family: var(--echo-font-body, system-ui, sans-serif);
        font-size: 0.78rem;
        color: var(--echo-text-2, rgba(255, 255, 255, 0.6));
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .cold-album {
        font-family: var(--echo-font-body, system-ui, sans-serif);
        font-size: 0.72rem;
        color: var(--echo-text-3, rgba(255, 255, 255, 0.45));
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }
</style>
