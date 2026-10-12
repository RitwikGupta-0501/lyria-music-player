<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { libraryStore } from "$lib/stores/library.svelte";
    import { homeStore } from "$lib/stores/home.svelte";
    import { audioStore } from "$lib/stores/audio.svelte";
    import { toastStore } from "$lib/stores/toast.svelte";
    import { Warning, CheckSquare, Square, X, Trash } from "phosphor-svelte";

    interface Props {
        onClose: () => void;
        onPurged?: () => void;
    }

    let { onClose, onPurged }: Props = $props();

    interface CategoryOption {
        id: string;
        label: string;
        description: string;
        isDestructive?: boolean;
    }

    const categories: CategoryOption[] = [
        {
            id: "local_library",
            label: "Local Library Index",
            description: "Deletes all indexed tracks and albums from the local database. Your physical audio files are not touched.",
        },
        {
            id: "playback_history",
            label: "Playback History & Analytics",
            description: "Clears listening logs, Heavy Rotation analytics, song play counts, and recommendation caches.",
            isDestructive: true,
        },
        {
            id: "playlists_and_likes",
            label: "Playlists & Liked Songs",
            description: "Deletes custom user playlists, saved remote albums, and liked songs telemetry.",
            isDestructive: true,
        },
        {
            id: "artwork_cache",
            label: "Artwork Disk Cache",
            description: "Purges extracted thumbnail images and cached album covers from disk (~/.local/share/Lyria/artwork).",
        },
        {
            id: "extensions",
            label: "Extensions & Plugin Storage",
            description: "Deletes all installed WASM extension bundles, isolated plugin settings, and metrics.",
            isDestructive: true,
        },
        {
            id: "settings",
            label: "Settings & Feature Flags",
            description: "Resets all audio preferences, theme settings, shortcuts, and feature flags to defaults.",
        },
        {
            id: "queue",
            label: "Active Queue & Playback State",
            description: "Stops audio playback and clears the active queue, shuffle history, and recovery snapshots.",
        },
    ];

    let selected = $state<Record<string, boolean>>({
        local_library: false,
        playback_history: false,
        playlists_and_likes: false,
        artwork_cache: false,
        extensions: false,
        settings: false,
        queue: false,
    });

    let isPurging = $state(false);

    let selectedCount = $derived(Object.values(selected).filter(Boolean).length);
    let hasDestructiveSelected = $derived(
        categories.some((c) => c.isDestructive && selected[c.id])
    );

    function toggleAll(value: boolean) {
        for (const cat of categories) {
            selected[cat.id] = value;
        }
    }

    function toggleCategory(id: string) {
        selected[id] = !selected[id];
    }

    async function handlePurge() {
        const toPurge = Object.keys(selected).filter((k) => selected[k]);
        if (toPurge.length === 0) return;

        if (hasDestructiveSelected) {
            const confirmed = confirm(
                "You have selected irreversible categories (such as Playback History or Playlists). Are you sure you want to delete this data?"
            );
            if (!confirmed) return;
        }

        isPurging = true;
        try {
            await invoke("purge_data", { categories: toPurge });

            // Synchronize frontend stores based on purged categories
            if (selected.local_library || selected.playlists_and_likes) {
                libraryStore.albums = [];
                libraryStore.playlists = [];
                libraryStore.likedSongs = [];
            }

            if (selected.playback_history) {
                homeStore.heavyRotation = { artists: [], albums: [] };
                homeStore.quickPicks = [];
                homeStore.keepListening = [];
            }

            if (selected.queue) {
                audioStore.clearQueue();
            }

            toastStore.success(`Purged ${toPurge.length} data ${toPurge.length === 1 ? "category" : "categories"}.`);
            onPurged?.();
            onClose();
        } catch (e: any) {
            console.error("Purge data failed:", e);
            toastStore.error(e?.toString() || "Failed to purge data.");
        } finally {
            isPurging = false;
        }
    }
</script>

<div class="modal-backdrop" onclick={onClose} role="presentation">
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="modal-card" onclick={(e) => e.stopPropagation()}>
        <header class="modal-header">
            <div class="header-left">
                <div class="warning-icon-wrap">
                    <Trash size={20} weight="bold" color="#ef4444" />
                </div>
                <div>
                    <h2 class="modal-title">Purge Application Data</h2>
                    <p class="modal-subtitle">Select the specific data categories you wish to clear.</p>
                </div>
            </div>
            <button class="close-btn" onclick={onClose} aria-label="Close modal">
                <X size={18} weight="bold" />
            </button>
        </header>

        <div class="selection-actions">
            <button 
                type="button" 
                class="text-action-btn" 
                onclick={() => toggleAll(true)}
            >
                Select All
            </button>
            <span class="action-sep">•</span>
            <button 
                type="button" 
                class="text-action-btn" 
                onclick={() => toggleAll(false)}
            >
                Clear Selection
            </button>
            <span class="selection-counter">{selectedCount} of {categories.length} selected</span>
        </div>

        <div class="categories-list">
            {#each categories as cat (cat.id)}
                {@const isChecked = selected[cat.id]}
                <!-- svelte-ignore a11y_click_events_have_key_events -->
                <div
                    class="category-item"
                    class:selected={isChecked}
                    class:destructive={cat.isDestructive && isChecked}
                    onclick={() => toggleCategory(cat.id)}
                    role="checkbox"
                    tabindex="0"
                    aria-checked={isChecked}
                >
                    <div class="check-box-icon">
                        {#if isChecked}
                            <CheckSquare size={19} weight="fill" color={cat.isDestructive ? "#ef4444" : "var(--echo-primary)"} />
                        {:else}
                            <Square size={19} weight="regular" color="var(--echo-text-3)" />
                        {/if}
                    </div>
                    <div class="category-meta">
                        <div class="category-label-row">
                            <span class="category-label">{cat.label}</span>
                            {#if cat.isDestructive}
                                <span class="badge-destructive">Permanent</span>
                            {/if}
                        </div>
                        <p class="category-desc">{cat.description}</p>
                    </div>
                </div>
            {/each}
        </div>

        {#if hasDestructiveSelected}
            <div class="destructive-warning-banner">
                <Warning size={18} weight="fill" color="#ef4444" />
                <span>Selected categories contain listening metrics or playlists that cannot be restored once purged.</span>
            </div>
        {/if}

        <footer class="modal-footer">
            <button 
                type="button" 
                class="btn secondary" 
                onclick={onClose} 
                disabled={isPurging}
            >
                Cancel
            </button>
            <button
                type="button"
                class="btn danger"
                onclick={handlePurge}
                disabled={selectedCount === 0 || isPurging}
            >
                {#if isPurging}
                    Purging...
                {:else}
                    Purge Selected ({selectedCount})
                {/if}
            </button>
        </footer>
    </div>
</div>

<style>
    .modal-backdrop {
        position: fixed;
        inset: 0;
        background: rgba(0, 0, 0, 0.72);
        backdrop-filter: blur(12px);
        -webkit-backdrop-filter: blur(12px);
        z-index: 200;
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 1.5rem;
        animation: fadeIn 0.15s ease-out;
    }

    .modal-card {
        background: var(--echo-raised, #18181b);
        border: 1px solid var(--echo-border-strong, rgba(255, 255, 255, 0.12));
        border-radius: 12px;
        width: 100%;
        max-width: 580px;
        max-height: 88vh;
        display: flex;
        flex-direction: column;
        box-shadow: 0 20px 48px rgba(0, 0, 0, 0.65);
        overflow: hidden;
    }

    .modal-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 1.25rem 1.5rem;
        border-bottom: 1px solid var(--echo-border);
    }

    .header-left {
        display: flex;
        align-items: center;
        gap: 0.85rem;
    }

    .warning-icon-wrap {
        width: 36px;
        height: 36px;
        border-radius: 8px;
        background: rgba(239, 68, 68, 0.12);
        border: 1px solid rgba(239, 68, 68, 0.25);
        display: flex;
        align-items: center;
        justify-content: center;
        flex-shrink: 0;
    }

    .modal-title {
        margin: 0;
        font-size: 1.1rem;
        font-weight: 600;
        color: var(--echo-text-1);
    }

    .modal-subtitle {
        margin: 0.2rem 0 0;
        font-size: 0.8rem;
        color: var(--echo-text-3);
    }

    .close-btn {
        background: transparent;
        border: none;
        color: var(--echo-text-3);
        padding: 0.4rem;
        border-radius: 6px;
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        transition: all 0.12s ease;
    }

    .close-btn:hover {
        color: var(--echo-text-1);
        background: rgba(255, 255, 255, 0.08);
    }

    .selection-actions {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        padding: 0.6rem 1.5rem;
        background: rgba(0, 0, 0, 0.18);
        border-bottom: 1px solid var(--echo-border);
        font-size: 0.78rem;
    }

    .text-action-btn {
        background: transparent;
        border: none;
        color: var(--echo-primary);
        cursor: pointer;
        padding: 0;
        font-size: inherit;
        font-weight: 500;
    }

    .text-action-btn:hover {
        text-decoration: underline;
    }

    .action-sep {
        color: var(--echo-text-3);
    }

    .selection-counter {
        margin-left: auto;
        color: var(--echo-text-3);
    }

    .categories-list {
        flex: 1;
        overflow-y: auto;
        padding: 0.75rem 1.5rem;
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .category-item {
        display: flex;
        align-items: flex-start;
        gap: 0.85rem;
        padding: 0.75rem 0.9rem;
        border-radius: 8px;
        background: rgba(255, 255, 255, 0.02);
        border: 1px solid var(--echo-border);
        cursor: pointer;
        transition: background 0.12s ease, border-color 0.12s ease;
        user-select: none;
    }

    .category-item:hover {
        background: rgba(255, 255, 255, 0.05);
        border-color: rgba(255, 255, 255, 0.15);
    }

    .category-item.selected {
        background: rgba(255, 255, 255, 0.06);
        border-color: var(--echo-primary);
    }

    .category-item.destructive.selected {
        border-color: rgba(239, 68, 68, 0.6);
        background: rgba(239, 68, 68, 0.05);
    }

    .check-box-icon {
        padding-top: 1px;
        display: flex;
        align-items: center;
        justify-content: center;
        flex-shrink: 0;
    }

    .category-meta {
        flex: 1;
        min-width: 0;
        display: flex;
        flex-direction: column;
        gap: 0.2rem;
    }

    .category-label-row {
        display: flex;
        align-items: center;
        gap: 0.5rem;
    }

    .category-label {
        font-size: 0.88rem;
        font-weight: 500;
        color: var(--echo-text-1);
    }

    .badge-destructive {
        font-size: 0.65rem;
        padding: 0.1rem 0.35rem;
        border-radius: 4px;
        background: rgba(239, 68, 68, 0.15);
        color: #ef4444;
        font-weight: 600;
        text-transform: uppercase;
        letter-spacing: 0.04em;
    }

    .category-desc {
        margin: 0;
        font-size: 0.76rem;
        color: var(--echo-text-3);
        line-height: 1.35;
    }

    .destructive-warning-banner {
        display: flex;
        align-items: center;
        gap: 0.65rem;
        padding: 0.75rem 1.5rem;
        background: rgba(239, 68, 68, 0.08);
        border-top: 1px solid rgba(239, 68, 68, 0.25);
        color: #ef4444;
        font-size: 0.76rem;
        line-height: 1.35;
    }

    .modal-footer {
        display: flex;
        align-items: center;
        justify-content: flex-end;
        gap: 0.75rem;
        padding: 1rem 1.5rem;
        border-top: 1px solid var(--echo-border);
        background: rgba(0, 0, 0, 0.15);
    }

    .btn {
        padding: 0.5rem 1.15rem;
        border-radius: 6px;
        font-family: inherit;
        font-size: 0.84rem;
        font-weight: 500;
        cursor: pointer;
        transition: all 0.15s ease;
    }

    .btn.secondary {
        background: transparent;
        border: 1px solid var(--echo-border);
        color: var(--echo-text-2);
    }

    .btn.secondary:hover:not(:disabled) {
        color: var(--echo-text-1);
        background: rgba(255, 255, 255, 0.05);
    }

    .btn.danger {
        background: #ef4444;
        border: 1px solid #dc2626;
        color: #ffffff;
        font-weight: 600;
    }

    .btn.danger:hover:not(:disabled) {
        background: #dc2626;
    }

    .btn:disabled {
        opacity: 0.45;
        cursor: not-allowed;
    }

    @keyframes fadeIn {
        from { opacity: 0; }
        to { opacity: 1; }
    }
</style>
