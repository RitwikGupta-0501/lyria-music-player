<script lang="ts">
    import { searchStore } from "$lib/stores/search.svelte";
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
        Clock, 
        ArrowClockwise 
    } from "phosphor-svelte";

    let { isOpen = $bindable(false) } = $props<{ isOpen?: boolean }>();

    let inputRef = $state<HTMLInputElement | null>(null);

    $effect(() => {
        if (isOpen && inputRef) {
            inputRef.focus();
        }
    });

    function handleKeyDown(e: KeyboardEvent) {
        if (e.key === 'Escape') {
            isOpen = false;
        } else if (e.key === 'Enter') {
            if (searchStore.resolvedUrl) {
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
                <MagnifyingGlass size={22} weight="bold" class="search-icon" />
                <input 
                    bind:this={inputRef}
                    type="text" 
                    placeholder="Search library, artists, tracks, or paste any stream URL..."
                    value={searchStore.query}
                    oninput={(e) => searchStore.handleQueryChange((e.target as HTMLInputElement).value)}
                />
                {#if searchStore.isSearching || searchStore.isResolvingUrl}
                    <div class="spinner-icon">
                        <ArrowClockwise size={18} weight="bold" />
                    </div>
                {/if}
                {#if searchStore.query}
                    <button class="clear-btn" onclick={() => searchStore.clear()} title="Clear query">
                        <X size={18} weight="bold" />
                    </button>
                {/if}
            </div>

            <!-- Filter Pills -->
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
                    <HardDrives size={14} weight="bold" />
                    <span>Local Library</span>
                </button>
                <button 
                    class="filter-pill" 
                    class:active={searchStore.activeFilter === "online"}
                    onclick={() => searchStore.activeFilter = "online"}
                >
                    <Globe size={14} weight="bold" />
                    <span>Online Extensions</span>
                </button>
            </div>

            <!-- Results Scrollable Body -->
            <div class="results-container">
                <!-- 1. Direct Resolved URL Card (Sandboxed Extension) -->
                {#if searchStore.resolvedUrl}
                    {@const r = searchStore.resolvedUrl}
                    <div class="direct-url-card">
                        <div class="url-badge">
                            <Lightning size={16} weight="fill" />
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
                                    <LinkSimple size={14} />
                                    Resolved via {r.provider_name}
                                </span>
                            </div>
                            <button 
                                class="url-play-btn" 
                                onclick={() => { searchStore.playResolvedUrl(); isOpen = false; }}
                            >
                                <Play size={18} weight="fill" />
                                <span>Play Stream</span>
                            </button>
                        </div>
                    </div>
                {/if}

                <!-- 2. Local Library Tracks -->
                {#if (searchStore.activeFilter === "all" || searchStore.activeFilter === "local") && searchStore.localTracks.length > 0}
                    <div class="result-group">
                        <div class="group-header">
                            <HardDrives size={16} weight="bold" />
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
                                            <Pause size={16} weight="fill" />
                                        {:else}
                                            <MusicNote size={16} />
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
                            <Globe size={16} weight="bold" />
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
                                            <MusicNote size={16} />
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
                        <p>No matches found for "{searchStore.query}".</p>
                    </div>
                {/if}

                <!-- Fresh Placeholder State -->
                {#if !searchStore.query}
                    <div class="search-tip">
                        <p>Type keywords to search local files and extensions simultaneously, or paste any direct stream URL.</p>
                    </div>
                {/if}
            </div>
        </div>
    </div>
{/if}

<style>
    .search-backdrop {
        position: fixed;
        inset: 0;
        background: rgba(0, 0, 0, 0.7);
        backdrop-filter: blur(12px);
        -webkit-backdrop-filter: blur(12px);
        z-index: 9999;
        display: flex;
        align-items: flex-start;
        justify-content: center;
        padding-top: 8vh;
    }

    .search-modal {
        background: rgba(20, 20, 24, 0.95);
        border: 1px solid rgba(255, 255, 255, 0.12);
        border-radius: 16px;
        width: 90%;
        max-width: 680px;
        max-height: 80vh;
        display: flex;
        flex-direction: column;
        overflow: hidden;
        box-shadow: 0 24px 60px rgba(0, 0, 0, 0.6);
        outline: none;
    }

    .search-input-wrapper {
        display: flex;
        align-items: center;
        gap: 0.9rem;
        padding: 1.25rem 1.5rem;
        border-bottom: 1px solid rgba(255, 255, 255, 0.08);
    }

    :global(.search-icon) {
        color: var(--text-muted, rgba(255, 255, 255, 0.6));
    }

    .search-input-wrapper input {
        flex: 1;
        background: transparent;
        border: none;
        outline: none;
        font-size: 1.1rem;
        color: #fff;
        font-weight: 500;
    }

    .search-input-wrapper input::placeholder {
        color: var(--text-muted, rgba(255, 255, 255, 0.4));
    }

    .spinner-icon {
        color: var(--text-muted, rgba(255, 255, 255, 0.6));
        animation: spin 1s linear infinite;
    }

    @keyframes spin {
        from { transform: rotate(0deg); }
        to { transform: rotate(360deg); }
    }

    .clear-btn {
        background: transparent;
        border: none;
        color: var(--text-muted, rgba(255, 255, 255, 0.5));
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 0.2rem;
    }

    .clear-btn:hover {
        color: #fff;
    }

    /* Filter Bar */
    .filter-bar {
        display: flex;
        gap: 0.5rem;
        padding: 0.75rem 1.5rem;
        background: rgba(255, 255, 255, 0.02);
        border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    }

    .filter-pill {
        display: flex;
        align-items: center;
        gap: 0.4rem;
        padding: 0.35rem 0.85rem;
        border-radius: 20px;
        background: rgba(255, 255, 255, 0.04);
        border: 1px solid rgba(255, 255, 255, 0.06);
        color: var(--text-muted, rgba(255, 255, 255, 0.7));
        font-size: 0.82rem;
        font-weight: 600;
        cursor: pointer;
        transition: all 0.15s ease;
    }

    .filter-pill:hover {
        background: rgba(255, 255, 255, 0.08);
        color: #fff;
    }

    .filter-pill.active {
        background: #fff;
        color: #000;
        border-color: #fff;
    }

    /* Results */
    .results-container {
        padding: 1rem 1.5rem;
        overflow-y: auto;
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
    }

    /* Direct URL Card */
    .direct-url-card {
        background: linear-gradient(135deg, rgba(255, 209, 102, 0.1) 0%, rgba(255, 209, 102, 0.02) 100%);
        border: 1px solid rgba(255, 209, 102, 0.3);
        border-radius: 12px;
        padding: 1.25rem;
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
    }

    .url-badge {
        display: flex;
        align-items: center;
        gap: 0.4rem;
        font-size: 0.8rem;
        font-weight: 700;
        color: #ffd166;
    }

    .url-content {
        display: flex;
        align-items: center;
        gap: 1rem;
    }

    .url-thumb {
        width: 54px;
        height: 54px;
        border-radius: 8px;
        object-fit: cover;
    }

    .url-meta {
        flex: 1;
        display: flex;
        flex-direction: column;
        gap: 0.2rem;
    }

    .url-title {
        font-size: 0.95rem;
        font-weight: 700;
    }

    .url-artist {
        font-size: 0.82rem;
        color: var(--text-muted, rgba(255, 255, 255, 0.7));
    }

    .url-provider {
        font-size: 0.75rem;
        color: #ffd166;
        display: flex;
        align-items: center;
        gap: 0.3rem;
    }

    .url-play-btn {
        display: flex;
        align-items: center;
        gap: 0.4rem;
        background: #ffd166;
        color: #000;
        border: none;
        padding: 0.6rem 1.1rem;
        border-radius: 8px;
        font-size: 0.88rem;
        font-weight: 700;
        cursor: pointer;
        transition: transform 0.15s ease;
    }

    .url-play-btn:hover {
        transform: scale(1.05);
    }

    /* Result Groups */
    .result-group {
        display: flex;
        flex-direction: column;
        gap: 0.4rem;
    }

    .group-header {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        font-size: 0.8rem;
        font-weight: 700;
        text-transform: uppercase;
        color: var(--text-muted, rgba(255, 255, 255, 0.5));
        margin-bottom: 0.2rem;
    }

    .track-row {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 0.55rem 0.75rem;
        border-radius: 8px;
        cursor: pointer;
        transition: background 0.15s ease;
    }

    .track-row:hover {
        background: rgba(255, 255, 255, 0.06);
    }

    .track-row.playing {
        background: rgba(255, 255, 255, 0.1);
    }

    .row-left {
        display: flex;
        align-items: center;
        gap: 0.8rem;
        min-width: 0;
    }

    .track-thumb {
        width: 38px;
        height: 38px;
        border-radius: 6px;
        object-fit: cover;
        flex-shrink: 0;
    }

    .row-icon {
        width: 38px;
        height: 38px;
        border-radius: 6px;
        background: rgba(255, 255, 255, 0.05);
        display: flex;
        align-items: center;
        justify-content: center;
        color: var(--text-muted, rgba(255, 255, 255, 0.6));
        flex-shrink: 0;
    }

    .row-info {
        display: flex;
        flex-direction: column;
        gap: 0.15rem;
        min-width: 0;
    }

    .row-title {
        font-size: 0.9rem;
        font-weight: 600;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .row-artist {
        font-size: 0.78rem;
        color: var(--text-muted, rgba(255, 255, 255, 0.6));
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .row-right {
        display: flex;
        align-items: center;
        gap: 0.75rem;
    }

    .local-badge {
        font-size: 0.72rem;
        font-weight: 700;
        color: #06d6a0;
        background: rgba(6, 214, 160, 0.1);
        padding: 0.2rem 0.5rem;
        border-radius: 4px;
    }

    .provider-badge {
        font-size: 0.72rem;
        font-weight: 700;
        color: var(--text-muted, rgba(255, 255, 255, 0.5));
        background: rgba(255, 255, 255, 0.05);
        padding: 0.2rem 0.5rem;
        border-radius: 4px;
    }

    .duration {
        font-size: 0.78rem;
        color: var(--text-muted, rgba(255, 255, 255, 0.5));
    }

    .no-results, .search-tip {
        padding: 2.5rem 1rem;
        text-align: center;
        color: var(--text-muted, rgba(255, 255, 255, 0.5));
        font-size: 0.9rem;
    }
</style>
