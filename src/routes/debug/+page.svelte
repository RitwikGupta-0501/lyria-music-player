<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { listen, type UnlistenFn } from "@tauri-apps/api/event";
    import { toastStore } from "$lib/stores/toast.svelte";
    import { Copy, Check, Play, Pause, Trash, FolderOpen, MagnifyingGlass } from "phosphor-svelte";
    import { onMount, onDestroy } from "svelte";

    interface LogEntry {
        id: number;
        timestamp: number;
        category: string;
        level: string;
        message: string;
    }

    let logs = $state<LogEntry[]>([]);
    let selectedCategory = $state<string>("All");
    let searchFilter = $state<string>("");
    let autoScroll = $state<boolean>(true);
    let isPaused = $state<boolean>(false);
    let isCopySuccess = $state(false);
    let copySuccessTimer: ReturnType<typeof setTimeout> | null = null;

    let activeLevels = $state<{ [key: string]: boolean }>({
        INFO: true,
        WARN: true,
        ERROR: true,
        DEBUG: true,
    });

    let unlistenBatch: UnlistenFn | null = null;
    let logContainer: HTMLElement | null = null;

    const categories = ["All", "Backend", "Frontend", "WASM", "JS Sandbox", "Audio", "Database", "Network", "System"];

    const filteredLogs = $derived(
        logs.filter((log) => {
            // Category Filter
            if (selectedCategory !== "All") {
                if (selectedCategory === "WASM") {
                    if (!log.category.includes("WASM") && !log.message.includes("[PLUGIN LOG]"))
                        return false;
                } else if (!log.category.toLowerCase().includes(selectedCategory.toLowerCase())) {
                    return false;
                }
            }

            // Severity Level Filter
            const lvl = log.level.toUpperCase();
            if (activeLevels[lvl] === false) return false;

            // Search Text Filter
            if (searchFilter.trim() !== "") {
                const query = searchFilter.toLowerCase();
                const matchMsg = log.message.toLowerCase().includes(query);
                const matchCat = log.category.toLowerCase().includes(query);
                if (!matchMsg && !matchCat) return false;
            }

            return true;
        })
    );

    onMount(async () => {
        try {
            const initialLogs = await invoke<LogEntry[]>("get_debug_logs");
            logs = initialLogs;
            scrollToBottom();
        } catch (e) {
            console.error("Failed to load initial debug logs:", e);
        }

        unlistenBatch = await listen<LogEntry[]>("debug-log-batch", (e) => {
            if (isPaused) return;
            logs = [...logs, ...e.payload].slice(-500);
            if (autoScroll) {
                setTimeout(scrollToBottom, 50);
            }
        });
    });

    onDestroy(() => {
        if (unlistenBatch) unlistenBatch();
    });

    function scrollToBottom() {
        if (logContainer) {
            logContainer.scrollTop = logContainer.scrollHeight;
        }
    }

    async function handleClear() {
        try {
            await invoke("clear_debug_logs");
            logs = [];
        } catch (e) {
            console.error("Failed to clear logs:", e);
        }
    }

    async function handleCopy() {
        try {
            const text = await invoke<string>("copy_debug_log_to_clipboard");
            await navigator.clipboard.writeText(text);
            if (copySuccessTimer) clearTimeout(copySuccessTimer);
            isCopySuccess = true;
            copySuccessTimer = setTimeout(() => {
                isCopySuccess = false;
                copySuccessTimer = null;
            }, 1200);
            toastStore.success("Debug logs copied to clipboard.");
        } catch (e) {
            console.error("Failed to copy logs:", e);
            toastStore.error("Failed to copy debug logs.");
        }
    }

    async function handleOpenLogFolder() {
        try {
            await invoke("open_log_directory");
        } catch (e) {
            console.error("Failed to open log directory:", e);
        }
    }

    function toggleLevel(level: string) {
        activeLevels[level] = !activeLevels[level];
    }

    function formatTime(ts: number): string {
        const d = new Date(ts);
        return d.toTimeString().split(" ")[0] + "." + String(d.getMilliseconds()).padStart(3, "0");
    }

    function getCategoryClass(cat: string): string {
        const lower = cat.toLowerCase();
        if (lower.includes("frontend")) return "frontend";
        if (lower.includes("wasm") || lower.includes("plugin")) return "wasm";
        if (lower.includes("sandbox") || lower.includes("js")) return "sandbox";
        if (lower.includes("audio")) return "audio";
        if (lower.includes("database") || lower.includes("db")) return "database";
        if (lower.includes("network") || lower.includes("http")) return "network";
        if (lower.includes("system")) return "system";
        return "backend";
    }
</script>

<div class="debug-page">
    <!-- Top Controls Header -->
    <header class="debug-header">
        <div class="header-title-row">
            <div class="title-group">
                <h1 class="title">Lyria Debug Console</h1>
                <span class="log-count">{filteredLogs.length} / {logs.length} logs</span>
            </div>

            <div class="action-buttons">
                <button
                    class="btn btn-secondary"
                    class:active={isPaused}
                    onclick={() => (isPaused = !isPaused)}
                    title={isPaused ? "Resume log stream" : "Pause log stream"}
                >
                    {#if isPaused}
                        <Play size={13} weight="fill" />
                        <span>Resume</span>
                    {:else}
                        <Pause size={13} weight="fill" />
                        <span>Pause</span>
                    {/if}
                </button>
                <button class="btn btn-secondary" onclick={handleClear} title="Clear debug log buffer">
                    <Trash size={13} weight="regular" />
                    <span>Clear</span>
                </button>
                <button class="btn btn-secondary copy-btn" class:copy-success={isCopySuccess} onclick={handleCopy} title="Copy logs to clipboard">
                    {#if isCopySuccess}
                        <Check size={13} weight="bold" />
                        <span>Copied</span>
                    {:else}
                        <Copy size={13} weight="regular" />
                        <span>Copy Logs</span>
                    {/if}
                </button>
                <button class="btn btn-primary" onclick={handleOpenLogFolder} title="Open log folder in file manager">
                    <FolderOpen size={13} weight="regular" />
                    <span>Log Folder</span>
                </button>
            </div>
        </div>

        <!-- Filter Controls Row -->
        <div class="filters-row">
            <!-- Category Pills -->
            <div class="category-pills">
                {#each categories as cat}
                    <button
                        class="pill"
                        class:active={selectedCategory === cat}
                        onclick={() => (selectedCategory = cat)}
                    >
                        {cat}
                    </button>
                {/each}
            </div>

            <!-- Severity Checks -->
            <div class="level-checks">
                {#each ["INFO", "WARN", "ERROR", "DEBUG"] as lvl}
                    <button
                        class="level-btn {lvl.toLowerCase()}"
                        class:active={activeLevels[lvl]}
                        onclick={() => toggleLevel(lvl)}
                    >
                        {lvl}
                    </button>
                {/each}
            </div>

            <!-- Search Input -->
            <div class="search-box">
                <MagnifyingGlass size={13} class="search-icon" />
                <input
                    type="text"
                    placeholder="Search logs..."
                    bind:value={searchFilter}
                    class="search-input"
                />
            </div>

            <!-- Auto Scroll Checkbox -->
            <label class="auto-scroll-toggle" title="Auto-scroll to latest log entries">
                <span class="custom-checkbox" class:checked={autoScroll}>
                    <input type="checkbox" bind:checked={autoScroll} />
                    {#if autoScroll}
                        <Check size={12} weight="bold" />
                    {/if}
                </span>
                <span class="label-text">Auto-Scroll</span>
            </label>
        </div>
    </header>

    <!-- Monospace Log Container -->
    <main class="log-viewer" bind:this={logContainer}>
        {#if filteredLogs.length === 0}
            <div class="empty-state">
                <p>No log entries found.</p>
            </div>
        {:else}
            {#each filteredLogs as log (log.id)}
                <div class="log-line {log.level.toLowerCase()}">
                    <span class="time">{formatTime(log.timestamp)}</span>
                    <span class="level-badge {log.level.toLowerCase()}">{log.level}</span>
                    <span class="cat-badge {getCategoryClass(log.category)}">[{log.category}]</span>
                    <span class="msg">{log.message}</span>
                </div>
            {/each}
        {/if}
    </main>
</div>

<style>
    .copy-btn {
        display: inline-flex;
        align-items: center;
        gap: 0.45rem;
    }

    .copy-btn.copy-success {
        animation: copySuccessPop 0.28s ease-out;
        background: rgba(16, 185, 129, 0.16);
        color: #34d399;
        border-color: rgba(16, 185, 129, 0.35);
    }

    .copy-btn.copy-success:hover {
        background: rgba(16, 185, 129, 0.2);
        border-color: rgba(16, 185, 129, 0.45);
    }

    @keyframes copySuccessPop {
        0% { transform: scale(1); }
        55% { transform: scale(1.05); }
        100% { transform: scale(1); }
    }

    .debug-page {
        display: flex;
        flex-direction: column;
        height: 100vh;
        background: var(--echo-void, #050507);
        color: var(--echo-text-1, #eae8e3);
        font-family: var(--lyria-font-body, system-ui, sans-serif);
        user-select: text;
    }

    .debug-header {
        display: flex;
        flex-direction: column;
        gap: 12px;
        padding: 14px 20px;
        background: rgba(16, 16, 20, 0.85);
        backdrop-filter: blur(16px);
        border-bottom: 1px solid var(--echo-border-medium, rgba(255, 255, 255, 0.08));
    }

    .header-title-row {
        display: flex;
        align-items: center;
        justify-content: space-between;
    }

    .title-group {
        display: flex;
        align-items: baseline;
        gap: 12px;
    }

    .title {
        margin: 0;
        font-family: var(--lyria-font-heading, "Newsreader", serif);
        font-size: 1.35rem;
        font-weight: 500;
        letter-spacing: -0.01em;
        color: var(--echo-text-1, #eae8e3);
    }

    .log-count {
        font-family: var(--lyria-font-mono, monospace);
        font-size: 11px;
        color: var(--echo-text-2, #7a7885);
        letter-spacing: 0.02em;
    }

    .action-buttons {
        display: flex;
        align-items: center;
        gap: 8px;
    }

    .btn {
        display: inline-flex;
        align-items: center;
        gap: 6px;
        padding: 5px 12px;
        font-family: var(--lyria-font-body, system-ui, sans-serif);
        font-size: 12px;
        font-weight: 500;
        border-radius: 6px;
        border: 1px solid var(--echo-border-medium, rgba(255, 255, 255, 0.1));
        cursor: pointer;
        transition: all 0.15s ease;
    }

    .btn-secondary {
        background: rgba(255, 255, 255, 0.04);
        color: var(--echo-text-1, #cbd5e1);
        border-color: var(--echo-border-strong, rgba(255, 255, 255, 0.12));
    }
    .btn-secondary:hover {
        background: rgba(255, 255, 255, 0.09);
        border-color: rgba(255, 255, 255, 0.22);
        color: #fff;
    }
    .btn-secondary.active {
        background: var(--echo-primary, #e2a973);
        color: #050507;
        border-color: var(--echo-primary, #e2a973);
        font-weight: 600;
    }

    .btn-primary {
        background: var(--echo-primary, #e2a973);
        color: #050507;
        border-color: var(--echo-primary, #e2a973);
        font-weight: 600;
    }
    .btn-primary:hover {
        background: var(--echo-primary-dark, #b58e62);
        border-color: var(--echo-primary-dark, #b58e62);
    }

    .filters-row {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 12px;
        flex-wrap: wrap;
    }

    .category-pills {
        display: flex;
        align-items: center;
        gap: 6px;
        flex-wrap: wrap;
    }

    .pill {
        padding: 3px 10px;
        font-family: var(--lyria-font-body, system-ui, sans-serif);
        font-size: 11px;
        font-weight: 500;
        border-radius: 9999px;
        background: rgba(255, 255, 255, 0.04);
        border: 1px solid var(--echo-border-medium, rgba(255, 255, 255, 0.08));
        color: var(--echo-text-2, #7a7885);
        cursor: pointer;
        transition: all 0.15s ease;
    }
    .pill:hover {
        background: rgba(255, 255, 255, 0.08);
        color: var(--echo-text-1, #eae8e3);
    }
    .pill.active {
        background: rgba(226, 169, 115, 0.15);
        border-color: var(--echo-primary, #e2a973);
        color: var(--echo-primary, #e2a973);
        font-weight: 600;
    }

    .level-checks {
        display: flex;
        align-items: center;
        gap: 4px;
    }

    .level-btn {
        padding: 3px 8px;
        font-family: var(--lyria-font-mono, monospace);
        font-size: 10px;
        font-weight: 600;
        border-radius: 4px;
        border: 1px solid transparent;
        cursor: pointer;
        background: transparent;
        opacity: 0.35;
        transition: all 0.15s ease;
        text-transform: uppercase;
        letter-spacing: 0.02em;
    }
    .level-btn.active {
        opacity: 1;
    }
    .level-btn.active.info { color: #38bdf8; border-color: rgba(56, 189, 248, 0.3); background: rgba(56, 189, 248, 0.1); }
    .level-btn.active.warn { color: #fbbf24; border-color: rgba(251, 191, 36, 0.3); background: rgba(251, 191, 36, 0.1); }
    .level-btn.active.error { color: #f87171; border-color: rgba(248, 113, 113, 0.3); background: rgba(248, 113, 113, 0.1); }
    .level-btn.active.debug { color: #c084fc; border-color: rgba(192, 132, 252, 0.3); background: rgba(192, 132, 252, 0.1); }

    .search-box {
        position: relative;
        flex: 1;
        min-width: 140px;
        display: flex;
        align-items: center;
    }

    :global(.search-box .search-icon) {
        position: absolute;
        left: 8px;
        color: var(--echo-text-2, #7a7885);
        pointer-events: none;
    }

    .search-input {
        width: 100%;
        padding: 5px 10px 5px 28px;
        font-family: var(--lyria-font-body, system-ui, sans-serif);
        font-size: 12px;
        background: rgba(0, 0, 0, 0.35);
        border: 1px solid var(--echo-border-medium, rgba(255, 255, 255, 0.08));
        border-radius: 6px;
        color: var(--echo-text-1, #eae8e3);
        transition: border-color 0.15s ease;
    }
    .search-input:focus {
        outline: none;
        border-color: var(--echo-primary, #e2a973);
    }

    .auto-scroll-toggle {
        display: inline-flex;
        align-items: center;
        gap: 8px;
        font-family: var(--lyria-font-body, system-ui, sans-serif);
        font-size: 12px;
        color: var(--echo-text-2, #7a7885);
        cursor: pointer;
        user-select: none;
        transition: color 0.15s ease;
    }

    .auto-scroll-toggle:hover {
        color: var(--echo-text-1, #eae8e3);
    }

    .auto-scroll-toggle .custom-checkbox {
        position: relative;
        display: inline-flex;
        align-items: center;
        justify-content: center;
        width: 16px;
        height: 16px;
        flex-shrink: 0;
        background-color: rgba(255, 255, 255, 0.05);
        border: 1px solid var(--echo-border-strong, rgba(255, 255, 255, 0.15));
        border-radius: 4px;
        color: var(--echo-void, #050507);
        transition: background-color 0.15s ease, border-color 0.15s ease, box-shadow 0.15s ease;
    }

    .auto-scroll-toggle:hover .custom-checkbox {
        border-color: rgba(255, 255, 255, 0.3);
        background-color: rgba(255, 255, 255, 0.08);
    }

    .auto-scroll-toggle .custom-checkbox.checked {
        background-color: var(--echo-primary, #e2a973);
        border-color: var(--echo-primary, #e2a973);
    }

    .auto-scroll-toggle input {
        position: absolute;
        opacity: 0;
        width: 0;
        height: 0;
        margin: 0;
        pointer-events: none;
    }

    .auto-scroll-toggle:has(input:focus-visible) .custom-checkbox {
        outline: 2px solid var(--echo-primary, #e2a973);
        outline-offset: 2px;
    }

    .log-viewer {
        flex: 1;
        overflow-y: auto;
        padding: 10px 18px;
        font-family: var(--lyria-font-mono, "IBM Plex Mono", monospace);
        font-size: 12px;
        line-height: 1.6;
        background: var(--echo-void, #050507);
    }

    .empty-state {
        display: flex;
        align-items: center;
        justify-content: center;
        height: 100%;
        font-family: var(--lyria-font-body, system-ui, sans-serif);
        color: var(--echo-text-2, #64748b);
        font-size: 14px;
    }

    .log-line {
        display: flex;
        align-items: flex-start;
        gap: 8px;
        padding: 3px 8px;
        border-bottom: 1px solid rgba(255, 255, 255, 0.03);
        border-left: 2px solid transparent;
        word-break: break-all;
        transition: background 0.15s ease;
    }

    .log-line:hover {
        background: rgba(255, 255, 255, 0.02);
    }

    .log-line.error {
        background: rgba(239, 68, 68, 0.08);
        border-left-color: #ef4444;
    }

    .log-line.error:hover {
        background: rgba(239, 68, 68, 0.14);
    }

    .log-line.warn {
        background: rgba(245, 158, 11, 0.05);
        border-left-color: #f59e0b;
    }

    .log-line.warn:hover {
        background: rgba(245, 158, 11, 0.10);
    }

    .time {
        color: var(--echo-text-2, #64748b);
        font-size: 11px;
        white-space: nowrap;
        font-variant-numeric: tabular-nums;
        line-height: 1.6;
    }

    .level-badge {
        font-weight: 700;
        font-size: 10px;
        white-space: nowrap;
        padding: 1px 5px;
        border-radius: 3px;
        text-transform: uppercase;
        letter-spacing: 0.03em;
        line-height: 1.4;
    }
    .level-badge.info { color: #38bdf8; background: rgba(56, 189, 248, 0.12); border: 1px solid rgba(56, 189, 248, 0.25); }
    .level-badge.warn { color: #fbbf24; background: rgba(251, 191, 36, 0.14); border: 1px solid rgba(251, 191, 36, 0.3); }
    .level-badge.error { color: #f87171; background: rgba(239, 68, 68, 0.18); border: 1px solid rgba(239, 68, 68, 0.35); }
    .level-badge.debug { color: #c084fc; background: rgba(192, 132, 252, 0.12); border: 1px solid rgba(192, 132, 252, 0.25); }

    .cat-badge {
        font-weight: 600;
        font-size: 11px;
        white-space: nowrap;
        padding: 0 4px;
        border-radius: 2px;
        line-height: 1.5;
    }
    .cat-badge.frontend { color: #34d399; background: rgba(52, 211, 153, 0.1); }
    .cat-badge.backend  { color: #38bdf8; background: rgba(56, 189, 248, 0.1); }
    .cat-badge.wasm     { color: #fb923c; background: rgba(251, 146, 60, 0.1); }
    .cat-badge.sandbox  { color: #c084fc; background: rgba(192, 132, 252, 0.1); }
    .cat-badge.audio    { color: #f472b6; background: rgba(244, 114, 182, 0.1); }
    .cat-badge.database { color: #818cf8; background: rgba(129, 140, 248, 0.1); }
    .cat-badge.network  { color: #2dd4bf; background: rgba(45, 212, 191, 0.1); }
    .cat-badge.system   { color: #94a3b8; background: rgba(148, 163, 184, 0.1); }

    .msg {
        color: var(--echo-text-1, #e2e8f0);
        line-height: 1.5;
        flex: 1;
    }

    .log-line.error .msg {
        color: #fca5a5;
        font-weight: 500;
    }
    .log-line.warn .msg {
        color: #fde68a;
    }
</style>