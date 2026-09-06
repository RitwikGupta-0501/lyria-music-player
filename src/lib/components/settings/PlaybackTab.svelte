<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { settingsStore } from "$lib/stores/settings.svelte";
    import CustomSelect from "$lib/components/ui/CustomSelect.svelte";
    import { onMount } from "svelte";

    const behaviorOptions = [
        { value: "interrupt", label: "Play Next & Switch" },
        { value: "clear", label: "Clear Queue & Play" },
        { value: "append", label: "Add to End of Queue" },
    ];

    const queueCompletionOptions = [
        { value: "retain_stopped", label: "Reset to Start & Keep Queue" },
        { value: "pause_end", label: "Pause at End of Song" },
        { value: "collapse_idle", label: "Clear Player When Finished" },
    ];

    const diversityCeilingOptions = [
        { value: 1, label: "1 Track (Maximum Diversity / Singletons)" },
        { value: 2, label: "2 Tracks (High Diversity)" },
        { value: 3, label: "3 Tracks (Balanced Catalog - Default)" },
        { value: 4, label: "4 Tracks (Deep Catalog)" },
        { value: 5, label: "5 Tracks (Relaxed Diversity)" },
    ];

    let availableProviders = $state<Array<{ value: string; label: string }>>([
        { value: "local", label: "Local Files Only (No Remote Fallback)" }
    ]);

    onMount(async () => {
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

    async function toggleKeepPlaying() {
        await settingsStore.setKeepPlayingOnQueueClear(!settingsStore.keepPlayingOnQueueClear);
    }
</script>

<section class="settings-section">
    <h3 class="section-title">Audio & Playback</h3>

    <div class="setting-row">
        <div class="setting-info">
            <p class="setting-label">Autoplay</p>
            <p class="setting-desc">Continuously discover and play similar music when the queue finishes (remote radio stream with local library fallback).</p>
        </div>
        <label class="switch">
            <input
                type="checkbox"
                checked={settingsStore.autoplay}
                onchange={() => settingsStore.setAutoplay(!settingsStore.autoplay)}
                disabled={!settingsStore.loaded}
            />
            <span class="slider round"></span>
        </label>
    </div>


    <div class="setting-row">
        <div class="setting-info">
            <p class="setting-label">Remote Stream Fallback</p>
            <p class="setting-desc">Automatically stream from your configured remote provider when a track's local file is missing in Quick Picks and home feeds.</p>
        </div>
        <label class="switch">
            <input
                type="checkbox"
                checked={settingsStore.remoteStreamingFallback}
                onchange={() => settingsStore.setRemoteStreamingFallback(!settingsStore.remoteStreamingFallback)}
                disabled={!settingsStore.loaded}
            />
            <span class="slider round"></span>
        </label>
    </div>

    <div class="setting-row">
        <div class="setting-info">
            <p class="setting-label">Keep Playing on Queue Clear</p>
            <p class="setting-desc">Allow the current song to finish even if the upcoming queue is wiped.</p>
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
            <p class="setting-desc">Behavior when playing a single track while a queue is active.</p>
        </div>
        <CustomSelect
            options={behaviorOptions}
            value={settingsStore.trackClickBehavior}
            onChange={(val) => settingsStore.setTrackClickBehavior(val)}
        />
    </div>

    <div class="setting-row">
        <div class="setting-info">
            <p class="setting-label">When queue completes</p>
            <p class="setting-desc">Behavior when the queue finishes playing all tracks.</p>
        </div>
        <CustomSelect
            options={queueCompletionOptions}
            value={settingsStore.queueCompletionBehavior}
            onChange={(val) => settingsStore.setQueueCompletionBehavior(val)}
        />
    </div>

    <div class="setting-row">
        <div class="setting-info">
            <p class="setting-label">Daily Discover Artist Ceiling</p>
            <p class="setting-desc">Maximum number of songs a single artist can have in the Daily Discover shelf.</p>
        </div>
        <CustomSelect
            options={diversityCeilingOptions}
            value={settingsStore.discoveryArtistDiversityCeiling}
            onChange={(val) => settingsStore.setDiscoveryArtistDiversityCeiling(val)}
        />
    </div>

    <div class="setting-row">
        <div class="setting-info">
            <p class="setting-label">Default Streaming / Fallback Provider</p>
            <p class="setting-desc">Select the fallback provider used to stream and resolve tracks when local audio files are missing.</p>
        </div>
        <CustomSelect
            options={availableProviders}
            value={settingsStore.defaultRemoteProvider}
            onChange={(val) => settingsStore.setDefaultRemoteProvider(val)}
        />
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
    .setting-row {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 1rem 0;
        gap: 1.5rem;
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
</style>
