<script lang="ts">
    import { searchStore } from "$lib/stores/search.svelte";
    import { exploreStore } from "$lib/stores/explore.svelte";
    import { audioStore } from "$lib/stores/audio.svelte";
    import { 
        MagnifyingGlass, 
        X, 
        Lightning, 
        LinkSimple, 
        HardDrives, 
        Globe, 
        Play, 
        Pause, 
        MusicNote, 
        ArrowClockwise,
        ArrowRight
    } from "phosphor-svelte";

    let { isOpen = $bindable(false) } = $props<{ isOpen?: boolean }>();

    let inputRef = $state<HTMLInputElement | null>(null);

    $effect(() => {
        if (isOpen && inputRef) {
            inputRef.focus();
        }
    });

    function openInExplore() {
        if (!searchStore.query.trim()) return;
        const q = searchStore.query.trim();
        exploreStore.setSearchQuery(q);
        isOpen = false;
        document.dispatchEvent(new CustomEvent('echo:navigate-explore'));
    }

    function handleKeyDown(e: KeyboardEvent) {
        if (e.key === 'Escape') {
            isOpen = false;
        } else if (e.key === 'Enter') {
            if (e.shiftKey) {
                openInExplore();
            } else if (searchStore.resolvedUrl) {
                searchStore.playResolvedUrl();
                isOpen = false;
            }
        }
    }

    function formatDuration(ms: number | null | undefined): string {
        if (!ms) return "";
        const totalSec = Math.floor(ms / 1000);
        const m = Math.floor(totalSec / 60);
        const s = totalSec % 60;
        return `${m}:${s.toString().padStart(2, '0')}`;
    }

    function isCurrent(track: any): boolean {
        const cur = audioStore.currentQueueTrack;
        if (!cur) return false;
        return cur.title.toLowerCase() === track.title.toLowerCase()
            && (cur.artist || "").toLowerCase() === (track.artist || "").toLowerCase();
    }
</script>

<svelte:window onkeydown={handleKeyDown} />

{#if isOpen}
    <div 
        class="search-backdrop" 
        role="button" 
        tabindex="-1"
        onclick={() => isOpen = false}
        onkeydown={(e) => { if (e.key === 'Escape') isOpen = false; }}
    >
        <div 
            class="search-modal" 
            role="dialog" 
            aria-modal="true" 
            tabindex="0"
            onclick={(e) => e.stopPropagation()}
            onkeydown={(e) => e.stopPropagation()}
        >
            <!-- Input Header -->
            <div class="search-input-wrapper">
                <MagnifyingGlass size={20} weight="bold" class="search-icon" />
                <input 
                    bind:this={inputRef}
                    type="text" 
                    placeholder="Search local library, streams, or paste any URL..." 
                    value={searchStore.query}
                    oninput={(e) => searchStore.handleQueryChange((e.target as HTMLInputElement).value)}
                />
                {#if searchStore.isSearching || searchStore.isResolvingUrl}
                    <div class="spinner-icon">
                        <ArrowClockwise size={16} weight="bold" />
                    </div>
                {/if}
                {#if searchStore.query}
                    <button class="clear-btn" onclick={() => searchStore.clear()} title="Clear query">
                        <X size={16} weight="bold" />
                    </button>
                {/if}
            </div>

            <!-- Filter Pills Bar -->
            <div class="filter-bar">
                <button 
                    class="filter-pill" 
                    class:active={searchStore.activeFilter === "all"}
                    onclick={() => searchStore.activeFilter = "all"}
                >
                    All
                </button>
                <button 
                    class="filter-pill" 
                    class:active={searchStore.activeFilter === "local"}
                    onclick={() => searchStore.activeFilter = "local"}
                >
                    <HardDrives size={13} weight="bold" />
                    <span>Local Library</span>
                </button>
                <button 
                    class="filter-pill" 
                    class:active={searchStore.activeFilter === "online"}
                    onclick={() => searchStore.activeFilter = "online"}
                >
                    <Globe size={13} weight="bold" />
                    <span>Online Streams</span>
                </button>
            </div>

            <!-- Scrollable Results Container (Fixed Height) -->
            <div class="results-container">
                <!-- 1. Direct Resolved URL Card (Sandboxed Extension) -->
                {#if searchStore.resolvedUrl}
                    {@const r = searchStore.resolvedUrl}
                    <div class="direct-url-card">
                        <div class="url-badge">
                            <Lightning size={15} weight="fill" />
                            <span>Direct Stream URL Intercepted</span>
                        </div>
                        <div class="url-content">
                            {#if r.track.cover_art_url}
                                <img src={r.track.cover_art_url} alt={r.track.title} class="url-thumb" />
                            {/if}
                            <div class="url-meta">
                                <span class="url-title">{r.track.title}</span>
                                <span class="url-artist">{r.track.artist}</span>
                                <span class="url-provider">
                                    <LinkSimple size={13} />
                                    Resolved via {r.provider_name}
                                </span>
                            </div>
                            <button 
                                class="url-play-btn" 
                                onclick={() => { searchStore.playResolvedUrl(); isOpen = false; }}
                            >
                                <Play size={16} weight="fill" />
                                <span>Play Stream</span>
                            </button>
                        </div>
                    </div>
                {/if}

                <!-- 2. Local Library Tracks -->
                {#if (searchStore.activeFilter === "all" || searchStore.activeFilter === "local") && searchStore.localTracks.length > 0}
                    <div class="result-group">
                        <div class="group-header">
                            <HardDrives size={14} weight="bold" />
                            <span>Local Lossless Tracks ({searchStore.localTracks.length})</span>
                        </div>
                        {#each searchStore.localTracks as track}
                            <div 
                                class="track-row"
                                class:playing={isCurrent(track)}
                                role="button"
                                tabindex="0"
                                onclick={() => { searchStore.playLocalTrack(track); isOpen = false; }}
                                onkeydown={(e) => { if (e.key === 'Enter') { searchStore.playLocalTrack(track); isOpen = false; } }}
                            >
                                <div class="row-left">
                                    <div class="row-icon">
                                        {#if isCurrent(track) && audioStore.playbackState === "Playing"}
                                            <Pause size={14} weight="fill" />
                                        {:else}
                                            <MusicNote size={14} />
                                        {/if}
                                    </div>
                                    <div class="row-info">
                                        <span class="row-title">{track.title}</span>
                                        <span class="row-artist">{track.artist || "Unknown Artist"} {track.album ? `• ${track.album}` : ''}</span>
                                    </div>
                                </div>
                                <div class="row-right">
                                    <span class="local-badge">FLAC / LOCAL</span>
                                </div>
                            </div>
                        {/each}
                    </div>
                {/if}

                <!-- 3. Remote Online Extensions Matches -->
                {#if (searchStore.activeFilter === "all" || searchStore.activeFilter === "online") && searchStore.remoteTracks.length > 0}
                    <div class="result-group">
                        <div class="group-header">
                            <Globe size={14} weight="bold" />
                            <span>Online Extensions ({searchStore.remoteTracks.length})</span>
                        </div>
                        {#each searchStore.remoteTracks as track}
                            <div 
                                class="track-row"
                                class:playing={isCurrent(track)}
                                role="button"
                                tabindex="0"
                                onclick={() => { searchStore.playRemoteTrack(track); isOpen = false; }}
                                onkeydown={(e) => { if (e.key === 'Enter') { searchStore.playRemoteTrack(track); isOpen = false; } }}
                            >
                                <div class="row-left">
                                    {#if track.cover_art_url}
                                        <img src={track.cover_art_url} alt={track.title} class="track-thumb" />
                                    {:else}
                                        <div class="row-icon">
                                            <MusicNote size={14} />
                                        </div>
                                    {/if}
                                    <div class="row-info">
                                        <span class="row-title">{track.title}</span>
                                        <span class="row-artist">{track.artist}</span>
                                    </div>
                                </div>
                                <div class="row-right">
                                    {#if track.duration_ms}
                                        <span class="duration">{formatDuration(track.duration_ms)}</span>
                                    {/if}
                                    <span class="provider-badge">{track.provider_name || 'YouTube'}</span>
                                </div>
                            </div>
                        {/each}
                    </div>
                {/if}

                <!-- Empty State -->
                {#if searchStore.query && !searchStore.isSearching && !searchStore.isResolvingUrl && searchStore.localTracks.length === 0 && searchStore.remoteTracks.length === 0 && !searchStore.resolvedUrl}
                    <div class="no-results">
                        <p>No immediate matches found for "{searchStore.query}".</p>
                        <button class="open-explore-btn" onclick={openInExplore}>
                            <span>Search in Explore Canvas</span>
                            <ArrowRight size={14} weight="bold" />
                        </button>
                    </div>
                {/if}

                <!-- Fresh Placeholder State -->
                {#if !searchStore.query}
                    <div class="search-tip">
                        <p>Type keywords to search local files and extensions simultaneously, or paste any stream URL.</p>
                    </div>
                {/if}
            </div>

            <!-- Fixed Modal Footer -->
            <footer class="modal-footer">
                <div class="footer-shortcuts">
                    <span class="shortcut-item"><kbd>↵</kbd> Play</span>
                    <span class="shortcut-dot">•</span>
                    <span class="shortcut-item"><kbd>Esc</kbd> Close</span>
                </div>

                {#if searchStore.query.trim()}
                    <button class="footer-explore-btn" onclick={openInExplore} title="View categorized shelves in Explore (Shift+Enter)">
                        <span>Explore full results</span>
                        <kbd>Shift + ↵</kbd>
                    </button>
                {/if}
            </footer>
        </div>
    </div>
{/if}

<style>
    .search-backdrop {
        position: fixed;
        inset: 0;
        background: rgba(0, 0, 0, 0.75);
        backdrop-filter: blur(14px);
        -webkit-backdrop-filter: blur(14px);
        z-index: 9999;
        display: flex;
        align-items: flex-start;
        justify-content: center;
        padding-top: 8vh;
    }

    /* Fixed Geometry: Never resizes or jumps */
    .search-modal {
        background: #141417;
        border: 1px solid rgba(255, 255, 255, 0.12);
        border-radius: 14px;
        width: 640px;
        max-width: 92vw;
        height: 500px;
        max-height: 500px;
        display: flex;
        flex-direction: column;
        overflow: hidden;
        box-shadow: 0 24px 60px rgba(0, 0, 0, 0.7), 0 0 0 1px rgba(255, 255, 255, 0.05);
        outline: none;
    }

    .search-input-wrapper {
        display: flex;
        align-items: center;
        gap: 0.85rem;
        padding: 1.1rem 1.35rem;
        border-bottom: 1px solid rgba(255, 255, 255, 0.08);
        background: #18181C;
    }

    :global(.search-icon) {
        color: #B58E62; /* Brass */
        flex-shrink: 0;
    }

    .search-input-wrapper input {
        flex: 1;
        background: transparent !important;
        border: none !important;
        outline: none !important;
        box-shadow: none !important;
        font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
        font-size: 0.95rem;
        color: #fff;
        font-weight: 500;
        padding: 0 !important;
        margin: 0 !important;
    }

    .search-input-wrapper input::placeholder {
        color: rgba(255, 255, 255, 0.4);
    }

    .spinner-icon {
        color: #B58E62;
        animation: spin 1s linear infinite;
    }

    @keyframes spin {
        from { transform: rotate(0deg); }
        to { transform: rotate(360deg); }
    }

    .clear-btn {
        background: transparent;
        border: none;
        color: rgba(255, 255, 255, 0.5);
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 0.2rem;
        border-radius: 4px;
        transition: color 0.15s ease;
    }

    .clear-btn:hover {
        color: #fff;
    }

    /* Filter Bar */
    .filter-bar {
        display: flex;
        gap: 0.5rem;
        padding: 0.65rem 1.35rem;
        background: rgba(255, 255, 255, 0.02);
        border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    }

    .filter-pill {
        display: flex;
        align-items: center;
        gap: 0.4rem;
        padding: 0.3rem 0.75rem;
        border-radius: 20px;
        background: rgba(255, 255, 255, 0.04);
        border: 1px solid rgba(255, 255, 255, 0.06);
        color: rgba(255, 255, 255, 0.7);
        font-size: 0.78rem;
        font-weight: 600;
        cursor: pointer;
        transition: all 0.15s ease;
    }

    .filter-pill:hover {
        background: rgba(255, 255, 255, 0.08);
        color: #fff;
    }

    .filter-pill.active {
        background: #B58E62;
        color: #000;
        border-color: #B58E62;
        font-weight: 700;
    }

    /* Results Scrollable Container */
    .results-container {
        flex: 1;
        padding: 0.85rem 1.25rem;
        overflow-y: auto;
        display: flex;
        flex-direction: column;
        gap: 1.1rem;
        scrollbar-width: thin;
        scrollbar-color: rgba(255, 255, 255, 0.15) transparent;
    }

    /* Direct URL Card */
    .direct-url-card {
        background: rgba(181, 142, 98, 0.1);
        border: 1px solid rgba(181, 142, 98, 0.3);
        border-radius: 10px;
        padding: 0.9rem 1.1rem;
        display: flex;
        flex-direction: column;
        gap: 0.6rem;
    }

    .url-badge {
        display: flex;
        align-items: center;
        gap: 0.4rem;
        font-size: 0.75rem;
        font-weight: 700;
        font-family: ui-monospace, monospace;
        color: #B58E62;
    }

    .url-content {
        display: flex;
        align-items: center;
        gap: 0.85rem;
    }

    .url-thumb {
        width: 44px;
        height: 44px;
        border-radius: 6px;
        object-fit: cover;
    }

    .url-meta {
        flex: 1;
        display: flex;
        flex-direction: column;
        gap: 0.15rem;
        min-width: 0;
    }

    .url-title {
        font-size: 0.88rem;
        font-weight: 700;
        color: #fff;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .url-artist {
        font-size: 0.78rem;
        color: rgba(255, 255, 255, 0.6);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .url-provider {
        font-size: 0.72rem;
        color: #B58E62;
        display: flex;
        align-items: center;
        gap: 0.25rem;
        font-family: ui-monospace, monospace;
    }

    .url-play-btn {
        display: flex;
        align-items: center;
        gap: 0.35rem;
        background: #B58E62;
        color: #000;
        border: none;
        padding: 0.45rem 0.9rem;
        border-radius: 6px;
        font-size: 0.82rem;
        font-weight: 700;
        cursor: pointer;
        transition: transform 0.15s ease;
    }

    .url-play-btn:hover {
        transform: scale(1.04);
    }

    /* Result Groups */
    .result-group {
        display: flex;
        flex-direction: column;
        gap: 0.35rem;
    }

    .group-header {
        display: flex;
        align-items: center;
        gap: 0.45rem;
        font-size: 0.74rem;
        font-weight: 700;
        font-family: ui-monospace, monospace;
        color: rgba(255, 255, 255, 0.45);
        letter-spacing: 0.05em;
        text-transform: uppercase;
        margin-bottom: 0.2rem;
    }

    .track-row {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 0.5rem 0.65rem;
        border-radius: 6px;
        background: rgba(255, 255, 255, 0.02);
        cursor: pointer;
        transition: background-color 0.12s ease;
    }

    .track-row:hover {
        background: rgba(255, 255, 255, 0.07);
    }

    .track-row.playing {
        background: rgba(181, 142, 98, 0.15);
        border: 1px solid rgba(181, 142, 98, 0.3);
    }

    .row-left {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        flex: 1;
        min-width: 0;
    }

    .row-icon {
        width: 32px;
        height: 32px;
        border-radius: 6px;
        background: #232328;
        color: #B58E62;
        display: flex;
        align-items: center;
        justify-content: center;
        flex-shrink: 0;
    }

    .track-thumb {
        width: 32px;
        height: 32px;
        border-radius: 6px;
        object-fit: cover;
        flex-shrink: 0;
    }

    .row-info {
        display: flex;
        flex-direction: column;
        gap: 0.1rem;
        min-width: 0;
    }

    .row-title {
        font-size: 0.86rem;
        font-weight: 600;
        color: #fff;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .row-artist {
        font-size: 0.76rem;
        color: rgba(255, 255, 255, 0.5);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .row-right {
        display: flex;
        align-items: center;
        gap: 0.6rem;
        margin-left: 0.5rem;
        flex-shrink: 0;
    }

    .local-badge {
        font-size: 0.65rem;
        font-family: ui-monospace, monospace;
        font-weight: 700;
        background: rgba(181, 142, 98, 0.15);
        border: 1px solid rgba(181, 142, 98, 0.35);
        color: #B58E62;
        padding: 0.1rem 0.4rem;
        border-radius: 4px;
    }

    .provider-badge {
        font-size: 0.65rem;
        font-family: ui-monospace, monospace;
        background: rgba(255, 255, 255, 0.05);
        border: 1px solid rgba(255, 255, 255, 0.1);
        color: rgba(255, 255, 255, 0.5);
        padding: 0.1rem 0.4rem;
        border-radius: 4px;
    }

    .duration {
        font-size: 0.74rem;
        font-family: ui-monospace, monospace;
        color: rgba(255, 255, 255, 0.4);
    }

    .no-results, .search-tip {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        padding: 3rem 1rem;
        text-align: center;
        gap: 0.85rem;
        color: rgba(255, 255, 255, 0.5);
        font-size: 0.86rem;
    }

    .open-explore-btn {
        background: transparent;
        border: 1px solid rgba(181, 142, 98, 0.4);
        color: #B58E62;
        padding: 0.45rem 0.9rem;
        border-radius: 6px;
        font-size: 0.82rem;
        font-weight: 600;
        cursor: pointer;
        display: flex;
        align-items: center;
        gap: 0.4rem;
        transition: all 0.15s ease;
    }

    .open-explore-btn:hover {
        background: rgba(181, 142, 98, 0.15);
        color: #fff;
    }

    /* Fixed Modal Footer */
    .modal-footer {
        height: 42px;
        background: #111114;
        border-top: 1px solid rgba(255, 255, 255, 0.06);
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 0 1.25rem;
        font-size: 0.75rem;
        color: rgba(255, 255, 255, 0.45);
        font-family: ui-monospace, monospace;
    }

    .footer-shortcuts {
        display: flex;
        align-items: center;
        gap: 0.5rem;
    }

    .shortcut-dot {
        color: rgba(255, 255, 255, 0.2);
    }

    kbd {
        background: rgba(255, 255, 255, 0.08);
        border: 1px solid rgba(255, 255, 255, 0.15);
        border-radius: 4px;
        padding: 0.1rem 0.35rem;
        color: rgba(255, 255, 255, 0.75);
        font-size: 0.7rem;
    }

    .footer-explore-btn {
        background: transparent;
        border: none;
        color: #B58E62;
        cursor: pointer;
        display: flex;
        align-items: center;
        gap: 0.5rem;
        font-size: 0.75rem;
        font-weight: 600;
        padding: 0;
        transition: color 0.15s ease;
    }

    .footer-explore-btn:hover {
        color: #fff;
    }
</style>
