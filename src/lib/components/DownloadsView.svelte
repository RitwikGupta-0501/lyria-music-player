<script lang="ts">
    import { onMount } from "svelte";
    import { downloadStore, type DownloadJobSummary, type DownloadItemRecord } from "$lib/stores/download.svelte";
    import { audioStore } from "$lib/stores/audio.svelte";
    import { isCurrentTrack } from "$lib/utils/format";
    import {
        Tray,
        FolderOpen,
        Trash,
        Pause,
        Play,
        X,
        ArrowClockwise,
        CheckCircle,
        WarningCircle,
        MusicNotes,
        FileAudio,
    } from "phosphor-svelte";

    let activeTab = $state<"all" | "active" | "completed" | "failed">("all");

    onMount(async () => {
        await downloadStore.fetchQueue();
        await downloadStore.fetchDownloadsDir();
    });

    // Split queue into active, completed, and failed
    let activeItems = $derived.by(() => {
        const list: Array<{ job: DownloadJobSummary["download"]; item: DownloadItemRecord }> = [];
        for (const job of downloadStore.queue) {
            for (const item of job.items) {
                if (item.status === "downloading" || item.status === "queued" || item.status === "paused" || item.status === "resolving") {
                    list.push({ job: job.download, item });
                }
            }
        }
        return list;
    });

    let completedItems = $derived.by(() => {
        const list: Array<{ job: DownloadJobSummary["download"]; item: DownloadItemRecord }> = [];
        for (const job of downloadStore.queue) {
            for (const item of job.items) {
                if (item.status === "completed") {
                    list.push({ job: job.download, item });
                }
            }
        }
        return list;
    });

    let failedItems = $derived.by(() => {
        const list: Array<{ job: DownloadJobSummary["download"]; item: DownloadItemRecord }> = [];
        for (const job of downloadStore.queue) {
            for (const item of job.items) {
                if (item.status === "failed") {
                    list.push({ job: job.download, item });
                }
            }
        }
        return list;
    });

    let totalCount = $derived(activeItems.length + completedItems.length + failedItems.length);

    $effect(() => {
        if (activeTab === "failed" && failedItems.length === 0) {
            activeTab = "all";
        }
    });

    function formatBytes(bytes?: number | null): string {
        if (!bytes || bytes <= 0) return "0 MB";
        const mb = bytes / (1024 * 1024);
        if (mb >= 1000) {
            return `${(mb / 1024).toFixed(2)} GB`;
        }
        return `${mb.toFixed(1)} MB`;
    }

    function formatSpeed(bytesPerSec?: number | null): string {
        if (!bytesPerSec || bytesPerSec <= 0) return "0 KB/s";
        const kb = bytesPerSec / 1024;
        if (kb >= 1024) {
            return `${(kb / 1024).toFixed(1)} MB/s`;
        }
        return `${Math.round(kb)} KB/s`;
    }

    function getAudioFormat(filePath?: string | null): string {
        if (!filePath) return "AUDIO";
        const ext = filePath.split(".").pop();
        return ext ? ext.toUpperCase() : "AUDIO";
    }

    function calculatePercent(item: DownloadItemRecord): number {
        const live = downloadStore.progressMap[item.id];
        const downloaded = live ? live.downloaded_bytes : item.downloaded_bytes;
        const total = live ? live.total_bytes : item.total_bytes;
        if (!total || total <= 0) return 0;
        return Math.min(100, Math.round((downloaded / total) * 100));
    }

    function isItemPlaying(item: DownloadItemRecord): boolean {
        return (
            isCurrentTrack(
                { title: item.track_title, artist: item.artist, file_path: item.dest_file_path },
                audioStore.currentQueueTrack
            ) && audioStore.playbackState === "Playing"
        );
    }

    function isItemActive(item: DownloadItemRecord): boolean {
        return isCurrentTrack(
            { title: item.track_title, artist: item.artist, file_path: item.dest_file_path },
            audioStore.currentQueueTrack
        );
    }

    function togglePlayCompleted(item: DownloadItemRecord) {
        if (!item.dest_file_path) return;
        if (isItemPlaying(item)) {
            audioStore.pause();
        } else if (isItemActive(item) && audioStore.playbackState === "Paused") {
            audioStore.play();
        } else {
            audioStore.playInterrupt({
                id: item.id,
                title: item.track_title,
                artist: item.artist,
                album: item.album || "Singles & EP",
                file_path: item.dest_file_path,
                is_local: true,
            });
        }
    }
</script>

<div class="downloads-view">
    <!-- Sticky Frosted Header matching LibraryHeader & ProvidersView -->
    <header class="downloads-header">
        <div class="header-top">
            <div class="header-title-block">
                <h1 class="page-title font-headline-lg">Downloads</h1>
                <p class="page-subtitle text-2">
                    Manage active transfers, offline caching, and download history.
                    {#if downloadStore.downloadsDir}
                        <span class="storage-separator">•</span>
                        <span class="storage-path" title={downloadStore.downloadsDir}>
                            Storage: <code>{downloadStore.downloadsDir}</code>
                        </span>
                    {/if}
                </p>
            </div>

            <div class="header-actions">
                <button 
                    class="ghost action-btn" 
                    onclick={() => downloadStore.openDownloadsFolder()}
                    title="Open downloads directory in file manager"
                >
                    <FolderOpen size={16} weight="regular" />
                    <span>Open Folder</span>
                </button>

                {#if completedItems.length > 0}
                    <button 
                        class="ghost action-btn" 
                        onclick={() => downloadStore.clearCompletedDownloads()}
                        title="Clear completed download records from history"
                    >
                        <Trash size={16} />
                        <span>Clear History</span>
                    </button>
                {/if}
            </div>
        </div>

        <!-- Segmented Tab Navigation -->
        <div class="tabs">
            <button 
                class="tab" 
                class:active={activeTab === "all"} 
                onclick={() => activeTab = "all"}
            >
                All {#if totalCount > 0}<span class="tab-count">{totalCount}</span>{/if}
            </button>
            <button 
                class="tab" 
                class:active={activeTab === "active"} 
                onclick={() => activeTab = "active"}
            >
                Downloading
                {#if activeItems.length > 0}
                    <span class="tab-badge">{activeItems.length}</span>
                {/if}
            </button>
            <button 
                class="tab" 
                class:active={activeTab === "completed"} 
                onclick={() => activeTab = "completed"}
            >
                Completed
                {#if completedItems.length > 0}
                    <span class="tab-count">{completedItems.length}</span>
                {/if}
            </button>
            {#if failedItems.length > 0}
                <button 
                    class="tab error" 
                    class:active={activeTab === "failed"} 
                    onclick={() => activeTab = "failed"}
                >
                    Failed
                    <span class="tab-badge error">{failedItems.length}</span>
                </button>
            {/if}
        </div>
    </header>

    <!-- Main Content Area -->
    <div class="downloads-content">
        {#if totalCount === 0}
            <!-- Global Empty State -->
            <div class="empty-state">
                <div class="empty-icon">
                    <Tray size={54} weight="thin" />
                </div>
                <h2 class="empty-heading font-headline-lg">No downloads yet</h2>
                <p class="empty-sub text-2">
                    Lossless FLAC and MP3 tracks downloaded from your extensions will appear here in real-time.
                </p>
                <button 
                    class="ghost" 
                    onclick={() => downloadStore.openDownloadsFolder()}
                    style="margin-top: 1.5rem;"
                >
                    <FolderOpen size={16} />
                    <span>Open Storage Folder</span>
                </button>
            </div>
        {:else}
            <!-- TAB 1: ALL OR FILTERED ACTIVE -->
            {#if (activeTab === "all" || activeTab === "active") && activeItems.length > 0}
                <section class="download-section">
                    <div class="section-heading">
                        <div class="heading-title-group">
                            <h2>Transfers in Progress</h2>
                            <span class="count-pill">{activeItems.length}</span>
                        </div>
                    </div>

                    <div class="items-grid">
                        {#each activeItems as { job, item } (item.id)}
                            {@const liveProgress = downloadStore.progressMap[item.id]}
                            {@const percent = calculatePercent(item)}
                            {@const speed = liveProgress ? formatSpeed(liveProgress.speed_bytes_per_sec) : null}
                            {@const downloaded = liveProgress ? liveProgress.downloaded_bytes : item.downloaded_bytes}
                            {@const total = liveProgress ? liveProgress.total_bytes : item.total_bytes}
                            {@const isPaused = item.status === "paused"}

                            <div class="active-card" class:is-paused={isPaused}>
                                <div class="card-top">
                                    <div class="track-meta">
                                        <h3 class="track-name" title={item.track_title}>{item.track_title}</h3>
                                        <p class="track-artist text-2">
                                            <span>{item.artist}</span>
                                            {#if item.album}
                                                <span class="dot">•</span>
                                                <span>{item.album}</span>
                                            {/if}
                                        </p>
                                    </div>

                                    <div class="card-controls">
                                        {#if isPaused}
                                            <button 
                                                class="control-btn resume" 
                                                onclick={() => downloadStore.resumeDownload(job.id)}
                                                title="Resume download"
                                                aria-label="Resume download"
                                            >
                                                <Play size={13} weight="fill" />
                                            </button>
                                        {:else}
                                            <button 
                                                class="control-btn pause" 
                                                onclick={() => downloadStore.pauseDownload(job.id)}
                                                title="Pause download"
                                                aria-label="Pause download"
                                            >
                                                <Pause size={13} weight="bold" />
                                            </button>
                                        {/if}
                                        <button 
                                            class="control-btn cancel" 
                                            onclick={() => downloadStore.cancelDownload(job.id)}
                                            title="Cancel download"
                                            aria-label="Cancel download"
                                        >
                                            <X size={14} weight="bold" />
                                        </button>
                                    </div>
                                </div>

                                <!-- Progress Bar -->
                                <div class="progress-track" role="progressbar" aria-valuenow={percent} aria-valuemin={0} aria-valuemax={100}>
                                    <div class="progress-fill" style="width: {percent}%;"></div>
                                </div>

                                <!-- Telemetry Footer -->
                                <div class="card-telemetry">
                                    <div class="telemetry-left">
                                        <span class="percent-text">{percent}%</span>
                                        {#if total > 0}
                                            <span class="bytes-text">{formatBytes(downloaded)} / {formatBytes(total)}</span>
                                        {/if}
                                    </div>
                                    <div class="telemetry-right">
                                        {#if isPaused}
                                            <span class="status-tag paused">Paused</span>
                                        {:else if speed}
                                            <span class="speed-text">{speed}</span>
                                        {:else}
                                            <span class="status-tag connecting">Connecting...</span>
                                        {/if}
                                    </div>
                                </div>
                            </div>
                        {/each}
                    </div>
                </section>
            {:else if activeTab === "active" && activeItems.length === 0}
                <div class="empty-state sub-empty">
                    <div class="empty-icon">
                        <Tray size={42} weight="thin" />
                    </div>
                    <h3 class="empty-heading font-headline-lg">No active downloads</h3>
                    <p class="empty-sub text-2">Active transfers and enqueued tracks will appear here.</p>
                </div>
            {/if}

            <!-- TAB 2: ALL OR FILTERED FAILED -->
            {#if (activeTab === "all" || activeTab === "failed") && failedItems.length > 0}
                <section class="download-section">
                    <div class="section-heading error">
                        <div class="heading-title-group">
                            <h2>Failed Downloads</h2>
                            <span class="count-pill error">{failedItems.length}</span>
                        </div>
                        {#if failedItems.length > 1}
                            <button 
                                class="retry-all-btn" 
                                onclick={() => downloadStore.retryAllFailed()}
                                title="Retry all failed downloads"
                            >
                                <ArrowClockwise size={13} weight="bold" />
                                <span>Retry All</span>
                            </button>
                        {/if}
                    </div>

                    <div class="failed-list">
                        {#each failedItems as { job, item } (item.id)}
                            <div class="failed-row">
                                <div class="failed-meta">
                                    <span class="failed-title">{item.track_title}</span>
                                    <span class="failed-artist text-2">{item.artist}</span>
                                    {#if item.error_message}
                                        <p class="error-desc" title={item.error_message}>{item.error_message}</p>
                                    {/if}
                                </div>
                                <div class="failed-actions">
                                    <button 
                                        class="retry-btn" 
                                        onclick={() => downloadStore.retryDownloadItem(item.id)}
                                        title="Retry download"
                                    >
                                        <ArrowClockwise size={13} weight="bold" />
                                        <span>Retry</span>
                                    </button>
                                    <button 
                                        class="control-btn cancel" 
                                        onclick={() => downloadStore.dismissDownloadItem(item.id)}
                                        title="Dismiss"
                                    >
                                        <X size={14} />
                                    </button>
                                </div>
                            </div>
                        {/each}
                    </div>
                </section>
            {:else if activeTab === "failed" && failedItems.length === 0}
                <div class="empty-state sub-empty">
                    <div class="empty-icon">
                        <CheckCircle size={42} weight="thin" color="var(--echo-primary)" />
                    </div>
                    <h3 class="empty-heading font-headline-lg">No failed downloads</h3>
                    <p class="empty-sub text-2">All completed tracks have been saved successfully.</p>
                </div>
            {/if}

            <!-- TAB 3: ALL OR FILTERED COMPLETED -->
            {#if (activeTab === "all" || activeTab === "completed") && completedItems.length > 0}
                <section class="download-section">
                    <div class="section-heading">
                        <div class="heading-title-group">
                            <h2>Completed Tracks</h2>
                            <span class="count-pill">{completedItems.length}</span>
                        </div>
                    </div>

                    <div class="completed-ledger">
                        {#each completedItems as { job, item }, i (item.id)}
                            {@const playing = isItemPlaying(item)}
                            {@const active = isItemActive(item)}
                            {@const format = getAudioFormat(item.dest_file_path)}

                            <!-- svelte-ignore a11y_click_events_have_key_events -->
                            <!-- svelte-ignore a11y_no_static_element_interactions -->
                            <div 
                                class="completed-row group" 
                                class:active={active}
                                onclick={() => togglePlayCompleted(item)}
                            >
                                <div class="index-col">
                                    <button 
                                        class="play-toggle-btn" 
                                        class:is-playing={playing}
                                        onclick={(e) => { e.stopPropagation(); togglePlayCompleted(item); }}
                                        title={playing ? "Pause track" : "Play track"}
                                        aria-label="{playing ? 'Pause' : 'Play'} {item.track_title}"
                                    >
                                        {#if playing}
                                            <Pause size={13} weight="fill" />
                                        {:else}
                                            <span class="icon-optical-wrap">
                                                <Play size={13} weight="fill" />
                                            </span>
                                        {/if}
                                    </button>
                                </div>

                                <div class="art-col">
                                    <div class="track-thumb">
                                        <MusicNotes size={18} weight="duotone" color="var(--echo-primary)" />
                                    </div>
                                </div>

                                <div class="info-col">
                                    <span class="track-title" class:active-title={active} title={item.track_title}>
                                        {item.track_title}
                                    </span>
                                    <div class="track-meta-row text-2">
                                        <span class="track-artist-name">{item.artist}</span>
                                        {#if item.album}
                                            <span class="dot">•</span>
                                            <span class="track-album-name">{item.album}</span>
                                        {/if}
                                    </div>
                                </div>

                                <div class="badge-col">
                                    <span class="format-pill">{format}</span>
                                    {#if item.total_bytes}
                                        <span class="size-pill">{formatBytes(item.total_bytes)}</span>
                                    {/if}
                                </div>

                                <div class="status-col">
                                    <CheckCircle size={18} weight="fill" color="var(--echo-primary)" />
                                </div>
                            </div>
                        {/each}
                    </div>
                </section>
            {:else if activeTab === "completed" && completedItems.length === 0}
                <div class="empty-state sub-empty">
                    <div class="empty-icon">
                        <MusicNotes size={42} weight="thin" />
                    </div>
                    <h3 class="empty-heading font-headline-lg">No completed downloads</h3>
                    <p class="empty-sub text-2">Tracks that finish downloading will appear here in your offline library.</p>
                </div>
            {/if}
        {/if}
    </div>
</div>

<style>
    /* ─── Downloads View Root ─────────────────────────────────────────── */
    .downloads-view {
        width: 100%;
        min-height: 100%;
        display: flex;
        flex-direction: column;
        background: transparent;
        color: var(--echo-text-1);
    }

    /* ─── Sticky Frosted Header (matching LibraryHeader) ─────────────── */
    .downloads-header {
        position: sticky;
        top: 0;
        background: rgba(5, 5, 7, 0.92);
        backdrop-filter: blur(24px);
        -webkit-backdrop-filter: blur(24px);
        z-index: 20;
        padding: 3rem 2.5rem 1.5rem;
        border-bottom: 1px solid rgba(255, 255, 255, 0.05);
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
    }

    .header-top {
        display: flex;
        justify-content: space-between;
        align-items: flex-end;
        gap: 1.5rem;
    }

    .header-title-block {
        display: flex;
        flex-direction: column;
        gap: 0.35rem;
        min-width: 0;
    }

    .page-title {
        font-family: var(--lyria-font-heading);
        font-size: 2.25rem;
        font-weight: 600;
        letter-spacing: -0.025em;
        line-height: 1.25;
        margin: 0;
        color: var(--echo-text-1);
    }

    .page-subtitle {
        font-size: 0.925rem;
        margin: 0;
        display: flex;
        align-items: center;
        flex-wrap: wrap;
        gap: 0.4rem;
        line-height: 1.4;
    }

    .storage-separator {
        opacity: 0.4;
    }

    .storage-path {
        display: inline-flex;
        align-items: center;
        gap: 0.3rem;
        color: var(--echo-text-2);
    }

    .storage-path code {
        font-family: var(--lyria-font-mono, monospace);
        font-size: 0.8rem;
        background: rgba(255, 255, 255, 0.05);
        border: 1px solid rgba(255, 255, 255, 0.07);
        padding: 0.15rem 0.45rem;
        border-radius: var(--radius-xs, 4px);
        color: var(--echo-text-1);
        max-width: 380px;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .header-actions {
        display: flex;
        align-items: center;
        gap: 0.65rem;
        flex-shrink: 0;
    }

    .action-btn {
        padding: 0.5rem 0.85rem;
        font-size: 0.8125rem;
    }

    /* ─── Navigation Tabs (matching LibraryHeader) ───────────────────── */
    .tabs {
        display: flex;
        gap: 0.5rem;
        font-size: 0.875rem;
        letter-spacing: 0.08em;
        text-transform: uppercase;
        font-weight: 500;
        color: var(--echo-text-2);
        margin-top: 0.25rem;
    }

    .tab {
        appearance: none;
        -webkit-appearance: none;
        background: transparent;
        border: none;
        outline: none;
        color: inherit;
        cursor: pointer;
        padding: 0.5rem 0.95rem;
        border-radius: var(--radius-md, 8px);
        transition: all 0.2s ease;
        display: inline-flex;
        align-items: center;
        gap: 0.45rem;
    }

    .tab:hover:not(.active) {
        color: var(--echo-text-1);
        background: rgba(255, 255, 255, 0.04);
    }

    .tab.active {
        color: var(--echo-primary);
        background: rgba(226, 169, 115, 0.1);
    }

    .tab.error.active {
        color: #f87171;
        background: rgba(248, 113, 113, 0.12);
    }

    .tab-count {
        font-size: 0.75rem;
        opacity: 0.7;
        font-variant-numeric: tabular-nums;
    }

    .tab-badge {
        font-size: 0.7rem;
        font-weight: 700;
        background: rgba(226, 169, 115, 0.22);
        color: var(--echo-primary);
        padding: 0.1rem 0.45rem;
        border-radius: var(--radius-pill, 9999px);
    }

    .tab-badge.error {
        background: rgba(248, 113, 113, 0.2);
        color: #f87171;
    }

    /* ─── Main Content Canvas ─────────────────────────────────────────── */
    .downloads-content {
        padding: 2.5rem 2.5rem var(--player-clearance, 10rem) 2.5rem;
        scroll-padding-bottom: var(--player-scroll-padding, 10rem);
        display: flex;
        flex-direction: column;
        gap: 3rem;
        width: 100%;
        box-sizing: border-box;
    }

    @media (max-width: 900px) {
        .downloads-header {
            padding: 2rem 1.5rem 1.25rem;
        }
        .downloads-content {
            padding: 1.5rem 1.5rem 10rem 1.5rem;
        }
    }

    /* ─── Section Header ─────────────────────────────────────────────── */
    .download-section {
        display: flex;
        flex-direction: column;
        gap: 1.25rem;
        width: 100%;
    }

    .section-heading {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 0.75rem;
    }

    .heading-title-group {
        display: flex;
        align-items: center;
        gap: 0.65rem;
    }

    .section-heading h2 {
        font-family: var(--lyria-font-heading);
        font-size: 1.25rem;
        font-weight: 600;
        letter-spacing: -0.02em;
        margin: 0;
        color: var(--echo-text-1);
    }

    .section-heading.error h2 {
        color: #f87171;
    }

    .count-pill {
        font-size: 0.75rem;
        font-weight: 600;
        background: rgba(255, 255, 255, 0.06);
        color: var(--echo-text-2);
        padding: 0.15rem 0.55rem;
        border-radius: var(--radius-pill, 9999px);
        font-variant-numeric: tabular-nums;
    }

    .count-pill.error {
        background: rgba(248, 113, 113, 0.15);
        color: #f87171;
    }

    .retry-all-btn {
        display: inline-flex;
        align-items: center;
        gap: 0.35rem;
        padding: 0.3rem 0.7rem;
        background: rgba(248, 113, 113, 0.12);
        border: 1px solid rgba(248, 113, 113, 0.25);
        color: #f87171;
        border-radius: var(--radius-sm, 6px);
        font-size: 0.78rem;
        font-weight: 600;
        cursor: pointer;
        transition: background 0.15s ease, transform 0.1s ease;
    }

    .retry-all-btn:hover {
        background: rgba(248, 113, 113, 0.2);
        transform: translateY(-1px);
    }

    /* ─── Active Transfer Cards (Liquid Glass) ───────────────────────── */
    .items-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(360px, 1fr));
        gap: 1.25rem;
    }

    .active-card {
        padding: 1.25rem;
        background: var(--liquid-glass-card-bg, rgba(20, 20, 26, 0.5));
        backdrop-filter: blur(16px);
        -webkit-backdrop-filter: blur(16px);
        border: 1px solid var(--echo-border);
        border-radius: var(--radius-lg, 12px);
        display: flex;
        flex-direction: column;
        gap: 0.95rem;
        transition: border-color 0.2s ease, box-shadow 0.2s ease;
    }

    .active-card:hover {
        border-color: var(--echo-border-medium);
        box-shadow: 0 8px 24px -4px rgba(0, 0, 0, 0.4);
    }

    .active-card.is-paused {
        opacity: 0.75;
    }

    .card-top {
        display: flex;
        justify-content: space-between;
        align-items: flex-start;
        gap: 0.85rem;
    }

    .track-meta {
        min-width: 0;
        flex: 1;
    }

    .track-name {
        margin: 0;
        font-size: 0.98rem;
        font-weight: 600;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        color: var(--echo-text-1);
    }

    .track-artist {
        margin: 0.25rem 0 0 0;
        font-size: 0.8125rem;
        display: flex;
        align-items: center;
        gap: 0.3rem;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .dot {
        opacity: 0.4;
    }

    .card-controls {
        display: flex;
        align-items: center;
        gap: 0.4rem;
        flex-shrink: 0;
    }

    .control-btn {
        width: 30px;
        height: 30px;
        border-radius: var(--radius-sm, 6px);
        background: rgba(255, 255, 255, 0.05);
        border: 1px solid var(--echo-border);
        color: var(--echo-text-2);
        display: flex;
        align-items: center;
        justify-content: center;
        cursor: pointer;
        padding: 0;
        transition: all 0.15s ease;
    }

    .control-btn:hover {
        background: rgba(255, 255, 255, 0.1);
        color: var(--echo-text-1);
        transform: scale(1.05);
    }

    .control-btn.resume:hover {
        color: var(--echo-primary);
        border-color: rgba(226, 169, 115, 0.35);
    }

    .control-btn.pause:hover {
        color: var(--echo-primary);
        border-color: rgba(226, 169, 115, 0.35);
    }

    .control-btn.cancel:hover {
        color: #f87171;
        border-color: rgba(248, 113, 113, 0.35);
    }

    .progress-track {
        height: 5px;
        width: 100%;
        background: rgba(255, 255, 255, 0.06);
        border-radius: var(--radius-pill, 9999px);
        overflow: hidden;
    }

    .progress-fill {
        height: 100%;
        background: linear-gradient(90deg, var(--echo-primary-dark), var(--echo-primary));
        border-radius: var(--radius-pill, 9999px);
        transition: width 0.25s ease-out;
    }

    .card-telemetry {
        display: flex;
        justify-content: space-between;
        align-items: center;
        font-size: 0.8rem;
    }

    .telemetry-left {
        display: flex;
        align-items: baseline;
        gap: 0.5rem;
    }

    .percent-text {
        font-weight: 700;
        color: var(--echo-primary);
        font-variant-numeric: tabular-nums;
    }

    .bytes-text {
        color: var(--echo-text-3);
        font-variant-numeric: tabular-nums;
        font-size: 0.775rem;
    }

    .speed-text {
        color: var(--echo-text-2);
        font-weight: 600;
        font-variant-numeric: tabular-nums;
    }

    .status-tag {
        font-size: 0.7rem;
        padding: 0.15rem 0.45rem;
        border-radius: var(--radius-xs, 4px);
        font-weight: 600;
        text-transform: uppercase;
        letter-spacing: 0.04em;
    }

    .status-tag.paused {
        background: rgba(226, 169, 115, 0.15);
        color: var(--echo-primary);
    }

    .status-tag.connecting {
        color: var(--echo-text-3);
    }

    /* ─── Failed Downloads ───────────────────────────────────────────── */
    .failed-list {
        display: flex;
        flex-direction: column;
        gap: 0.65rem;
    }

    .failed-row {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 0.95rem 1.25rem;
        background: rgba(248, 113, 113, 0.04);
        border: 1px solid rgba(248, 113, 113, 0.15);
        border-radius: var(--radius-md, 8px);
        gap: 1rem;
    }

    .failed-meta {
        display: flex;
        flex-direction: column;
        gap: 0.15rem;
        min-width: 0;
        flex: 1;
    }

    .failed-title {
        font-size: 0.95rem;
        font-weight: 600;
        color: var(--echo-text-1);
    }

    .failed-artist {
        font-size: 0.825rem;
    }

    .error-desc {
        margin: 0.25rem 0 0 0;
        font-size: 0.78rem;
        color: #f87171;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .failed-actions {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        flex-shrink: 0;
    }

    .retry-btn {
        display: inline-flex;
        align-items: center;
        gap: 0.35rem;
        padding: 0.35rem 0.75rem;
        background: rgba(248, 113, 113, 0.12);
        border: 1px solid rgba(248, 113, 113, 0.25);
        color: #f87171;
        border-radius: var(--radius-sm, 6px);
        font-size: 0.8rem;
        font-weight: 600;
        cursor: pointer;
        transition: background 0.15s ease;
    }

    .retry-btn:hover {
        background: rgba(248, 113, 113, 0.2);
    }

    /* ─── Completed Tracks Ledger (CollectionDetail style) ───────────── */
    .completed-ledger {
        display: flex;
        flex-direction: column;
        background: rgba(18, 18, 22, 0.45);
        border: 1px solid var(--echo-border);
        border-radius: var(--radius-lg, 12px);
        overflow: hidden;
    }

    .completed-row {
        display: flex;
        align-items: center;
        gap: 1rem;
        padding: 0.75rem 1.25rem;
        border-bottom: 1px solid rgba(255, 255, 255, 0.035);
        cursor: pointer;
        user-select: none;
        transition: background 0.15s ease;
    }

    .completed-row:last-child {
        border-bottom: none;
    }

    .completed-row:hover {
        background: rgba(226, 169, 115, 0.04);
    }

    .completed-row.active {
        background: rgba(226, 169, 115, 0.08);
    }

    .index-col {
        width: 32px;
        display: flex;
        align-items: center;
        justify-content: center;
        flex-shrink: 0;
    }

    .play-toggle-btn {
        width: 30px;
        height: 30px;
        border-radius: 50%;
        background: rgba(255, 255, 255, 0.08);
        border: 1px solid rgba(255, 255, 255, 0.12);
        color: var(--echo-text-1);
        display: flex;
        align-items: center;
        justify-content: center;
        cursor: pointer;
        padding: 0;
        transition: transform 0.15s ease, background 0.15s ease, color 0.15s ease;
    }

    .play-toggle-btn.is-playing,
    .play-toggle-btn:hover {
        background: var(--echo-primary);
        color: var(--echo-void);
        border-color: var(--echo-primary);
        transform: scale(1.08);
    }

    .icon-optical-wrap {
        display: flex;
        align-items: center;
        justify-content: center;
        transform: translateX(1px);
    }

    .art-col {
        flex-shrink: 0;
    }

    .track-thumb {
        width: 38px;
        height: 38px;
        border-radius: var(--radius-sm, 6px);
        background: rgba(255, 255, 255, 0.04);
        border: 1px solid var(--echo-border);
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .info-col {
        flex: 1;
        min-width: 0;
        display: flex;
        flex-direction: column;
        gap: 0.15rem;
    }

    .track-title {
        font-size: 0.925rem;
        font-weight: 500;
        color: var(--echo-text-1);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .track-title.active-title {
        color: var(--echo-primary);
    }

    .track-meta-row {
        font-size: 0.8rem;
        display: flex;
        align-items: center;
        gap: 0.3rem;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .track-artist-name, .track-album-name {
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .badge-col {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        flex-shrink: 0;
    }

    .format-pill {
        font-family: var(--lyria-font-mono, monospace);
        font-size: 0.7rem;
        font-weight: 600;
        background: rgba(226, 169, 115, 0.12);
        color: var(--echo-primary);
        border: 1px solid rgba(226, 169, 115, 0.25);
        padding: 0.1rem 0.45rem;
        border-radius: var(--radius-xs, 4px);
        letter-spacing: 0.05em;
    }

    .size-pill {
        font-size: 0.775rem;
        color: var(--echo-text-3);
        font-variant-numeric: tabular-nums;
    }

    .status-col {
        width: 30px;
        display: flex;
        align-items: center;
        justify-content: flex-end;
        flex-shrink: 0;
    }

    /* ─── Empty States (matching AlbumGrid) ───────────────────────────── */
    .empty-state {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        height: 50vh;
        text-align: center;
        gap: 0.5rem;
    }

    .empty-state.sub-empty {
        height: 35vh;
    }

    .empty-icon {
        color: var(--echo-text-3);
        margin-bottom: 1rem;
        opacity: 0.6;
    }

    .empty-heading {
        font-size: 1.75rem;
        font-weight: 500;
        color: var(--echo-text-1);
        letter-spacing: -0.025em;
        margin: 0;
    }

    .empty-sub {
        font-size: 0.95rem;
        max-width: 340px;
        line-height: 1.5;
        margin: 0;
    }
</style>
