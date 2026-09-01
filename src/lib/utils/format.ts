/**
 * Centralized formatting and comparison utilities for Echo Desktop.
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
export function isCurrentTrack(
    track: { title: string; artist?: string | null } | null | undefined,
    currentQueueTrack: { title: string; artist?: string | null } | null | undefined
): boolean {
    if (!track || !currentQueueTrack) return false;
    const sameTitle = (track.title || "").trim().toLowerCase() === (currentQueueTrack.title || "").trim().toLowerCase();
    if (!sameTitle) return false;

    // If both have artists, compare them
    if (track.artist && currentQueueTrack.artist) {
        return track.artist.trim().toLowerCase() === currentQueueTrack.artist.trim().toLowerCase();
    }
    return true;
}
