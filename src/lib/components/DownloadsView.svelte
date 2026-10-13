<script lang="ts">
    import { downloadStore, type DownloadJobSummary, type DownloadItemRecord } from "$lib/stores/download.svelte";
    import { audioStore } from "$lib/stores/audio.svelte";
    import { formatDuration } from "$lib/utils/format";
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
        ArrowSquareOut,
    } from "phosphor-svelte";

    // Split queue into active, completed, and failed
    let activeItems = $derived.by(() => {
        const list: Array<{ job: DownloadJobSummary["download"]; item: DownloadItemRecord }> = [];
        for (const job of downloadStore.queue) {
            for (const item of job.items) {
                if (item.status === "downloading" || item.status === "queued" || item.status === "paused") {
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

    function calculatePercent(item: DownloadItemRecord): number {
        const live = downloadStore.progressMap[item.id];
        const downloaded = live ? live.downloaded_bytes : item.downloaded_bytes;
        const total = live ? live.total_bytes : item.total_bytes;
        if (!total || total <= 0) return 0;
        return Math.min(100, Math.round((downloaded / total) * 100));
    }

    function handlePlayCompleted(item: DownloadItemRecord) {
        if (!item.dest_file_path) return;
        audioStore.playInterrupt({
            id: item.id,
            title: item.track_title,
            artist: item.artist,
            album: item.album || "Singles & EP",
            file_path: item.dest_file_path,
        });
    }
</script>

<div class="downloads-page">
    <!-- Header -->
    <header class="downloads-header">
        <div class="header-left">
            <div class="title-row">
                <h2>Downloads</h2>
                {#if activeItems.length > 0}
                    <span class="active-badge">{activeItems.length} active</span>
                {/if}
            </div>
            {#if downloadStore.downloadsDir}
                <div class="dir-meta">
                    <span class="dir-label">Storage:</span>
                    <span class="dir-path" title={downloadStore.downloadsDir}>{downloadStore.downloadsDir}</span>
                </div>
            {/if}
        </div>

        <div class="header-actions">
            <button 
                class="header-btn" 
                onclick={() => downloadStore.openDownloadsFolder()}
                title="Open downloads directory in file manager"
            >
                <FolderOpen size={16} weight="bold" />
                <span>Open Folder</span>
            </button>

            {#if completedItems.length > 0}
                <button 
                    class="header-btn ghost text-muted" 
                    onclick={() => downloadStore.clearCompletedDownloads()}
                    title="Clear completed download records from history"
                >
                    <Trash size={16} />
                    <span>Clear History</span>
                </button>
            {/if}
        </div>
    </header>

    <!-- Main Content Area -->
    <div class="downloads-content">
        {#if activeItems.length === 0 && completedItems.length === 0 && failedItems.length === 0}
            <div class="empty-state">
                <div class="empty-icon-wrap">
                    <Tray size={48} weight="light" />
                </div>
                <h3>No active or recent downloads</h3>
                <p>Downloaded lossless FLAC and MP3 tracks from your extensions will appear here in real-time.</p>
            </div>
        {:else}
            <!-- 1. ACTIVE & QUEUED TRANSFERS -->
            {#if activeItems.length > 0}
                <section class="download-section">
                    <div class="section-heading">
                        <h3>Transfers in Progress</h3>
                        <span class="count-pill">{activeItems.length}</span>
                    </div>

                    <div class="items-grid">
                        {#each activeItems as { job, item } (item.id)}
                            {@const liveProgress = downloadStore.progressMap[item.id]}
                            {@const percent = calculatePercent(item)}
                            {@const speed = liveProgress ? formatSpeed(liveProgress.speed_bytes_per_sec) : null}
                            {@const downloaded = liveProgress ? liveProgress.downloaded_bytes : item.downloaded_bytes}
                            {@const total = liveProgress ? liveProgress.total_bytes : item.total_bytes}
                            {@const isPaused = item.status === "paused"}

                            <div class="active-card glass-panel" class:is-paused={isPaused}>
                                <div class="card-top">
                                    <div class="track-meta">
                                        <h4 class="track-name" title={item.track_title}>{item.track_title}</h4>
                                        <p class="track-artist">
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
                                                class="action-icon-btn resume" 
                                                onclick={() => downloadStore.resumeDownload(job.id)}
                                                title="Resume download"
                                                aria-label="Resume download"
                                            >
                                                <Play size={15} weight="fill" />
                                            </button>
                                        {:else}
                                            <button 
                                                class="action-icon-btn pause" 
                                                onclick={() => downloadStore.pauseDownload(job.id)}
                                                title="Pause download"
                                                aria-label="Pause download"
                                            >
                                                <Pause size={15} weight="bold" />
                                            </button>
                                        {/if}
                                        <button 
                                            class="action-icon-btn cancel" 
                                            onclick={() => downloadStore.cancelDownload(job.id)}
                                            title="Cancel download"
                                            aria-label="Cancel download"
                                        >
                                            <X size={15} weight="bold" />
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
            {/if}

            <!-- 2. FAILED DOWNLOADS -->
            {#if failedItems.length > 0}
                <section class="download-section">
                    <div class="section-heading error">
                        <h3>Failed Downloads</h3>
                        <span class="count-pill error">{failedItems.length}</span>
                    </div>

                    <div class="items-list">
                        {#each failedItems as { job, item } (item.id)}
                            <div class="failed-row glass-panel">
                                <div class="failed-meta">
                                    <span class="track-name">{item.track_title}</span>
                                    <span class="track-artist">{item.artist}</span>
                                    {#if item.error_message}
                                        <p class="error-desc" title={item.error_message}>{item.error_message}</p>
                                    {/if}
                                </div>
                                <div class="failed-actions">
                                    <button 
                                        class="retry-btn" 
                                        onclick={() => downloadStore.resumeDownload(job.id)}
                                        title="Retry download"
                                    >
                                        <ArrowClockwise size={14} weight="bold" />
                                        <span>Retry</span>
                                    </button>
                                    <button 
                                        class="action-icon-btn cancel" 
                                        onclick={() => downloadStore.cancelDownload(job.id)}
                                        title="Dismiss"
                                    >
                                        <X size={15} />
                                    </button>
                                </div>
                            </div>
                        {/each}
                    </div>
                </section>
            {/if}

            <!-- 3. COMPLETED DOWNLOADS -->
            {#if completedItems.length > 0}
                <section class="download-section">
                    <div class="section-heading">
                        <h3>Completed</h3>
                        <span class="count-pill">{completedItems.length}</span>
                    </div>

                    <div class="completed-table glass-panel">
                        {#each completedItems as { job, item } (item.id)}
                            <div class="completed-row group">
                                <div class="thumb-col">
                                    <div class="completed-icon">
                                        <CheckCircle size={18} weight="fill" color="#34d399" />
                                    </div>
                                </div>

                                <div class="title-col">
                                    <span class="completed-title" title={item.track_title}>{item.track_title}</span>
                                    <div class="sub-col">
                                        <span class="completed-artist">{item.artist}</span>
                                        {#if item.album}
                                            <span class="dot">•</span>
                                            <span class="completed-album">{item.album}</span>
                                        {/if}
                                    </div>
                                </div>

                                <div class="meta-col">
                                    {#if item.total_bytes}
                                        <span class="size-tag">{formatBytes(item.total_bytes)}</span>
                                    {/if}
                                </div>

                                <div class="play-action-col">
                                    <button 
                                        class="play-circle-btn" 
                                        onclick={() => handlePlayCompleted(item)}
                                        title="Play track"
                                        aria-label="Play {item.track_title}"
                                    >
                                        <Play size={13} weight="fill" />
                                    </button>
                                </div>
                            </div>
                        {/each}
                    </div>
                </section>
            {/if}
        {/if}
    </div>
</div>

<style>
    .downloads-page {
        display: flex;
        flex-direction: column;
        padding: 2.5rem 3rem;
        min-height: 100%;
        max-width: 1100px;
        margin: 0 auto;
        color: #ffffff;
    }

    .downloads-header {
        display: flex;
        justify-content: space-between;
        align-items: flex-start;
        padding-bottom: 2rem;
        border-bottom: 1px solid rgba(255, 255, 255, 0.06);
        margin-bottom: 2rem;
    }

    .title-row {
        display: flex;
        align-items: center;
        gap: 0.85rem;
    }

    .title-row h2 {
        margin: 0;
        font-size: 1.85rem;
        font-weight: 700;
        letter-spacing: -0.02em;
    }

    .active-badge {
        font-size: 0.72rem;
        font-weight: 700;
        text-transform: uppercase;
        letter-spacing: 0.04em;
        background: rgba(255, 210, 133, 0.15);
        color: #ffd285;
        border: 1px solid rgba(255, 210, 133, 0.3);
        padding: 0.2rem 0.55rem;
        border-radius: 9999px;
    }

    .dir-meta {
        display: flex;
        align-items: center;
        gap: 0.4rem;
        margin-top: 0.5rem;
        font-size: 0.82rem;
        color: rgba(255, 255, 255, 0.45);
    }

    .dir-label {
        color: rgba(255, 255, 255, 0.3);
    }

    .dir-path {
        font-family: var(--lyria-font-mono, monospace);
        background: rgba(255, 255, 255, 0.04);
        padding: 0.1rem 0.45rem;
        border-radius: 4px;
        max-width: 450px;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .header-actions {
        display: flex;
        align-items: center;
        gap: 0.6rem;
    }

    .header-btn {
        display: flex;
        align-items: center;
        gap: 0.45rem;
        background: rgba(255, 255, 255, 0.06);
        border: 1px solid rgba(255, 255, 255, 0.1);
        color: #ffffff;
        padding: 0.5rem 0.95rem;
        border-radius: 8px;
        font-size: 0.85rem;
        font-weight: 500;
        cursor: pointer;
        transition: background 0.15s ease, border-color 0.15s ease, transform 0.12s ease;
    }

    .header-btn:hover {
        background: rgba(255, 255, 255, 0.12);
        border-color: rgba(255, 255, 255, 0.2);
        transform: translateY(-1px);
    }

    .header-btn:active {
        transform: translateY(0);
    }

    .header-btn.ghost {
        background: transparent;
        border-color: transparent;
    }

    .header-btn.ghost:hover {
        background: rgba(255, 255, 255, 0.06);
    }

    .downloads-content {
        display: flex;
        flex-direction: column;
        gap: 2.5rem;
    }

    .empty-state {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        padding: 6rem 1rem;
        text-align: center;
        color: rgba(255, 255, 255, 0.4);
    }

    .empty-icon-wrap {
        margin-bottom: 1rem;
        color: rgba(255, 255, 255, 0.2);
    }

    .empty-state h3 {
        margin: 0;
        font-size: 1.15rem;
        font-weight: 600;
        color: rgba(255, 255, 255, 0.7);
    }

    .empty-state p {
        margin: 0.4rem 0 0 0;
        font-size: 0.88rem;
        max-width: 380px;
        line-height: 1.45;
    }

    .download-section {
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }

    .section-heading {
        display: flex;
        align-items: center;
        gap: 0.6rem;
    }

    .section-heading h3 {
        margin: 0;
        font-size: 1.1rem;
        font-weight: 600;
    }

    .section-heading.error h3 {
        color: #f87171;
    }

    .count-pill {
        font-size: 0.75rem;
        font-weight: 700;
        background: rgba(255, 255, 255, 0.08);
        color: rgba(255, 255, 255, 0.6);
        padding: 0.15rem 0.5rem;
        border-radius: 9999px;
    }

    .count-pill.error {
        background: rgba(248, 113, 113, 0.15);
        color: #f87171;
    }

    .items-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
        gap: 1rem;
    }

    .active-card {
        padding: 1.15rem;
        background: rgba(255, 255, 255, 0.03);
        border: 1px solid rgba(255, 255, 255, 0.07);
        border-radius: 12px;
        display: flex;
        flex-direction: column;
        gap: 0.85rem;
        transition: border-color 0.15s ease;
    }

    .active-card:hover {
        border-color: rgba(255, 255, 255, 0.14);
    }

    .active-card.is-paused {
        opacity: 0.75;
    }

    .card-top {
        display: flex;
        justify-content: space-between;
        align-items: flex-start;
        gap: 0.75rem;
    }

    .track-meta {
        min-width: 0;
        flex: 1;
    }

    .track-name {
        margin: 0;
        font-size: 0.95rem;
        font-weight: 600;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        color: #ffffff;
    }

    .track-artist {
        margin: 0.2rem 0 0 0;
        font-size: 0.8rem;
        color: rgba(255, 255, 255, 0.5);
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
        gap: 0.35rem;
        flex-shrink: 0;
    }

    .action-icon-btn {
        width: 28px;
        height: 28px;
        border-radius: 6px;
        background: rgba(255, 255, 255, 0.06);
        border: 1px solid rgba(255, 255, 255, 0.08);
        color: rgba(255, 255, 255, 0.7);
        display: flex;
        align-items: center;
        justify-content: center;
        cursor: pointer;
        padding: 0;
        transition: background 0.12s ease, color 0.12s ease, transform 0.1s ease;
    }

    .action-icon-btn:hover {
        background: rgba(255, 255, 255, 0.12);
        color: #ffffff;
        transform: scale(1.05);
    }

    .action-icon-btn.resume:hover {
        color: #34d399;
        border-color: rgba(52, 211, 153, 0.3);
    }

    .action-icon-btn.pause:hover {
        color: #ffd285;
        border-color: rgba(255, 210, 133, 0.3);
    }

    .action-icon-btn.cancel:hover {
        color: #f87171;
        border-color: rgba(248, 113, 113, 0.3);
    }

    .progress-track {
        height: 5px;
        width: 100%;
        background: rgba(255, 255, 255, 0.06);
        border-radius: 9999px;
        overflow: hidden;
    }

    .progress-fill {
        height: 100%;
        background: linear-gradient(90deg, #ffd285, #ffb547);
        border-radius: 9999px;
        transition: width 0.25s ease-out;
    }

    .card-telemetry {
        display: flex;
        justify-content: space-between;
        align-items: center;
        font-size: 0.78rem;
    }

    .telemetry-left {
        display: flex;
        align-items: baseline;
        gap: 0.5rem;
    }

    .percent-text {
        font-weight: 700;
        color: #ffd285;
        font-variant-numeric: tabular-nums;
    }

    .bytes-text {
        color: rgba(255, 255, 255, 0.4);
        font-variant-numeric: tabular-nums;
    }

    .speed-text {
        color: rgba(255, 255, 255, 0.65);
        font-weight: 600;
        font-variant-numeric: tabular-nums;
    }

    .status-tag {
        font-size: 0.7rem;
        padding: 0.1rem 0.4rem;
        border-radius: 4px;
        font-weight: 600;
        text-transform: uppercase;
        letter-spacing: 0.03em;
    }

    .status-tag.paused {
        background: rgba(255, 210, 133, 0.12);
        color: #ffd285;
    }

    .status-tag.connecting {
        color: rgba(255, 255, 255, 0.4);
    }

    .items-list {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .failed-row {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 0.85rem 1rem;
        background: rgba(248, 113, 113, 0.04);
        border: 1px solid rgba(248, 113, 113, 0.15);
        border-radius: 10px;
    }

    .failed-meta {
        display: flex;
        flex-direction: column;
        gap: 0.15rem;
        min-width: 0;
        flex: 1;
    }

    .error-desc {
        margin: 0.2rem 0 0 0;
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
        display: flex;
        align-items: center;
        gap: 0.35rem;
        padding: 0.35rem 0.75rem;
        background: rgba(248, 113, 113, 0.12);
        border: 1px solid rgba(248, 113, 113, 0.25);
        color: #f87171;
        border-radius: 6px;
        font-size: 0.8rem;
        font-weight: 600;
        cursor: pointer;
        transition: background 0.15s ease;
    }

    .retry-btn:hover {
        background: rgba(248, 113, 113, 0.2);
    }

    .completed-table {
        display: flex;
        flex-direction: column;
        background: rgba(255, 255, 255, 0.02);
        border: 1px solid rgba(255, 255, 255, 0.05);
        border-radius: 12px;
        overflow: hidden;
    }

    .completed-row {
        display: flex;
        align-items: center;
        gap: 0.85rem;
        padding: 0.7rem 1.15rem;
        border-bottom: 1px solid rgba(255, 255, 255, 0.04);
        transition: background 0.12s ease;
    }

    .completed-row:last-child {
        border-bottom: none;
    }

    .completed-row:hover {
        background: rgba(255, 255, 255, 0.04);
    }

    .thumb-col {
        flex-shrink: 0;
    }

    .title-col {
        flex: 1;
        min-width: 0;
    }

    .completed-title {
        display: block;
        font-size: 0.9rem;
        font-weight: 500;
        color: #ffffff;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .sub-col {
        display: flex;
        align-items: center;
        gap: 0.3rem;
        font-size: 0.78rem;
        color: rgba(255, 255, 255, 0.45);
        margin-top: 0.1rem;
    }

    .completed-artist, .completed-album {
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .meta-col {
        flex-shrink: 0;
    }

    .size-tag {
        font-size: 0.78rem;
        color: rgba(255, 255, 255, 0.4);
        font-variant-numeric: tabular-nums;
    }

    .play-action-col {
        flex-shrink: 0;
    }

    .play-circle-btn {
        width: 30px;
        height: 30px;
        border-radius: 50%;
        background: rgba(255, 255, 255, 0.08);
        border: 1px solid rgba(255, 255, 255, 0.12);
        color: #ffffff;
        display: flex;
        align-items: center;
        justify-content: center;
        cursor: pointer;
        transition: transform 0.15s ease, background 0.15s ease;
    }

    .play-circle-btn:hover {
        background: #ffd285;
        color: #121212;
        transform: scale(1.08);
    }
</style>
