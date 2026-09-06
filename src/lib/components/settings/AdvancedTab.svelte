<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { settingsStore } from "$lib/stores/settings.svelte";
    import { flagsStore } from "$lib/stores/flags.svelte";
    import { toastStore } from "$lib/stores/toast.svelte";
    import { TerminalWindow, Copy, FolderOpen, Check, Flask, ArrowsClockwise } from "phosphor-svelte";

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

    async function handleFlagToggle(key: string, currentVal: boolean) {
        await flagsStore.toggle(key, !currentVal);
        toastStore.success(`Feature flag '${key}' updated.`);
    }

    async function handleResetFlags() {
        await flagsStore.resetToDefaults();
        toastStore.success("All feature flags reset to defaults.");
    }

    let flagGroups = $derived.by(() => {
        const homePage = flagsStore.details.find(f => f.key === "page_home");
        const homeSections = flagsStore.details.filter(f => f.key.startsWith("home_"));

        const explorePage = flagsStore.details.find(f => f.key === "page_explore");
        const exploreSections = flagsStore.details.filter(f => f.key.startsWith("explore_"));

        const otherFlags = flagsStore.details.filter(
            f => f.key !== "page_home" && 
                 f.key !== "page_explore" && 
                 !f.key.startsWith("home_") && 
                 !f.key.startsWith("explore_")
        );

        return {
            groups: [
                {
                    id: "home",
                    title: "Home Page",
                    description: "Main algorithmic cockpit, dynamic discovery carousels, and listening history shelves.",
                    pageFlag: homePage,
                    sections: homeSections,
                },
                {
                    id: "explore",
                    title: "Explore Page",
                    description: "Global music catalog, editorial spotlights, top charts, and genre category hub.",
                    pageFlag: explorePage,
                    sections: exploreSections,
                },
            ],
            others: otherFlags,
        };
    });
</script>

<section class="settings-section">
    <h3 class="section-title">Developer & Debugging</h3>

    <div class="setting-row">
        <div class="setting-info">
            <p class="setting-label">Enable Diagnostic Log Collection</p>
            <p class="setting-desc">
                Record WASM plugin calls, BotGuard JS VM signals, network statuses, and audio engine events to local memory and log files. 
                <strong style="color: var(--echo-text-1);">Lyria never transmits remote telemetry — all diagnostic logs stay 100% on your device.</strong>
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

    <!-- Feature Flags & Experiments Section -->
    <div class="experiments-header">
        <div class="title-with-icon">
            <Flask size={20} weight="duotone" color="#d4a86e" />
            <h3 class="section-title no-border">Runtime Feature Flags & Experiments</h3>
        </div>
        <button class="reset-btn" onclick={handleResetFlags} title="Reset all flags to defaults">
            <ArrowsClockwise size={14} weight="bold" />
            <span>Reset Defaults</span>
        </button>
    </div>

    <p class="experiments-desc">
        Configure runtime capabilities and bleeding-edge audio / UI experiments. Flags can also be overridden via environment variables (e.g. <code>ECHO_FF_EXPERIMENTAL_DSP=1</code>).
    </p>

    <div class="flag-groups-container">
        {#each flagGroups.groups as group}
            <div class="page-flag-group" class:is-disabled={group.pageFlag && !group.pageFlag.enabled}>
                <!-- Master Page Toggle Header -->
                {#if group.pageFlag}
                    <div class="master-flag-card">
                        <div class="flag-info">
                            <div class="flag-header-line">
                                <span class="master-page-name">{group.title}</span>
                                <span class="group-badge">Page Master</span>
                                <span class="stage-badge {group.pageFlag.stage}">{group.pageFlag.stage}</span>
                                {#if group.pageFlag.is_overridden_by_env}
                                    <span class="env-badge" title="Overridden by system environment variable">ENV Override</span>
                                {/if}
                            </div>
                            <p class="master-flag-desc">{group.pageFlag.description || group.description}</p>
                            <span class="flag-meta">Key: <code>{group.pageFlag.key}</code></span>
                        </div>

                        <label class="switch master-switch">
                            <input
                                type="checkbox"
                                checked={group.pageFlag.enabled}
                                onchange={() => handleFlagToggle(group.pageFlag!.key, group.pageFlag!.enabled)}
                                disabled={group.pageFlag.is_overridden_by_env}
                            />
                            <span class="slider round"></span>
                        </label>
                    </div>
                {/if}

                <!-- Sub-sections List -->
                {#if group.sections.length > 0}
                    <div class="sub-sections-container">
                        <div class="sub-sections-header">
                            <span class="sub-sections-title">Individual Sections ({group.sections.length})</span>
                            {#if group.pageFlag && !group.pageFlag.enabled}
                                <span class="sub-disabled-notice">Inactive while {group.title} is turned off</span>
                            {/if}
                        </div>

                        <div class="sub-sections-list">
                            {#each group.sections as section}
                                <div class="sub-flag-card">
                                    <div class="flag-info">
                                        <div class="flag-header-line">
                                            <span class="sub-flag-name">{section.name}</span>
                                            {#if section.is_overridden_by_env}
                                                <span class="env-badge" title="Overridden by system environment variable">ENV Override</span>
                                            {/if}
                                        </div>
                                        <p class="sub-flag-desc">{section.description}</p>
                                        <span class="flag-meta">Key: <code>{section.key}</code></span>
                                    </div>

                                    <label class="switch sub-switch">
                                        <input
                                            type="checkbox"
                                            checked={section.enabled}
                                            onchange={() => handleFlagToggle(section.key, section.enabled)}
                                            disabled={section.is_overridden_by_env || (group.pageFlag && !group.pageFlag.enabled)}
                                        />
                                        <span class="slider round"></span>
                                    </label>
                                </div>
                            {/each}
                        </div>
                    </div>
                {/if}
            </div>
        {/each}

        <!-- Other System & Engine Flags if any -->
        {#if flagGroups.others.length > 0}
            <div class="page-flag-group other-flags-group">
                <div class="sub-sections-header">
                    <span class="sub-sections-title">Other Capabilities & Experiments</span>
                </div>
                <div class="sub-sections-list no-indent">
                    {#each flagGroups.others as flag}
                        <div class="flag-card">
                            <div class="flag-info">
                                <div class="flag-header-line">
                                    <span class="flag-name">{flag.name}</span>
                                    <span class="stage-badge {flag.stage}">{flag.stage}</span>
                                    {#if flag.is_overridden_by_env}
                                        <span class="env-badge">ENV Override</span>
                                    {/if}
                                </div>
                                <p class="flag-desc">{flag.description}</p>
                                <span class="flag-meta">Key: <code>{flag.key}</code> • Category: {flag.category}</span>
                            </div>

                            <label class="switch">
                                <input
                                    type="checkbox"
                                    checked={flag.enabled}
                                    onchange={() => handleFlagToggle(flag.key, flag.enabled)}
                                    disabled={flag.is_overridden_by_env}
                                />
                                <span class="slider round"></span>
                            </label>
                        </div>
                    {/each}
                </div>
            </div>
        {/if}
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
        font-family: var(--lyria-font-body);
        font-size: 1.25rem;
        font-weight: 500;
        color: var(--echo-text-1);
        padding-bottom: 0.5rem;
        border-bottom: 1px solid var(--echo-border);
        margin-bottom: 0.5rem;
    }
    .section-title.no-border {
        border-bottom: none;
        padding-bottom: 0;
        margin-bottom: 0;
    }
    .experiments-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-top: 1.5rem;
        padding-bottom: 0.5rem;
        border-bottom: 1px solid var(--echo-border);
    }
    .title-with-icon {
        display: flex;
        align-items: center;
        gap: 0.6rem;
    }
    .experiments-desc {
        font-size: 0.85rem;
        color: var(--echo-text-2);
        line-height: 1.4;
        margin: -0.5rem 0 0.5rem 0;
    }
    .experiments-desc code {
        font-family: var(--lyria-font-mono);
        font-size: 0.78rem;
        background: rgba(255, 255, 255, 0.08);
        padding: 0.15rem 0.35rem;
        border-radius: 4px;
        color: #d4a86e;
    }
    .reset-btn {
        display: flex;
        align-items: center;
        gap: 0.4rem;
        font-size: 0.78rem;
        font-weight: 500;
        color: var(--echo-text-2);
        background: rgba(255, 255, 255, 0.05);
        border: 1px solid rgba(255, 255, 255, 0.1);
        padding: 0.35rem 0.7rem;
        border-radius: 6px;
        cursor: pointer;
        transition: all 0.2s ease;
    }
    .reset-btn:hover {
        color: var(--echo-text-1);
        background: rgba(255, 255, 255, 0.1);
        border-color: rgba(255, 255, 255, 0.2);
    }
    
    .flag-groups-container {
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
    }

    .page-flag-group {
        display: flex;
        flex-direction: column;
        background: rgba(255, 255, 255, 0.02);
        border: 1px solid rgba(255, 255, 255, 0.07);
        border-radius: 12px;
        overflow: hidden;
        transition: all 0.25s ease;
    }

    .page-flag-group:hover {
        border-color: rgba(255, 255, 255, 0.12);
    }

    .page-flag-group.is-disabled {
        border-color: rgba(255, 255, 255, 0.04);
        background: rgba(255, 255, 255, 0.01);
    }

    .master-flag-card {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 1.25rem 1.5rem;
        background: rgba(255, 255, 255, 0.03);
        border-bottom: 1px solid rgba(255, 255, 255, 0.06);
        gap: 1.5rem;
    }

    .master-page-name {
        font-family: var(--lyria-font-body);
        font-size: 1.15rem;
        font-weight: 600;
        color: var(--echo-text-1);
    }

    .group-badge {
        font-size: 0.65rem;
        font-weight: 700;
        text-transform: uppercase;
        letter-spacing: 0.05em;
        padding: 0.15rem 0.5rem;
        border-radius: 4px;
        background: rgba(212, 168, 110, 0.15);
        color: #d4a86e;
        border: 1px solid rgba(212, 168, 110, 0.3);
    }

    .master-flag-desc {
        font-size: 0.85rem;
        color: var(--echo-text-2);
        margin: 0.15rem 0 0.25rem 0;
        line-height: 1.4;
    }

    .sub-sections-container {
        display: flex;
        flex-direction: column;
        padding: 1.25rem 1.5rem;
        gap: 0.85rem;
        transition: opacity 0.25s ease;
    }

    .is-disabled .sub-sections-container {
        opacity: 0.4;
    }

    .sub-sections-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 0.25rem;
    }

    .sub-sections-title {
        font-size: 0.78rem;
        font-weight: 600;
        text-transform: uppercase;
        letter-spacing: 0.06em;
        color: var(--echo-text-3);
    }

    .sub-disabled-notice {
        font-size: 0.75rem;
        font-style: italic;
        color: #f87171;
    }

    .sub-sections-list {
        display: flex;
        flex-direction: column;
        gap: 0.6rem;
        padding-left: 1rem;
        border-left: 2px solid rgba(255, 255, 255, 0.06);
    }

    .sub-sections-list.no-indent {
        padding-left: 0;
        border-left: none;
    }

    .sub-flag-card {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 0.75rem 1rem;
        background: rgba(255, 255, 255, 0.02);
        border: 1px solid rgba(255, 255, 255, 0.04);
        border-radius: 8px;
        gap: 1.25rem;
        transition: all 0.2s ease;
    }

    .sub-flag-card:hover {
        background: rgba(255, 255, 255, 0.035);
        border-color: rgba(255, 255, 255, 0.08);
    }

    .sub-flag-name {
        font-size: 0.88rem;
        font-weight: 500;
        color: var(--echo-text-1);
    }

    .sub-flag-desc {
        font-size: 0.78rem;
        color: var(--echo-text-2);
        margin: 0.1rem 0;
        line-height: 1.35;
    }

    .other-flags-group {
        padding: 1.25rem 1.5rem;
    }

    .flags-list {
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
    }
    .flag-card {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 1rem 1.25rem;
        background: rgba(255, 255, 255, 0.025);
        border: 1px solid rgba(255, 255, 255, 0.06);
        border-radius: 8px;
        gap: 1.5rem;
        transition: border-color 0.2s ease, background 0.2s ease;
    }
    .flag-card:hover {
        background: rgba(255, 255, 255, 0.04);
        border-color: rgba(255, 255, 255, 0.12);
    }
    .flag-info {
        display: flex;
        flex-direction: column;
        gap: 0.35rem;
        flex: 1;
        min-width: 0;
    }
    .flag-header-line {
        display: flex;
        align-items: center;
        gap: 0.6rem;
        flex-wrap: wrap;
    }
    .flag-name {
        font-size: 0.95rem;
        font-weight: 600;
        color: var(--echo-text-1);
    }
    .stage-badge {
        font-size: 0.68rem;
        font-weight: 600;
        text-transform: uppercase;
        letter-spacing: 0.04em;
        padding: 0.15rem 0.45rem;
        border-radius: 4px;
    }
    .stage-badge.stable {
        background: rgba(52, 211, 153, 0.15);
        color: #34d399;
        border: 1px solid rgba(52, 211, 153, 0.3);
    }
    .stage-badge.beta {
        background: rgba(56, 189, 248, 0.15);
        color: #38bdf8;
        border: 1px solid rgba(56, 189, 248, 0.3);
    }
    .stage-badge.experimental {
        background: rgba(251, 191, 36, 0.15);
        color: #fbbf24;
        border: 1px solid rgba(251, 191, 36, 0.3);
    }
    .env-badge {
        font-size: 0.68rem;
        font-weight: 600;
        background: rgba(168, 85, 247, 0.15);
        color: #c084fc;
        border: 1px solid rgba(168, 85, 247, 0.3);
        padding: 0.15rem 0.45rem;
        border-radius: 4px;
    }
    .flag-desc {
        font-size: 0.82rem;
        color: var(--echo-text-2);
        line-height: 1.35;
        margin: 0;
    }
    .flag-meta {
        font-size: 0.72rem;
        color: rgba(255, 255, 255, 0.35);
        font-family: var(--echo-font-body);
    }
    .flag-meta code {
        font-family: var(--lyria-font-mono);
        color: rgba(255, 255, 255, 0.5);
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
        font-size: 0.82rem;
        color: var(--echo-text-2);
        line-height: 1.4;
        margin: 0;
    }
    .action-btn {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        padding: 0.6rem 1rem;
        background: transparent;
        border: 1px solid var(--echo-border-medium);
        color: var(--echo-text-1);
        border-radius: 6px;
        font-size: 0.85rem;
        font-weight: 500;
        cursor: pointer;
        transition: all 0.2s ease;
        white-space: nowrap;
    }
    .action-btn:hover:not(:disabled) {
        background: rgba(255, 255, 255, 0.05);
        border-color: var(--echo-text-2);
    }
    .action-btn.primary {
        background: var(--echo-primary);
        border-color: var(--echo-primary);
        color: #000;
    }
    .action-btn.primary:hover:not(:disabled) {
        background: #e2ba84;
        border-color: #e2ba84;
    }
    .action-btn.copy-action {
        min-width: 110px;
        justify-content: center;
    }
    .action-btn.copy-success {
        background: rgba(74, 222, 128, 0.15);
        border-color: #4ade80;
        color: #4ade80;
    }
    .action-btn:disabled {
        opacity: 0.4;
        cursor: not-allowed;
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
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        background-color: rgba(255, 255, 255, 0.1);
        transition: 0.2s cubic-bezier(0.4, 0, 0.2, 1);
        border: 1px solid var(--echo-border-medium);
    }
    .slider:before {
        position: absolute;
        content: "";
        height: 16px;
        width: 16px;
        left: 3px;
        bottom: 3px;
        background-color: var(--echo-text-2);
        transition: 0.2s cubic-bezier(0.4, 0, 0.2, 1);
    }
    input:checked + .slider {
        background-color: var(--echo-primary);
        border-color: var(--echo-primary);
    }
    input:checked + .slider:before {
        transform: translateX(20px);
        background-color: #000;
    }
    .slider.round {
        border-radius: 24px;
    }
    .slider.round:before {
        border-radius: 50%;
    }
    @keyframes fadeIn {
        from { opacity: 0; transform: translateY(4px); }
        to { opacity: 1; transform: translateY(0); }
    }
</style>
