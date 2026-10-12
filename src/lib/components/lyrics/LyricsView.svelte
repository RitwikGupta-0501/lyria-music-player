<script lang="ts">
    import { lyricsStore, type LyricLine } from "$lib/stores/lyrics.svelte";
    import { audioStore } from "$lib/stores/audio.svelte";
    import { MicrophoneStage, ArrowClockwise, Sparkle } from "phosphor-svelte";

    let { variant = "drawer" } = $props<{
        variant?: "drawer" | "fullscreen";
    }>();

    let containerEl = $state<HTMLElement | null>(null);
    let scrollTimeout: ReturnType<typeof setTimeout> | null = null;
    let manualScrollPause = $state(false);

    // Auto-scroll the active line into center view
    $effect(() => {
        const activeIdx = lyricsStore.activeLineIndex;
        if (activeIdx < 0 || manualScrollPause || !containerEl) return;

        const el = containerEl.querySelector(`[data-index="${activeIdx}"]`) as HTMLElement | null;
        if (el) {
            el.scrollIntoView({
                behavior: "smooth",
                block: "center",
            });
        }
    });

    // Detect user manual scroll to prevent fighting the auto-scroll for 3.5 seconds
    function handleUserScroll() {
        manualScrollPause = true;
        if (scrollTimeout) clearTimeout(scrollTimeout);
        scrollTimeout = setTimeout(() => {
            manualScrollPause = false;
        }, 3500);
    }

    function handleLineClick(index: number) {
        lyricsStore.seekToLine(index);
    }

    function retryWithProvider(providerId: string) {
        lyricsStore.fetchForCurrentTrack(providerId);
    }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
    class="lyrics-container {variant}"
    bind:this={containerEl}
    onwheel={handleUserScroll}
    ontouchstart={handleUserScroll}
>
    {#if lyricsStore.isLoading}
        <div class="status-box">
            <div class="spinner"></div>
            <p class="status-text">Fetching lyrics...</p>
        </div>
    {:else if lyricsStore.error}
        <div class="status-box">
            <MicrophoneStage size={36} weight="light" class="status-icon" />
            <p class="status-title">Unable to Load Lyrics</p>
            <p class="status-desc">{lyricsStore.error}</p>
            <button class="retry-btn" onclick={() => lyricsStore.fetchForCurrentTrack()}>
                <ArrowClockwise size={16} />
                <span>Retry</span>
            </button>
        </div>
    {:else if !lyricsStore.hasLyrics}
        <div class="status-box">
            <MicrophoneStage size={40} weight="thin" class="status-icon" />
            <p class="status-title">
                {lyricsStore.currentLyrics?.instrumental ? "Instrumental Track" : "No Lyrics Available"}
            </p>
            <p class="status-desc">
                {audioStore.currentQueueTrack?.artist || "Unknown Artist"} — {audioStore.currentQueueTrack?.title || "Unknown Title"}
            </p>

            <!-- Provider fallback pills -->
            <div class="provider-switch">
                <span class="provider-switch-label">Try searching with:</span>
                <div class="provider-pills">
                    {#each lyricsStore.providers as p}
                        <button
                            class="provider-pill"
                            class:active={lyricsStore.currentLyrics?.provider_id === p.id}
                            onclick={() => retryWithProvider(p.id)}
                        >
                            <Sparkle size={12} weight="fill" />
                            <span>{p.name.replace(" Lyrics Provider", "")}</span>
                        </button>
                    {/each}
                </div>
            </div>
        </div>
    {:else if lyricsStore.isSynced}
        <div class="lyrics-list">
            {#each lyricsStore.currentLyrics?.lines || [] as line, index}
                {@const isActive = index === lyricsStore.activeLineIndex}
                {@const isPast = lyricsStore.activeLineIndex >= 0 && index < lyricsStore.activeLineIndex}
                <button
                    class="lyric-line"
                    class:active={isActive}
                    class:past={isPast}
                    data-index={index}
                    onclick={() => handleLineClick(index)}
                    title="Click to jump to line"
                >
                    <span class="line-text">{line.text || "♪"}</span>
                </button>
            {/each}

            {#if lyricsStore.currentLyrics?.provider_name}
                <footer class="lyrics-footer">
                    <span>Lyrics provided by {lyricsStore.currentLyrics.provider_name}</span>
                </footer>
            {/if}
        </div>
    {:else}
        <!-- Plain unsynchronized lyrics -->
        <div class="plain-lyrics">
            <pre class="plain-text">{lyricsStore.currentLyrics?.raw_plain || ""}</pre>
            {#if lyricsStore.currentLyrics?.provider_name}
                <footer class="lyrics-footer">
                    <span>Lyrics provided by {lyricsStore.currentLyrics.provider_name}</span>
                </footer>
            {/if}
        </div>
    {/if}
</div>

<style>
    .lyrics-container {
        height: 100%;
        width: 100%;
        overflow-y: auto;
        display: flex;
        flex-direction: column;
        scroll-behavior: smooth;
        scrollbar-width: thin;
        scrollbar-color: rgba(255, 255, 255, 0.1) transparent;
        padding: 2rem 1.5rem;
        box-sizing: border-box;
    }

    .lyrics-container.fullscreen {
        padding: 4rem 3rem;
        align-items: flex-start;
        justify-content: flex-start;
    }

    .lyrics-list {
        display: flex;
        flex-direction: column;
        gap: 1.25rem;
        width: 100%;
        padding-top: 30vh;
        padding-bottom: 45vh;
    }

    .lyric-line {
        background: transparent;
        border: none;
        text-align: left;
        padding: 0.5rem 0.75rem;
        border-radius: 0.5rem;
        cursor: pointer;
        color: var(--echo-text-2, #888);
        font-family: var(--echo-font-body, inherit);
        font-size: 1.15rem;
        line-height: 1.5;
        font-weight: 500;
        transition: all 0.25s cubic-bezier(0.16, 1, 0.3, 1);
        user-select: text;
        width: 100%;
    }

    .fullscreen .lyric-line {
        font-size: 1.85rem;
        line-height: 1.6;
        padding: 0.75rem 1rem;
    }

    .lyric-line:hover {
        color: var(--echo-text-1, #eee);
        background-color: rgba(255, 255, 255, 0.04);
    }

    .lyric-line.active {
        color: #ffffff;
        font-weight: 700;
        font-size: 1.35rem;
        transform: scale(1.02);
        transform-origin: left center;
        text-shadow: 0 0 16px rgba(255, 255, 255, 0.25);
    }

    .fullscreen .lyric-line.active {
        font-size: 2.25rem;
        text-shadow: 0 0 24px rgba(255, 255, 255, 0.35);
    }

    .lyric-line.past {
        color: rgba(255, 255, 255, 0.35);
    }

    .line-text {
        display: inline-block;
    }

    .plain-lyrics {
        padding-top: 2rem;
        padding-bottom: 4rem;
        width: 100%;
    }

    .plain-text {
        font-family: var(--echo-font-body, inherit);
        font-size: 1.1rem;
        line-height: 1.8;
        color: var(--echo-text-1, #e0e0e0);
        white-space: pre-wrap;
        margin: 0;
    }

    .status-box {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        margin: auto;
        text-align: center;
        gap: 1rem;
        max-width: 320px;
        padding: 2rem 1rem;
    }

    .status-icon {
        color: var(--echo-text-2, #666);
    }

    .status-title {
        font-size: 1.1rem;
        font-weight: 600;
        color: var(--echo-text-1, #eee);
        margin: 0;
    }

    .status-desc {
        font-size: 0.875rem;
        color: var(--echo-text-2, #888);
        margin: 0;
    }

    .status-text {
        font-size: 0.95rem;
        color: var(--echo-text-2, #aaa);
    }

    .spinner {
        width: 28px;
        height: 28px;
        border: 2px solid rgba(255, 255, 255, 0.1);
        border-top-color: var(--echo-primary, #fff);
        border-radius: 50%;
        animation: spin 0.8s linear infinite;
    }

    @keyframes spin {
        to {
            transform: rotate(360deg);
        }
    }

    .retry-btn {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        padding: 0.5rem 1rem;
        background: rgba(255, 255, 255, 0.08);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 9999px;
        color: var(--echo-text-1, #fff);
        font-size: 0.85rem;
        cursor: pointer;
        transition: all 0.2s;
    }

    .retry-btn:hover {
        background: rgba(255, 255, 255, 0.15);
    }

    .provider-switch {
        margin-top: 1rem;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 0.6rem;
    }

    .provider-switch-label {
        font-size: 0.75rem;
        text-transform: uppercase;
        letter-spacing: 0.05em;
        color: var(--echo-text-2, #777);
    }

    .provider-pills {
        display: flex;
        flex-wrap: wrap;
        gap: 0.5rem;
        justify-content: center;
    }

    .provider-pill {
        display: flex;
        align-items: center;
        gap: 0.35rem;
        background: rgba(255, 255, 255, 0.05);
        border: 1px solid rgba(255, 255, 255, 0.08);
        color: var(--echo-text-1, #ddd);
        padding: 0.35rem 0.75rem;
        border-radius: 9999px;
        font-size: 0.8rem;
        cursor: pointer;
        transition: all 0.2s;
    }

    .provider-pill:hover,
    .provider-pill.active {
        background: rgba(255, 255, 255, 0.12);
        border-color: rgba(255, 255, 255, 0.25);
        color: #fff;
    }

    .lyrics-footer {
        padding-top: 2rem;
        padding-bottom: 1rem;
        text-align: center;
        font-size: 0.75rem;
        color: var(--echo-text-2, #666);
        text-transform: uppercase;
        letter-spacing: 0.04em;
    }
</style>
