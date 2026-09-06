<script lang="ts">
    import type { Component } from "svelte";
    import { settingsStore } from "$lib/stores/settings.svelte";

    export interface TabItem<T extends string = string> {
        id: T;
        label: string;
        icon?: Component<any> | any;
        disabled?: boolean;
    }

    interface Props<T extends string = string> {
        tabs: TabItem<T>[];
        activeTab?: T;
        onchange?: (tabId: T) => void;
        size?: "sm" | "md";
        isGlass?: boolean;
        ariaLabel?: string;
    }

    let {
        tabs,
        activeTab = $bindable(""),
        onchange,
        size = "sm",
        isGlass,
        ariaLabel = "Tabs",
    }: Props<any> = $props();

    let effectiveGlass = $derived(isGlass ?? settingsStore.glassyPlayerBar);

    function selectTab(id: string, disabled?: boolean) {
        if (disabled) return;
        activeTab = id;
        onchange?.(id);
    }
</script>

<div 
    class="tab-pills" 
    class:is-glass={effectiveGlass}
    class:size-md={size === "md"}
    role="tablist"
    aria-label={ariaLabel}
>
    {#each tabs as tab (tab.id)}
        {@const isActive = activeTab === tab.id}
        <button
            type="button"
            role="tab"
            aria-selected={isActive}
            disabled={tab.disabled}
            class="pill-btn"
            class:active={isActive}
            onclick={() => selectTab(tab.id, tab.disabled)}
        >
            {#if tab.icon}
                {@const IconComp = tab.icon}
                <IconComp size={size === "md" ? 15 : 13} weight={isActive ? "fill" : "bold"} />
            {/if}
            <span>{tab.label}</span>
        </button>
    {/each}
</div>

<style>
    .tab-pills {
        display: inline-flex;
        align-items: center;
        gap: 0.35rem;
        background: rgba(18, 18, 22, 0.75);
        border: 1px solid rgba(255, 255, 255, 0.08);
        border-radius: 20px;
        padding: 0.2rem 0.3rem;
        box-shadow: 0 4px 16px rgba(0, 0, 0, 0.25);
    }

    /* Glass Housing: Translucent Tinted Frosted Track */
    .tab-pills.is-glass {
        backdrop-filter: blur(16px) saturate(130%) brightness(1.06);
        -webkit-backdrop-filter: blur(16px) saturate(130%) brightness(1.06);
        background: linear-gradient(180deg, rgba(255, 255, 255, 0.07) 0%, rgba(24, 20, 26, 0.32) 60%, rgba(14, 12, 16, 0.40) 100%);
        border: 1.5px solid rgba(255, 255, 255, 0.10);
        border-top-color: rgba(255, 255, 255, 0.26);
        border-bottom-color: rgba(10, 8, 12, 0.25);
        box-shadow: 
            inset 0 1px 1.5px rgba(255, 255, 255, 0.12),
            inset 0 -1px 2px rgba(0, 0, 0, 0.35),
            0 6px 20px rgba(0, 0, 0, 0.35);
    }

    .tab-pills.size-md {
        padding: 0.25rem 0.4rem;
        border-radius: 24px;
        gap: 0.45rem;
    }

    .pill-btn {
        display: inline-flex;
        align-items: center;
        gap: 0.35rem;
        font-size: 0.72rem;
        font-weight: 600;
        color: rgba(255, 255, 255, 0.55);
        background: transparent;
        border: 1.5px solid transparent;
        outline: none;
        -webkit-tap-highlight-color: transparent;
        padding: 0.25rem 0.6rem;
        border-radius: 14px;
        cursor: pointer;
        user-select: none;
        transition:
            background 0.15s ease,
            border-color 0.15s ease,
            box-shadow 0.15s ease,
            transform 0.1s ease,
            color 0.15s ease;
    }

    .tab-pills.size-md .pill-btn {
        font-size: 0.8rem;
        padding: 0.35rem 0.8rem;
        border-radius: 16px;
        gap: 0.45rem;
    }

    .pill-btn:hover:not(:disabled):not(.active) {
        color: #fff;
        background: rgba(255, 255, 255, 0.06);
    }

    .pill-btn:active:not(:disabled) {
        transform: scale(0.96);
    }

    .pill-btn.active {
        color: #0E0E10;
        background: #D4A86E;
        box-shadow: 0 2px 8px rgba(212, 168, 110, 0.35);
        transition: none;
    }

    /* Glass pill-btn states: Translucent Tinted Frosted Glass */
    .tab-pills.is-glass .pill-btn {
        color: rgba(255, 255, 255, 0.70);
    }

    .tab-pills.is-glass .pill-btn:hover:not(:disabled):not(.active) {
        color: #ffffff;
        background: linear-gradient(180deg, rgba(255, 255, 255, 0.12) 0%, rgba(255, 255, 255, 0.03) 100%);
        border-color: rgba(255, 255, 255, 0.14);
        border-top-color: rgba(255, 255, 255, 0.32);
        box-shadow: inset 0 1px 1px rgba(255, 255, 255, 0.2);
    }

    /* Active Tab: Transparent Echo Primary (#e2a973) Amber-Brass Tinted Glass */
    .tab-pills.is-glass .pill-btn.active {
        color: #ffffff;
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
            0 3px 14px rgba(226, 169, 115, 0.35),
            0 1px 4px rgba(0, 0, 0, 0.4);
        text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
    }

    .pill-btn:focus-visible {
        outline: 1px solid rgba(212, 168, 110, 0.6);
        outline-offset: 1px;
    }

    .pill-btn:disabled {
        opacity: 0.3;
        cursor: not-allowed;
    }

    .pill-btn :global(svg) {
        flex-shrink: 0;
    }
</style>
