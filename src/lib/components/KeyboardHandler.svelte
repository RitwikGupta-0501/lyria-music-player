<script lang="ts">
    import { audioStore } from "$lib/stores/audio.svelte";
    import { homeStore } from "$lib/stores/home.svelte";
    import { settingsStore } from "$lib/stores/settings.svelte";
    import { matchesBinding, type KeyAction } from "$lib/stores/keymap";

    $effect(() => {
        function handleKeydown(e: KeyboardEvent) {
            // Don't intercept when user is typing in an input/textarea
            const target = e.target as HTMLElement;
            if (target.tagName === "INPUT" || target.tagName === "TEXTAREA" || target.isContentEditable) {
                return;
            }

            // If a shortcut recorder is active anywhere in the DOM, let it capture the event
            if (document.querySelector(".is-recording")) {
                return;
            }

            const activeKeymap = settingsStore.getKeymap();

            // Find matching action from the active keymap
            let matchedAction: KeyAction | null = null;
            for (const [action, binding] of Object.entries(activeKeymap)) {
                if (matchesBinding(e, binding)) {
                    matchedAction = action as KeyAction;
                    break;
                }
            }

            if (!matchedAction) return;

            // Prevent default browser behavior for matched shortcuts
            e.preventDefault();

            // Dispatch the action
            switch (matchedAction) {
                case "refreshRecommendations":
                    homeStore.loadHome(true);
                    document.dispatchEvent(new CustomEvent("echo:refresh"));
                    break;

                case "playPause":
                    if (audioStore.playbackState === "Playing") {
                        audioStore.pause();
                    } else {
                        audioStore.play();
                    }
                    break;

                case "seekBack":
                    audioStore.seek(Math.max(0, audioStore.currentTime - 5));
                    break;

                case "seekForward":
                    audioStore.seek(Math.min(audioStore.duration, audioStore.currentTime + 5));
                    break;

                case "prevTrack":
                    audioStore.previous();
                    break;

                case "nextTrack":
                    audioStore.next();
                    break;

                case "volumeUp":
                    audioStore.setVolume(Math.min(1, audioStore.volume + 0.05));
                    break;

                case "volumeDown":
                    audioStore.setVolume(Math.max(0, audioStore.volume - 0.05));
                    break;

                case "toggleShuffle":
                    audioStore.toggleShuffle();
                    break;

                case "cycleRepeat":
                    audioStore.cycleRepeat();
                    break;

                case "escape":
                    document.dispatchEvent(new CustomEvent("echo:escape"));
                    break;

                case "search":
                    document.dispatchEvent(new CustomEvent("echo:search"));
                    break;
            }
        }

        document.addEventListener("keydown", handleKeydown);
        return () => document.removeEventListener("keydown", handleKeydown);
    });
</script>
