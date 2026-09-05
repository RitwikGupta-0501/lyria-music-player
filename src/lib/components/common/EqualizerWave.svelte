<script lang="ts">
    interface Props {
        class?: string;
        color?: string;
    }

    let { class: className = "", color }: Props = $props();
</script>

<div class="playing-visualizer {className}" style={color ? `--eq-color: ${color};` : ""} aria-hidden="true">
    <div class="bar"></div>
    <div class="bar"></div>
    <div class="bar"></div>
    <div class="bar"></div>
</div>

<style>
    .playing-visualizer {
        display: flex;
        align-items: flex-end;
        justify-content: center;
        gap: 2px;
        height: 14px;
        width: 18px;
        flex-shrink: 0;
    }

    .playing-visualizer .bar {
        width: 3px;
        background-color: var(--eq-color, var(--echo-primary));
        border-radius: 2px;
        transform-origin: bottom;
    }

    .playing-visualizer .bar:nth-child(1) { height: 100%; animation: eq-bar-1 1.2s ease-in-out infinite; }
    .playing-visualizer .bar:nth-child(2) { height: 100%; animation: eq-bar-2 1.5s ease-in-out infinite; }
    .playing-visualizer .bar:nth-child(3) { height: 100%; animation: eq-bar-3 1.1s ease-in-out infinite; }
    .playing-visualizer .bar:nth-child(4) { height: 100%; animation: eq-bar-4 1.4s ease-in-out infinite; }

    @keyframes eq-bar-1 {
        0%, 100% { transform: scaleY(0.3); }
        25% { transform: scaleY(0.9); }
        50% { transform: scaleY(0.5); }
        75% { transform: scaleY(1.0); }
    }

    @keyframes eq-bar-2 {
        0%, 100% { transform: scaleY(0.6); }
        25% { transform: scaleY(0.2); }
        50% { transform: scaleY(1.0); }
        75% { transform: scaleY(0.4); }
    }

    @keyframes eq-bar-3 {
        0%, 100% { transform: scaleY(0.8); }
        25% { transform: scaleY(0.4); }
        50% { transform: scaleY(0.9); }
        75% { transform: scaleY(0.3); }
    }

    @keyframes eq-bar-4 {
        0%, 100% { transform: scaleY(0.4); }
        25% { transform: scaleY(1.0); }
        50% { transform: scaleY(0.3); }
        75% { transform: scaleY(0.8); }
    }
</style>
