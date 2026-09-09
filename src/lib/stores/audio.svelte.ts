import { toastStore } from "./toast.svelte";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { settingsStore } from "./settings.svelte";

interface PlayerSyncPayload {
    state: string;
    position: number;
    duration: number;
    track: string;
}

interface QueueChangePayload {
    tracks: QueueTrack[];
    current_position: number;
    current_track: QueueTrack | null;
    repeat_mode: string;
    queue_mode: string;
}


interface TrackSource {
    type: 'Local' | 'Remote';
    // Local fields
    track_id?: number;
    file_path?: string;
    album_id?: number | null;
    // Remote fields
    provider_id?: string;
    remote_track_id?: string;
    stream_url?: string | null;
    quality_hint?: string | null;
    cover_art_url?: string | null;
    duration_ms?: number | null;
}

export interface QueueTrack {
    instanceId: string;
    title: string;
    artist: string | null;
    trackNumber?: number | null;
    source: TrackSource;
}

export class AudioStore {
    // ══════════════════════════════════════════
    // PLAYBACK STATE (NOT cached from backend)
    // ══════════════════════════════════════════

    playbackState = $state("Stopped");
    currentTrack = $state("None");
    currentTime = $state(0);
    duration = $state(0);
    volume = $state(1.0);
    isMuted = $state(false);

    trackClickBehavior = $derived(settingsStore.trackClickBehavior);
    queueCompletionBehavior = $derived(settingsStore.queueCompletionBehavior);

    // ══════════════════════════════════════════
    // QUEUE STATE (Cached from backend events)
    // ══════════════════════════════════════════

    queue = $state<QueueTrack[]>([]);
    currentPosition = $state(0);
    currentQueueId = $state<string | null>(null);
    repeatMode = $state("Off");
    shuffleEnabled = $state(false);

    // ══════════════════════════════════════════
    // INTERNAL: Interpolation for smooth playback
    // ══════════════════════════════════════════

    private _syncPosition = 0;
    private _syncTimestamp = 0;
    private _isPlaying = false;
    // _clockTimer used instead of _rafId
    private _autoAdvancing = false;
    private _stagedAutoplayTrack: QueueTrack | null = null;

    private unlistenSync: UnlistenFn | null = null;
    private unlistenTrackEnded: UnlistenFn | null = null;
    private unlistenQueueChanged: UnlistenFn | null = null;
    private _volumeSaveTimer: ReturnType<typeof setTimeout> | null = null;

    // ══════════════════════════════════════════
    // DERIVED STATE
    // ══════════════════════════════════════════

    currentQueueTrack = $derived(
        this.currentQueueId
            ? this.queue.find((t) => t.instanceId === this.currentQueueId) || null
            : null
    );

    private lastLoggedInstanceId: string | null = null;

    private logPlayback(t: QueueTrack | null | undefined) {
        if (!t) return;
        if (this.lastLoggedInstanceId === t.instanceId) return;
        this.lastLoggedInstanceId = t.instanceId;

        const isRemote = t.source.type === "Remote";
        const providerId = isRemote ? (t.source.provider_id || settingsStore.getEffectiveRemoteProvider()) : "local";
        const sourceId = isRemote ? (t.source.remote_track_id || t.instanceId) : String(t.source.track_id || t.instanceId);
        const coverArtUrl = isRemote ? (t.source.cover_art_url || null) : null;
        const durationMs = isRemote ? (t.source.duration_ms || null) : (this.duration ? Math.round(this.duration * 1000) : null);

        invoke("record_track_play", {
            title: t.title,
            artist: t.artist || "Unknown Artist",
            album: null,
            coverArtUrl,
            providerId,
            sourceId,
            durationMs,
        }).catch((err) => console.warn("Failed to record track play:", err));
    }

    // ══════════════════════════════════════════
    // LIFECYCLE
    // ══════════════════════════════════════════

    async init() {
        // Listen to playback sync events (backend → frontend)
        this.unlistenSync = await listen<PlayerSyncPayload>("player-sync", (e) => {
            const payload = e.payload;

            this._syncPosition = payload.position;
            this._syncTimestamp = performance.now();
            this._isPlaying = payload.state === "Playing";
            if (this._isPlaying) {
                this.startClock();
            } else {
                this.stopClock();
            }

            this.playbackState = payload.state;
            const fallbackDuration = this.currentQueueTrack?.source?.type === "Remote" && this.currentQueueTrack.source.duration_ms
                ? this.currentQueueTrack.source.duration_ms / 1000
                : 0;
            this.duration = payload.duration > 0 ? payload.duration : (this.duration > 0 ? this.duration : fallbackDuration);
            this.currentTrack = payload.track || "None";
            this.currentTime = payload.position;

            if (payload.state === "Playing" && this.currentQueueTrack) {
                // Log playback only after listening for >= 10s or >= 25% of duration (prevents immediate skip pollution)
                if (payload.position >= 10 || (payload.duration > 0 && (payload.position / payload.duration) >= 0.25)) {
                    this.logPlayback(this.currentQueueTrack);
                }
            }

            if (payload.state === "Stopped") {
                if (this.queueCompletionBehavior === "collapse_idle") {
                    this.currentTime = 0;
                    this.duration = 0;
                    this.currentTrack = "None";
                } else if (this.queueCompletionBehavior === "pause_end") {
                    this.playbackState = "Paused";
                    this.currentTime = this.duration;
                } else {
                    // "retain_stopped" (Default): Retain track info & duration for replay, reset position to 0
                    this.currentTime = 0;
                }
            }
        });

        // Listen to queue changes from backend
        this.unlistenQueueChanged = await listen<QueueChangePayload>(
            "queue-changed",
            (e) => {
                const payload = e.payload;
                this.queue = payload.tracks;
                this.currentPosition = payload.current_position;
                this.currentQueueId = payload.current_track?.instanceId || null;
                this.repeatMode = payload.repeat_mode;
                this.shuffleEnabled = payload.queue_mode === "Shuffle";
            }
        );

        // Track advancement detection (for gapless playback)
        this.unlistenTrackEnded = await listen("track-advanced", async () => {
            // Re-anchor the playback clock immediately on track transition
            this._autoAdvancing = true;
            this.currentTime = 0;
            this._syncPosition = 0;
            this._syncTimestamp = performance.now();

            if (this._stagedAutoplayTrack) {
                const committingTrack = this._stagedAutoplayTrack;
                this._stagedAutoplayTrack = null;
                await invoke("add_to_queue", { track: committingTrack });
                await invoke("skip_forward", { count: 1 });
            } else {
                await invoke("skip_forward", { count: 1 });
            }

            this.queueNextAudio();
        });

        // Load persisted settings via settingsStore
        try {
            await settingsStore.init();
            if (settingsStore.settings["volume"]) {
                this.volume = parseFloat(settingsStore.settings["volume"]);
                await invoke("set_volume", { volume: this.volume });
            }
            if (settingsStore.settings["mute"]) {
                this.isMuted = settingsStore.settings["mute"] === "true";
                await invoke("set_mute", { mute: this.isMuted });
            }
        } catch (e) {
            console.error("Failed to load settings:", e);
        }

        try {
            await invoke("sync_playback_state");
        } catch (e) {
            console.error("Failed to sync playback state on boot:", e);
        }

        if (this._isPlaying) {
            this.startClock();
        }
    }

    destroy() {
        if (this.unlistenSync) this.unlistenSync();
        if (this.unlistenTrackEnded) this.unlistenTrackEnded();
        if (this.unlistenQueueChanged) this.unlistenQueueChanged();
        this.stopClock();
        if (this._volumeSaveTimer) clearTimeout(this._volumeSaveTimer);
    }

    // ══════════════════════════════════════════
    // PLAYBACK CLOCK (Pure 4Hz Timer - Zero RAF overhead)
    // ══════════════════════════════════════════

    private _clockTimer: ReturnType<typeof setInterval> | null = null;

    private tick = () => {
        if (!this._isPlaying) {
            this.stopClock();
            return;
        }

        const now = performance.now();
        const elapsed = (now - this._syncTimestamp) / 1000;
        const fallbackDuration = this.currentQueueTrack?.source?.type === "Remote" && this.currentQueueTrack.source.duration_ms
            ? this.currentQueueTrack.source.duration_ms / 1000
            : 0;
        const effectiveDuration = this.duration > 0 ? this.duration : fallbackDuration;

        if (effectiveDuration > 0) {
            this.currentTime = Math.min(this._syncPosition + elapsed, effectiveDuration);
        } else {
            this.currentTime = this._syncPosition + elapsed;
        }
    };

    private startClock() {
        if (this._clockTimer === null && this._isPlaying) {
            this.tick();
            this._clockTimer = setInterval(this.tick, 250);
        }
    }

    private stopClock() {
        if (this._clockTimer !== null) {
            clearInterval(this._clockTimer);
            this._clockTimer = null;
        }
    }

    // ══════════════════════════════════════════
    // PLAYBACK COMMANDS
    // ══════════════════════════════════════════

    async play() {
        try {
            await invoke("play_audio");
        } catch (e) {
            console.error("Play failed:", e);
        }
    }

    async pause() {
        this._isPlaying = false;
        this.stopClock();
        try {
            await invoke("pause_audio");
        } catch (e) {
            console.error("Pause failed:", e);
        }
    }

    async stop() {
        this._isPlaying = false;
        this.stopClock();
        try {
            await invoke("stop_audio");
        } catch (e) {
            console.error("Stop failed:", e);
        }
    }

    async seek(position: number) {
        position = Math.max(0, Math.min(position, this.duration));

        this._syncPosition = position;
        this._syncTimestamp = performance.now();
        this.currentTime = position;

        try {
            await invoke("seek_audio", { position });
        } catch (e) {
            console.error("Seek failed:", e);
        }
    }

    // ══════════════════════════════════════════
    // VOLUME COMMANDS
    // ══════════════════════════════════════════

    async setVolume(volume: number) {
        this.volume = Math.max(0, Math.min(1, volume));

        if (this._volumeSaveTimer) clearTimeout(this._volumeSaveTimer);
        this._volumeSaveTimer = setTimeout(async () => {
            try {
                await invoke("set_setting", {
                    key: "volume",
                    value: this.volume.toString(),
                });
            } catch (e) {
                console.error("Failed to save volume:", e);
            }
        }, 500);

        try {
            await invoke("set_volume", { volume: this.volume });
        } catch (e) {
            console.error("Set volume failed:", e);
        }
    }

    async setMute(mute: boolean) {
        this.isMuted = mute;

        try {
            await invoke("set_setting", { key: "mute", value: mute.toString() });
            await invoke("set_mute", { mute });
        } catch (e) {
            console.error("Set mute failed:", e);
        }
    }

    async setTrackClickBehavior(behavior: "interrupt" | "clear" | "append") {
        this.trackClickBehavior = behavior;
        try {
            await invoke("set_setting", { key: "track_click_behavior", value: behavior });
        } catch (e) {
            console.error("Failed to save track click behavior:", e);
        }
    }

    async setQueueCompletionBehavior(behavior: "retain_stopped" | "pause_end" | "collapse_idle") {
        this.queueCompletionBehavior = behavior;
        try {
            await invoke("set_setting", { key: "queue_completion_behavior", value: behavior });
        } catch (e) {
            console.error("Failed to save queue completion behavior:", e);
        }
    }

    // ══════════════════════════════════════════
    // QUEUE COMMANDS (All via backend)
    // ══════════════════════════════════════════

    private formatQueueTrack(t: any): QueueTrack {
        const instanceId = crypto.randomUUID();
        const filePath = t.source?.file_path ?? t.file_path ?? t.filePath ?? '';
        const isRemoteUri = typeof filePath === 'string' && filePath.startsWith('remote://');

        let providerId = t.source?.provider_id ?? t.provider_id ?? t.providerId ?? '';
        let rawRemoteId = t.source?.remote_track_id ?? t.remote_track_id ?? t.remoteTrackId ?? t.id ?? null;

        if (isRemoteUri) {
            // e.g. "remote://youtube-wasm/7r63lUR6Wz4"
            const trimmed = filePath.substring('remote://'.length);
            const slashIdx = trimmed.indexOf('/');
            if (slashIdx !== -1) {
                providerId = trimmed.substring(0, slashIdx);
                rawRemoteId = trimmed.substring(slashIdx + 1);
            }
        }

        const isExplicitLocal = (t.source?.type === 'Local') || (t.type === 'Local') || (providerId === 'local' && !!filePath && !isRemoteUri);
        const isRemote = !isExplicitLocal && (
            (t.source?.type === 'Remote') ||
            (providerId && providerId !== 'local') ||
            !!(t.stream_url) ||
            (t.type === "Remote") ||
            isRemoteUri
        );

        if (!providerId || (providerId === 'local' && isRemote)) {
            providerId = isRemote ? settingsStore.getEffectiveRemoteProvider() : 'local';
        }

        if (typeof rawRemoteId === 'string' && providerId && rawRemoteId.startsWith(`remote-${providerId}-`)) {
            rawRemoteId = rawRemoteId.substring(`remote-${providerId}-`.length);
        } else if (typeof rawRemoteId === 'string' && rawRemoteId.startsWith('remote-youtube-wasm-')) {
            rawRemoteId = rawRemoteId.substring('remote-youtube-wasm-'.length);
        }

        const source: TrackSource = isRemote
            ? {
                type: 'Remote',
                provider_id: providerId,
                remote_track_id: rawRemoteId ? String(rawRemoteId) : String(t.id || ''),
                stream_url: t.source?.stream_url ?? t.stream_url,
                quality_hint: t.source?.quality_hint ?? t.quality_hint ?? null,
                cover_art_url: t.source?.cover_art_url ?? t.cover_art_url ?? null,
                duration_ms: t.source?.duration_ms ?? t.duration_ms ?? t.durationMs ?? (typeof t.duration === 'number' && t.duration > 0 ? (t.duration > 1000 ? Math.round(t.duration) : Math.round(t.duration * 1000)) : null) ?? null,
            }
            : {
                type: 'Local',
                track_id: t.source?.track_id ?? t.id ?? t.track_id ?? t.trackId ?? -1,
                file_path: filePath,
                album_id: t.source?.album_id ?? t.album_id ?? t.albumId ?? null,
            };

        return {
            instanceId,
            title: t.title,
            artist: t.artist ?? null,
            trackNumber: t.trackNumber ?? t.track_number ?? null,
            source,
        };
    }

    async handleTrackClick(trackPayload: any) {
        if (this.trackClickBehavior === "interrupt") {
            await this.playInterrupt(trackPayload);
        } else if (this.trackClickBehavior === "append") {
            await this.addToQueue(trackPayload);
        } else {
            await this.setQueue([trackPayload], 0);
        }
    }

    async setQueue(tracks: any[], startIndex: number = 0) {
        this._stagedAutoplayTrack = null;
        try {
            const tracksWithIds = tracks.map((t) => this.formatQueueTrack(t));
            await invoke("set_queue", { tracks: tracksWithIds, startIndex });

            // Load the first track immediately
            if (tracksWithIds.length > startIndex) {
                const t = tracksWithIds[startIndex];
                try {
                    await invoke("load_audio", {
                        source: t.source,
                        title: t.title,
                        artist: t.artist || null,
                        album: null
                    });
                    this.queueNextAudio();
                } catch (loadErr) {
                    console.warn(`Failed to load audio for track '${t.title}':`, loadErr);
                    toastStore.error(`Failed to play ${t.title}`);
                }
            }
        } catch (e) {
            console.error("Set queue failed:", e);
        }
    }

    async addToQueue(track: any) {
        try {
            const trackWithId = this.formatQueueTrack(track);
            await invoke("add_to_queue", { track: trackWithId });
        } catch (e) {
            console.error("Add to queue failed:", e);
        }
    }

    async playNext(track: any) {
        try {
            const trackWithId = this.formatQueueTrack(track);
            const event = await invoke<QueueChangePayload>("add_to_queue", { track: trackWithId });

            if (event && event.tracks && event.tracks.length > 0) {
                if (event.tracks.length === 1) {
                    // Queue was empty, play it immediately
                    await this.jumpToTrack(trackWithId.instanceId);
                } else {
                    const fromIndex = event.tracks.length - 1;
                    const toIndex = Math.min(event.current_position + 1, event.tracks.length - 1);

                    if (fromIndex !== toIndex) {
                        await invoke("reorder_queue", { fromIndex, toIndex });
                    }
                }
            }
        } catch (e) {
            console.error("Play next failed:", e);
        }
    }

    async playInterrupt(track: any) {
        try {
            const trackWithId = this.formatQueueTrack(track);
            const event = await invoke<QueueChangePayload>("add_to_queue", { track: trackWithId });

            if (event && event.tracks && event.tracks.length > 0) {
                if (event.tracks.length > 1) {
                    const fromIndex = event.tracks.length - 1;
                    const toIndex = Math.min(event.current_position + 1, event.tracks.length - 1);

                    if (fromIndex !== toIndex) {
                        await invoke("reorder_queue", { fromIndex, toIndex });
                    }
                }
                await this.jumpToTrack(trackWithId.instanceId);
            }
        } catch (e) {
            console.error("Play interrupt failed:", e);
        }
    }

    async clearQueue() {
        this._stagedAutoplayTrack = null;
        try {
            await invoke("clear_queue");
        } catch (e) {
            console.error("Clear queue failed:", e);
        }
    }

    async skipForward(count: number = 1) {
        try {
            if (this.currentPosition >= this.queue.length - 1 && settingsStore.autoplay && this.currentQueueTrack) {
                try {
                    const autoplayTrack = await invoke<QueueTrack | null>("resolve_autoplay_next_track", {
                        seed: this.currentQueueTrack
                    });
                    if (autoplayTrack) {
                        await invoke("add_to_queue", { track: autoplayTrack });
                    }
                } catch (autoErr) {
                    console.warn("Failed to resolve autoplay track on skip:", autoErr);
                }
            }

            const event = await invoke<QueueChangePayload>("skip_forward", { count });
            if (event.current_track) {
                const t = event.current_track;
                try {
                    await invoke("load_audio", {
                        source: t.source,
                        title: t.title,
                        artist: t.artist || null,
                        album: null
                    });
                    this.queueNextAudio();
                } catch (loadErr) {
                    console.warn(`Failed to load audio for track '${t.title}':`, loadErr);
                    toastStore.error(`Failed to play ${t.title}`);
                }
            }
        } catch (e) {
            console.error("Skip forward failed:", e);
        }
    }

    async skipBackward(count: number = 1) {
        try {
            const event = await invoke<QueueChangePayload>("skip_backward", { count });
            if (event.current_track) {
                const t = event.current_track;
                try {
                    await invoke("load_audio", {
                        source: t.source,
                        title: t.title,
                        artist: t.artist || null,
                        album: null
                    });
                    this.queueNextAudio();
                } catch (loadErr) {
                    console.warn(`Failed to load audio for track '${t.title}':`, loadErr);
                    toastStore.error(`Failed to play ${t.title}`);
                }
            }
        } catch (e) {
            console.error("Skip backward failed:", e);
        }
    }

    async jumpToTrack(instanceId: string) {
        try {
            const event = await invoke<QueueChangePayload>("jump_to_track", {
                instanceId,
            });
            if (event.current_track) {
                const t = event.current_track;
                try {
                    await invoke("load_audio", {
                        source: t.source,
                        title: t.title,
                        artist: t.artist || null,
                        album: null
                    });
                    this.queueNextAudio();
                } catch (loadErr) {
                    console.warn(`Failed to load audio for track '${t.title}':`, loadErr);
                    toastStore.error(`Failed to play ${t.title}`);
                }
            }
        } catch (e) {
            console.error("Jump to track failed:", e);
        }
    }

    private async queueNextAudio() {
        try {
            const nextTrack = await invoke<QueueTrack | null>("get_next_track");

            if (nextTrack) {
                this._stagedAutoplayTrack = null;
                await invoke("queue_next_audio", {
                    source: nextTrack.source,
                    title: nextTrack.title,
                    artist: nextTrack.artist || null,
                    album: null
                });
            } else if (settingsStore.autoplay && this.currentQueueTrack) {
                // Infinite Core Loop: Just-In-Time Background Staging (pre-buffers audio sink without cluttering visible queue)
                try {
                    const autoplayTrack = await invoke<QueueTrack | null>("resolve_autoplay_next_track", {
                        seed: this.currentQueueTrack
                    });
                    if (autoplayTrack) {
                        this._stagedAutoplayTrack = autoplayTrack;
                        await invoke("queue_next_audio", {
                            source: autoplayTrack.source,
                            title: autoplayTrack.title,
                            artist: autoplayTrack.artist || null,
                            album: null
                        });
                    }
                } catch (autoErr) {
                    console.warn("Failed to stage autoplay track:", autoErr);
                }
            } else {
                this._stagedAutoplayTrack = null;
            }
        } catch (e) {
            console.error("Failed to queue next audio:", e);
        }
    }

    async reorderQueue(fromIndex: number, toIndex: number) {
        if (fromIndex === toIndex || fromIndex < 0 || toIndex < 0 || fromIndex >= this.queue.length || toIndex >= this.queue.length) {
            return;
        }

        // Optimistic local update
        const previousQueue = [...this.queue];
        const newQueue = [...this.queue];
        const [moved] = newQueue.splice(fromIndex, 1);
        newQueue.splice(toIndex, 0, moved);
        this.queue = newQueue;

        try {
            await invoke("reorder_queue", {
                fromIndex,
                toIndex,
            });
        } catch (e) {
            console.error("Reorder queue failed, reverting:", e);
            this.queue = previousQueue;
        }
    }

    async setRepeatMode(mode: "Off" | "All" | "One") {
        try {
            await invoke("set_repeat_mode", { mode });
        } catch (e) {
            console.error("Set repeat mode failed:", e);
        }
    }

    async setShuffle(enabled: boolean) {
        try {
            await invoke("set_shuffle", { enabled });
        } catch (e) {
            console.error("Set shuffle failed:", e);
        }
    }

    // ══════════════════════════════════════════
    // HELPER METHODS (for UI convenience)
    // ══════════════════════════════════════════

    async toggleShuffle() {
        await this.setShuffle(!this.shuffleEnabled);
    }

    async toggleMute() {
        await this.setMute(!this.isMuted);
    }

    async cycleRepeat() {
        const modes: ("Off" | "All" | "One")[] = ["Off", "All", "One"];
        const currentIndex = modes.indexOf(this.repeatMode as "Off" | "All" | "One");
        const nextMode = modes[(currentIndex + 1) % modes.length];
        await this.setRepeatMode(nextMode);
    }

    async next() {
        await this.skipForward(1);
    }

    async previous() {
        await this.skipBackward(1);
    }
}

export const audioStore = new AudioStore();
