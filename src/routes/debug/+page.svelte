<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { listen, type UnlistenFn } from "@tauri-apps/api/event";
    import { toastStore } from "$lib/stores/toast.svelte";
    import { Copy, Check } from "phosphor-svelte";
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

    const categories = ["All", "WASM", "JS Sandbox", "Audio", "Network", "System"];

    const filteredLogs = $derived(
        logs.filter((log) => {
            // Category Filter
            if (selectedCategory !== "All") {
                if (
                    selectedCategory === "WASM" &&
                    !log.category.includes("WASM") &&
                    !log.message.includes("[PLUGIN LOG]")
                )
                    return false;
                if (selectedCategory === "JS Sandbox" && !log.category.includes("JS Sandbox"))
                    return false;
                if (selectedCategory === "Audio" && !log.category.includes("Audio"))
                    return false;
                if (selectedCategory === "Network" && !log.category.includes("Network"))
                    return false;
                if (selectedCategory === "System" && !log.category.includes("System"))
                    return false;
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
</script>

<div class="debug-page">
    <!-- Top Controls Header -->
    <header class="debug-header">
        <div class="header-title-row">
            <div class="title-group">
                <span class="pulse-dot" class:paused={isPaused}></span>
                <h1 class="title">Lyria Debug Console</h1>
                <span class="log-count">{filteredLogs.length} / {logs.length} logs</span>
            </div>

            <div class="action-buttons">
                <button
                    class="btn btn-secondary"
                    class:active={isPaused}
                    onclick={() => (isPaused = !isPaused)}
                >
                    {isPaused ? "▶ Resume" : "⏸ Pause"}
                </button>
                <button class="btn btn-secondary" onclick={handleClear}>Clear</button>
                <button class="btn btn-secondary copy-btn" class:copy-success={isCopySuccess} onclick={handleCopy}>
                    {#if isCopySuccess}
                        <Check size={16} weight="bold" />
                        <span>Copied</span>
                    {:else}
                        <Copy size={16} weight="regular" />
                        <span>Copy Logs</span>
                    {/if}
                </button>
                <button class="btn btn-primary" onclick={handleOpenLogFolder}>Log Folder 📁</button>
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
                <input
                    type="text"
                    placeholder="Search logs..."
                    bind:value={searchFilter}
                    class="search-input"
                />
            </div>

            <!-- Auto Scroll Switch -->
            <label class="auto-scroll-toggle">
                <input type="checkbox" bind:checked={autoScroll} />
                <span>Auto-Scroll</span>
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
                    <span class="level-badge {log.level.toLowerCase()}">[{log.level}]</span>
                    <span class="cat-badge">[{log.category}]</span>
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
        gap: 0.5rem;
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
        0% {
            transform: scale(1);
        }
        55% {
            transform: scale(1.05);
        }
        100% {
            transform: scale(1);
        }
    }


    .debug-page {
        display: flex;
        flex-direction: column;
        height: 100vh;
        background: #090a0f;
        color: #e2e8f0;
        font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
        user-select: text;
    }

    .debug-header {
        display: flex;
        flex-direction: column;
        gap: 10px;
        padding: 14px 18px;
        background: rgba(18, 20, 29, 0.85);
        backdrop-filter: blur(12px);
        border-bottom: 1px solid rgba(255, 255, 255, 0.08);
    }

    .header-title-row {
        display: flex;
        align-items: center;
        justify-content: space-between;
    }

    .title-group {
        display: flex;
        align-items: center;
        gap: 10px;
    }

    .pulse-dot {
        width: 8px;
        height: 8px;
        border-radius: 50%;
        background: #10b981;
        box-shadow: 0 0 8px #10b981;
    }

    .pulse-dot.paused {
        background: #f59e0b;
        box-shadow: 0 0 8px #f59e0b;
    }

    .title {
        margin: 0;
        font-size: 16px;
        font-weight: 600;
        letter-spacing: -0.01em;
    }

    .log-count {
        font-size: 12px;
        color: #64748b;
    }

    .action-buttons {
        display: flex;
        align-items: center;
        gap: 8px;
    }

    .btn {
        padding: 5px 12px;
        font-size: 12px;
        font-weight: 500;
        border-radius: 6px;
        border: 1px solid rgba(255, 255, 255, 0.1);
        cursor: pointer;
        transition: all 0.15s ease;
    }

    .btn-secondary {
        background: rgba(255, 255, 255, 0.05);
        color: #cbd5e1;
    }
    .btn-secondary:hover {
        background: rgba(255, 255, 255, 0.12);
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
    }

    .pill {
        padding: 4px 10px;
        font-size: 11px;
        font-weight: 500;
        border-radius: 20px;
        background: rgba(255, 255, 255, 0.04);
        color: #94a3b8;
        border: 1px solid transparent;
        cursor: pointer;
    }
    .pill:hover {
        color: #f1f5f9;
        background: rgba(255, 255, 255, 0.08);
    }
    .pill.active {
        background: rgba(226, 169, 115, 0.18);
        color: #e2a973;
        border-color: rgba(226, 169, 115, 0.3);
    }

    .level-checks {
        display: flex;
        gap: 4px;
    }

    .level-btn {
        padding: 3px 8px;
        font-size: 10px;
        font-weight: 600;
        border-radius: 4px;
        border: 1px solid rgba(255, 255, 255, 0.08);
        background: transparent;
        color: #475569;
        cursor: pointer;
    }
    .level-btn.active.info { color: #38bdf8; border-color: rgba(56, 189, 248, 0.3); background: rgba(56, 189, 248, 0.1); }
    .level-btn.active.warn { color: #fbbf24; border-color: rgba(251, 191, 36, 0.3); background: rgba(251, 191, 36, 0.1); }
    .level-btn.active.error { color: #f87171; border-color: rgba(248, 113, 113, 0.3); background: rgba(248, 113, 113, 0.1); }
    .level-btn.active.debug { color: #c084fc; border-color: rgba(192, 132, 252, 0.3); background: rgba(192, 132, 252, 0.1); }

    .search-box {
        flex: 1;
        min-width: 140px;
    }

    .search-input {
        width: 100%;
        padding: 4px 10px;
        font-size: 12px;
        background: rgba(0, 0, 0, 0.3);
        border: 1px solid rgba(255, 255, 255, 0.08);
        border-radius: 6px;
        color: #f1f5f9;
    }
    .search-input:focus {
        outline: none;
        border-color: var(--echo-primary, #e2a973);
    }

    .auto-scroll-toggle {
        display: flex;
        align-items: center;
        gap: 6px;
        font-size: 12px;
        color: #94a3b8;
        cursor: pointer;
    }

    .log-viewer {
        flex: 1;
        overflow-y: auto;
        padding: 12px 18px;
        font-family: "Cascadia Code", "Fira Code", Consolas, Monaco, monospace;
        font-size: 12px;
        line-height: 1.6;
        background: #090a0f;
    }

    .empty-state {
        display: flex;
        justify-content: center;
        align-items: center;
        height: 100%;
        color: #475569;
    }

    .log-line {
        display: flex;
        align-items: flex-start;
        gap: 8px;
        padding: 2px 0;
        border-bottom: 1px solid rgba(255, 255, 255, 0.02);
        word-break: break-all;
    }

    .time {
        color: #475569;
        font-size: 11px;
        white-space: nowrap;
    }

    .level-badge {
        font-weight: 600;
        font-size: 11px;
        white-space: nowrap;
    }
    .level-badge.info { color: #38bdf8; }
    .level-badge.warn { color: #fbbf24; }
    .level-badge.error { color: #f87171; }
    .level-badge.debug { color: #c084fc; }

    .cat-badge {
        color: #818cf8;
        white-space: nowrap;
    }

    .msg {
        color: #e2e8f0;
    }

    .log-line.error .msg {
        color: #fca5a5;
    }
    .log-line.warn .msg {
        color: #fde68a;
    }
</style>
