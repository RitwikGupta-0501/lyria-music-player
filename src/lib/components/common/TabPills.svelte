<script lang="ts">
    import type { Component } from "svelte";

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
        isGlass = false,
        ariaLabel = "Tabs",
    }: Props<any> = $props();

    function selectTab(id: string, disabled?: boolean) {
        if (disabled) return;
        activeTab = id;
        onchange?.(id);
    }
</script>

<div 
    class="tab-pills" 
    class:is-glass={isGlass}
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

    .tab-pills.is-glass {
        backdrop-filter: blur(12px);
        -webkit-backdrop-filter: blur(12px);
        background: rgba(25, 25, 32, 0.45);
        border-color: rgba(255, 255, 255, 0.12);
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
        border: none;
        outline: none;
        -webkit-tap-highlight-color: transparent;
        padding: 0.25rem 0.6rem;
        border-radius: 14px;
        cursor: pointer;
        user-select: none;
        transition:
            background-color 0.12s ease,
            box-shadow 0.12s ease,
            transform 0.1s ease;
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
