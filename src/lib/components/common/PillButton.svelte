<script lang="ts">
    import type { Snippet, Component } from "svelte";
    import { settingsStore } from "$lib/stores/settings.svelte";

    let {
        variant = "primary",
        size = "md",
        icon,
        label,
        disabled = false,
        title,
        isGlass,
        onclick,
        children,
    } = $props<{
        variant?: "primary" | "secondary" | "danger" | "ghost";
        size?: "sm" | "md" | "lg";
        icon?: Component<any>;
        label?: string;
        disabled?: boolean;
        title?: string;
        isGlass?: boolean;
        onclick?: (e: MouseEvent) => void;
        children?: Snippet;
    }>();

    let effectiveGlass = $derived(isGlass ?? settingsStore.glassyPlayerBar);
    let iconSize = $derived(size === "sm" ? 14 : (size === "lg" ? 18 : 16));
</script>

<button 
    type="button"
    class="echo-pill-btn variant-{variant} size-{size}" 
    class:is-glass={effectiveGlass}
    {disabled}
    {onclick}
    {title}
    aria-label={title || label}
>
    {#if icon}
        {@const Icon = icon}
        <Icon size={iconSize} weight={variant === "primary" ? "fill" : "bold"} />
    {/if}
    {#if label}
        <span class="btn-label">{label}</span>
    {/if}
    {@render children?.()}
</button>

<style>
    .echo-pill-btn {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        font-family: inherit;
        font-weight: 600;
        cursor: pointer;
        border: 1px solid transparent;
        outline: none;
        user-select: none;
        white-space: nowrap;
        transition: transform 0.18s cubic-bezier(0.16, 1, 0.3, 1), background 0.18s ease, border-color 0.18s ease, color 0.18s ease, box-shadow 0.18s ease;
    }

    .echo-pill-btn:disabled {
        opacity: 0.45;
        cursor: not-allowed;
        transform: none !important;
    }

    /* SIZES */
    .size-sm {
        padding: 0.42rem 0.9rem;
        font-size: 0.8rem;
        border-radius: 20px;
        gap: 0.42rem;
    }

    .size-md {
        padding: 0.6rem 1.3rem;
        font-size: 0.88rem;
        border-radius: 24px;
        gap: 0.5rem;
    }

    .size-lg {
        padding: 0.75rem 1.6rem;
        font-size: 0.95rem;
        border-radius: 28px;
        gap: 0.6rem;
    }

    /* PRIMARY VARIANT */
    .variant-primary {
        background: var(--echo-primary, #B58E62);
        color: #0E0E10 !important;
        box-shadow: 0 4px 14px rgba(0, 0, 0, 0.35);
    }

    .variant-primary:hover:not(:disabled) {
        background: #f3c292;
        color: #0E0E10 !important;
        transform: scale(1.03);
        box-shadow: 0 6px 20px rgba(226, 169, 115, 0.35);
    }

    .variant-primary :global(svg) {
        color: #0E0E10 !important;
        fill: currentColor;
    }

    .variant-primary:hover:not(:disabled) :global(svg) {
        color: #0E0E10 !important;
    }

    .variant-primary:active:not(:disabled) {
        transform: scale(0.96);
    }

    /* Primary Glassmorphic (Transparent Brass Glass - Tab Pill Effect) */
    .variant-primary.is-glass {
        color: #ffffff !important;
        background: linear-gradient(
            180deg,
            rgba(235, 175, 115, 0.40) 0%,
            rgba(224, 162, 102, 0.32) 50%,
            rgba(214, 150, 90, 0.35) 100%
        );
        border: 1.5px solid rgba(235, 175, 115, 0.65);
        border-top-color: rgba(255, 238, 208, 0.95);
        border-bottom-color: rgba(170, 110, 55, 0.40);
        backdrop-filter: blur(12px) saturate(155%) brightness(1.15);
        -webkit-backdrop-filter: blur(12px) saturate(155%) brightness(1.15);
        font-weight: 650;
        box-shadow: 
            inset 0 1px 2px 0 rgba(255, 240, 215, 0.45),
            inset 0 -1px 2px 0 rgba(140, 95, 50, 0.30),
            0 4px 16px rgba(226, 169, 115, 0.35);
        text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
    }

    .variant-primary.is-glass :global(svg) {
        color: #ffffff !important;
        fill: currentColor;
    }

    .variant-primary.is-glass:hover:not(:disabled) {
        color: #ffffff !important;
        background: linear-gradient(
            180deg,
            rgba(245, 185, 125, 0.50) 0%,
            rgba(234, 172, 112, 0.40) 50%,
            rgba(224, 160, 100, 0.44) 100%
        );
        border-color: rgba(245, 185, 125, 0.80);
        border-top-color: #ffffff;
        box-shadow: 
            inset 0 1px 2px 0 #ffffff,
            inset 0 -1px 2px 0 rgba(140, 95, 50, 0.40),
            0 6px 22px rgba(226, 169, 115, 0.45);
        transform: scale(1.03);
    }

    .variant-primary.is-glass:hover:not(:disabled) :global(svg) {
        color: #ffffff !important;
        fill: currentColor;
    }

    .variant-primary.is-glass:active:not(:disabled) {
        transform: scale(0.96);
        box-shadow: 
            inset 0 1px 1px 0 rgba(255, 240, 215, 0.30),
            inset 0 1px 3px 0 rgba(140, 95, 50, 0.50),
            0 2px 6px rgba(0, 0, 0, 0.3);
    }

    /* SECONDARY VARIANT */
    .variant-secondary {
        background: rgba(255, 255, 255, 0.07);
        border: 1px solid rgba(255, 255, 255, 0.12);
        color: #FFFFFF;
        backdrop-filter: blur(8px);
        -webkit-backdrop-filter: blur(8px);
    }

    .variant-secondary:hover:not(:disabled) {
        background: rgba(255, 255, 255, 0.14);
        border-color: rgba(255, 255, 255, 0.25);
        transform: scale(1.03);
    }

    .variant-secondary:active:not(:disabled) {
        transform: scale(0.96);
    }

    /* Secondary Glassmorphic (Optical Refraction Lens - White Translucent) */
    .variant-secondary.is-glass {
        background: linear-gradient(180deg, rgba(255, 255, 255, 0.10) 0%, rgba(255, 255, 255, 0.02) 100%);
        border: 1px solid rgba(255, 255, 255, 0.12);
        border-top-color: rgba(255, 255, 255, 0.38);
        border-bottom-color: rgba(255, 255, 255, 0.05);
        color: #FFFFFF;
        backdrop-filter: blur(14px) saturate(135%) brightness(1.08);
        -webkit-backdrop-filter: blur(14px) saturate(135%) brightness(1.08);
        box-shadow: 
            inset 0 1px 1.5px 0 rgba(255, 255, 255, 0.25),
            inset 0 -1px 2px 0 rgba(0, 0, 0, 0.35),
            0 4px 16px rgba(0, 0, 0, 0.3);
    }

    .variant-secondary.is-glass:hover:not(:disabled) {
        background: linear-gradient(180deg, rgba(255, 255, 255, 0.16) 0%, rgba(255, 255, 255, 0.05) 100%);
        border-color: rgba(255, 255, 255, 0.20);
        border-top-color: rgba(255, 255, 255, 0.52);
        border-bottom-color: rgba(255, 255, 255, 0.08);
        box-shadow: 
            inset 0 1px 2px 0 rgba(255, 255, 255, 0.35),
            inset 0 -1px 2px 0 rgba(0, 0, 0, 0.4),
            0 6px 20px rgba(0, 0, 0, 0.45);
        transform: scale(1.03);
    }

    .variant-secondary.is-glass:active:not(:disabled) {
        transform: scale(0.96);
        box-shadow: 
            inset 0 1px 1px 0 rgba(255, 255, 255, 0.15),
            inset 0 1px 3px 0 rgba(0, 0, 0, 0.5),
            0 2px 6px rgba(0, 0, 0, 0.3);
    }

    /* DANGER VARIANT */
    .variant-danger {
        background: rgba(239, 68, 68, 0.12);
        border: 1px solid rgba(239, 68, 68, 0.25);
        color: #ef4444;
    }

    .variant-danger:hover:not(:disabled) {
        background: rgba(239, 68, 68, 0.22);
        border-color: rgba(239, 68, 68, 0.4);
        transform: scale(1.03);
    }

    .variant-danger:active:not(:disabled) {
        transform: scale(0.96);
    }

    /* Danger Glassmorphic (Ruby Refraction) */
    .variant-danger.is-glass {
        background: linear-gradient(180deg, rgba(239, 68, 68, 0.22) 0%, rgba(220, 38, 38, 0.10) 100%);
        border: 1px solid rgba(239, 68, 68, 0.30);
        border-top-color: rgba(255, 180, 180, 0.55);
        border-bottom-color: rgba(180, 20, 20, 0.20);
        color: #ff7575;
        backdrop-filter: blur(12px) saturate(135%);
        -webkit-backdrop-filter: blur(12px) saturate(135%);
        box-shadow: 
            inset 0 1px 1px 0 rgba(255, 200, 200, 0.3),
            inset 0 -1px 2px 0 rgba(150, 20, 20, 0.35),
            0 4px 14px rgba(239, 68, 68, 0.2);
    }

    .variant-danger.is-glass:hover:not(:disabled) {
        background: linear-gradient(180deg, rgba(239, 68, 68, 0.32) 0%, rgba(220, 38, 38, 0.16) 100%);
        border-color: rgba(239, 68, 68, 0.45);
        border-top-color: rgba(255, 200, 200, 0.75);
        color: #ffffff;
        box-shadow: 
            inset 0 1px 2px 0 rgba(255, 200, 200, 0.45),
            0 6px 18px rgba(239, 68, 68, 0.3);
    }

    /* GHOST VARIANT */
    .variant-ghost {
        background: transparent;
        border: 1px solid transparent;
        color: rgba(255, 255, 255, 0.75);
    }

    .variant-ghost:hover:not(:disabled) {
        background: rgba(255, 255, 255, 0.08);
        color: #fff;
    }

    .variant-ghost:active:not(:disabled) {
        transform: scale(0.96);
    }

    /* Ghost Glassmorphic (Minimalist Refractive Lens) */
    .variant-ghost.is-glass {
        background: rgba(255, 255, 255, 0.02);
        border: 1px solid rgba(255, 255, 255, 0.06);
        border-top-color: rgba(255, 255, 255, 0.22);
        border-bottom-color: rgba(255, 255, 255, 0.02);
        color: rgba(255, 255, 255, 0.85);
        backdrop-filter: blur(10px) saturate(130%) brightness(1.06);
        -webkit-backdrop-filter: blur(10px) saturate(130%) brightness(1.06);
        box-shadow: 
            inset 0 1px 1px 0 rgba(255, 255, 255, 0.16),
            0 2px 8px rgba(0, 0, 0, 0.2);
    }

    .variant-ghost.is-glass:hover:not(:disabled) {
        background: linear-gradient(180deg, rgba(255, 255, 255, 0.10) 0%, rgba(255, 255, 255, 0.03) 100%);
        border-color: rgba(255, 255, 255, 0.15);
        border-top-color: rgba(255, 255, 255, 0.38);
        color: #FFFFFF;
        box-shadow: 
            inset 0 1px 1px 0 rgba(255, 255, 255, 0.25),
            0 4px 14px rgba(0, 0, 0, 0.35);
        transform: scale(1.03);
    }

    .variant-ghost.is-glass:active:not(:disabled) {
        transform: scale(0.96);
    }

    .btn-label {
        line-height: 1;
    }
</style>
