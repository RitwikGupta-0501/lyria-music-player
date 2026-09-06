<script lang="ts">
    import { settingsStore } from "$lib/stores/settings.svelte";
    import { toastStore } from "$lib/stores/toast.svelte";
    import { DEFAULT_KEYMAP, formatBinding, eventToBinding, type KeyAction } from "$lib/stores/keymap";
    import { ArrowCounterClockwise } from "phosphor-svelte";

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
</script>

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

<style>
    .settings-section {
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
        animation: fadeIn 0.2s ease;
    }
    .section-title-row {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding-bottom: 0.5rem;
        border-bottom: 1px solid var(--echo-border);
        margin-bottom: 0.5rem;
    }
    .section-title {
        font-family: var(--lyria-font-body);
        font-size: 1.25rem;
        font-weight: 500;
        color: var(--echo-text-1);
        margin: 0;
    }
    .section-desc {
        font-size: 0.85rem;
        color: var(--echo-text-2);
        margin-top: 0.2rem;
    }
    .shortcuts-group {
        display: flex;
        flex-direction: column;
        gap: 0.8rem;
        margin-top: 0.5rem;
    }
    .group-title {
        font-size: 0.85rem;
        text-transform: uppercase;
        letter-spacing: 0.05em;
        color: var(--echo-text-2);
        font-weight: 600;
    }
    .shortcuts-ledger {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }
    .shortcut-row {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 0.75rem 1rem;
        background: var(--echo-surface);
        border: 1px solid var(--echo-border);
        border-radius: 8px;
        gap: 1rem;
    }
    .shortcut-info {
        display: flex;
        flex-direction: column;
        gap: 0.2rem;
        flex: 1;
    }
    .setting-label {
        font-size: 0.92rem;
        font-weight: 500;
        color: var(--echo-text-1);
    }
    .setting-desc {
        font-size: 0.8rem;
        color: var(--echo-text-2);
    }
    .keybind-btn {
        background: var(--echo-raised);
        border: 1px solid var(--echo-border-medium);
        color: var(--echo-text-1);
        padding: 0.4rem 0.8rem;
        border-radius: 6px;
        cursor: pointer;
        font-family: var(--lyria-font-mono);
        font-size: 0.82rem;
        font-weight: 600;
        transition: all 0.15s ease;
    }
    .keybind-btn:hover {
        border-color: rgba(226, 169, 115, 0.5);
        color: var(--echo-primary);
    }
    .keybind-btn.is-recording {
        border-color: var(--echo-primary);
        background: rgba(226, 169, 115, 0.15);
        color: var(--echo-primary);
    }
    .recording-pulse {
        animation: pulse 1s infinite alternate;
    }
    @keyframes pulse {
        from { opacity: 0.6; }
        to { opacity: 1; }
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
        transition: all 0.2s ease;
    }
    .action-btn.secondary {
        background: var(--echo-surface);
        border: 1px solid var(--echo-border-medium);
        color: var(--echo-text-2);
    }
    .action-btn.secondary:hover {
        color: var(--echo-text-1);
        border-color: var(--echo-border-strong);
    }
</style>
