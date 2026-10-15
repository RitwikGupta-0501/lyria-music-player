<script lang="ts">
    import { downloadStore, type DownloadCandidateItem } from "$lib/stores/download.svelte";
    import { resolveCoverArt } from "$lib/utils/media";
    import { formatDuration } from "$lib/utils/format";
    import { 
        MagnifyingGlass, 
        X, 
        DownloadSimple, 
        ArrowClockwise, 
        MusicNotes,
        CheckCircle,
        WarningCircle
    } from "phosphor-svelte";
    import Loader from "$lib/components/common/Loader.svelte";

    let searchInput = $state("");
    let debounceTimer: ReturnType<typeof setTimeout> | null = null;
    let inputRef = $state<HTMLInputElement | null>(null);

    // Sync search input when modal opens or query changes
    $effect(() => {
        if (downloadStore.isCandidateSearchOpen) {
            searchInput = downloadStore.searchQuery;
            setTimeout(() => {
                inputRef?.focus();
                inputRef?.select();
            }, 50);
        }
    });

    function handleInput() {
        if (debounceTimer) clearTimeout(debounceTimer);
        debounceTimer = setTimeout(() => {
            if (downloadStore.searchProviderId && searchInput.trim()) {
                downloadStore.executeCandidateSearch(searchInput.trim(), downloadStore.searchProviderId);
            }
        }, 350);
    }

    function handleKeydown(e: KeyboardEvent) {
        if (e.key === "Escape") {
            downloadStore.closeCandidateSearch();
        } else if (e.key === "Enter") {
            if (debounceTimer) clearTimeout(debounceTimer);
            if (downloadStore.searchProviderId && searchInput.trim()) {
                downloadStore.executeCandidateSearch(searchInput.trim(), downloadStore.searchProviderId);
            }
        }
    }

    function switchProvider(providerId: string) {
        downloadStore.searchProviderId = providerId;
        if (searchInput.trim()) {
            downloadStore.executeCandidateSearch(searchInput.trim(), providerId);
        }
    }

    async function handleSelect(candidate: DownloadCandidateItem) {
        await downloadStore.selectCandidateAndDownload(candidate);
    }

    function getDurationDiff(candidateDurMs?: number | null): string | null {
        const targetDurMs = downloadStore.targetTrackInfo?.duration_ms;
        if (!targetDurMs || !candidateDurMs) return null;
        const diffSec = Math.round((candidateDurMs - targetDurMs) / 1000);
        if (diffSec === 0) return "Exact match";
        if (diffSec > 0) return `+${diffSec}s`;
        return `${diffSec}s`;
    }
</script>

{#if downloadStore.isCandidateSearchOpen}
    <div 
        class="modal-backdrop" 
        onclick={() => downloadStore.closeCandidateSearch()} 
        onkeydown={handleKeydown} 
        role="dialog" 
        aria-modal="true"
        aria-labelledby="candidate-modal-title"
        tabindex="-1"
    >
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
        <div class="modal-card glass-panel" onclick={(e) => e.stopPropagation()} role="document">
            <!-- Header -->
            <header class="modal-header">
                <div class="title-meta">
                    <h3 id="candidate-modal-title">Select Download Source</h3>
                    {#if downloadStore.targetTrackInfo}
                        <p class="target-sub">
                            Target: <span class="highlight">{downloadStore.targetTrackInfo.title}</span> — {downloadStore.targetTrackInfo.artist}
                            {#if downloadStore.targetTrackInfo.duration_ms}
                                <span class="dur-tag">({formatDuration(downloadStore.targetTrackInfo.duration_ms)})</span>
                            {/if}
                        </p>
                    {/if}
                </div>
                <button 
                    class="close-btn" 
                    onclick={() => downloadStore.closeCandidateSearch()} 
                    aria-label="Close download candidate modal"
                >
                    <X size={18} weight="bold" />
                </button>
            </header>

            <!-- Search Bar & Provider Pills -->
            <div class="search-section">
                <div class="search-bar-wrapper">
                    <MagnifyingGlass size={16} class="search-icon" />
                    <input 
                        bind:this={inputRef}
                        type="text" 
                        bind:value={searchInput} 
                        oninput={handleInput}
                        placeholder="Search track title, artist, or keywords..."
                        spellcheck="false"
                        aria-label="Search candidates"
                    />
                    {#if downloadStore.isSearching}
                        <Loader size={16} color="primary" />
                    {/if}
                </div>

                <!-- Provider Selector Tabs -->
                <div class="provider-tabs" role="tablist" aria-label="Download Providers">
                    {#each downloadStore.providers as p}
                        {@const isSelected = downloadStore.searchProviderId === p.id}
                        <button 
                            class="provider-pill" 
                            class:active={isSelected}
                            onclick={() => switchProvider(p.id)}
                            role="tab"
                            aria-selected={isSelected}
                        >
                            <span>{p.name}</span>
                            {#if p.id.includes("monochrome")}
                                <span class="quality-badge flac">FLAC</span>
                            {:else if p.id.includes("freemp3s")}
                                <span class="quality-badge mp3">320k</span>
                            {/if}
                        </button>
                    {/each}
                </div>
            </div>

            <!-- Candidate List Results -->
            <div class="candidates-container">
                {#if downloadStore.isSearching && downloadStore.searchResults.length === 0}
                    <div class="state-message">
                        <Loader size={28} color="primary" />
                        <p>Querying {downloadStore.searchProviderId} for audio candidates...</p>
                    </div>
                {:else if downloadStore.searchError}
                    <div class="state-message error">
                        <WarningCircle size={28} />
                        <p>{downloadStore.searchError}</p>
                        <button class="retry-btn" onclick={() => downloadStore.searchProviderId && downloadStore.executeCandidateSearch(searchInput.trim(), downloadStore.searchProviderId)}>
                            Retry Search
                        </button>
                    </div>
                {:else if downloadStore.searchResults.length === 0}
                    <div class="state-message empty">
                        <MusicNotes size={32} />
                        <p>No playable audio matches found for this query.</p>
                        <span class="subtext">Try refining your search terms above or switch providers.</span>
                    </div>
                {:else}
                    <div class="candidates-list">
                        {#each downloadStore.searchResults as candidate, idx (candidate.native_id + idx)}
                            {@const diff = getDurationDiff(candidate.duration_ms)}
                            {@const isExact = diff === "Exact match"}
                            <div class="candidate-row group">
                                <!-- Art thumbnail -->
                                <div class="art-thumb">
                                    {#if candidate.artwork_url}
                                        <img src={resolveCoverArt(candidate.artwork_url)} alt={candidate.title} loading="lazy" />
                                    {:else}
                                        <div class="art-fallback">
                                            <MusicNotes size={16} />
                                        </div>
                                    {/if}
                                </div>

                                <!-- Metadata -->
                                <div class="track-info">
                                    <div class="title-row">
                                        <span class="track-title" title={candidate.title}>{candidate.title}</span>
                                        {#if isExact}
                                            <span class="match-tag" title="Exact duration match">
                                                <CheckCircle size={12} weight="fill" /> Match
                                            </span>
                                        {/if}
                                    </div>
                                    <div class="artist-sub">
                                        <span class="artist-name">{candidate.artist}</span>
                                        {#if candidate.album}
                                            <span class="album-dot">•</span>
                                            <span class="album-name">{candidate.album}</span>
                                        {/if}
                                    </div>
                                    {#if candidate.extra_info}
                                        <div class="extra-info-text">{candidate.extra_info}</div>
                                    {/if}
                                </div>

                                <!-- Audio Quality & Duration -->
                                <div class="audio-meta">
                                    <div class="badges-row">
                                        {#if candidate.codec}
                                            <span class="codec-tag {candidate.codec.toLowerCase()}">{candidate.codec.toUpperCase()}</span>
                                        {/if}
                                        {#if candidate.bitrate_kbps}
                                            <span class="bitrate-tag">{candidate.bitrate_kbps}k</span>
                                        {/if}
                                    </div>
                                    <div class="duration-row">
                                        {#if candidate.duration_ms}
                                            <span class="dur-text">{formatDuration(candidate.duration_ms)}</span>
                                        {/if}
                                        {#if diff}
                                            <span class="diff-tag" class:exact={isExact}>{diff}</span>
                                        {/if}
                                    </div>
                                </div>

                                <!-- Action button -->
                                <div class="action-col">
                                    <button 
                                        class="dl-btn pill-accent-bg" 
                                        onclick={() => handleSelect(candidate)}
                                        aria-label="Download candidate {candidate.title}"
                                    >
                                        <DownloadSimple size={15} weight="bold" />
                                        <span>Download</span>
                                    </button>
                                </div>
                            </div>
                        {/each}
                    </div>
                {/if}
            </div>
        </div>
    </div>
{/if}

<style>
    .modal-backdrop {
        position: fixed;
        inset: 0;
        background: rgba(0, 0, 0, 0.75);
        backdrop-filter: blur(10px);
        display: flex;
        justify-content: center;
        align-items: center;
        z-index: 2000;
        padding: 1.5rem;
    }

    .modal-card {
        width: 100%;
        max-width: 680px;
        max-height: 85vh;
        display: flex;
        flex-direction: column;
        background: rgba(18, 18, 18, 0.95);
        border: 1px solid rgba(255, 255, 255, 0.08);
        border-radius: 16px;
        box-shadow: 0 24px 60px rgba(0, 0, 0, 0.6);
        overflow: hidden;
    }

    .modal-header {
        display: flex;
        justify-content: space-between;
        align-items: flex-start;
        padding: 1.25rem 1.5rem;
        border-bottom: 1px solid rgba(255, 255, 255, 0.06);
    }

    .title-meta h3 {
        margin: 0;
        font-size: 1.15rem;
        font-weight: 600;
        color: #ffffff;
    }

    .target-sub {
        margin: 0.25rem 0 0 0;
        font-size: 0.82rem;
        color: rgba(255, 255, 255, 0.5);
    }

    .highlight {
        color: #ffd285;
        font-weight: 500;
    }

    .dur-tag {
        font-variant-numeric: tabular-nums;
        color: rgba(255, 255, 255, 0.4);
    }

    .close-btn {
        background: transparent;
        border: none;
        color: rgba(255, 255, 255, 0.5);
        cursor: pointer;
        padding: 0.4rem;
        border-radius: 8px;
        display: flex;
        align-items: center;
        justify-content: center;
        transition: color 0.15s ease, background 0.15s ease;
    }

    .close-btn:hover {
        color: #ffffff;
        background: rgba(255, 255, 255, 0.08);
    }

    .close-btn:focus-visible {
        outline: 2px solid #ffd285;
    }

    .search-section {
        padding: 1rem 1.5rem 0.75rem 1.5rem;
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
        border-bottom: 1px solid rgba(255, 255, 255, 0.04);
        background: rgba(0, 0, 0, 0.2);
    }

    .search-bar-wrapper {
        position: relative;
        display: flex;
        align-items: center;
        background: rgba(255, 255, 255, 0.05);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 10px;
        padding: 0 0.85rem;
        transition: border-color 0.15s ease;
    }

    .search-bar-wrapper:focus-within {
        border-color: #ffd285;
    }

    :global(.search-bar-wrapper .search-icon) {
        color: rgba(255, 255, 255, 0.4);
        margin-right: 0.6rem;
        flex-shrink: 0;
    }

    .search-bar-wrapper input {
        flex: 1;
        background: transparent;
        border: none;
        color: #ffffff;
        font-size: 0.92rem;
        padding: 0.65rem 0;
        outline: none;
    }

    .search-bar-wrapper input::placeholder {
        color: rgba(255, 255, 255, 0.35);
    }


    .provider-tabs {
        display: flex;
        gap: 0.5rem;
    }

    .provider-pill {
        display: flex;
        align-items: center;
        gap: 0.4rem;
        padding: 0.35rem 0.8rem;
        background: rgba(255, 255, 255, 0.04);
        border: 1px solid rgba(255, 255, 255, 0.08);
        border-radius: 9999px;
        color: rgba(255, 255, 255, 0.65);
        font-size: 0.82rem;
        cursor: pointer;
        transition: all 0.15s ease;
    }

    .provider-pill:hover {
        background: rgba(255, 255, 255, 0.08);
        color: #ffffff;
    }

    .provider-pill.active {
        background: rgba(255, 210, 133, 0.12);
        border-color: rgba(255, 210, 133, 0.4);
        color: #ffd285;
        font-weight: 500;
    }

    .quality-badge {
        font-size: 0.68rem;
        font-weight: 700;
        padding: 0.1rem 0.35rem;
        border-radius: 4px;
        text-transform: uppercase;
        letter-spacing: 0.03em;
    }

    .quality-badge.flac {
        background: rgba(45, 212, 191, 0.2);
        color: #2dd4bf;
    }

    .quality-badge.mp3 {
        background: rgba(52, 211, 153, 0.2);
        color: #34d399;
    }

    .candidates-container {
        flex: 1;
        overflow-y: auto;
        padding: 0.75rem 1rem;
        min-height: 240px;
    }

    .state-message {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        gap: 0.6rem;
        padding: 3rem 1rem;
        color: rgba(255, 255, 255, 0.5);
        text-align: center;
        font-size: 0.9rem;
    }

    .state-message.error {
        color: #f87171;
    }

    .retry-btn {
        margin-top: 0.5rem;
        padding: 0.4rem 1rem;
        background: rgba(248, 113, 113, 0.15);
        border: 1px solid rgba(248, 113, 113, 0.3);
        color: #f87171;
        border-radius: 6px;
        cursor: pointer;
        font-size: 0.82rem;
    }

    .subtext {
        font-size: 0.78rem;
        color: rgba(255, 255, 255, 0.35);
    }

    .candidates-list {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .candidate-row {
        display: flex;
        align-items: center;
        gap: 0.85rem;
        padding: 0.65rem 0.85rem;
        background: rgba(255, 255, 255, 0.02);
        border: 1px solid rgba(255, 255, 255, 0.04);
        border-radius: 10px;
        transition: background 0.15s ease, border-color 0.15s ease;
    }

    .candidate-row:hover {
        background: rgba(255, 255, 255, 0.05);
        border-color: rgba(255, 255, 255, 0.09);
    }

    .art-thumb {
        width: 44px;
        height: 44px;
        border-radius: 6px;
        overflow: hidden;
        background: rgba(0, 0, 0, 0.4);
        flex-shrink: 0;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .art-thumb img {
        width: 100%;
        height: 100%;
        object-fit: cover;
    }

    .art-fallback {
        color: rgba(255, 255, 255, 0.3);
    }

    .track-info {
        flex: 1;
        min-width: 0;
    }

    .title-row {
        display: flex;
        align-items: center;
        gap: 0.5rem;
    }

    .track-title {
        color: #ffffff;
        font-size: 0.9rem;
        font-weight: 500;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .match-tag {
        display: inline-flex;
        align-items: center;
        gap: 0.2rem;
        font-size: 0.68rem;
        font-weight: 600;
        padding: 0.1rem 0.4rem;
        border-radius: 4px;
        background: rgba(52, 211, 153, 0.15);
        color: #34d399;
        flex-shrink: 0;
    }

    .artist-sub {
        display: flex;
        align-items: center;
        gap: 0.3rem;
        font-size: 0.78rem;
        color: rgba(255, 255, 255, 0.5);
        margin-top: 0.15rem;
    }

    .artist-name, .album-name {
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .album-dot {
        opacity: 0.5;
    }

    .extra-info-text {
        font-size: 0.7rem;
        color: rgba(255, 255, 255, 0.35);
        margin-top: 0.15rem;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .audio-meta {
        display: flex;
        flex-direction: column;
        align-items: flex-end;
        gap: 0.25rem;
        flex-shrink: 0;
    }

    .badges-row {
        display: flex;
        align-items: center;
        gap: 0.3rem;
    }

    .codec-tag {
        font-size: 0.68rem;
        font-weight: 700;
        padding: 0.1rem 0.35rem;
        border-radius: 4px;
    }

    .codec-tag.flac {
        background: rgba(45, 212, 191, 0.15);
        color: #2dd4bf;
    }

    .codec-tag.mp3 {
        background: rgba(52, 211, 153, 0.15);
        color: #34d399;
    }

    .bitrate-tag {
        font-size: 0.68rem;
        color: rgba(255, 255, 255, 0.4);
        font-variant-numeric: tabular-nums;
    }

    .duration-row {
        display: flex;
        align-items: center;
        gap: 0.35rem;
    }

    .dur-text {
        font-size: 0.76rem;
        color: rgba(255, 255, 255, 0.6);
        font-variant-numeric: tabular-nums;
    }

    .diff-tag {
        font-size: 0.68rem;
        padding: 0.05rem 0.3rem;
        border-radius: 3px;
        background: rgba(255, 255, 255, 0.06);
        color: rgba(255, 255, 255, 0.45);
        font-variant-numeric: tabular-nums;
    }

    .diff-tag.exact {
        background: rgba(52, 211, 153, 0.15);
        color: #34d399;
    }

    .action-col {
        flex-shrink: 0;
    }

    .dl-btn {
        display: flex;
        align-items: center;
        gap: 0.35rem;
        padding: 0.45rem 0.85rem;
        border-radius: 8px;
        border: none;
        background: #ffd285;
        color: #121212;
        font-size: 0.8rem;
        font-weight: 600;
        cursor: pointer;
        transition: transform 0.12s ease, opacity 0.15s ease;
    }

    .dl-btn:hover {
        opacity: 0.9;
        transform: scale(1.02);
    }

    .dl-btn:active {
        transform: scale(0.98);
    }

    .dl-btn:focus-visible {
        outline: 2px solid #ffffff;
    }
</style>
