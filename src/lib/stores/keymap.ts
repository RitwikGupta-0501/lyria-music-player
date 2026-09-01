/**
 * Keyboard shortcut mapping system.
 * 
 * Configurable hotkey management with persistence to SQLite settings table.
 * The KeyboardHandler component reads from settingsStore at runtime.
 */

export interface KeyBinding {
    /** The key value (e.g., " " for Space, "ArrowLeft", "r", "F5") */
    key: string;
    /** Modifier flags */
    ctrl?: boolean;
    shift?: boolean;
    alt?: boolean;
}

export interface KeymapEntry {
    /** Human-readable label for the Settings UI */
    label: string;
    /** Description of what the hotkey triggers */
    description?: string;
    /** Logical category for grouped display */
    category: "playback" | "navigation" | "discovery";
    /** The action identifier */
    action: KeyAction;
    /** The key binding */
    binding: KeyBinding;
}

/** The action identifiers used throughout the app */
export type KeyAction =
    | "playPause"
    | "seekBack"
    | "seekForward"
    | "prevTrack"
    | "nextTrack"
    | "volumeUp"
    | "volumeDown"
    | "toggleShuffle"
    | "cycleRepeat"
    | "refreshRecommendations"
    | "search"
    | "escape";

/** Factory default keymap */
export const DEFAULT_KEYMAP: Record<KeyAction, KeymapEntry> = {
    refreshRecommendations: {
        label: "Refresh Recommendations",
        description: "Reload recommendation feeds, radios, and algorithmic shelves",
        category: "discovery",
        action: "refreshRecommendations",
        binding: { key: "r" },
    },
    playPause: {
        label: "Play / Pause",
        description: "Toggle playback state for the active track",
        category: "playback",
        action: "playPause",
        binding: { key: " " },
    },
    prevTrack: {
        label: "Previous Track",
        description: "Skip to previous track in queue",
        category: "playback",
        action: "prevTrack",
        binding: { key: "ArrowLeft", ctrl: true },
    },
    nextTrack: {
        label: "Next Track",
        description: "Skip to next track in queue",
        category: "playback",
        action: "nextTrack",
        binding: { key: "ArrowRight", ctrl: true },
    },
    seekBack: {
        label: "Seek Back 5s",
        description: "Rewind current playback by 5 seconds",
        category: "playback",
        action: "seekBack",
        binding: { key: "ArrowLeft" },
    },
    seekForward: {
        label: "Seek Forward 5s",
        description: "Fast-forward current playback by 5 seconds",
        category: "playback",
        action: "seekForward",
        binding: { key: "ArrowRight" },
    },
    volumeUp: {
        label: "Volume Up",
        description: "Increase master output volume by 5%",
        category: "playback",
        action: "volumeUp",
        binding: { key: "ArrowUp", ctrl: true },
    },
    volumeDown: {
        label: "Volume Down",
        description: "Decrease master output volume by 5%",
        category: "playback",
        action: "volumeDown",
        binding: { key: "ArrowDown", ctrl: true },
    },
    toggleShuffle: {
        label: "Toggle Shuffle",
        description: "Toggle shuffle order on current playback queue",
        category: "playback",
        action: "toggleShuffle",
        binding: { key: "s", ctrl: true },
    },
    cycleRepeat: {
        label: "Cycle Repeat",
        description: "Cycle repeat modes (Off, All, One)",
        category: "playback",
        action: "cycleRepeat",
        binding: { key: "r", ctrl: true },
    },
    search: {
        label: "Global Search",
        description: "Focus search bar across local library and streaming providers",
        category: "navigation",
        action: "search",
        binding: { key: "k", ctrl: true },
    },
    escape: {
        label: "Close / Back",
        description: "Close active drawer, modals, or full-screen view",
        category: "navigation",
        action: "escape",
        binding: { key: "Escape" },
    },
};

export function formatKey(key: string): string {
    if (key === " ") return "Space";
    if (key === "ArrowLeft") return "Left";
    if (key === "ArrowRight") return "Right";
    if (key === "ArrowUp") return "Up";
    if (key === "ArrowDown") return "Down";
    if (key === "Escape") return "Esc";
    if (key.length === 1) return key.toUpperCase();
    return key;
}

export function formatBinding(binding: KeyBinding): string {
    const parts: string[] = [];
    if (binding.ctrl) parts.push("Ctrl");
    if (binding.alt) parts.push("Alt");
    if (binding.shift) parts.push("Shift");
    parts.push(formatKey(binding.key));
    return parts.join(" + ");
}

export function eventToBinding(event: KeyboardEvent): KeyBinding | null {
    // Ignore pure modifier presses
    if (["Control", "Alt", "Shift", "Meta"].includes(event.key)) {
        return null;
    }

    return {
        key: event.key.length === 1 ? event.key.toLowerCase() : event.key,
        ctrl: event.ctrlKey || event.metaKey,
        alt: event.altKey,
        shift: event.shiftKey,
    };
}

/**
 * Check if a KeyboardEvent matches a KeyBinding.
 */
export function matchesBinding(event: KeyboardEvent, binding: KeyBinding): boolean {
    const eventKey = event.key.length === 1 ? event.key.toLowerCase() : event.key;
    const bindingKey = binding.key.length === 1 ? binding.key.toLowerCase() : binding.key;

    if (eventKey !== bindingKey) return false;
    if ((binding.ctrl ?? false) !== (event.ctrlKey || event.metaKey)) return false;
    if ((binding.shift ?? false) !== event.shiftKey) return false;
    if ((binding.alt ?? false) !== event.altKey) return false;
    return true;
}
