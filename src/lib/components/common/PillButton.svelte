<script lang="ts">
    import type { Snippet, Component } from "svelte";

    let {
        variant = "primary",
        size = "md",
        icon,
        label,
        disabled = false,
        title,
        onclick,
        children,
    } = $props<{
        variant?: "primary" | "secondary" | "danger" | "ghost";
        size?: "sm" | "md" | "lg";
        icon?: Component<any>;
        label?: string;
        disabled?: boolean;
        title?: string;
        onclick?: (e: MouseEvent) => void;
        children?: Snippet;
    }>();

    let iconSize = $derived(size === "sm" ? 14 : (size === "lg" ? 18 : 16));
</script>

<button 
    type="button"
    class="echo-pill-btn variant-{variant} size-{size}" 
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
        border: none;
        outline: none;
        user-select: none;
        white-space: nowrap;
        transition: transform 0.18s cubic-bezier(0.16, 1, 0.3, 1), background-color 0.18s ease, border-color 0.18s ease, color 0.18s ease, box-shadow 0.18s ease;
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

    /* VARIANTS */
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

    .variant-secondary {
        background: rgba(255, 255, 255, 0.07);
        border: 1px solid rgba(255, 255, 255, 0.12);
        color: #FFFFFF;
        backdrop-filter: blur(8px);
    }

    .variant-secondary:hover:not(:disabled) {
        background: rgba(255, 255, 255, 0.14);
        border-color: rgba(255, 255, 255, 0.25);
        transform: scale(1.03);
    }

    .variant-secondary:active:not(:disabled) {
        transform: scale(0.96);
    }

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

    .variant-ghost {
        background: transparent;
        color: rgba(255, 255, 255, 0.75);
    }

    .variant-ghost:hover:not(:disabled) {
        background: rgba(255, 255, 255, 0.08);
        color: #fff;
    }

    .variant-ghost:active:not(:disabled) {
        transform: scale(0.96);
    }

    .btn-label {
        line-height: 1;
    }
</style>
