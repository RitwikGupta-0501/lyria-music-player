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
        contain: strict;
        transform: translateZ(0);
        will-change: transform;
    }

    .playing-visualizer .bar {
        width: 3px;
        background-color: var(--eq-color, var(--echo-primary));
        border-radius: 2px;
        transform-origin: bottom;
        will-change: transform;
        transform: translateZ(0);
    }

    .playing-visualizer .bar {
        height: 100%;
        animation: eq-pulse 1.2s ease-in-out infinite;
    }
    .playing-visualizer .bar:nth-child(1) { animation-delay: 0s; }
    .playing-visualizer .bar:nth-child(2) { animation-delay: 0.35s; }
    .playing-visualizer .bar:nth-child(3) { animation-delay: 0.15s; }
    .playing-visualizer .bar:nth-child(4) { animation-delay: 0.5s; }

    @keyframes eq-pulse {
        0%, 100% { transform: scaleY(0.25) translateZ(0); }
        50% { transform: scaleY(1.0) translateZ(0); }
    }
</style>
