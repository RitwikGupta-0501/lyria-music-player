<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { libraryStore } from "$lib/stores/library.svelte";
    import { settingsStore } from "$lib/stores/settings.svelte";
    import { toastStore } from "$lib/stores/toast.svelte";
    import { CaretDown, Check, TerminalWindow, Copy, FolderOpen, ArrowCounterClockwise, Keyboard } from "phosphor-svelte";
    import { DEFAULT_KEYMAP, formatBinding, eventToBinding, type KeyAction, type KeyBinding } from "$lib/stores/keymap";
    import { onMount } from "svelte";

    let activeTab = $state<"playback" | "appearance" | "shortcuts" | "data" | "advanced">("playback");
    let recordingAction = $state<KeyAction | null>(null);

    function startRecording(action: KeyAction) {
        recordingAction = action;
    }

    function stopRecording() {
        recordingAction = null;
    }

    async function handleKeyRecord(e: KeyboardEvent, action: KeyAction) {
        e.preventDefault();
        e.stopPropagation();

        if (e.key === "Escape") {
            stopRecording();
            return;
        }

        const newBinding = eventToBinding(e);
        if (newBinding) {
            await settingsStore.setKeybinding(action, newBinding);
            toastStore.success(`Updated shortcut for ${DEFAULT_KEYMAP[action]?.label || action}`);
            stopRecording();
        }
    }

    async function handleResetKeymap() {
        await settingsStore.resetKeymap();
        toastStore.success("Restored factory default shortcuts");
    }

    let isDropdownOpen = $state(false);
    let isCopyLogsSuccess = $state(false);
    let copyLogsSuccessTimer: ReturnType<typeof setTimeout> | null = null;

    const behaviorOptions = [
        { value: "interrupt", label: "Play Next & Switch" },
        { value: "clear", label: "Clear Queue & Play" },
        { value: "append", label: "Add to End of Queue" },
    ];

    let isQueueCompletionOpen = $state(false);

    const queueCompletionOptions = [
        { value: "retain_stopped", label: "Reset to Start & Keep Queue" },
        { value: "pause_end", label: "Pause at End of Song" },
        { value: "collapse_idle", label: "Clear Player When Finished" },
    ];

    let isDiversityCeilingOpen = $state(false);

    const diversityCeilingOptions = [
        { value: 1, label: "1 Track (Maximum Diversity / Singletons)" },
        { value: 2, label: "2 Tracks (High Diversity)" },
        { value: 3, label: "3 Tracks (Balanced Catalog - Default)" },
        { value: 4, label: "4 Tracks (Deep Catalog)" },
        { value: 5, label: "5 Tracks (Relaxed Diversity)" },
    ];

    let isProviderDropdownOpen = $state(false);
    let availableProviders = $state<Array<{ value: string; label: string }>>([
        { value: "local", label: "Local Files Only (No Remote Fallback)" }
    ]);

    onMount(async () => {
        await settingsStore.init();
        try {
            const providers = await invoke<any[]>("get_providers");
            if (providers && providers.length > 0) {
                const resolverProviders = providers
                    .filter((p: any) => {
                        if (p.status !== "enabled") return false;
                        const caps: string[] = p.capabilities || [];
                        return caps.some(c => ["resolve", "stream", "url_resolver"].includes(c.toLowerCase()));
                    })
                    .map((p: any) => ({
                        value: p.id,
                        label: `${p.name || p.id} (WASM)`,
                    }));
                
                availableProviders = [
                    { value: "local", label: "Local Files Only (No Remote Fallback)" },
                    ...resolverProviders,
                ];
            }
        } catch (e) {
            console.error("Failed to load providers in settings:", e);
        }
    });

    async function selectProvider(val: string) {
        await settingsStore.setDefaultRemoteProvider(val);
        isProviderDropdownOpen = false;
    }

    async function toggleKeepPlaying() {
        await settingsStore.setKeepPlayingOnQueueClear(!settingsStore.keepPlayingOnQueueClear);
    }

    async function selectBehavior(val: "interrupt" | "clear" | "append") {
        await settingsStore.setTrackClickBehavior(val);
        isDropdownOpen = false;
    }

    async function selectQueueCompletion(val: "retain_stopped" | "pause_end" | "collapse_idle") {
        await settingsStore.setQueueCompletionBehavior(val);
        isQueueCompletionOpen = false;
    }

    async function selectDiversityCeiling(val: number) {
        await settingsStore.setDiscoveryArtistDiversityCeiling(val);
        isDiversityCeilingOpen = false;
    }

    function handleOutsideClick(e: MouseEvent) {
        if (isDropdownOpen) {
            isDropdownOpen = false;
        }
        if (isQueueCompletionOpen) {
            isQueueCompletionOpen = false;
        }
        if (isDiversityCeilingOpen) {
            isDiversityCeilingOpen = false;
        }
        if (isProviderDropdownOpen) {
            isProviderDropdownOpen = false;
        }
    }

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

        async function clearRecommendationCache() {
        try {
            await invoke("clear_recommendations_cache");
            toastStore.success("Recommendation cache cleared & plugins reloaded.");
        } catch (e: any) {
            console.error("Failed to clear recommendation cache:", e);
            toastStore.error(e?.toString() || "Failed to clear cache.");
        }
    }

    async function factoryReset() {
        const yes = confirm(
            "Are you sure you want to completely wipe your library and settings?",
        );
        if (yes) {
            try {
                await invoke("factory_reset");
                libraryStore.albums = [];
                libraryStore.playlists = [];
            } catch (e) {
                console.error("Factory reset failed:", e);
                alert("Failed to reset library.");
            }
        }
    }
</script>

<svelte:window onclick={handleOutsideClick} />

<div class="settings-layout">
    <!-- Left Pane: Navigation -->
    <aside class="settings-nav">
        <h2 class="settings-title">Settings</h2>

        <nav class="nav-list">
            <button
                class="nav-item"
                class:active={activeTab === "playback"}
                onclick={() => (activeTab = "playback")}
            >
                Playback
            </button>
            <button
                class="nav-item"
                class:active={activeTab === "appearance"}
                onclick={() => (activeTab = "appearance")}
            >
                Appearance
            </button>
            <button
                class="nav-item"
                class:active={activeTab === "shortcuts"}
                onclick={() => (activeTab = "shortcuts")}
            >
                Shortcuts & Keybinds
            </button>
            <button
                class="nav-item"
                class:active={activeTab === "data"}
                onclick={() => (activeTab = "data")}
            >
                Data & Privacy
            </button>
            <button
                class="nav-item"
                class:active={activeTab === "advanced"}
                onclick={() => (activeTab = "advanced")}
            >
                Advanced & Debug
            </button>
        </nav>
    </aside>

    <!-- Right Pane: Content -->
    <div
        class="settings-content"
        role="presentation"
        onclick={(e) => e.stopPropagation()}
    >
        <div class="content-container">
            {#if activeTab === "playback"}
                <section class="settings-section">
                    <h3 class="section-title">Audio & Playback</h3>

                    <div class="setting-row">
                        <div class="setting-info">
                            <p class="setting-label">
                                Keep Playing on Queue Clear
                            </p>
                            <p class="setting-desc">
                                Allow the current song to finish even if the
                                upcoming queue is wiped.
                            </p>
                        </div>
                        <label class="switch">
                            <input
                                type="checkbox"
                                checked={settingsStore.keepPlayingOnQueueClear}
                                onchange={toggleKeepPlaying}
                                disabled={!settingsStore.loaded}
                            />
                            <span class="slider round"></span>
                        </label>
                    </div>

                    <div class="setting-row">
                        <div class="setting-info">
                            <p class="setting-label">When clicking a track</p>
                            <p class="setting-desc">
                                Behavior when playing a single track while a
                                queue is active.
                            </p>
                        </div>

                        <div class="custom-select-container">
                            <button
                                class="select-trigger"
                                onclick={(e) => {
                                    e.stopPropagation();
                                    isDropdownOpen = !isDropdownOpen;
                                }}
                                disabled={!settingsStore.loaded}
                            >
                                <span
                                    >{behaviorOptions.find(
                                        (o) => o.value === settingsStore.trackClickBehavior,
                                    )?.label || "Select..."}</span
                                >
                                <CaretDown
                                    size={14}
                                    weight="bold"
                                    class={isDropdownOpen ? "rotated" : ""}
                                />
                            </button>

                            {#if isDropdownOpen}
                                <div class="custom-select-menu glass">
                                    {#each behaviorOptions as opt}
                                        <button
                                            class="select-option"
                                            class:selected={settingsStore.trackClickBehavior ===
                                                opt.value}
                                            onclick={(e) => {
                                                e.stopPropagation();
                                                selectBehavior(
                                                    opt.value as any,
                                                );
                                            }}
                                        >
                                            <span class="opt-label"
                                                >{opt.label}</span
                                            >
                                            {#if settingsStore.trackClickBehavior === opt.value}
                                                <Check
                                                    size={14}
                                                    weight="bold"
                                                    class="check-icon"
                                                />
                                            {/if}
                                        </button>
                                    {/each}
                                </div>
                            {/if}
                        </div>
                    </div>

                    <div class="setting-row">
                        <div class="setting-info">
                            <p class="setting-label">When queue completes</p>
                            <p class="setting-desc">
                                Behavior when the queue finishes playing all tracks.
                            </p>
                        </div>

                        <div class="custom-select-container">
                            <button
                                class="select-trigger"
                                onclick={(e) => {
                                    e.stopPropagation();
                                    isQueueCompletionOpen = !isQueueCompletionOpen;
                                }}
                                disabled={!settingsStore.loaded}
                            >
                                <span
                                    >{queueCompletionOptions.find(
                                        (o) => o.value === settingsStore.queueCompletionBehavior,
                                    )?.label || "Select..."}</span
                                >
                                <CaretDown
                                    size={14}
                                    weight="bold"
                                    class={isQueueCompletionOpen ? "rotated" : ""}
                                />
                            </button>

                            {#if isQueueCompletionOpen}
                                <div class="custom-select-menu glass">
                                    {#each queueCompletionOptions as opt}
                                        <button
                                            class="select-option"
                                            class:selected={settingsStore.queueCompletionBehavior ===
                                                opt.value}
                                            onclick={(e) => {
                                                e.stopPropagation();
                                                selectQueueCompletion(
                                                    opt.value as any,
                                                );
                                            }}
                                        >
                                            <span class="opt-label"
                                                >{opt.label}</span
                                            >
                                            {#if settingsStore.queueCompletionBehavior === opt.value}
                                                <Check
                                                    size={14}
                                                    weight="bold"
                                                    class="check-icon"
                                                />
                                            {/if}
                                        </button>
                                    {/each}
                                </div>
                            {/if}
                        </div>
                    </div>

                    <div class="setting-row">
                        <div class="setting-info">
                            <p class="setting-label">Daily Discover Artist Ceiling</p>
                            <p class="setting-desc">
                                Maximum number of songs a single artist can have in the Daily Discover shelf.
                            </p>
                        </div>

                        <div class="custom-select-container">
                            <button
                                class="select-trigger"
                                onclick={(e) => {
                                    e.stopPropagation();
                                    isDiversityCeilingOpen = !isDiversityCeilingOpen;
                                    isDropdownOpen = false;
                                    isQueueCompletionOpen = false;
                                }}
                                disabled={!settingsStore.loaded}
                            >
                                <span
                                    >{diversityCeilingOptions.find(
                                        (o) => o.value === settingsStore.discoveryArtistDiversityCeiling,
                                    )?.label || "3 Tracks (Balanced Catalog)"}</span
                                >
                                <CaretDown
                                    size={14}
                                    weight="bold"
                                    class={isDiversityCeilingOpen ? "rotated" : ""}
                                />
                            </button>

                            {#if isDiversityCeilingOpen}
                                <div class="custom-select-menu glass">
                                    {#each diversityCeilingOptions as opt}
                                        <button
                                            class="select-option"
                                            class:selected={settingsStore.discoveryArtistDiversityCeiling ===
                                                opt.value}
                                            onclick={(e) => {
                                                e.stopPropagation();
                                                selectDiversityCeiling(opt.value);
                                            }}
                                        >
                                            <span class="opt-label"
                                                >{opt.label}</span
                                            >
                                            {#if settingsStore.discoveryArtistDiversityCeiling === opt.value}
                                                <Check
                                                    size={14}
                                                    weight="bold"
                                                    class="check-icon"
                                                />
                                            {/if}
                                        </button>
                                    {/each}
                                </div>
                            {/if}
                        </div>
                    </div>

                    <div class="setting-row">
                        <div class="setting-info">
                            <p class="setting-label">Default Streaming / Fallback Provider</p>
                            <p class="setting-desc">
                                Select the fallback provider used to stream and resolve tracks when local audio files are missing.
                            </p>
                        </div>

                        <div class="custom-select-container">
                            <button
                                class="select-trigger"
                                onclick={(e) => {
                                    e.stopPropagation();
                                    isProviderDropdownOpen = !isProviderDropdownOpen;
                                    isDropdownOpen = false;
                                    isQueueCompletionOpen = false;
                                    isDiversityCeilingOpen = false;
                                }}
                                disabled={!settingsStore.loaded}
                            >
                                <span
                                    >{availableProviders.find(
                                        (o) => o.value === settingsStore.defaultRemoteProvider,
                                    )?.label || (settingsStore.defaultRemoteProvider === "local" ? "Local Files Only (No Remote Fallback)" : settingsStore.defaultRemoteProvider)}</span
                                >
                                <CaretDown
                                    size={14}
                                    weight="bold"
                                    class={isProviderDropdownOpen ? "rotated" : ""}
                                />
                            </button>

                            {#if isProviderDropdownOpen}
                                <div class="custom-select-menu glass">
                                    {#each availableProviders as opt}
                                        <button
                                            class="select-option"
                                            class:selected={settingsStore.defaultRemoteProvider ===
                                                opt.value}
                                            onclick={(e) => {
                                                e.stopPropagation();
                                                selectProvider(opt.value);
                                            }}
                                        >
                                            <span class="opt-label"
                                                >{opt.label}</span
                                            >
                                            {#if settingsStore.defaultRemoteProvider === opt.value}
                                                <Check
                                                    size={14}
                                                    weight="bold"
                                                    class="check-icon"
                                                />
                                            {/if}
                                        </button>
                                    {/each}
                                </div>
                            {/if}
                        </div>
                    </div>
                </section>
            {:else if activeTab === "appearance"}
                <section class="settings-section">
                    <h3 class="section-title">Appearance</h3>

                    <div class="setting-row">
                        <div class="setting-info">
                            <p class="setting-label">Glassy Player Bar</p>
                            <p class="setting-desc">
                                Enable a sleek, semi-transparent frosted glass
                                effect for the bottom player.
                            </p>
                        </div>
                        <label class="switch">
                            <input
                                type="checkbox"
                                checked={settingsStore.glassyPlayerBar}
                                onchange={(e) =>
                                    settingsStore.setGlassyPlayerBar(
                                        e.currentTarget.checked,
                                    )}
                                disabled={!settingsStore.loaded}
                            />
                            <span class="slider round"></span>
                        </label>
                    </div>
                </section>
            {:else if activeTab === "shortcuts"}
                <section class="settings-section">
                    <div class="section-title-row">
                        <div>
                            <h3 class="section-title">Shortcuts & Keybindings</h3>
                            <p class="section-desc">Click any keybinding to record a new key combination.</p>
                        </div>
                        <button class="action-btn secondary" onclick={handleResetKeymap} title="Reset to Factory Defaults">
                            <ArrowCounterClockwise size={14} weight="bold" />
                            <span>Reset Defaults</span>
                        </button>
                    </div>

                    <!-- Category: Discovery & Navigation -->
                    <div class="shortcuts-group">
                        <h4 class="group-title">Discovery & Navigation</h4>
                        <div class="shortcuts-ledger">
                            {#each (Object.entries(DEFAULT_KEYMAP).filter(([_, e]) => e.category === "discovery" || e.category === "navigation")) as [action, entry]}
                                <div class="shortcut-row">
                                    <div class="shortcut-info">
                                        <p class="setting-label">{entry.label}</p>
                                        <p class="setting-desc">{entry.description}</p>
                                    </div>
                                    <div class="shortcut-recorder-wrap">
                                        <button 
                                            class="keybind-btn" 
                                            class:is-recording={recordingAction === action}
                                            onclick={() => startRecording(action as KeyAction)}
                                            onkeydown={(e) => handleKeyRecord(e, action as KeyAction)}
                                            title="Click to change shortcut"
                                        >
                                            {#if recordingAction === action}
                                                <span class="recording-pulse">Press key... (Esc to cancel)</span>
                                            {:else}
                                                <span class="keybind-tag">{formatBinding(settingsStore.getKeybinding(action as KeyAction))}</span>
                                            {/if}
                                        </button>
                                    </div>
                                </div>
                            {/each}
                        </div>
                    </div>

                    <!-- Category: Playback Control -->
                    <div class="shortcuts-group">
                        <h4 class="group-title">Playback & Audio Controls</h4>
                        <div class="shortcuts-ledger">
                            {#each (Object.entries(DEFAULT_KEYMAP).filter(([_, e]) => e.category === "playback")) as [action, entry]}
                                <div class="shortcut-row">
                                    <div class="shortcut-info">
                                        <p class="setting-label">{entry.label}</p>
                                        <p class="setting-desc">{entry.description}</p>
                                    </div>
                                    <div class="shortcut-recorder-wrap">
                                        <button 
                                            class="keybind-btn" 
                                            class:is-recording={recordingAction === action}
                                            onclick={() => startRecording(action as KeyAction)}
                                            onkeydown={(e) => handleKeyRecord(e, action as KeyAction)}
                                            title="Click to change shortcut"
                                        >
                                            {#if recordingAction === action}
                                                <span class="recording-pulse">Press key... (Esc to cancel)</span>
                                            {:else}
                                                <span class="keybind-tag">{formatBinding(settingsStore.getKeybinding(action as KeyAction))}</span>
                                            {/if}
                                        </button>
                                    </div>
                                </div>
                            {/each}
                        </div>
                    </div>
                </section>
            {:else if activeTab === "data"}
                <section class="settings-section">
                    <h3 class="section-title">Data Management</h3>

                    <div class="setting-row">
                        <div class="setting-info">
                            <p class="setting-label">Clear Recommendation Cache</p>
                            <p class="setting-desc">
                                Purge all cached Daily Discover recommendations and reload sandboxed extension plugins into memory.
                            </p>
                        </div>
                        <button class="action-btn primary" onclick={clearRecommendationCache}>
                            <span>Clear Cache</span>
                        </button>
                    </div>

                    <div
                        class="setting-row"
                        style="flex-direction: column; align-items: flex-start; gap: 1.5rem; margin-top: 1.5rem;"
                    >
                        <div class="setting-info">
                            <p class="setting-desc" style="font-size: 0.95rem;">
                                This will delete the local SQLite database and
                                clear all cached artwork and extensions. Your
                                actual music files will not be touched.
                            </p>
                        </div>
                        <button class="danger-btn" onclick={factoryReset}>
                            Factory Reset (Wipe Database)
                        </button>
                    </div>
                </section>
            {:else if activeTab === "advanced"}
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
            {/if}
        </div>
    </div>
</div>

<style>
    .settings-layout {
        display: flex;
        height: 100vh;
        width: 100%;
    }

    /* Left Pane */
    .settings-nav {
        width: 250px;
        min-width: 250px;
        border-right: 1px solid var(--echo-border);
        padding: 3rem 1.5rem;
        display: flex;
        flex-direction: column;
        gap: 2rem;
    }

    .settings-title {
        font-family: var(--echo-font-heading);
        font-size: 1.5rem;
        font-weight: 600;
        color: var(--echo-text-1);
        margin: 0;
        padding-left: 0.5rem;
    }

    .nav-list {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .nav-item {
        background: transparent;
        border: none;
        color: var(--echo-text-2);
        font-family: var(--echo-font-body);
        font-size: 0.95rem;
        font-weight: 500;
        text-align: left;
        padding: 0.6rem 0.8rem;
        border-radius: 6px;
        cursor: pointer;
        transition: all 0.2s ease;
        position: relative;
    }

    .nav-item:hover:not(.active) {
        background: rgba(255, 255, 255, 0.05);
        color: var(--echo-text-1);
    }

    .nav-item.active {
        color: var(--echo-text-1);
        background: rgba(226, 169, 115, 0.08); /* subtle primary tint */
    }

    .nav-item.active::before {
        content: "";
        position: absolute;
        left: 0;
        top: 20%;
        bottom: 20%;
        width: 3px;
        background: var(--echo-primary);
        border-radius: 0 4px 4px 0;
    }

    /* Right Pane */
    .settings-content {
        flex: 1;
        overflow-y: auto;
        padding: 3rem 4rem var(--player-clearance, 10rem) 4rem;
        scroll-padding-bottom: var(--player-scroll-padding, 10rem);
        display: flex;
        flex-direction: column;
    }

    .content-container {
        max-width: 650px;
        width: 100%;
    }

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
    }

    .setting-info {
        display: flex;
        flex-direction: column;
        gap: 0.3rem;
        margin-right: 2rem;
    }

    .setting-label {
        font-size: 1rem;
        font-weight: 500;
        color: var(--echo-text-1);
        margin: 0;
    }

    .section-title-row {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding-bottom: 0.5rem;
        border-bottom: 1px solid var(--echo-border);
        margin-bottom: 0.5rem;
    }

    .section-title-row .section-title {
        border-bottom: none;
        padding-bottom: 0;
        margin-bottom: 0.2rem;
    }

    .section-desc {
        font-size: 0.82rem;
        color: var(--echo-text-2);
        margin: 0;
    }

    .shortcuts-group {
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
        margin-top: 0.5rem;
    }

    .group-title {
        font-family: var(--echo-font-mono, monospace);
        font-size: 0.72rem;
        font-weight: 700;
        letter-spacing: 0.08em;
        text-transform: uppercase;
        color: var(--echo-primary, #e2a973);
        margin: 0;
    }

    .shortcuts-ledger {
        display: flex;
        flex-direction: column;
        background: rgba(255, 255, 255, 0.02);
        border: 1px solid var(--echo-border);
        border-radius: 8px;
        overflow: hidden;
    }

    .shortcut-row {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 0.85rem 1.1rem;
        border-bottom: 1px solid rgba(255, 255, 255, 0.04);
        transition: background 0.15s ease;
    }

    .shortcut-row:last-child {
        border-bottom: none;
    }

    .shortcut-row:hover {
        background: rgba(255, 255, 255, 0.03);
    }

    .shortcut-info {
        display: flex;
        flex-direction: column;
        gap: 0.2rem;
        margin-right: 1.5rem;
    }

    .keybind-btn {
        background: rgba(255, 255, 255, 0.05);
        border: 1px solid rgba(255, 255, 255, 0.12);
        border-radius: 6px;
        padding: 0.4rem 0.8rem;
        cursor: pointer;
        display: inline-flex;
        align-items: center;
        justify-content: center;
        min-width: 90px;
        transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
    }

    .keybind-btn:hover:not(.is-recording) {
        background: rgba(255, 255, 255, 0.1);
        border-color: rgba(226, 169, 115, 0.4);
        color: var(--echo-primary, #e2a973);
    }

    .keybind-btn.is-recording {
        background: rgba(226, 169, 115, 0.15);
        border-color: var(--echo-primary, #e2a973);
        box-shadow: 0 0 12px rgba(226, 169, 115, 0.25);
    }

    .keybind-tag {
        font-family: var(--echo-font-mono, monospace);
        font-size: 0.78rem;
        font-weight: 600;
        color: var(--echo-text-1, #fff);
        letter-spacing: 0.04em;
    }

    .recording-pulse {
        font-family: var(--echo-font-mono, monospace);
        font-size: 0.74rem;
        font-weight: 600;
        color: var(--echo-primary, #e2a973);
        animation: pulse 1s infinite;
    }

    @keyframes pulse {
        0%, 100% { opacity: 1; }
        50% { opacity: 0.4; }
    }

    .setting-desc {
        font-size: 0.85rem;
        color: var(--echo-text-2);
        margin: 0;
        line-height: 1.5;
    }

    .danger-btn {
        background-color: rgba(220, 38, 38, 0.1);
        color: #ef4444;
        border: 1px solid rgba(220, 38, 38, 0.3);
        padding: 0.6rem 1.5rem;
        border-radius: 8px;
        font-size: 0.9rem;
        font-weight: 500;
        cursor: pointer;
        transition: all 0.2s ease;
    }

    .action-btn {
        display: inline-flex;
        align-items: center;
        gap: 0.55rem;
        background: var(--echo-surface);
        color: var(--echo-text-1);
        border: 1px solid var(--echo-border-medium);
        padding: 0.6rem 1.2rem;
        border-radius: 8px;
        font-family: var(--echo-font-body);
        font-size: 0.85rem;
        font-weight: 500;
        cursor: pointer;
        transition: all 0.2s ease;
        white-space: nowrap;
    }
    .action-btn:disabled {
        opacity: 0.45;
        cursor: not-allowed;
        filter: grayscale(0.15);
    }
    .action-btn:hover {
        background: rgba(255, 255, 255, 0.04);
        border-color: var(--echo-text-2);
    }
    .action-btn:disabled:hover {
        background: var(--echo-surface);
        border-color: var(--echo-border-medium);
    }
    .action-btn.primary {
        background: rgba(226, 169, 115, 0.12);
        color: var(--echo-primary);
        border-color: rgba(226, 169, 115, 0.3);
    }
    .action-btn.primary:hover {
        background: rgba(226, 169, 115, 0.22);
        border-color: var(--echo-primary);
    }

    .copy-action.copy-success {
        background: rgba(16, 185, 129, 0.14);
        color: #34d399;
        border-color: rgba(16, 185, 129, 0.35);
        animation: copySuccessPop 0.28s ease-out;
    }

    .copy-action.copy-success:hover:not(:disabled) {
        background: rgba(16, 185, 129, 0.18);
        border-color: rgba(16, 185, 129, 0.45);
    }

    .danger-btn:hover {
        background-color: rgba(220, 38, 38, 0.2);
        border-color: rgba(220, 38, 38, 0.5);
    }

    .dependent-group {
        margin-left: 1rem;
        padding-left: 1rem;
        border-left: 1px solid rgba(255, 255, 255, 0.08);
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
    }

    .dependent-group-label {
        margin: 0;
        font-size: 0.78rem;
        font-weight: 600;
        letter-spacing: 0.04em;
        text-transform: uppercase;
        color: var(--echo-text-2);
    }

    .dependent-row {
        padding: 0.75rem 0;
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

    /* Dropdown UI */
    .custom-select-container {
        position: relative;
        min-width: 200px;
    }

    .select-trigger {
        width: 100%;
        display: flex;
        justify-content: space-between;
        align-items: center;
        background: var(--echo-surface);
        border: 1px solid var(--echo-border-medium);
        color: var(--echo-text-1);
        padding: 0.6rem 1rem;
        border-radius: 8px;
        font-family: var(--echo-font-body);
        font-size: 0.9rem;
        cursor: pointer;
        transition: all 0.2s ease;
    }

    .select-trigger:hover:not(:disabled) {
        border-color: var(--echo-text-2);
        background: rgba(255, 255, 255, 0.04);
    }

    .select-trigger :global(svg) {
        transition: transform 0.2s cubic-bezier(0.4, 0, 0.2, 1);
        color: var(--echo-text-2);
    }

    .select-trigger :global(svg.rotated) {
        transform: rotate(180deg);
    }

    .custom-select-menu {
        position: absolute;
        top: calc(100% + 0.5rem);
        right: 0;
        width: max-content;
        min-width: 100%;
        display: flex;
        flex-direction: column;
        background: var(--echo-surface);
        border: 1px solid var(--echo-border-medium);
        border-radius: 8px;
        padding: 0.3rem;
        z-index: 50;
        box-shadow: 0 10px 30px -10px rgba(0, 0, 0, 0.6);
        animation: slideDown 0.15s cubic-bezier(0.16, 1, 0.3, 1) forwards;
        transform-origin: top center;
    }

    .select-option {
        display: flex;
        justify-content: space-between;
        align-items: center;
        width: 100%;
        background: transparent;
        border: none;
        color: var(--echo-text-2);
        padding: 0.6rem 0.8rem;
        border-radius: 6px;
        font-family: var(--echo-font-body);
        font-size: 0.85rem;
        cursor: pointer;
        transition: all 0.15s ease;
        text-align: left;
    }

    .select-option:hover {
        background: rgba(255, 255, 255, 0.05);
        color: var(--echo-text-1);
    }

    .select-option.selected {
        color: var(--echo-primary);
        background: rgba(255, 255, 255, 0.02);
        font-weight: 500;
    }

    .select-option :global(.check-icon) {
        color: var(--echo-primary);
        margin-left: 1rem;
    }

    /* Switch CSS */
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
        background-color: var(--echo-raised);
        border: 1px solid var(--echo-border-medium);
        transition: 0.4s;
    }
    .slider:before {
        position: absolute;
        content: "";
        height: 16px;
        width: 16px;
        left: 3px;
        bottom: 3px;
        background-color: var(--echo-text-2);
        transition: 0.4s;
    }
    input:checked + .slider {
        background-color: var(--echo-primary);
        border-color: var(--echo-primary);
    }
    input:checked + .slider:before {
        transform: translateX(20px);
        background-color: var(--echo-void);
    }
    .slider.round {
        border-radius: 34px;
    }
    .slider.round:before {
        border-radius: 50%;
    }

    @keyframes fadeIn {
        from {
            opacity: 0;
            transform: translateY(5px);
        }
        to {
            opacity: 1;
            transform: translateY(0);
        }
    }
    @keyframes slideDown {
        0% {
            opacity: 0;
            transform: translateY(-4px) scale(0.98);
        }
        100% {
            opacity: 1;
            transform: translateY(0) scale(1);
        }
    }
</style>
