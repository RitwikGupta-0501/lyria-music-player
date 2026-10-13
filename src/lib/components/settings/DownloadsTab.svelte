<script lang="ts">
    import { downloadStore } from "$lib/stores/download.svelte";
    import { settingsStore } from "$lib/stores/settings.svelte";
    import { toastStore } from "$lib/stores/toast.svelte";
    import CustomSelect from "$lib/components/ui/CustomSelect.svelte";
    import { FolderOpen, Broom, CheckCircle } from "phosphor-svelte";
    import { onMount } from "svelte";

    onMount(async () => {
        await downloadStore.init();
    });

    let providerOptions = $derived([
        { value: "auto", label: "Automatic (Highest Available Quality)" },
        ...downloadStore.providers.map(p => ({
            value: p.id,
            label: p.id === "monochrome" 
                ? "Monochrome (FLAC Lossless)" 
                : p.id === "freemp3s" 
                    ? "FreeMP3s (MP3 320kbps)" 
                    : p.name,
        }))
    ]);

    let selectedProvider = $derived(downloadStore.preferredProvider || "auto");

    async function handleProviderChange(val: string) {
        const pref = val === "auto" ? null : val;
        await downloadStore.setPreferredProvider(pref);
        await settingsStore.setSetting("preferred_download_provider", pref || "");
        toastStore.success(`Default download provider set to ${val === "auto" ? "Automatic" : val}`);
    }

    async function handleOpenFolder() {
        try {
            await downloadStore.openDownloadsFolder();
        } catch (e: any) {
            toastStore.error(e?.toString() || "Failed to open downloads folder");
        }
    }

    async function handleClearCompleted() {
        try {
            await downloadStore.clearCompletedDownloads();
            toastStore.info("Cleared completed download history");
        } catch (e: any) {
            toastStore.error(e?.toString() || "Failed to clear completed history");
        }
    }
</script>

<section class="settings-section">
    <h3 class="section-title">Downloads & Offline Library</h3>

    <!-- Preferred Provider -->
    <div class="setting-row">
        <div class="setting-info">
            <p class="setting-label">Preferred Audio Quality & Provider</p>
            <p class="setting-desc">
                Default provider used for single-click downloads and album exports. Automatic attempts lossless FLAC first and cascades to 320kbps MP3.
            </p>
        </div>
        <div class="setting-control">
            <CustomSelect
                options={providerOptions}
                value={selectedProvider}
                onChange={handleProviderChange}
            />
        </div>
    </div>

    <!-- Download Location -->
    <div class="setting-row">
        <div class="setting-info">
            <p class="setting-label">Storage Location</p>
            <p class="setting-desc">
                Completed audio files are organized by artist and album into this directory and automatically indexed into your offline library.
            </p>
            {#if downloadStore.downloadsDir}
                <div class="path-display" title={downloadStore.downloadsDir}>
                    <code>{downloadStore.downloadsDir}</code>
                </div>
            {/if}
        </div>
        <button class="action-btn secondary" onclick={handleOpenFolder} title="Open downloads folder in file manager">
            <FolderOpen size={16} />
            <span>Open Folder</span>
        </button>
    </div>

    <!-- Download History -->
    <div class="setting-row">
        <div class="setting-info">
            <p class="setting-label">Download Task History</p>
            <p class="setting-desc">
                Remove finished items from the active downloads monitor. Downloaded music files on your disk and in your library will not be affected.
            </p>
        </div>
        <button class="action-btn ghost" onclick={handleClearCompleted} title="Clear finished tasks from queue">
            <Broom size={16} />
            <span>Clear Completed</span>
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
        gap: 0.35rem;
        flex: 1;
        min-width: 0;
    }

    .setting-label {
        font-size: 0.95rem;
        font-weight: 500;
        color: var(--echo-text-1);
        margin: 0;
    }

    .setting-desc {
        font-size: 0.85rem;
        color: var(--echo-text-2);
        margin: 0;
        line-height: 1.4;
    }

    .path-display {
        margin-top: 0.35rem;
        padding: 0.35rem 0.65rem;
        background: rgba(255, 255, 255, 0.04);
        border: 1px solid rgba(255, 255, 255, 0.08);
        border-radius: 6px;
        max-width: 100%;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }

    .path-display code {
        font-family: monospace;
        font-size: 0.8rem;
        color: var(--echo-primary, #ffd285);
    }

    .setting-control {
        min-width: 260px;
    }

    .action-btn {
        display: inline-flex;
        align-items: center;
        gap: 0.5rem;
        padding: 0.5rem 1rem;
        border-radius: 6px;
        font-size: 0.875rem;
        font-weight: 500;
        cursor: pointer;
        transition: all 0.2s ease;
        white-space: nowrap;
        border: none;
    }

    .action-btn.secondary {
        background: rgba(255, 255, 255, 0.08);
        color: var(--echo-text-1);
        border: 1px solid rgba(255, 255, 255, 0.12);
    }

    .action-btn.secondary:hover {
        background: rgba(255, 255, 255, 0.14);
        border-color: rgba(255, 255, 255, 0.2);
    }

    .action-btn.ghost {
        background: transparent;
        color: var(--echo-text-2);
        border: 1px solid rgba(255, 255, 255, 0.08);
    }

    .action-btn.ghost:hover {
        background: rgba(255, 255, 255, 0.05);
        color: var(--echo-text-1);
        border-color: rgba(255, 255, 255, 0.15);
    }
</style>
