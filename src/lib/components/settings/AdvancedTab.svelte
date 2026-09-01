<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { settingsStore } from "$lib/stores/settings.svelte";
    import { toastStore } from "$lib/stores/toast.svelte";
    import { TerminalWindow, Copy, FolderOpen, Check } from "phosphor-svelte";

    let isCopyLogsSuccess = $state(false);
    let copyLogsSuccessTimer: ReturnType<typeof setTimeout> | null = null;

    async function openDebugWindow() {
        try {
            await invoke("open_debug_window");
        } catch (e) {
            console.error("Failed to open debug window:", e);
        }
    }

    async function copyDebugLogs() {
        try {
            const logs = await invoke<string>("copy_debug_log_to_clipboard");
            await navigator.clipboard.writeText(logs);
            if (copyLogsSuccessTimer) clearTimeout(copyLogsSuccessTimer);
            isCopyLogsSuccess = true;
            copyLogsSuccessTimer = setTimeout(() => {
                isCopyLogsSuccess = false;
                copyLogsSuccessTimer = null;
            }, 1200);
            toastStore.success("Debug logs copied to clipboard.");
        } catch (e) {
            console.error("Failed to copy debug logs:", e);
            toastStore.error("Failed to copy debug logs.");
        }
    }

    async function openLogFolder() {
        try {
            await invoke("open_log_directory");
        } catch (e) {
            console.error("Failed to open log folder:", e);
        }
    }

    async function toggleLogCollection() {
        await settingsStore.setLogCollectionEnabled(!settingsStore.logCollectionEnabled);
    }
</script>

<section class="settings-section">
    <h3 class="section-title">Developer & Debugging</h3>

    <div class="setting-row">
        <div class="setting-info">
            <p class="setting-label">Enable Diagnostic Log Collection</p>
            <p class="setting-desc">
                Record WASM plugin calls, BotGuard JS VM signals, network statuses, and audio engine events to local memory and log files. 
                <strong style="color: var(--echo-text-1);">Echo never transmits remote telemetry — all diagnostic logs stay 100% on your device.</strong>
            </p>
        </div>
        <label class="switch">
            <input
                type="checkbox"
                checked={settingsStore.logCollectionEnabled}
                onchange={toggleLogCollection}
                disabled={!settingsStore.loaded}
            />
            <span class="slider round"></span>
        </label>
    </div>

    <div class="dependent-group">
        <p class="dependent-group-label">Requires diagnostic log collection</p>

        <div class="setting-row dependent-row">
            <div class="setting-info">
                <p class="setting-label">Real-Time Debug Console</p>
                <p class="setting-desc">
                    Launch a separate window to monitor live WASM plugin calls, BotGuard JS VM executions, network requests, and audio engine events.
                </p>
            </div>
            <button
                class="action-btn primary"
                onclick={openDebugWindow}
                disabled={!settingsStore.loaded || !settingsStore.logCollectionEnabled}
            >
                <TerminalWindow size={16} weight="regular" />
                <span>Open Debug Console</span>
            </button>
        </div>

        <div class="setting-row dependent-row">
            <div class="setting-info">
                <p class="setting-label">Copy Debug Logs</p>
                <p class="setting-desc">
                    Copy recent in-memory log buffer to clipboard for GitHub bug reports.
                </p>
            </div>
            <button
                class="action-btn copy-action"
                class:copy-success={isCopyLogsSuccess}
                onclick={copyDebugLogs}
                disabled={!settingsStore.loaded || !settingsStore.logCollectionEnabled}
            >
                {#if isCopyLogsSuccess}
                    <Check size={16} weight="bold" />
                    <span>Copied</span>
                {:else}
                    <Copy size={16} weight="regular" />
                    <span>Copy Logs</span>
                {/if}
            </button>
        </div>
    </div>

    <div class="setting-row">
        <div class="setting-info">
            <p class="setting-label">Log File Directory</p>
            <p class="setting-desc">
                Open the system file explorer folder containing daily rolling log files.
            </p>
        </div>
        <button class="action-btn" onclick={openLogFolder}>
            <FolderOpen size={16} weight="regular" />
            <span>Open Log Folder</span>
        </button>
    </div>
</section>

<style>
    .settings-section {
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
        animation: fadeIn 0.2s ease;
    }
    .section-title {
        font-family: var(--echo-font-heading);
        font-size: 1.25rem;
        font-weight: 500;
        color: var(--echo-text-1);
        padding-bottom: 0.5rem;
        border-bottom: 1px solid var(--echo-border);
        margin-bottom: 0.5rem;
    }
    .setting-row {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 1rem 0;
        gap: 1.5rem;
    }
    .dependent-group {
        display: flex;
        flex-direction: column;
        border-left: 2px solid var(--echo-border-medium);
        padding-left: 1.25rem;
        margin-left: 0.25rem;
        gap: 0.5rem;
    }
    .dependent-group-label {
        font-size: 0.75rem;
        text-transform: uppercase;
        letter-spacing: 0.05em;
        color: var(--echo-text-2);
        margin-bottom: 0.25rem;
    }
    .dependent-row {
        padding: 0.75rem 0;
    }
    .setting-info {
        display: flex;
        flex-direction: column;
        gap: 0.3rem;
        flex: 1;
    }
    .setting-label {
        font-size: 0.95rem;
        font-weight: 500;
        color: var(--echo-text-1);
    }
    .setting-desc {
        font-size: 0.85rem;
        color: var(--echo-text-2);
        line-height: 1.4;
    }
    .switch {
        position: relative;
        display: inline-block;
        width: 44px;
        height: 24px;
        flex-shrink: 0;
    }
    .switch input {
        opacity: 0;
        width: 0;
        height: 0;
    }
    .slider {
        position: absolute;
        cursor: pointer;
        inset: 0;
        background-color: var(--echo-raised);
        border: 1px solid var(--echo-border-medium);
        transition: 0.2s cubic-bezier(0.16, 1, 0.3, 1);
        border-radius: 24px;
    }
    .slider:before {
        position: absolute;
        content: "";
        height: 16px;
        width: 16px;
        left: 3px;
        bottom: 3px;
        background-color: var(--echo-text-2);
        transition: 0.2s cubic-bezier(0.16, 1, 0.3, 1);
        border-radius: 50%;
    }
    input:checked + .slider {
        background-color: var(--echo-primary);
        border-color: var(--echo-primary);
    }
    input:checked + .slider:before {
        transform: translateX(20px);
        background-color: var(--echo-void);
    }
    .action-btn {
        display: flex;
        align-items: center;
        gap: 0.4rem;
        padding: 0.45rem 0.85rem;
        border-radius: 6px;
        font-family: inherit;
        font-size: 0.82rem;
        font-weight: 500;
        cursor: pointer;
        background: var(--echo-surface);
        border: 1px solid var(--echo-border-medium);
        color: var(--echo-text-2);
        transition: all 0.2s ease;
    }
    .action-btn:hover:not(:disabled) {
        color: var(--echo-text-1);
        border-color: var(--echo-border-strong);
    }
    .action-btn:disabled {
        opacity: 0.4;
        cursor: not-allowed;
    }
    .action-btn.primary {
        background: var(--echo-raised);
        border: 1px solid var(--echo-border-strong);
        color: var(--echo-text-1);
    }
    .action-btn.primary:hover:not(:disabled) {
        border-color: var(--echo-primary);
        color: var(--echo-primary);
    }
    .copy-action.copy-success {
        border-color: #22c55e;
        color: #22c55e;
    }
</style>
