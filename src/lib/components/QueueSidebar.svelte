<script lang="ts">
    import { audioStore } from "$lib/stores/audio.svelte";
    import { libraryStore } from "$lib/stores/library.svelte";
    import { toastStore } from "$lib/stores/toast.svelte";
    import { createVirtualizer } from "@tanstack/svelte-virtual";
    import { X, Trash, Pause, Play, BookmarkSimple, DotsSixVertical } from "phosphor-svelte";
    import PromptModal from "$lib/components/PromptModal.svelte";
    import EqualizerWave from "$lib/components/common/EqualizerWave.svelte";
    let { open = $bindable(false) } = $props<{ open?: boolean }>();

    let draggedIndex = $state(-1);
    let dragoverIndex = $state(-1);
    let dropPosition = $state<"top" | "bottom" | null>(null);
    let justReorderedIndex = $state(-1);
    let isLoading = $state(false);

    let scrollContainer = $state<HTMLElement | null>(null);

    let virtStore = $derived.by(() => {
        const container = scrollContainer;
        return createVirtualizer({
            count: audioStore.queue.length,
            getScrollElement: () => container,
            estimateSize: () => 52,
            overscan: 5,
        });
    });

    let queueWithIndex = $derived(
        audioStore.queue.map((track, i) => ({ track, index: i }))
    );

    function jumpToTrack(instanceId: string) {
        audioStore.jumpToTrack(instanceId);
    }

    function handleDragStart(e: DragEvent, index: number) {
        draggedIndex = index;
        if (e.dataTransfer) {
            e.dataTransfer.effectAllowed = "move";
            e.dataTransfer.setData("text/plain", index.toString());
        }
    }

    function handleDragOver(e: DragEvent, index: number) {
        e.preventDefault();
        if (draggedIndex === index) {
            dragoverIndex = -1;
            dropPosition = null;
            return;
        }

        const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
        const relY = e.clientY - rect.top;
        dropPosition = relY < rect.height / 2 ? "top" : "bottom";
        dragoverIndex = index;

        if (e.dataTransfer) {
            e.dataTransfer.dropEffect = "move";
        }
    }

    function handleDragLeave() {
        // Handled cleanly on dragend and ondrop to eliminate child hover flickering
    }

    async function handleDrop(e: DragEvent, targetIndex: number) {
        e.preventDefault();
        const fromIdx = draggedIndex;
        const pos = dropPosition;

        dragoverIndex = -1;
        dropPosition = null;
        draggedIndex = -1;

        if (fromIdx !== -1 && fromIdx !== targetIndex) {
            let toIdx = targetIndex;
            if (pos === "bottom" && fromIdx < targetIndex) {
                toIdx = targetIndex;
            } else if (pos === "top" && fromIdx > targetIndex) {
                toIdx = targetIndex;
            } else if (pos === "bottom" && fromIdx > targetIndex) {
                toIdx = targetIndex + 1;
            } else if (pos === "top" && fromIdx < targetIndex) {
                toIdx = targetIndex - 1;
            }

            toIdx = Math.max(0, Math.min(toIdx, audioStore.queue.length - 1));

            if (fromIdx !== toIdx) {
                isLoading = true;
                try {
                    await audioStore.reorderQueue(fromIdx, toIdx);
                    justReorderedIndex = toIdx;
                    setTimeout(() => {
                        justReorderedIndex = -1;
                    }, 400);
                } catch (error) {
                    console.error("Reorder failed:", error);
                } finally {
                    isLoading = false;
                }
            }
        }
    }

    let showSaveModal = $state(false);

    function getDefaultPlaylistName(): string {
        const d = new Date();
        const month = d.toLocaleString("default", { month: "short" });
        const day = d.getDate();
        const hours = d.getHours().toString().padStart(2, "0");
        const minutes = d.getMinutes().toString().padStart(2, "0");
        return `Queue Mix (${month} ${day}, ${hours}:${minutes})`;
    }

    async function handleSaveQueue(name: string) {
        showSaveModal = false;
        if (!name.trim() || audioStore.queue.length === 0) return;
        isLoading = true;
        try {
            await libraryStore.saveQueueAsPlaylist(name.trim(), audioStore.queue);
        } catch (e: any) {
            console.error("Failed to save queue as playlist:", e);
            toastStore.show(e?.toString() || "Failed to save playlist", "error");
        } finally {
            isLoading = false;
        }
    }

    async function handleClearQueue() {
        if (confirm("Clear entire queue?")) {
            isLoading = true;
            try {
                await audioStore.clearQueue();
            } finally {
                isLoading = false;
            }
        }
    }
</script>

{#if open}
    <div class="queue-view">
        <div class="queue-header">
            <span class="queue-title">Queue</span>
            <div class="header-actions">
                {#if audioStore.queue.length > 0}
                    <button
                        class="icon-btn"
                        onclick={() => (showSaveModal = true)}
                        disabled={isLoading}
                        title="Save queue as playlist"
                    >
                        <BookmarkSimple size={16} weight="bold" />
                    </button>
                    <button
                        class="icon-btn"
                        onclick={handleClearQueue}
                        disabled={isLoading}
                        title="Clear queue"
                    >
                        <Trash size={16} weight="bold" />
                    </button>
                {/if}
            </div>
        </div>

        {#if audioStore.queue.length === 0}
            <div class="queue-empty">
                <p class="empty-label">Queue is empty</p>
                <p class="empty-hint">Play an album or playlist to populate it.</p>
            </div>
        {:else}
            <div class="virtual-list-container" class:is-reordering={draggedIndex !== -1} bind:this={scrollContainer}>
                <div style="position: relative; width: 100%; height: {$virtStore.getTotalSize()}px;">
                    {#each $virtStore.getVirtualItems() as row (row.index)}
                        {@const i = row.index}
                        {@const track = queueWithIndex[i].track}
                        {@const isPlaying = track.instanceId === audioStore.currentQueueId}
                        {@const isPast = i < audioStore.currentPosition}
                        {@const isDragging = draggedIndex === i}
                        {@const isOverTop = dragoverIndex === i && dropPosition === "top" && draggedIndex !== i}
                        {@const isOverBottom = dragoverIndex === i && dropPosition === "bottom" && draggedIndex !== i}

                        <div
                            class="queue-row"
                            class:playing={isPlaying}
                            class:past-track={isPast && !isPlaying}
                            class:is-dragging={isDragging}
                            class:drag-over-top={isOverTop}
                            class:drag-over-bottom={isOverBottom}
                            class:just-reordered={justReorderedIndex === i}
                            role="button"
                            tabindex="0"
                            draggable="true"
                            style="position: absolute; top: 0; left: 0; width: 100%; transform: translateY({row.start}px);"
                            ondragstart={(e) => handleDragStart(e, i)}
                            ondragover={(e) => handleDragOver(e, i)}
                            ondragleave={handleDragLeave}
                            ondrop={(e) => handleDrop(e, i)}
                            ondragend={() => {
                                dragoverIndex = -1;
                                dropPosition = null;
                                draggedIndex = -1;
                            }}
                            onclick={() => jumpToTrack(track.instanceId)}
                            onkeydown={(e) => e.key === 'Enter' && jumpToTrack(track.instanceId)}
                        >
                            <span class="row-num">
                                {#if isPlaying}
                                    <span class="playing-indicator">
                                        {#if audioStore.playbackState === "Playing"}
                                            <EqualizerWave />
                                        {:else}
                                            <Pause size={18} weight="bold" color="var(--echo-primary)" />
                                        {/if}
                                    </span>
                                {:else}
                                    {i + 1}
                                {/if}
                            </span>
                            <div class="row-info">
                                <span class="row-title">{track.title}</span>
                                <span class="row-artist">{track.artist || "Unknown"}</span>
                            </div>
                            <div class="row-actions">
                                <span class="drag-handle" title="Drag to reorder">
                                    <DotsSixVertical size={16} weight="bold" />
                                </span>
                            </div>
                        </div>
                    {/each}
                </div>
            </div>
        {/if}
    </div>

    {#if showSaveModal}
        <PromptModal
            title="Save Queue as Playlist"
            defaultValue={getDefaultPlaylistName()}
            onSubmit={handleSaveQueue}
            onClose={() => (showSaveModal = false)}
        />
    {/if}
{/if}

<style>
    .queue-view {
        display: flex;
        flex-direction: column;
        height: 100%;
        width: 100%;
        padding-bottom: var(--drawer-scroll-padding, 8rem);
        scroll-padding-bottom: var(--drawer-scroll-padding, 8rem);
    }

    .queue-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 1.25rem;
        border-bottom: 1px solid var(--echo-border);
        flex-shrink: 0;
    }

    .queue-title {
        font-size: 0.8rem;
        font-weight: 600;
        letter-spacing: 0.06em;
        text-transform: uppercase;
        color: var(--echo-text-2);
    }

    .header-actions {
        display: flex;
        gap: 4px;
        align-items: center;
    }

    .icon-btn {
        background: transparent;
        border: none;
        color: var(--echo-text-3);
        padding: 0.3rem;
        border-radius: 6px;
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        transition: all 0.12s ease;
    }

    .icon-btn:hover:not(:disabled) {
        color: var(--echo-text-1);
        background: rgba(255 255 255 / 0.07);
    }

    .icon-btn:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }

    .icon-btn:active:not(:disabled) {
        transform: scale(0.94);
    }

    .queue-empty {
        flex: 1;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        gap: 0.4rem;
        padding: 2rem;
        text-align: center;
    }

    .empty-label {
        font-size: 0.85rem;
        color: var(--echo-text-2);
        font-weight: 500;
    }

    .empty-hint {
        font-size: 0.75rem;
        color: var(--echo-text-3);
        max-width: 200px;
        line-height: 1.5;
    }

    .virtual-list-container {
        flex: 1;
        overflow-y: auto;
        padding: 0.375rem 0;
    }

    .virtual-list-container.is-reordering .queue-row * {
        pointer-events: none;
    }

    .virtual-list-container.is-reordering .queue-row {
        pointer-events: auto;
    }

    .queue-row {
        position: relative;
        display: flex;
        align-items: center;
        gap: 0.85rem;
        padding: 0.6rem 1.25rem;
        cursor: pointer;
        transition: background 0.12s ease, opacity 0.15s ease, transform 0.15s ease;
        user-select: none;
    }

    .queue-row:hover {
        background: rgba(255 255 255 / 0.04);
    }

    .queue-row.is-dragging {
        opacity: 0.25;
        background: rgba(255 255 255 / 0.02);
    }

    /* Precision Gliding Insertion Line Indicator */
    .queue-row.drag-over-top::before {
        content: '';
        position: absolute;
        top: -1px;
        left: 1.25rem;
        right: 1.25rem;
        height: 2px;
        background: var(--echo-primary);
        box-shadow: 0 0 10px var(--echo-primary);
        border-radius: 2px;
        pointer-events: none;
        z-index: 20;
    }

    .queue-row.drag-over-top::after {
        content: '';
        position: absolute;
        top: -3px;
        left: 1.15rem;
        width: 6px;
        height: 6px;
        border-radius: 50%;
        background: var(--echo-primary);
        box-shadow: 0 0 8px var(--echo-primary);
        pointer-events: none;
        z-index: 21;
    }

    .queue-row.drag-over-bottom::before {
        content: '';
        position: absolute;
        bottom: -1px;
        left: 1.25rem;
        right: 1.25rem;
        height: 2px;
        background: var(--echo-primary);
        box-shadow: 0 0 10px var(--echo-primary);
        border-radius: 2px;
        pointer-events: none;
        z-index: 20;
    }

    .queue-row.drag-over-bottom::after {
        content: '';
        position: absolute;
        bottom: -3px;
        left: 1.15rem;
        width: 6px;
        height: 6px;
        border-radius: 50%;
        background: var(--echo-primary);
        box-shadow: 0 0 8px var(--echo-primary);
        pointer-events: none;
        z-index: 21;
    }

    .queue-row.playing {
        background: rgba(255 255 255 / 0.04);
    }

    .queue-row.past-track {
        opacity: 0.45;
        filter: grayscale(40%);
    }

    .queue-row.just-reordered {
        background: rgba(255, 255, 255, 0.08);
        animation: pulse-highlight 0.35s ease-out;
    }

    @keyframes pulse-highlight {
        0% {
            background: rgba(255, 255, 255, 0.14);
        }
        50% {
            background: rgba(255, 255, 255, 0.08);
        }
        100% {
            background: rgba(255, 255, 255, 0.04);
        }
    }

    .row-num {
        font-size: 0.72rem;
        color: var(--echo-text-3);
        width: 1.5rem;
        text-align: right;
        font-variant-numeric: tabular-nums;
        display: flex;
        align-items: center;
        justify-content: flex-end;
    }

    .drag-handle {
        background: transparent;
        border: none;
        display: flex;
        align-items: center;
        justify-content: center;
        color: var(--echo-text-3);
        opacity: 0;
        padding: 4px;
        border-radius: 4px;
        cursor: grab;
        transition: opacity 0.12s ease, color 0.12s ease, background 0.12s ease;
    }

    .queue-row:hover .drag-handle {
        opacity: 0.6;
    }

    .drag-handle:hover {
        opacity: 1 !important;
        color: var(--echo-text-1);
        background: rgba(255 255 255 / 0.08);
    }

    .drag-handle:active {
        cursor: grabbing;
    }
</style>
