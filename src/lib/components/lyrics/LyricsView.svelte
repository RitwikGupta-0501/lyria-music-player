<script lang="ts">
    import { lyricsStore, type LyricLine } from "$lib/stores/lyrics.svelte";
    import { audioStore } from "$lib/stores/audio.svelte";
    import { MicrophoneStage, ArrowClockwise, Sparkle } from "phosphor-svelte";
    import { fade } from "svelte/transition";

    let { variant = "drawer" } = $props<{
        variant?: "drawer" | "fullscreen";
    }>();

    let containerEl = $state<HTMLElement | null>(null);
    let scrollTimeout: ReturnType<typeof setTimeout> | null = null;
    let manualScrollPause = $state(false);

    // Smoothly scroll the container to center a specific line element
    function scrollToIndex(index: number, smooth = true) {
        if (!containerEl || index < 0) return;
        const el = containerEl.querySelector(`[data-index="${index}"]`) as HTMLElement | null;
        if (!el) return;

        const containerRect = containerEl.getBoundingClientRect();
        const elRect = el.getBoundingClientRect();
        const targetScrollTop =
            containerEl.scrollTop + (elRect.top - containerRect.top) - containerEl.clientHeight / 2 + elRect.height / 2;

        containerEl.scrollTo({
            top: Math.max(0, targetScrollTop),
            behavior: smooth ? "smooth" : "instant",
        });
    }

    // Auto-scroll the active line into center view
    $effect(() => {
        const activeIdx = lyricsStore.activeLineIndex;
        if (activeIdx < 0 || manualScrollPause || !containerEl) return;
        scrollToIndex(activeIdx, true);
    });

    // Detect user manual scroll to prevent fighting the auto-scroll for 4 seconds
    function handleUserScroll() {
        manualScrollPause = true;
        if (scrollTimeout) clearTimeout(scrollTimeout);
        scrollTimeout = setTimeout(() => {
            manualScrollPause = false;
        }, 4000);
    }

    function handleLineClick(index: number) {
        if (scrollTimeout) {
            clearTimeout(scrollTimeout);
            scrollTimeout = null;
        }
        manualScrollPause = false;
        lyricsStore.seekToLine(index, 150);
        scrollToIndex(index, true);
    }

    function handleInterludeClick(targetMs: number) {
        if (scrollTimeout) {
            clearTimeout(scrollTimeout);
            scrollTimeout = null;
        }
        manualScrollPause = false;
        audioStore.seek(targetMs / 1000);
    }

    function resumeSyncNow() {
        if (scrollTimeout) {
            clearTimeout(scrollTimeout);
            scrollTimeout = null;
        }
        manualScrollPause = false;
        if (lyricsStore.activeLineIndex >= 0) {
            scrollToIndex(lyricsStore.activeLineIndex, true);
        }
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
        {@const lines = lyricsStore.currentLyrics?.lines || []}
        {@const curMs = audioStore.currentTime * 1000}
        <div class="lyrics-list">
            <!-- Instrumental intro before first line -->
            {#if lines.length > 0 && lines[0].time_ms >= 6000}
                {@const isIntroActive = curMs < lines[0].time_ms}
                <button
                    type="button"
                    class="instrumental-break"
                    class:active={isIntroActive}
                    onclick={() => handleInterludeClick(0)}
                    title="Instrumental intro (jump to start)"
                >
                    <span class="break-dot"></span>
                    <span class="break-dot"></span>
                    <span class="break-dot"></span>
                </button>
            {/if}

            {#each lines as line, index}
                {@const isActive = index === lyricsStore.activeLineIndex}
                {@const isPast = lyricsStore.activeLineIndex >= 0 && index < lyricsStore.activeLineIndex}
                {@const nextLine = lines[index + 1]}
                {@const gapMs = nextLine ? nextLine.time_ms - line.time_ms : 0}

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

                <!-- Instrumental break between lines if gap >= 7 seconds -->
                {#if gapMs >= 7000}
                    {@const isBreakActive = curMs >= line.time_ms + 2500 && curMs < nextLine.time_ms}
                    <button
                        type="button"
                        class="instrumental-break"
                        class:active={isBreakActive}
                        onclick={() => handleInterludeClick(line.time_ms + 2500)}
                        title="Instrumental break (jump to section)"
                    >
                        <span class="break-dot"></span>
                        <span class="break-dot"></span>
                        <span class="break-dot"></span>
                    </button>
                {/if}
            {/each}

            {#if lyricsStore.currentLyrics?.provider_name}
                <footer class="lyrics-footer">
                    <span>Lyrics provided by {lyricsStore.currentLyrics.provider_name}</span>
                </footer>
            {/if}
        </div>

        {#if manualScrollPause && lyricsStore.activeLineIndex >= 0}
            <button
                type="button"
                class="sync-pill-btn"
                transition:fade={{ duration: 150 }}
                onclick={resumeSyncNow}
                title="Re-sync with current playback"
            >
                <Sparkle size={13} weight="fill" />
                <span>Sync with playback</span>
            </button>
        {/if}
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

    .instrumental-break {
        display: inline-flex;
        align-items: center;
        gap: 0.35rem;
        padding: 0.4rem 0.75rem;
        background: transparent;
        border: none;
        cursor: pointer;
        border-radius: 9999px;
        opacity: 0.35;
        transition: all 0.25s ease;
        align-self: flex-start;
        margin: -0.25rem 0;
    }

    .fullscreen .instrumental-break {
        padding: 0.6rem 1rem;
        gap: 0.5rem;
    }

    .instrumental-break:hover {
        opacity: 0.8;
        background-color: rgba(255, 255, 255, 0.05);
    }

    .instrumental-break.active {
        opacity: 0.95;
    }

    .break-dot {
        width: 6px;
        height: 6px;
        border-radius: 50%;
        background-color: var(--echo-text-2, #888);
        transition: all 0.3s ease;
    }

    .fullscreen .break-dot {
        width: 8px;
        height: 8px;
    }

    .instrumental-break.active .break-dot {
        background-color: #ffffff;
        box-shadow: 0 0 8px rgba(255, 255, 255, 0.4);
        animation: pulseDot 1.4s infinite ease-in-out;
    }

    .instrumental-break.active .break-dot:nth-child(2) {
        animation-delay: 0.2s;
    }

    .instrumental-break.active .break-dot:nth-child(3) {
        animation-delay: 0.4s;
    }

    @keyframes pulseDot {
        0%, 100% {
            transform: scale(0.85);
            opacity: 0.5;
        }
        50% {
            transform: scale(1.35);
            opacity: 1;
        }
    }

    .sync-pill-btn {
        position: sticky;
        bottom: 1.5rem;
        align-self: center;
        display: inline-flex;
        align-items: center;
        gap: 0.5rem;
        background: rgba(18, 18, 24, 0.88);
        backdrop-filter: blur(16px);
        -webkit-backdrop-filter: blur(16px);
        border: 1px solid rgba(255, 255, 255, 0.15);
        color: #ffffff;
        font-family: var(--echo-font-body, inherit);
        font-size: 0.8125rem;
        font-weight: 600;
        letter-spacing: 0.01em;
        padding: 0.5rem 1.125rem;
        border-radius: 9999px;
        cursor: pointer;
        box-shadow: 0 6px 20px rgba(0, 0, 0, 0.5);
        transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
        z-index: 10;
        margin-top: -2.75rem;
    }

    .sync-pill-btn:hover {
        background: rgba(30, 30, 38, 0.98);
        border-color: rgba(255, 255, 255, 0.3);
        transform: translateY(-2px);
        box-shadow: 0 8px 24px rgba(0, 0, 0, 0.65);
    }
</style>
