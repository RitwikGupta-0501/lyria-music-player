/**
 * Centralized formatting and comparison utilities for Lyria.
 */

/**
 * Formats milliseconds into a `MM:SS` duration string.
 * Returns fallback (default: "--:--") if ms is null, undefined, or <= 0.
 */
export function formatDuration(ms?: number | null, fallback: string = "--:--"): string {
    if (ms === null || ms === undefined || isNaN(ms) || ms < 0) {
        return fallback;
    }
    const totalSecs = Math.floor(ms / 1000);
    const mins = Math.floor(totalSecs / 60);
    const secs = totalSecs % 60;
    return `${mins}:${secs.toString().padStart(2, '0')}`;
}

/**
 * Formats seconds into a `M:SS` or `MM:SS` duration string.
 */
export function formatSeconds(seconds: number, fallback: string = "0:00"): string {
    if (isNaN(seconds) || seconds < 0) {
        return fallback;
    }
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
}

/**
 * Extracts the single uppercase initial for typography avatar squircle fallbacks.
 */
export function getInitial(name?: string | null, fallback: string = "?"): string {
    if (!name || !name.trim()) {
        return fallback;
    }
    return name.trim().charAt(0).toUpperCase();
}

/**
 * Case-insensitive comparison checking if a track matches the currently playing queue track.
 */
function getTrackIdentity(t: any): { id?: string; filePath?: string; title: string; artist?: string } | null {
    if (!t) return null;
    let id: string | undefined = undefined;
    let filePath: string | undefined = undefined;

    if (t.source) {
        if (t.source.remote_track_id) {
            id = String(t.source.remote_track_id);
        } else if (t.source.track_id !== undefined && t.source.track_id !== null && t.source.track_id !== -1) {
            id = String(t.source.track_id);
        }
        if (t.source.file_path) {
            filePath = t.source.file_path;
        }
    }

    if (!id && t.remote_track_id) {
        id = String(t.remote_track_id);
    }
    if (!id && t.id !== undefined && t.id !== null) {
        id = String(t.id).replace(/^local-/, "");
    }
    if (!id && t.track_id !== undefined && t.track_id !== null && t.track_id !== -1) {
        id = String(t.track_id).replace(/^local-/, "");
    }

    if (!filePath && t.file_path) {
        filePath = t.file_path;
    }

    const title = (t.title || "").trim().toLowerCase();
    const artist = (t.artist || "").trim().toLowerCase();

    return { id, filePath, title, artist };
}

/**
 * Robust, deterministic comparison checking if a track matches the currently playing queue track.
 * Prioritizes exact remote/local ID and file path matching to prevent false duplicate highlights.
 */
export function isCurrentTrack(trackA: any, trackB: any): boolean {
    const a = getTrackIdentity(trackA);
    const b = getTrackIdentity(trackB);
    if (!a || !b) return false;

    // 1. Strict match on unique ID if both items provide an ID
    if (a.id && b.id) {
        return a.id === b.id;
    }

    // 2. Strict match on file path if both provide a file path
    if (a.filePath && b.filePath) {
        return a.filePath === b.filePath;
    }

    // 3. Fallback: Only match by metadata if both title AND artist are present and match
    if (a.title && b.title && a.title === b.title) {
        if (a.artist && b.artist) {
            return a.artist === b.artist;
        }
    }

    return false;
}

export function sanitizeAlbumName(album?: string | null): string | null {
    if (!album) return null;
    const trimmed = album.trim();
    if (!trimmed || ['&', '.', '•', '-', ',', '·'].includes(trimmed)) {
        return null;
    }
    return trimmed;
}

/**
 * Returns prioritized display metric: duration > plays > views > null (hidden).
 */
export function getTrackDisplayMetric(track?: {
    duration_ms?: number | null;
    plays?: string | number | null;
    play_count?: string | number | null;
    views?: string | number | null;
} | null): string | null {
    if (!track) return null;

    // 1. Duration (positive milliseconds)
    if (typeof track.duration_ms === 'number' && track.duration_ms > 0) {
        return formatDuration(track.duration_ms, "");
    }

    // 2. Play count / Plays
    const plays = track.plays || track.play_count;
    if (plays) {
        const pStr = plays.toString().trim();
        if (pStr) return pStr.toLowerCase().includes('play') ? pStr : `${pStr} plays`;
    }

    // 3. Views
    if (track.views) {
        const vStr = track.views.toString().trim();
        if (vStr) return vStr.toLowerCase().includes('view') ? vStr : `${vStr} views`;
    }

    // 4. Fallback -> cleanly hidden
    return null;
}
