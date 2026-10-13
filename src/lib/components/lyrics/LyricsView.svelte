<script lang="ts">
    import { lyricsStore, type LyricLine, type LyricsSearchResultItem } from "$lib/stores/lyrics.svelte";
    import { audioStore } from "$lib/stores/audio.svelte";
    import { MicrophoneStage, ArrowClockwise, Sparkle, MagnifyingGlass, ArrowLeft, X, Check } from "phosphor-svelte";
    import { formatDuration } from "$lib/utils/format";
    import { fade } from "svelte/transition";

    let { variant = "drawer" } = $props<{
        variant?: "drawer" | "fullscreen";
    }>();

    let containerEl = $state<HTMLElement | null>(null);
    let scrollTimeout: ReturnType<typeof setTimeout> | null = null;
    let manualScrollPause = $state(false);
    let searchDebounceTimeout: ReturnType<typeof setTimeout> | null = null;

    let playingDurationMs = $derived(Math.round(audioStore.duration * 1000));
    let targetTitle = $derived(audioStore.currentQueueTrack?.title || "Unknown Title");
    let targetArtist = $derived(audioStore.currentQueueTrack?.artist || "Unknown Artist");

    // Smoothly scroll the container to center a specific line element
    function scrollToIndex(index: number, smooth = true) {
        if (!containerEl || index < 0) return;
        const el = containerEl.querySelector(`[data-index="${index}"]`) as HTMLElement | null;
        if (!el) return;

        const containerRect = containerEl.getBoundingClientRect();
        const elRect = el.getBoundingClientRect();
        const targetScrollTop =
            containerEl.scrollTop + (elRect.top - containerRect.top) - containerEl.clientHeight / 2 + elRect.height / 2;

        containerEl.scrollTo({
            top: Math.max(0, targetScrollTop),
            behavior: smooth ? "smooth" : "instant",
        });
    }

    // Auto-scroll the active line into center view
    $effect(() => {
        const activeIdx = lyricsStore.activeLineIndex;
        if (activeIdx < 0 || manualScrollPause || !containerEl || lyricsStore.isSearchOpen) return;
        scrollToIndex(activeIdx, true);
    });

    // Detect user manual scroll to prevent fighting the auto-scroll for 4 seconds
    function handleUserScroll() {
        if (lyricsStore.isSearchOpen) return;
        manualScrollPause = true;
        if (scrollTimeout) clearTimeout(scrollTimeout);
        scrollTimeout = setTimeout(() => {
            manualScrollPause = false;
        }, 4000);
    }

    function handleLineClick(index: number) {
        if (scrollTimeout) {
            clearTimeout(scrollTimeout);
            scrollTimeout = null;
        }
        manualScrollPause = false;
        lyricsStore.seekToLine(index, 150);
        scrollToIndex(index, true);
    }

    function handleInterludeClick(targetMs: number) {
        if (scrollTimeout) {
            clearTimeout(scrollTimeout);
            scrollTimeout = null;
        }
        manualScrollPause = false;
        audioStore.seek(targetMs / 1000);
    }

    function resumeSyncNow() {
        if (scrollTimeout) {
            clearTimeout(scrollTimeout);
            scrollTimeout = null;
        }
        manualScrollPause = false;
        if (lyricsStore.activeLineIndex >= 0) {
            scrollToIndex(lyricsStore.activeLineIndex, true);
        }
    }

    function retryWithProvider(providerId: string) {
        lyricsStore.fetchForCurrentTrack(providerId);
    }

    function handleSearchInput() {
        if (searchDebounceTimeout) clearTimeout(searchDebounceTimeout);
        searchDebounceTimeout = setTimeout(() => {
            lyricsStore.executeSearch(lyricsStore.searchQuery, lyricsStore.searchProviderOverride || undefined);
        }, 300);
    }

    function handleSearchKeydown(e: KeyboardEvent) {
        if (e.key === "Enter") {
            if (searchDebounceTimeout) clearTimeout(searchDebounceTimeout);
            lyricsStore.executeSearch(lyricsStore.searchQuery, lyricsStore.searchProviderOverride || undefined);
        } else if (e.key === "Escape") {
            lyricsStore.closeSearch();
        }
    }

    function handleSelectProviderFilter(providerId: string | null) {
        lyricsStore.searchProviderOverride = providerId;
        lyricsStore.executeSearch(lyricsStore.searchQuery, providerId || undefined);
    }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
    class="lyrics-container {variant}"
    bind:this={containerEl}
    onwheel={handleUserScroll}
    ontouchstart={handleUserScroll}
>
    {#if lyricsStore.isSearchOpen}
        <div class="search-panel" transition:fade={{ duration: 150 }}>
            <!-- Search Top Bar -->
            <div class="search-topbar">
                <button
                    type="button"
                    class="search-back-btn"
                    onclick={() => lyricsStore.closeSearch()}
                    title="Back to lyrics"
                >
                    <ArrowLeft size={18} weight="bold" />
                </button>
                <div class="search-input-shell">
                    <MagnifyingGlass size={16} class="search-input-icon" />
                    <input
                        type="text"
                        class="search-text-input"
                        bind:value={lyricsStore.searchQuery}
                        oninput={handleSearchInput}
                        onkeydown={handleSearchKeydown}
                        placeholder="Search title, artist, or tags..."
                    />
                    {#if lyricsStore.searchQuery.length > 0}
                        <button
                            type="button"
                            class="search-clear-btn"
                            onclick={() => { lyricsStore.searchQuery = ""; }}
                            aria-label="Clear query"
                        >
                            <X size={13} weight="bold" />
                        </button>
                    {/if}
                </div>
            </div>

            <!-- Provider filter pills -->
            <div class="search-providers-bar">
                <button
                    type="button"
                    class="provider-filter-pill"
                    class:active={!lyricsStore.searchProviderOverride}
                    onclick={() => handleSelectProviderFilter(null)}
                >
                    All Providers
                </button>
                {#each lyricsStore.providers as p}
                    <button
                        type="button"
                        class="provider-filter-pill"
                        class:active={lyricsStore.searchProviderOverride === p.id}
                        onclick={() => handleSelectProviderFilter(p.id)}
                    >
                        {p.name.replace(" Lyrics Provider", "")}
                    </button>
                {/each}
            </div>

            <!-- Context track info & Reset option -->
            <div class="search-context-banner">
                <div class="context-text">
                    <span class="context-label">For:</span>
                    <span class="context-song">{targetArtist} — {targetTitle}</span>
                </div>
                {#if lyricsStore.currentLyrics}
                    <button
                        type="button"
                        class="revert-link-btn"
                        onclick={() => lyricsStore.revertToAuto()}
                        title="Clear manual override and reset to automatic match"
                    >
                        Reset to auto
                    </button>
                {/if}
            </div>

            <!-- Search Results State Area -->
            <div class="search-results-container">
                {#if lyricsStore.isSearching}
                    <div class="status-box">
                        <div class="spinner"></div>
                        <p class="status-text">Searching lyrics providers...</p>
                    </div>
                {:else if lyricsStore.searchError}
                    <div class="status-box">
                        <p class="status-title">Search Error</p>
                        <p class="status-desc">{lyricsStore.searchError}</p>
                        <button
                            class="retry-btn"
                            onclick={() => lyricsStore.executeSearch(lyricsStore.searchQuery, lyricsStore.searchProviderOverride || undefined)}
                        >
                            <ArrowClockwise size={15} />
                            <span>Retry</span>
                        </button>
                    </div>
                {:else if lyricsStore.searchResults.length === 0}
                    <div class="status-box">
                        <MicrophoneStage size={36} weight="thin" class="status-icon" />
                        <p class="status-title">No Lyrics Found</p>
                        <p class="status-desc">
                            Try simplifying your query (e.g. removing "(Remastered)" or "[Live]").
                        </p>
                    </div>
                {:else}
                    <div class="candidate-ledger">
                        <div class="results-header">
                            <span class="results-count">Matches ({lyricsStore.searchResults.length})</span>
                            <span class="results-hint">Click to apply & pin</span>
                        </div>
                        {#each lyricsStore.searchResults as candidate}
                            {@const candDur = candidate.duration_ms || 0}
                            {@const diffSec = playingDurationMs > 0 && candDur > 0 ? Math.round((candDur - playingDurationMs) / 1000) : 0}
                            {@const isExactDuration = Math.abs(diffSec) <= 3}

                            <button
                                type="button"
                                class="candidate-card"
                                onclick={() => lyricsStore.selectAndApply(candidate)}
                            >
                                <div class="candidate-header">
                                    <span class="candidate-title">{candidate.title}</span>
                                    <span class="candidate-artist">
                                        {candidate.artist}{candidate.album ? ` • ${candidate.album}` : ""}
                                    </span>
                                </div>

                                {#if candidate.preview_text}
                                    <p class="candidate-snippet">"{candidate.preview_text}"</p>
                                {/if}

                                <div class="candidate-pills">
                                    <span class="format-badge" class:synced={candidate.format === "synced"}>
                                        {candidate.format === "synced" ? "Synced LRC" : "Plain"}
                                    </span>
                                    {#if candDur > 0}
                                        <span class="duration-badge" class:exact-match={isExactDuration}>
                                            {formatDuration(candDur)}
                                            {#if playingDurationMs > 0 && diffSec !== 0}
                                                <span class="dur-diff">({diffSec > 0 ? `+${diffSec}` : diffSec}s)</span>
                                            {/if}
                                        </span>
                                    {/if}
                                    <span class="provider-badge">{candidate.provider_name}</span>
                                </div>
                            </button>
                        {/each}
                    </div>
                {/if}
            </div>
        </div>
    {:else if lyricsStore.isLoading}
        <div class="status-box">
            <div class="spinner"></div>
            <p class="status-text">Fetching lyrics...</p>
        </div>
    {:else if lyricsStore.error}
        <div class="status-box">
            <MicrophoneStage size={36} weight="light" class="status-icon" />
            <p class="status-title">Unable to Load Lyrics</p>
            <p class="status-desc">{lyricsStore.error}</p>
            <div class="status-actions">
                <button class="retry-btn" onclick={() => lyricsStore.fetchForCurrentTrack()}>
                    <ArrowClockwise size={16} />
                    <span>Retry</span>
                </button>
                <button class="manual-search-btn" onclick={() => lyricsStore.openSearch()}>
                    <MagnifyingGlass size={15} weight="bold" />
                    <span>Search Manually</span>
                </button>
            </div>
        </div>
    {:else if !lyricsStore.hasLyrics}
        <div class="status-box">
            <MicrophoneStage size={40} weight="thin" class="status-icon" />
            <p class="status-title">
                {lyricsStore.currentLyrics?.instrumental ? "Instrumental Track" : "No Lyrics Available"}
            </p>
            <p class="status-desc">
                {audioStore.currentQueueTrack?.artist || "Unknown Artist"} — {audioStore.currentQueueTrack?.title || "Unknown Title"}
            </p>

            <button class="manual-search-pill" onclick={() => lyricsStore.openSearch()}>
                <MagnifyingGlass size={14} weight="bold" />
                <span>Search Lyrics Manually</span>
            </button>

            <!-- Provider fallback pills -->
            <div class="provider-switch">
                <span class="provider-switch-label">Try searching with:</span>
                <div class="provider-pills">
                    {#each lyricsStore.providers as p}
                        <button
                            class="provider-pill"
                            class:active={lyricsStore.currentLyrics?.provider_id === p.id}
                            onclick={() => retryWithProvider(p.id)}
                        >
                            <Sparkle size={12} weight="fill" />
                            <span>{p.name.replace(" Lyrics Provider", "")}</span>
                        </button>
                    {/each}
                </div>
            </div>
        </div>
    {:else if lyricsStore.isSynced}
        {@const lines = lyricsStore.currentLyrics?.lines || []}
        {@const curMs = audioStore.currentTime * 1000}
        <div class="lyrics-top-bar">
            <button
                type="button"
                class="search-mode-link"
                onclick={() => lyricsStore.openSearch()}
                title="Search and select different lyrics"
            >
                <MagnifyingGlass size={13} weight="bold" />
                <span>Wrong lyrics?</span>
            </button>
        </div>

        <div class="lyrics-list">
            <!-- Instrumental intro before first line -->
            {#if lines.length > 0 && lines[0].time_ms >= 6000}
                {@const isIntroActive = curMs < lines[0].time_ms}
                <button
                    type="button"
                    class="instrumental-break"
                    class:active={isIntroActive}
                    onclick={() => handleInterludeClick(0)}
                    title="Instrumental intro (jump to start)"
                >
                    <span class="break-dot"></span>
                    <span class="break-dot"></span>
                    <span class="break-dot"></span>
                </button>
            {/if}

            {#each lines as line, index}
                {@const isActive = index === lyricsStore.activeLineIndex}
                {@const isPast = lyricsStore.activeLineIndex >= 0 && index < lyricsStore.activeLineIndex}
                {@const nextLine = lines[index + 1]}
                {@const gapMs = nextLine ? nextLine.time_ms - line.time_ms : 0}

                <button
                    class="lyric-line"
                    class:active={isActive}
                    class:past={isPast}
                    data-index={index}
                    onclick={() => handleLineClick(index)}
                    title="Click to jump to line"
                >
                    <span class="line-text">{line.text || "♪"}</span>
                </button>

                <!-- Instrumental break between lines if gap >= 7 seconds -->
                {#if gapMs >= 7000}
                    {@const isBreakActive = curMs >= line.time_ms + 2500 && curMs < nextLine.time_ms}
                    <button
                        type="button"
                        class="instrumental-break"
                        class:active={isBreakActive}
                        onclick={() => handleInterludeClick(line.time_ms + 2500)}
                        title="Instrumental break (jump to section)"
                    >
                        <span class="break-dot"></span>
                        <span class="break-dot"></span>
                        <span class="break-dot"></span>
                    </button>
                {/if}
            {/each}

            {#if lyricsStore.currentLyrics?.provider_name}
                <footer class="lyrics-footer">
                    <span>Lyrics provided by {lyricsStore.currentLyrics.provider_name}</span>
                    <span class="footer-dot">•</span>
                    <button type="button" class="footer-search-link" onclick={() => lyricsStore.openSearch()}>
                        Wrong lyrics?
                    </button>
                </footer>
            {/if}
        </div>

        {#if manualScrollPause && lyricsStore.activeLineIndex >= 0}
            <button
                type="button"
                class="sync-pill-btn"
                transition:fade={{ duration: 150 }}
                onclick={resumeSyncNow}
                title="Re-sync with current playback"
            >
                <Sparkle size={13} weight="fill" />
                <span>Sync with playback</span>
            </button>
        {/if}
    {:else}
        <!-- Plain unsynchronized lyrics -->
        <div class="lyrics-top-bar">
            <button
                type="button"
                class="search-mode-link"
                onclick={() => lyricsStore.openSearch()}
                title="Search and select different lyrics"
            >
                <MagnifyingGlass size={13} weight="bold" />
                <span>Wrong lyrics?</span>
            </button>
        </div>

        <div class="plain-lyrics">
            <pre class="plain-text">{lyricsStore.currentLyrics?.raw_plain || ""}</pre>
            {#if lyricsStore.currentLyrics?.provider_name}
                <footer class="lyrics-footer">
                    <span>Lyrics provided by {lyricsStore.currentLyrics.provider_name}</span>
                    <span class="footer-dot">•</span>
                    <button type="button" class="footer-search-link" onclick={() => lyricsStore.openSearch()}>
                        Wrong lyrics?
                    </button>
                </footer>
            {/if}
        </div>
    {/if}
</div>

<style>
    .lyrics-container {
        height: 100%;
        width: 100%;
        overflow-y: auto;
        display: flex;
        flex-direction: column;
        scroll-behavior: smooth;
        scrollbar-width: thin;
        scrollbar-color: rgba(255, 255, 255, 0.1) transparent;
        padding: 2rem 1.5rem;
        box-sizing: border-box;
    }

    .lyrics-container.fullscreen {
        padding: 4rem 3rem;
        align-items: flex-start;
        justify-content: flex-start;
    }

    .lyrics-list {
        display: flex;
        flex-direction: column;
        gap: 1.25rem;
        width: 100%;
        padding-top: 30vh;
        padding-bottom: 45vh;
    }

    .lyric-line {
        background: transparent;
        border: none;
        text-align: left;
        padding: 0.5rem 0.75rem;
        border-radius: 0.5rem;
        cursor: pointer;
        color: var(--echo-text-2, #888);
        font-family: var(--echo-font-body, inherit);
        font-size: 1.15rem;
        line-height: 1.5;
        font-weight: 500;
        transition: all 0.25s cubic-bezier(0.16, 1, 0.3, 1);
        user-select: text;
        width: 100%;
    }

    .fullscreen .lyric-line {
        font-size: 1.85rem;
        line-height: 1.6;
        padding: 0.75rem 1rem;
    }

    .lyric-line:hover {
        color: var(--echo-text-1, #eee);
        background-color: rgba(255, 255, 255, 0.04);
    }

    .lyric-line.active {
        color: #ffffff;
        font-weight: 700;
        font-size: 1.35rem;
        transform: scale(1.02);
        transform-origin: left center;
        text-shadow: 0 0 16px rgba(255, 255, 255, 0.25);
    }

    .fullscreen .lyric-line.active {
        font-size: 2.25rem;
        text-shadow: 0 0 24px rgba(255, 255, 255, 0.35);
    }

    .lyric-line.past {
        color: rgba(255, 255, 255, 0.35);
    }

    .line-text {
        display: inline-block;
    }

    .plain-lyrics {
        padding-top: 2rem;
        padding-bottom: 4rem;
        width: 100%;
    }

    .plain-text {
        font-family: var(--echo-font-body, inherit);
        font-size: 1.1rem;
        line-height: 1.8;
        color: var(--echo-text-1, #e0e0e0);
        white-space: pre-wrap;
        margin: 0;
    }

    .status-box {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        margin: auto;
        text-align: center;
        gap: 1rem;
        max-width: 320px;
        padding: 2rem 1rem;
    }

    .status-icon {
        color: var(--echo-text-2, #666);
    }

    .status-title {
        font-size: 1.1rem;
        font-weight: 600;
        color: var(--echo-text-1, #eee);
        margin: 0;
    }

    .status-desc {
        font-size: 0.875rem;
        color: var(--echo-text-2, #888);
        margin: 0;
    }

    .status-text {
        font-size: 0.95rem;
        color: var(--echo-text-2, #aaa);
    }

    .spinner {
        width: 28px;
        height: 28px;
        border: 2px solid rgba(255, 255, 255, 0.1);
        border-top-color: var(--echo-primary, #fff);
        border-radius: 50%;
        animation: spin 0.8s linear infinite;
    }

    @keyframes spin {
        to {
            transform: rotate(360deg);
        }
    }

    .retry-btn {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        padding: 0.5rem 1rem;
        background: rgba(255, 255, 255, 0.08);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 9999px;
        color: var(--echo-text-1, #fff);
        font-size: 0.85rem;
        cursor: pointer;
        transition: all 0.2s;
    }

    .retry-btn:hover {
        background: rgba(255, 255, 255, 0.15);
    }

    .provider-switch {
        margin-top: 1rem;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 0.6rem;
    }

    .provider-switch-label {
        font-size: 0.75rem;
        text-transform: uppercase;
        letter-spacing: 0.05em;
        color: var(--echo-text-2, #777);
    }

    .provider-pills {
        display: flex;
        flex-wrap: wrap;
        gap: 0.5rem;
        justify-content: center;
    }

    .provider-pill {
        display: flex;
        align-items: center;
        gap: 0.35rem;
        background: rgba(255, 255, 255, 0.05);
        border: 1px solid rgba(255, 255, 255, 0.08);
        color: var(--echo-text-1, #ddd);
        padding: 0.35rem 0.75rem;
        border-radius: 9999px;
        font-size: 0.8rem;
        cursor: pointer;
        transition: all 0.2s;
    }

    .provider-pill:hover,
    .provider-pill.active {
        background: rgba(255, 255, 255, 0.12);
        border-color: rgba(255, 255, 255, 0.25);
        color: #fff;
    }

    .lyrics-footer {
        padding-top: 2rem;
        padding-bottom: 1rem;
        text-align: center;
        font-size: 0.75rem;
        color: var(--echo-text-2, #666);
        text-transform: uppercase;
        letter-spacing: 0.04em;
    }

    .instrumental-break {
        display: inline-flex;
        align-items: center;
        gap: 0.35rem;
        padding: 0.4rem 0.75rem;
        background: transparent;
        border: none;
        cursor: pointer;
        border-radius: 9999px;
        opacity: 0.35;
        transition: all 0.25s ease;
        align-self: flex-start;
        margin: -0.25rem 0;
    }

    .fullscreen .instrumental-break {
        padding: 0.6rem 1rem;
        gap: 0.5rem;
    }

    .instrumental-break:hover {
        opacity: 0.8;
        background-color: rgba(255, 255, 255, 0.05);
    }

    .instrumental-break.active {
        opacity: 0.95;
    }

    .break-dot {
        width: 6px;
        height: 6px;
        border-radius: 50%;
        background-color: var(--echo-text-2, #888);
        transition: all 0.3s ease;
    }

    .fullscreen .break-dot {
        width: 8px;
        height: 8px;
    }

    .instrumental-break.active .break-dot {
        background-color: #ffffff;
        box-shadow: 0 0 8px rgba(255, 255, 255, 0.4);
        animation: pulseDot 1.4s infinite ease-in-out;
    }

    .instrumental-break.active .break-dot:nth-child(2) {
        animation-delay: 0.2s;
    }

    .instrumental-break.active .break-dot:nth-child(3) {
        animation-delay: 0.4s;
    }

    @keyframes pulseDot {
        0%, 100% {
            transform: scale(0.85);
            opacity: 0.5;
        }
        50% {
            transform: scale(1.35);
            opacity: 1;
        }
    }

    .sync-pill-btn {
        position: sticky;
        bottom: 1.5rem;
        align-self: center;
        display: inline-flex;
        align-items: center;
        gap: 0.5rem;
        background: rgba(18, 18, 24, 0.88);
        backdrop-filter: blur(16px);
        -webkit-backdrop-filter: blur(16px);
        border: 1px solid rgba(255, 255, 255, 0.15);
        color: #ffffff;
        font-family: var(--echo-font-body, inherit);
        font-size: 0.8125rem;
        font-weight: 600;
        letter-spacing: 0.01em;
        padding: 0.5rem 1.125rem;
        border-radius: 9999px;
        cursor: pointer;
        box-shadow: 0 6px 20px rgba(0, 0, 0, 0.5);
        transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
        z-index: 10;
        margin-top: -2.75rem;
    }

    .sync-pill-btn:hover {
        background: rgba(30, 30, 38, 0.98);
        border-color: rgba(255, 255, 255, 0.3);
        transform: translateY(-2px);
        box-shadow: 0 8px 24px rgba(0, 0, 0, 0.65);
    }

    /* Top actions & footer triggers */
    .lyrics-top-bar {
        display: flex;
        justify-content: flex-end;
        width: 100%;
        margin-bottom: -1rem;
        z-index: 5;
    }

    .search-mode-link {
        display: inline-flex;
        align-items: center;
        gap: 0.35rem;
        background: rgba(255, 255, 255, 0.04);
        border: 1px solid rgba(255, 255, 255, 0.08);
        color: var(--echo-text-2, #888);
        font-family: var(--echo-font-body, inherit);
        font-size: 0.75rem;
        font-weight: 500;
        padding: 0.25rem 0.65rem;
        border-radius: 9999px;
        cursor: pointer;
        transition: all 0.2s ease;
    }

    .search-mode-link:hover {
        background: rgba(255, 255, 255, 0.08);
        border-color: rgba(255, 255, 255, 0.16);
        color: var(--echo-text-1, #eee);
    }

    .footer-dot {
        margin: 0 0.4rem;
        opacity: 0.4;
    }

    .footer-search-link {
        background: transparent;
        border: none;
        color: var(--echo-text-2, #888);
        font-family: var(--echo-font-body, inherit);
        font-size: 0.75rem;
        text-transform: uppercase;
        letter-spacing: 0.04em;
        cursor: pointer;
        text-decoration: underline;
        text-underline-offset: 3px;
        padding: 0;
        transition: color 0.2s ease;
    }

    .footer-search-link:hover {
        color: #ffffff;
    }

    .status-actions {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        flex-wrap: wrap;
        justify-content: center;
    }

    .manual-search-btn {
        display: inline-flex;
        align-items: center;
        gap: 0.45rem;
        padding: 0.5rem 1rem;
        background: rgba(255, 255, 255, 0.06);
        border: 1px solid rgba(255, 255, 255, 0.12);
        border-radius: 9999px;
        color: var(--echo-text-1, #fff);
        font-family: var(--echo-font-body, inherit);
        font-size: 0.85rem;
        cursor: pointer;
        transition: all 0.2s;
    }

    .manual-search-btn:hover {
        background: rgba(255, 255, 255, 0.14);
        border-color: rgba(255, 255, 255, 0.22);
    }

    .manual-search-pill {
        display: inline-flex;
        align-items: center;
        gap: 0.45rem;
        padding: 0.45rem 1.1rem;
        background: rgba(255, 255, 255, 0.08);
        border: 1px solid rgba(255, 255, 255, 0.12);
        border-radius: 9999px;
        color: var(--echo-text-1, #eee);
        font-family: var(--echo-font-body, inherit);
        font-size: 0.825rem;
        font-weight: 500;
        cursor: pointer;
        transition: all 0.2s;
        margin-top: 0.5rem;
    }

    .manual-search-pill:hover {
        background: rgba(255, 255, 255, 0.16);
        color: #fff;
    }

    /* Search Panel Layout */
    .search-panel {
        width: 100%;
        display: flex;
        flex-direction: column;
        gap: 1rem;
        box-sizing: border-box;
    }

    .search-topbar {
        display: flex;
        align-items: center;
        gap: 0.65rem;
        width: 100%;
    }

    .search-back-btn {
        width: 34px;
        height: 34px;
        border-radius: 50%;
        background: rgba(255, 255, 255, 0.05);
        border: 1px solid rgba(255, 255, 255, 0.08);
        color: var(--echo-text-1, #eee);
        display: flex;
        align-items: center;
        justify-content: center;
        cursor: pointer;
        flex-shrink: 0;
        transition: all 0.2s ease;
    }

    .search-back-btn:hover {
        background: rgba(255, 255, 255, 0.12);
        color: #fff;
    }

    .search-input-shell {
        flex: 1;
        display: flex;
        align-items: center;
        gap: 0.5rem;
        background: rgba(255, 255, 255, 0.06);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 9999px;
        padding: 0.45rem 0.85rem;
        transition: border-color 0.2s ease;
    }

    .search-input-shell:focus-within {
        border-color: rgba(255, 255, 255, 0.3);
        background: rgba(255, 255, 255, 0.09);
    }

    .search-input-icon {
        color: var(--echo-text-2, #888);
        flex-shrink: 0;
    }

    .search-text-input {
        flex: 1;
        background: transparent;
        border: none;
        outline: none;
        color: #ffffff;
        font-family: var(--echo-font-body, inherit);
        font-size: 0.85rem;
    }

    .search-text-input::placeholder {
        color: var(--echo-text-2, #777);
    }

    .search-clear-btn {
        background: transparent;
        border: none;
        color: var(--echo-text-2, #888);
        cursor: pointer;
        padding: 2px;
        display: flex;
        align-items: center;
        justify-content: center;
        transition: color 0.15s ease;
    }

    .search-clear-btn:hover {
        color: #ffffff;
    }

    .search-providers-bar {
        display: flex;
        align-items: center;
        gap: 0.4rem;
        flex-wrap: wrap;
    }

    .provider-filter-pill {
        background: rgba(255, 255, 255, 0.04);
        border: 1px solid rgba(255, 255, 255, 0.07);
        color: var(--echo-text-2, #888);
        font-family: var(--echo-font-body, inherit);
        font-size: 0.75rem;
        padding: 0.25rem 0.65rem;
        border-radius: 9999px;
        cursor: pointer;
        transition: all 0.2s ease;
    }

    .provider-filter-pill:hover {
        color: var(--echo-text-1, #eee);
        background: rgba(255, 255, 255, 0.08);
    }

    .provider-filter-pill.active {
        background: rgba(255, 255, 255, 0.16);
        border-color: rgba(255, 255, 255, 0.25);
        color: #ffffff;
        font-weight: 600;
    }

    .search-context-banner {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 0.5rem;
        background: rgba(255, 255, 255, 0.03);
        border: 1px solid rgba(255, 255, 255, 0.05);
        border-radius: 0.5rem;
        padding: 0.5rem 0.75rem;
        font-size: 0.775rem;
    }

    .context-text {
        display: flex;
        align-items: baseline;
        gap: 0.4rem;
        min-width: 0;
        overflow: hidden;
    }

    .context-label {
        color: var(--echo-text-2, #777);
        text-transform: uppercase;
        font-size: 0.7rem;
        letter-spacing: 0.05em;
        flex-shrink: 0;
    }

    .context-song {
        color: var(--echo-text-1, #eee);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        font-weight: 500;
    }

    .revert-link-btn {
        background: transparent;
        border: none;
        color: var(--echo-text-2, #888);
        font-family: var(--echo-font-body, inherit);
        font-size: 0.725rem;
        text-decoration: underline;
        cursor: pointer;
        flex-shrink: 0;
        padding: 0;
        transition: color 0.15s ease;
    }

    .revert-link-btn:hover {
        color: #ffffff;
    }

    .search-results-container {
        display: flex;
        flex-direction: column;
        width: 100%;
        margin-top: 0.5rem;
    }

    .candidate-ledger {
        display: flex;
        flex-direction: column;
        gap: 0.6rem;
        width: 100%;
    }

    .results-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        font-size: 0.725rem;
        text-transform: uppercase;
        letter-spacing: 0.04em;
        color: var(--echo-text-2, #777);
        padding: 0 0.25rem 0.25rem;
    }

    .results-count {
        font-weight: 600;
    }

    .results-hint {
        font-size: 0.7rem;
        opacity: 0.75;
    }

    .candidate-card {
        background: rgba(255, 255, 255, 0.03);
        border: 1px solid rgba(255, 255, 255, 0.06);
        border-radius: 0.65rem;
        padding: 0.75rem 0.85rem;
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
        text-align: left;
        cursor: pointer;
        transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
        width: 100%;
        box-sizing: border-box;
    }

    .candidate-card:hover {
        background: rgba(255, 255, 255, 0.07);
        border-color: rgba(255, 255, 255, 0.16);
        transform: translateY(-1px);
    }

    .candidate-header {
        display: flex;
        flex-direction: column;
        gap: 0.15rem;
    }

    .candidate-title {
        font-size: 0.95rem;
        font-weight: 600;
        color: var(--echo-text-1, #fff);
        line-height: 1.3;
    }

    .candidate-artist {
        font-size: 0.8rem;
        color: var(--echo-text-2, #aaa);
    }

    .candidate-snippet {
        margin: 0;
        font-size: 0.775rem;
        color: var(--echo-text-2, #888);
        font-style: italic;
        line-height: 1.4;
        display: -webkit-box;
        -webkit-line-clamp: 2;
        line-clamp: 2;
        -webkit-box-orient: vertical;
        overflow: hidden;
    }

    .candidate-pills {
        display: flex;
        align-items: center;
        gap: 0.4rem;
        flex-wrap: wrap;
        margin-top: 0.2rem;
    }

    .format-badge {
        font-size: 0.675rem;
        font-weight: 600;
        text-transform: uppercase;
        letter-spacing: 0.04em;
        padding: 0.15rem 0.5rem;
        border-radius: 9999px;
        background: rgba(255, 255, 255, 0.06);
        color: var(--echo-text-2, #888);
        border: 1px solid rgba(255, 255, 255, 0.08);
    }

    .format-badge.synced {
        background: rgba(255, 255, 255, 0.14);
        color: #ffffff;
        border-color: rgba(255, 255, 255, 0.25);
    }

    .duration-badge {
        font-size: 0.7rem;
        font-family: var(--echo-font-body, monospace);
        color: var(--echo-text-2, #aaa);
        background: rgba(255, 255, 255, 0.04);
        padding: 0.15rem 0.45rem;
        border-radius: 9999px;
    }

    .duration-badge.exact-match {
        color: #4ade80;
        background: rgba(74, 222, 128, 0.1);
        border: 1px solid rgba(74, 222, 128, 0.2);
    }

    .dur-diff {
        opacity: 0.7;
        margin-left: 2px;
    }

    .provider-badge {
        font-size: 0.675rem;
        color: var(--echo-text-2, #777);
        text-transform: uppercase;
        letter-spacing: 0.04em;
        margin-left: auto;
    }
</style>
