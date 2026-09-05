<script lang="ts">
    import { Play, Pause } from "phosphor-svelte";

    let {
        isPlaying = false,
        size = "md",
        title = "Play",
        onclick
    } = $props<{
        isPlaying?: boolean;
        size?: "sm" | "md" | "lg";
        title?: string;
        onclick: (e: MouseEvent) => void;
    }>();

    let iconSize = $derived(size === "sm" ? 14 : (size === "lg" ? 22 : 18));
</script>

<button 
    type="button"
    class="echo-card-play-btn size-{size}" 
    class:is-playing={isPlaying}
    {onclick}
    {title}
    aria-label={title}
>
    {#if isPlaying}
        <Pause weight="fill" size={iconSize} />
    {:else}
        <span class="icon-optical-wrap size-{size}">
            <Play weight="fill" size={iconSize} />
        </span>
    {/if}
</button>

<style>
    .echo-card-play-btn {
        border-radius: 50%;
        background-color: var(--echo-primary, #B58E62);
        color: #0E0E10;
        border: none;
        display: flex;
        align-items: center;
        justify-content: center;
        box-shadow: 0 4px 16px rgba(0, 0, 0, 0.5);
        cursor: pointer;
        padding: 0;
        pointer-events: auto;
        transform: scale(0.9);
        transition: transform 0.2s cubic-bezier(0.16, 1, 0.3, 1), background-color 0.2s ease, box-shadow 0.2s ease;
    }

    .size-sm {
        width: 32px;
        height: 32px;
    }

    .size-md {
        width: 42px;
        height: 42px;
    }

    .size-lg {
        width: 48px;
        height: 48px;
    }

    .icon-optical-wrap {
        display: flex;
        align-items: center;
        justify-content: center;
        transform: translateX(1px);
    }

    .icon-optical-wrap.size-lg {
        transform: translateX(1.5px);
    }

    .icon-optical-wrap.size-sm {
        transform: translateX(0.75px);
    }

    :global(.group:hover) .echo-card-play-btn,
    :global(.album-card:hover) .echo-card-play-btn,
    :global(.track-card:hover) .echo-card-play-btn,
    :global(.playlist-card:hover) .echo-card-play-btn,
    :global(.video-card:hover) .echo-card-play-btn,
    :global(.favorites-card:hover) .echo-card-play-btn {
        transform: scale(1);
    }

    .echo-card-play-btn:hover {
        background-color: #f3c292 !important;
        transform: scale(1.08) !important;
        box-shadow: 0 6px 20px rgba(226, 169, 115, 0.35);
    }

    .echo-card-play-btn:active {
        transform: scale(0.95) !important;
    }

    :global(.echo-card-play-btn svg) {
        color: #0E0E10 !important;
        fill: currentColor;
    }
</style>
