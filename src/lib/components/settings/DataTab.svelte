<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { libraryStore } from "$lib/stores/library.svelte";
    import { toastStore } from "$lib/stores/toast.svelte";

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
                toastStore.success("Database and library wiped.");
            } catch (e) {
                console.error("Factory reset failed:", e);
                toastStore.error("Failed to reset library.");
            }
        }
    }
</script>

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
        class="setting-row danger-block"
    >
        <div class="setting-info">
            <p class="setting-label danger-text">Factory Reset</p>
            <p class="setting-desc">
                This will delete the local SQLite database and clear all cached artwork and extensions. Your
                actual music files will not be touched.
            </p>
        </div>
        <button class="danger-btn" onclick={factoryReset}>
            Factory Reset (Wipe Database)
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
    .danger-block {
        flex-direction: column;
        align-items: flex-start;
        gap: 1.2rem;
        margin-top: 1rem;
        padding: 1.25rem;
        border: 1px solid rgba(220, 38, 38, 0.25);
        border-radius: 8px;
        background: rgba(220, 38, 38, 0.05);
    }
    .danger-text {
        color: #ef4444;
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
    .action-btn.primary {
        background: var(--echo-raised);
        border: 1px solid var(--echo-border-strong);
        color: var(--echo-text-1);
    }
    .action-btn.primary:hover {
        border-color: var(--echo-primary);
        color: var(--echo-primary);
    }
    .danger-btn {
        background: rgba(220, 38, 38, 0.15);
        border: 1px solid rgba(220, 38, 38, 0.35);
        color: #ef4444;
        padding: 0.5rem 1rem;
        border-radius: 6px;
        font-family: inherit;
        font-size: 0.85rem;
        font-weight: 600;
        cursor: pointer;
        transition: all 0.2s ease;
    }
    .danger-btn:hover {
        background: rgba(220, 38, 38, 0.25);
        border-color: #ef4444;
    }
</style>
