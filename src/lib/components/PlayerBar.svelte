<script lang="ts">
    import { audioStore } from "$lib/stores/audio.svelte";
    import { settingsStore } from "$lib/stores/settings.svelte";
    import { libraryStore, getCanonicalKey } from "$lib/stores/library.svelte";
    import {
        Play,
        Pause,
        SkipBack,
        SkipForward,
        Shuffle,
        Repeat,
        RepeatOnce,
        SpeakerHigh,
        SpeakerX,
        ListNumbers,
        CornersOut,
        Heart,
    } from "phosphor-svelte";
    
    let { queueOpen = $bindable(false), fullScreenOpen = $bindable(false) } =
        $props<{
            queueOpen?: boolean;
            fullScreenOpen?: boolean;
        }>();

    /* ── Album art for now-playing track ── */
    let playerArtUrl = $state<string | null>(null);
    let currentArtReqId = 0;

    $effect(() => {
        const track = audioStore.currentQueueTrack;
        const reqId = ++currentArtReqId;
        if (
            track &&
            track.source.type === "Local" &&
            track.source.track_id &&
            track.source.file_path
        ) {
            libraryStore
                .getArtworkUrl(track.source.track_id, track.source.file_path)
                .then((url) => {
                    if (reqId === currentArtReqId) playerArtUrl = url;
                })
                .catch(() => {
                    if (reqId === currentArtReqId) playerArtUrl = null;
                });
        } else if (
            track &&
            track.source.type === "Remote" &&
            track.source.cover_art_url
        ) {
            playerArtUrl = track.source.cover_art_url;
        } else {
            playerArtUrl = null;
        }
    });

    /* ── Playback helpers ── */
    function handlePlayPause() {
        if (audioStore.playbackState === "Playing") {
            audioStore.pause();
        } else {
            audioStore.play();
        }
    }

    // -- Seek State --
    let isSeeking = $state(false);
    let seekProgress = $state(0);

    let progress = $derived(
        audioStore.duration > 0
            ? (audioStore.currentTime / audioStore.duration) * 100
            : 0,
    );

    let displayProgress = $derived(isSeeking ? seekProgress : progress);

    let volumeProgress = $derived(
        audioStore.isMuted ? 0 : audioStore.volume * 100,
    );

    function formatTime(seconds: number): string {
        if (isNaN(seconds) || seconds < 0) return "0:00";
        const m = Math.floor(seconds / 60);
        const s = Math.floor(seconds % 60);
        return `${m}:${s.toString().padStart(2, "0")}`;
    }

    function handleSeekPointerDown() {
        if (audioStore.duration <= 0) return;
        isSeeking = true;
        seekProgress = progress;
    }

    function handleSeekInput(e: Event) {
        if (audioStore.duration <= 0) return;
        isSeeking = true;
        const input = e.target as HTMLInputElement;
        seekProgress = parseFloat(input.value);
    }

    function handleSeekChange(e: Event) {
        if (audioStore.duration <= 0) return;
        const input = e.target as HTMLInputElement;
        const pct = parseFloat(input.value) / 100;
        audioStore.seek(pct * audioStore.duration);
        isSeeking = false;
    }

    function handleVolume(e: Event) {
        const input = e.target as HTMLInputElement;
        audioStore.setVolume(parseFloat(input.value));
    }

    let trackTitle = $derived.by(() => {
        if (audioStore.currentQueueTrack)
            return audioStore.currentQueueTrack.title;
        if (audioStore.currentTrack !== "None") return audioStore.currentTrack;
        return "Nothing playing";
    });

    let isPlaying = $derived(audioStore.playbackState === "Playing");
    let hasTrack = $derived(
        audioStore.currentQueueTrack !== null ||
            audioStore.currentTrack !== "None",
    );
    let pillState = $derived(
        hasTrack ? (isPlaying ? "playing" : "paused") : "idle",
    );
</script>

<div
    class="player-pill-container"
    style="--pill-bg: {settingsStore.glassyPlayerBar
        ? 'var(--liquid-glass-card-bg)'
        : 'var(--echo-surface)'};"
>
    <div class="player-pill-wrapper" data-state={pillState}>
        <!-- 1. Fitts's Law Top-Edge Seek Bar -->
        <div class="seek-hitbox group">
            <div class="seek-track">
                <div
                    class="seek-fill pill-accent-bg"
                    style="width: {displayProgress}%"
                ></div>
            </div>
            <div
                class="seek-thumb pill-accent-bg"
                style="left: {displayProgress}%"
            ></div>
            <input
                type="range"
                class="range-overlay"
                min="0"
                max="100"
                step="0.1"
                value={displayProgress}
                onpointerdown={handleSeekPointerDown}
                oninput={handleSeekInput}
                onchange={handleSeekChange}
                aria-label="Seek"
            />
        </div>

        <!-- 2. Main Pill Body (Liquid Glass Capsule) -->
        <div class="pill-body" class:is-glass={settingsStore.glassyPlayerBar} class:liquid-glass-hybrid={settingsStore.glassyPlayerBar}>
            <!-- Liquid Glass Specular Catch-Lights (Only in Glass Mode) -->
            {#if settingsStore.glassyPlayerBar}
                <div class="liquid-specular-rim player-specular-rim" aria-hidden="true"></div>
            {/if}

            <!-- Left Flank: Album Art & Song Details -->
            <div class="flank flank-left">
                <div class="album-art-container">
                    {#if playerArtUrl}
                        <img src={playerArtUrl} alt="Now playing artwork" />
                    {:else}
                        <div
                            class="placeholder"
                            style="background-color: #27272a;"
                        ></div>
                    {/if}
                </div>

                <div class="song-details">
                    <div class="song-title-row">
                        <span class="song-title">
                            {trackTitle}
                        </span>
                        {#if hasTrack}
                            {@const currentTrackObj = audioStore.currentQueueTrack}
                            {@const trackTitleStr = currentTrackObj?.title || (audioStore.currentTrack !== "None" ? audioStore.currentTrack : "")}
                            {@const trackArtistStr = currentTrackObj?.artist || "unknown"}
                            {@const trackKey = getCanonicalKey({ title: trackTitleStr, artist: trackArtistStr })}
                            {@const isLiked = libraryStore.likedSongs.some(s => s.canonical_key.toLowerCase().trim() === trackKey.toLowerCase().trim())}
                            <button 
                                class="like-btn" 
                                class:is-glass={settingsStore.glassyPlayerBar}
                                class:liked={isLiked}
                                onclick={() => {
                                    if (trackTitleStr) {
                                        libraryStore.toggleLike({
                                            title: trackTitleStr,
                                            artist: trackArtistStr,
                                            canonical_key: trackKey,
                                            cover_art_url: (currentTrackObj?.source as any)?.cover_art_url || playerArtUrl || undefined,
                                            duration_ms: audioStore.duration ? Math.floor(audioStore.duration * 1000) : undefined,
                                        });
                                    }
                                }}
                                title={isLiked ? "Unlike" : "Like song"}
                                aria-label={isLiked ? "Unlike song" : "Like song"}
                            >
                                <Heart size={13} weight={isLiked ? "fill" : "bold"} color={isLiked ? "#ffd285" : "rgba(255,255,255,0.7)"} />
                            </button>
                        {/if}
                    </div>
                    <div class="song-time">
                        <span>{formatTime(audioStore.currentTime)}</span>
                        <span class="text-white-20">/</span>
                        <span>{formatTime(audioStore.duration)}</span>
                    </div>
                </div>
            </div>

            <!-- Center: Transport Controls -->
            <div class="center-controls">
                <button
                    class="ctrl-btn"
                    class:active={audioStore.shuffleEnabled}
                    onclick={() => audioStore.toggleShuffle()}
                    title="Shuffle"
                >
                    <Shuffle size={16} weight="bold" />
                </button>
                <button class="ctrl-btn" onclick={() => audioStore.previous()} title="Previous track">
                    <SkipBack size={20} weight="fill" />
                </button>

                <button
                    class="play-pause-btn pill-accent-bg pill-accent-shadow"
                    onclick={handlePlayPause}
                    disabled={!hasTrack}
                    title={isPlaying ? "Pause" : "Play"}
                >
                    {#if isPlaying}
                        <Pause size={20} weight="fill" />
                    {:else}
                        <Play size={20} weight="fill" />
                    {/if}
                </button>

                <button class="ctrl-btn" onclick={() => audioStore.next()} title="Next track">
                    <SkipForward size={20} weight="fill" />
                </button>
                <button
                    class="ctrl-btn"
                    class:active={audioStore.repeatMode !== "Off"}
                    onclick={() => audioStore.cycleRepeat()}
                    title="Repeat"
                >
                    {#if audioStore.repeatMode === "One"}
                        <RepeatOnce size={16} weight="bold" />
                    {:else}
                        <Repeat size={16} weight="bold" />
                    {/if}
                </button>
            </div>

            <!-- Right Flank: Utilities -->
            <div class="flank flank-right">
                <button
                    class="ctrl-btn"
                    class:active={fullScreenOpen}
                    onclick={() => (fullScreenOpen = true)}
                    title="Fullscreen Player"
                >
                    <CornersOut size={16} weight="bold" />
                </button>
                <button
                    class="ctrl-btn"
                    class:active={queueOpen}
                    onclick={() => { queueOpen = !queueOpen; }}
                    title="Queue"
                >
                    <ListNumbers size={16} weight="bold" />
                </button>

                <div class="vol-wrapper group">
                    <button
                        class="vol-icon text-muted"
                        onclick={() => audioStore.toggleMute()}
                        title={audioStore.isMuted ? "Unmute" : "Mute"}
                    >
                        {#if audioStore.isMuted || audioStore.volume === 0}
                            <SpeakerX size={16} weight="bold" />
                        {:else}
                            <SpeakerHigh size={16} weight="bold" />
                        {/if}
                    </button>
                    <div class="vol-hitbox">
                        <div class="vol-track">
                            <div
                                class="vol-fill pill-accent-bg"
                                style="width: {volumeProgress}%"
                            ></div>
                        </div>
                        <div
                            class="vol-thumb pill-accent-bg"
                            style="left: {volumeProgress}%"
                        ></div>
                        <input
                            type="range"
                            class="range-overlay"
                            min="0"
                            max="1"
                            step="0.01"
                            value={audioStore.isMuted ? 0 : audioStore.volume}
                            oninput={handleVolume}
                            aria-label="Volume"
                        />
                    </div>
                </div>
            </div>
        </div>
    </div>
</div>

<style>
    :global(body) {
        --text-main: var(--echo-text-1);
        --muted: var(--echo-text-2);
    }

    .text-muted {
        color: var(--muted);
    }
    .text-white-20 {
        color: rgba(255, 255, 255, 0.2);
    }

    .player-pill-container {
        position: fixed;
        bottom: 2rem;
        left: var(--sidebar-w, 80px);
        right: 0px;
        height: 72px;
        z-index: 100;
        display: flex;
        justify-content: center;
        pointer-events: none;
    }

        .player-pill-wrapper {
        position: relative;
        height: 100%;
        transition:
            width 0.5s cubic-bezier(0.16, 1, 0.3, 1),
            background-color 0.4s ease;
        pointer-events: auto;
    }

    .player-pill-wrapper[data-state="idle"] {
        width: 340px;
    }
    .player-pill-wrapper[data-state="playing"],
    .player-pill-wrapper[data-state="paused"] {
        width: 720px;
        max-width: calc(100% - 2rem);
    }

    /* Fitts's Law Top-Edge Seek Bar */
    .seek-hitbox {
        position: absolute;
        top: -10px;
        left: 32px;
        right: 32px;
        height: 20px;
        cursor: pointer;
        z-index: 20;
    }
    .seek-track {
        position: absolute;
        top: 50%;
        left: 0;
        width: 100%;
        height: 2px;
        background-color: rgba(255, 255, 255, 0.08);
        border-top-left-radius: 9999px;
        border-top-right-radius: 9999px;
        transform: translateY(-50%);
        transition: all 0.2s var(--ease-liquid, ease);
        overflow: hidden;
    }
    .seek-hitbox:hover .seek-track {
        height: 4px;
    }
    .seek-fill {
        height: 100%;
        border-top-right-radius: 9999px;
        border-bottom-right-radius: 9999px;
        box-shadow: 0 0 8px rgba(226, 169, 115, 0.35);
    }
    .seek-thumb {
        position: absolute;
        top: 50%;
        transform: translate(-50%, -50%) scale(0.5);
        width: 10px;
        height: 10px;
        border-radius: 50%;
        opacity: 0;
        box-shadow: 0 0 8px rgba(226, 169, 115, 0.5);
        transition:
            opacity 0.2s,
            transform 0.2s var(--ease-liquid, ease);
        pointer-events: none;
    }
    .seek-hitbox:hover .seek-thumb {
        opacity: 1;
        transform: translate(-50%, -50%) scale(1);
    }
    .range-overlay {
        position: absolute;
        inset: 0;
        width: 100%;
        height: 100%;
        opacity: 0;
        cursor: pointer;
        margin: 0;
    }

    /* Main Pill Body (Crystalline Glass Sheet) */
    .pill-body {
        width: 100%;
        height: 100%;
        border-radius: 9999px;
        overflow: hidden;
        display: flex;
        align-items: center;
        justify-content: center;
        position: relative;
        z-index: 10;
        background-color: var(--pill-bg);
        border: 1px solid rgba(255, 255, 255, 0.08);
        box-shadow: 0 8px 32px rgba(0, 0, 0, 0.35);
        transition: all 0.3s var(--ease-liquid, ease);
    }

    .pill-body.is-glass {
        background: var(--liquid-glass-card-bg);
        backdrop-filter: blur(var(--liquid-glass-card-blur)) 
                         saturate(var(--liquid-glass-saturate)) 
                         contrast(var(--liquid-glass-contrast)) 
                         brightness(var(--liquid-glass-brightness));
        -webkit-backdrop-filter: blur(var(--liquid-glass-card-blur)) 
                                 saturate(var(--liquid-glass-saturate)) 
                                 contrast(var(--liquid-glass-contrast)) 
                                 brightness(var(--liquid-glass-brightness));
        border: 1.5px solid transparent;
        border-color: var(--liquid-border-card);
        box-shadow: inset 0 1.5px 0 0 rgba(255, 255, 255, 0.50), inset 0 -2px 5px 0 rgba(0, 0, 0, 0.28), 0 14px 38px -4px rgba(0, 0, 0, 0.48);
    }

    /* Active & Paused Track Caustic Internal Scatter */
    .player-pill-wrapper[data-state="playing"] .pill-body.is-glass,
    .player-pill-wrapper[data-state="paused"] .pill-body.is-glass {
        background: var(--liquid-glass-amber-bg);
        border-color: var(--liquid-border-amber);
        box-shadow: inset 0 1.5px 0 0 rgba(255, 235, 205, 0.60), inset 0 -2px 5px 0 rgba(0, 0, 0, 0.30), 0 16px 44px -4px rgba(0, 0, 0, 0.55);
    }

    /* 135deg Sub-surface Ambient Light Sheen */
    
    /* Glassmorphic Controls: Luminous on Playing, Dull on Paused/Idle */
    .pill-body.is-glass .ctrl-btn {
        color: rgba(255, 255, 255, 0.48);
        filter: drop-shadow(0 2px 6px rgba(0, 0, 0, 0.6));
        transition:
            color 0.25s ease,
            filter 0.25s ease,
            background-color 0.2s var(--ease-liquid, ease),
            transform 0.15s var(--ease-liquid, ease);
    }

    .player-pill-wrapper[data-state="playing"] .pill-body.is-glass .ctrl-btn {
        color: rgba(255, 255, 255, 0.85);
        filter: drop-shadow(0 2px 8px rgba(0, 0, 0, 0.8));
    }

    .pill-body.is-glass .vol-icon {
        color: rgba(255, 255, 255, 0.48);
        filter: drop-shadow(0 2px 6px rgba(0, 0, 0, 0.6));
        transition: color 0.25s ease, filter 0.25s ease, transform 0.15s var(--ease-liquid, ease);
    }

    .player-pill-wrapper[data-state="playing"] .pill-body.is-glass .vol-icon {
        color: rgba(255, 255, 255, 0.85);
        filter: drop-shadow(0 2px 8px rgba(0, 0, 0, 0.8));
    }

    .pill-body.is-glass .song-time {
        color: rgba(255, 255, 255, 0.45);
        transition: color 0.25s ease;
    }

    .player-pill-wrapper[data-state="playing"] .pill-body.is-glass .song-time {
        color: rgba(255, 255, 255, 0.70);
    }

    .pill-body.is-glass .vol-track {
        height: 2px;
        background-color: rgba(0, 0, 0, 0.35);
        box-shadow: 0 1px 2px rgba(0, 0, 0, 0.35);
        transition: background-color 0.25s ease;
    }

    .player-pill-wrapper[data-state="playing"] .pill-body.is-glass .vol-track {
        background-color: rgba(0, 0, 0, 0.35);
        box-shadow: 0 1px 2px rgba(0, 0, 0, 0.40);
    }

    .pill-body.is-glass .vol-fill {
        background: linear-gradient(90deg, #c8955c 0%, #e2a973 100%) !important;
        box-shadow: 0 0 4px rgba(226, 169, 115, 0.35);
    }

    .player-pill-wrapper[data-state="playing"] .pill-body.is-glass .vol-fill {
        background: linear-gradient(90deg, #d49b65 0%, #f5cb99 100%) !important;
        box-shadow: 0 0 4px rgba(226, 169, 115, 0.40);
    }

    .pill-body.is-glass .vol-thumb {
        background: radial-gradient(circle at 35% 35%, #ffdca8 0%, #e2a973 70%, #b87b42 100%) !important;
        box-shadow: 0 0 6px rgba(226, 169, 115, 0.5), 0 1px 3px rgba(0, 0, 0, 0.5);
    }

    .pill-body.is-glass .ctrl-btn:hover {
        color: #ffffff !important;
        background-color: rgba(255, 255, 255, 0.14);
        border-color: rgba(255, 255, 255, 0.20);
    }

    .pill-body.is-glass .vol-wrapper:hover .vol-icon {
        color: #ffffff !important;
    }

    .pill-body.is-glass::after {
        content: "";
        position: absolute;
        inset: 0;
        border-radius: inherit;
        pointer-events: none;
        background: linear-gradient(
            135deg, 
            rgba(255, 255, 255, 0.10) 0%, 
            rgba(255, 255, 255, 0.02) 30%, 
            rgba(255, 255, 255, 0) 65%
        );
        z-index: 2;
    }

    /* Soft & Elegant Crest Highlight (Glass Mode Only) */
    .player-specular-rim {
        inset-inline: 24px;
        top: 0;
        height: 2.5px;
        opacity: 0.80;
        background: linear-gradient(90deg, transparent 0%, rgba(255, 255, 255, 0.85) 50%, transparent 100%);
        pointer-events: none;
        z-index: 4;
        transition: opacity 0.3s ease, background 0.3s ease;
    }

    .player-pill-wrapper[data-state="playing"] .player-specular-rim,
    .player-pill-wrapper[data-state="paused"] .player-specular-rim {
        opacity: 0.95;
        background: linear-gradient(
            90deg, 
            transparent 0%, 
            rgba(255, 215, 165, 0.50) 25%, 
            rgba(255, 235, 205, 0.90) 50%, 
            rgba(255, 215, 165, 0.50) 75%, 
            transparent 100%
        );
    }

    /* Flanks */
    .flank {
        opacity: 0;
        pointer-events: none;
        transition: opacity 0.3s ease;
        transition-delay: 0s;
    }
    .player-pill-wrapper[data-state="playing"] .flank,
    .player-pill-wrapper[data-state="paused"] .flank {
        opacity: 1;
        pointer-events: auto;
        transition-delay: 0.2s;
    }

    /* Left Flank */
    .flank-left {
        position: absolute;
        left: 24px;
        right: calc(50% + 130px);
        display: flex;
        align-items: center;
        gap: 10px;
        overflow: hidden;
        z-index: 15;
    }
    .album-art-container {
        width: 44px;
        height: 44px;
        flex-shrink: 0;
        border-radius: 12px;
        background-color: #1a1a20;
        border: 1px solid rgba(255, 255, 255, 0.15);
        overflow: hidden;
        box-shadow:
            inset 0 1px 1px rgba(255, 255, 255, 0.2),
            0 4px 12px rgba(0, 0, 0, 0.4);
    }
    .album-art-container img,
    .album-art-container .placeholder {
        width: 100%;
        height: 100%;
        object-fit: cover;
        background-position: center;
        background-size: cover;
    }
    .song-details {
        display: flex;
        flex-direction: column;
        justify-content: center;
        min-width: 0;
        width: 100%;
        padding-top: 2px;
    }
    .song-title {
        font-size: 15px;
        font-weight: 500;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        display: block;
        line-height: 1.25;
        letter-spacing: 0.025em;
        color: var(--text-main);
        text-shadow: 0 1px 4px rgba(0, 0, 0, 0.8);
    }
    .song-time {
        display: flex;
        align-items: center;
        gap: 6px;
        font-size: 11px;
        color: var(--muted);
        font-weight: 500;
        margin-top: 2px;
        text-shadow: 0 1px 4px rgba(0, 0, 0, 0.8);
    }

    /* Center Controls */
    .center-controls {
        display: flex;
        align-items: center;
        gap: 4px;
        flex-shrink: 0;
        z-index: 20;
    }
    .ctrl-btn {
        color: var(--echo-text-2);
        transition:
            color 0.15s ease,
            background-color 0.2s var(--ease-liquid, ease),
            transform 0.15s var(--ease-liquid, ease);
        width: 34px;
        height: 34px;
        padding: 0;
        display: flex;
        align-items: center;
        justify-content: center;
        border-radius: 9999px;
        background: transparent;
        border: 1.5px solid transparent;
        cursor: pointer;
        position: relative;
        filter: drop-shadow(0 2px 6px rgba(0, 0, 0, 0.8));
    }
    .ctrl-btn:hover {
        color: #ffffff;
        background-color: rgba(255, 255, 255, 0.08);
        border-color: rgba(255, 255, 255, 0.12);
        transform: scale(1.08);
    }
    .ctrl-btn:active {
        transform: scale(0.92);
    }
    .ctrl-btn.active {
        color: #f5cb99;
    }
    .ctrl-btn.active::after {
        content: "";
        position: absolute;
        bottom: 3px;
        left: 50%;
        transform: translateX(-50%);
        width: 4px;
        height: 4px;
        border-radius: 50%;
        background-color: #f5cb99;
        box-shadow: 0 0 6px rgba(245, 203, 153, 0.6);
    }

    .play-pause-btn {
        width: 44px;
        height: 44px;
        padding: 0;
        border-radius: 9999px;
        display: flex;
        align-items: center;
        justify-content: center;
        transition: all 0.25s var(--ease-liquid, ease);
        margin: 0 4px;
        color: var(--echo-void);
        border: none;
        cursor: pointer;
        box-shadow: inset 0 1px 1px rgba(255, 255, 255, 0.4), 0 4px 16px rgba(0, 0, 0, 0.4);
    }
    .play-pause-btn:not(:disabled):hover {
        transform: scale(1.08);
    }
    .play-pause-btn:not(:disabled):active {
        transform: scale(0.94);
    }
    .play-pause-btn:disabled {
        cursor: not-allowed;
    }

    /* Right Flank */
    .flank-right {
        position: absolute;
        right: 24px;
        left: calc(50% + 130px);
        display: flex;
        align-items: center;
        justify-content: flex-end;
        gap: 6px;
        overflow: visible;
        z-index: 15;
    }
    .vol-wrapper {
        display: flex;
        align-items: center;
        gap: 6px;
        cursor: pointer;
    }
    .vol-icon {
        background: transparent;
        border: none;
        padding: 0;
        display: flex;
        align-items: center;
        justify-content: center;
        cursor: pointer;
        transition: color 0.15s ease, transform 0.15s var(--ease-liquid, ease);
        filter: drop-shadow(0 2px 6px rgba(0, 0, 0, 0.8));
    }
    .vol-wrapper:hover .vol-icon {
        color: var(--text-main);
        transform: scale(1.05);
    }
    .vol-hitbox {
        position: relative;
        width: 48px;
        height: 20px;
        display: flex;
        align-items: center;
        overflow: visible;
    }
    .vol-track {
        position: absolute;
        top: 50%;
        transform: translateY(-50%);
        left: 0;
        width: 100%;
        height: 2px;
        background-color: rgba(255, 255, 255, 0.12);
        border-radius: 9999px;
        overflow: hidden;
    }
    .vol-fill {
        height: 100%;
        transition: background-color 0.4s;
    }
    .vol-thumb {
        position: absolute;
        top: 50%;
        transform: translate(-50%, -50%) scale(0.5);
        width: 8px;
        height: 8px;
        border-radius: 50%;
        opacity: 0;
        box-shadow: 0 0 6px rgba(226, 169, 115, 0.5);
        transition:
            opacity 0.2s,
            transform 0.2s;
        pointer-events: none;
    }
    .vol-wrapper:hover .vol-thumb {
        opacity: 1;
        transform: translate(-50%, -50%) scale(1);
    }

    /* State-based styling */
    /* Base / Idle State Colors */
    .player-pill-wrapper[data-state="idle"] .pill-accent-bg {
        background-color: #4a3c2b;
    }
    .player-pill-wrapper[data-state="idle"] .pill-accent-shadow {
        box-shadow: inset 0 2px 6px rgba(0, 0, 0, 0.6);
    }
    .player-pill-wrapper[data-state="idle"] .play-pause-btn {
        color: #1a140d;
    }

    /* Accent Colors & System Status Transitions */
    .pill-accent-bg,
    .pill-accent-shadow,
    .album-art-container,
    .song-title {
        transition: all 0.4s var(--ease-liquid, ease);
    }

    /* Playing State */
    .player-pill-wrapper[data-state="playing"] .pill-accent-bg {
        background: linear-gradient(180deg, #f5cb99 0%, #b58e62 100%);
    }
    .player-pill-wrapper[data-state="playing"] .pill-accent-shadow {
        box-shadow: 0 0 16px rgba(245, 203, 153, 0.45);
    }
    .player-pill-wrapper[data-state="playing"] .song-title {
        color: var(--text-main);
    }
    .player-pill-wrapper[data-state="playing"] .album-art-container {
        filter: grayscale(0%) brightness(1);
    }

    /* Paused State */
    .player-pill-wrapper[data-state="paused"] .pill-accent-bg {
        background: linear-gradient(180deg, #a88258 0%, #6e5232 100%);
    }
    .player-pill-wrapper[data-state="paused"] .pill-accent-shadow {
        box-shadow: none;
    }
    .player-pill-wrapper[data-state="paused"] .play-pause-btn :global(svg) {
        transform: translateX(1px);
    }
    .player-pill-wrapper[data-state="paused"] .song-title {
        color: var(--echo-text-3);
    }
    .player-pill-wrapper[data-state="paused"] .album-art-container {
        filter: grayscale(40%) brightness(0.6);
    }

    .song-title-row {
        display: flex;
        align-items: center;
        gap: 0.5rem;
    }

    .like-btn {
        width: 26px;
        height: 26px;
        border-radius: 50%;
        display: inline-flex;
        align-items: center;
        justify-content: center;
        background: rgba(255, 255, 255, 0.05);
        border: 1px solid rgba(255, 255, 255, 0.08);
        cursor: pointer;
        padding: 0;
        flex-shrink: 0;
        transition: transform 0.2s var(--ease-liquid, ease), background 0.15s ease, border-color 0.15s ease, box-shadow 0.15s ease;
    }

    .like-btn.is-glass {
        background: linear-gradient(180deg, rgba(255, 255, 255, 0.09) 0%, rgba(255, 255, 255, 0.03) 100%);
        border: 1.5px solid rgba(255, 255, 255, 0.12);
        border-top-color: rgba(255, 255, 255, 0.28);
        border-bottom-color: rgba(255, 255, 255, 0.08);
        box-shadow: inset 0 1px 2px rgba(255, 255, 255, 0.12), 0 2px 8px rgba(0, 0, 0, 0.3);
    }

    .like-btn:hover {
        transform: scale(1.15);
        background: rgba(255, 255, 255, 0.10);
        border-color: rgba(226, 169, 115, 0.45);
    }

    .like-btn.is-glass:hover {
        background: rgba(255, 255, 255, 0.14);
        border-color: rgba(226, 169, 115, 0.45);
        border-top-color: rgba(255, 225, 185, 0.65);
        box-shadow: inset 0 1px 2px rgba(255, 255, 255, 0.22), 0 4px 12px rgba(0, 0, 0, 0.4);
    }

    .like-btn:active {
        transform: scale(0.9);
    }

    .like-btn.liked {
        color: #ffd285;
        background: rgba(45, 35, 25, 0.85);
        border-color: rgba(224, 184, 143, 0.35);
    }

    .like-btn.is-glass.liked {
        background: linear-gradient(180deg, rgba(226, 169, 115, 0.32) 0%, rgba(185, 130, 80, 0.18) 100%);
        border: 1.5px solid rgba(224, 184, 143, 0.55);
        border-top-color: rgba(255, 235, 205, 0.85);
        border-bottom-color: rgba(160, 105, 55, 0.35);
        box-shadow: inset 0 1px 2px rgba(255, 235, 205, 0.35), 0 2px 10px rgba(226, 169, 115, 0.3);
    }
</style>
