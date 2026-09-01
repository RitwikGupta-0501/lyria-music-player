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
    private _rafId: number | null = null;
    private _autoAdvancing = false;

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
        const providerId = isRemote ? (t.source.provider_id || "youtube-wasm") : "local";
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

            this.playbackState = payload.state;
            this.duration = payload.duration;
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
            // Tell backend we advanced (this updates current_position, but we don't load_audio since it's already playing)
            this._autoAdvancing = true;
            await invoke("skip_forward", { count: 1 });
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

        this.startClock();
    }

    destroy() {
        if (this.unlistenSync) this.unlistenSync();
        if (this.unlistenTrackEnded) this.unlistenTrackEnded();
        if (this.unlistenQueueChanged) this.unlistenQueueChanged();
        this.stopClock();
        if (this._volumeSaveTimer) clearTimeout(this._volumeSaveTimer);
    }

    // ══════════════════════════════════════════
    // PLAYBACK CLOCK (60fps interpolation)
    // ══════════════════════════════════════════

    private tick = () => {
        if (this._isPlaying && this.duration > 0) {
            const elapsed = (performance.now() - this._syncTimestamp) / 1000;
            this.currentTime = Math.min(this._syncPosition + elapsed, this.duration);
        }
        this._rafId = requestAnimationFrame(this.tick);
    };

    private startClock() {
        if (this._rafId === null) {
            this._rafId = requestAnimationFrame(this.tick);
        }
    }

    private stopClock() {
        if (this._rafId !== null) {
            cancelAnimationFrame(this._rafId);
            this._rafId = null;
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
        try {
            await invoke("pause_audio");
        } catch (e) {
            console.error("Pause failed:", e);
        }
    }

    async stop() {
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
        const isRemote = (t.source?.type === 'Remote') ||
                         !!(t.source?.provider_id) ||
                         !!(t.stream_url) ||
                         !!(t.provider_id) ||
                         (t.type === "Remote");

        let rawRemoteId = t.source?.remote_track_id ?? t.remote_track_id ?? t.remoteTrackId ?? t.id ?? null;
        const providerId = t.source?.provider_id ?? t.provider_id ?? t.providerId ?? 'unknown';

        if (typeof rawRemoteId === 'string' && providerId && rawRemoteId.startsWith(`remote-${providerId}-`)) {
            rawRemoteId = rawRemoteId.substring(`remote-${providerId}-`.length);
        } else if (typeof rawRemoteId === 'string' && rawRemoteId.startsWith('remote-youtube-wasm-')) {
            rawRemoteId = rawRemoteId.substring('remote-youtube-wasm-'.length);
        }

        const source: TrackSource = isRemote
            ? {
                type: 'Remote',
                provider_id: providerId,
                remote_track_id: rawRemoteId,
                stream_url: t.source?.stream_url ?? t.stream_url,
                quality_hint: t.source?.quality_hint ?? t.quality_hint ?? null,
                cover_art_url: t.source?.cover_art_url ?? t.cover_art_url ?? null,
                duration_ms: t.source?.duration_ms ?? t.duration_ms ?? null,
            }
            : {
                type: 'Local',
                track_id: t.source?.track_id ?? t.id ?? t.track_id ?? t.trackId ?? -1,
                file_path: t.source?.file_path ?? t.file_path ?? t.filePath ?? '',
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
        try {
            await invoke("clear_queue");
        } catch (e) {
            console.error("Clear queue failed:", e);
        }
    }

    async skipForward(count: number = 1) {
        try {
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
                await invoke("queue_next_audio", {
                    source: nextTrack.source,
                    title: nextTrack.title,
                    artist: nextTrack.artist || null,
                    album: null
                });
            }
        } catch (e) {
            console.error("Failed to queue next audio:", e);
        }
    }

    async reorderQueue(fromIndex: number, toIndex: number) {
        try {
            await invoke("reorder_queue", {
                fromIndex,
                toIndex,
            });
        } catch (e) {
            console.error("Reorder queue failed:", e);
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
