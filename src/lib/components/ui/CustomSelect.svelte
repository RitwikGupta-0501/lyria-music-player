<script lang="ts">
    import { CaretDown, Check } from "phosphor-svelte";

    interface Option<T = any> {
        value: T;
        label: string;
        badge?: string;
    }

    let {
        options = [],
        value = $bindable(),
        placeholder = "Select an option",
        prefix = "",
        icon: IconComponent = null,
        onChange = (val: any) => {},
    } = $props<{
        options: Option[];
        value: any;
        placeholder?: string;
        prefix?: string;
        icon?: any;
        onChange?: (val: any) => void;
    }>();

    let isOpen = $state(false);

    let selectedOption = $derived(options.find((o: Option) => o.value === value));
    let displayLabel = $derived(selectedOption ? selectedOption.label : placeholder);

    function toggleOpen(e: MouseEvent) {
        e.stopPropagation();
        isOpen = !isOpen;
    }

    function selectOption(optValue: any, e?: MouseEvent) {
        if (e) e.stopPropagation();
        value = optValue;
        isOpen = false;
        onChange(optValue);
    }

    function handleKeydown(e: KeyboardEvent) {
        if (e.key === "Escape" && isOpen) {
            isOpen = false;
            e.stopPropagation();
        }
    }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="custom-select-container">
    <button 
        class="select-trigger" 
        type="button"
        onclick={toggleOpen}
        aria-expanded={isOpen}
        aria-haspopup="listbox"
    >
        <div class="select-trigger-left">
            {#if IconComponent}
                <IconComponent size={14} weight="bold" />
            {/if}
            {#if prefix}
                <span class="select-prefix">{prefix}</span>
            {/if}
            <span class="select-label">{displayLabel}</span>
            {#if selectedOption?.badge}
                <span class="option-badge">{selectedOption.badge}</span>
            {/if}
        </div>
        <CaretDown size={13} weight="bold" class={isOpen ? 'rotated' : ''} />
    </button>

    {#if isOpen}
        <div 
            class="select-backdrop" 
            role="button" 
            tabindex="-1" 
            onclick={() => isOpen = false}
            onkeydown={(e) => { if (e.key === 'Escape') isOpen = false; }}
        ></div>
        <div class="custom-select-menu" role="listbox">
            {#each options as opt}
                <button
                    type="button"
                    class="select-option"
                    class:selected={opt.value === value}
                    role="option"
                    aria-selected={opt.value === value}
                    onclick={(e) => selectOption(opt.value, e)}
                >
                    <span class="option-label">{opt.label}</span>
                    <div class="option-right">
                        {#if opt.badge}
                            <span class="option-badge">{opt.badge}</span>
                        {/if}
                        {#if opt.value === value}
                            <Check size={14} weight="bold" class="check-icon" />
                        {/if}
                    </div>
                </button>
            {/each}
        </div>
    {/if}
</div>

<style>
    .custom-select-container {
        position: relative;
        flex-shrink: 0;
        display: inline-block;
    }

    .select-trigger {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 0.65rem;
        background: var(--echo-surface, #161618);
        border: 1px solid var(--echo-border-medium, rgba(255, 255, 255, 0.1));
        color: var(--echo-text-1, #FFFFFF);
        padding: 0.42rem 0.85rem;
        border-radius: 8px;
        font-family: var(--echo-font-body, inherit);
        font-size: 0.82rem;
        font-weight: 500;
        cursor: pointer;
        transition: all 0.2s ease;
        min-width: 140px;
    }

    .select-trigger:hover {
        border-color: rgba(181, 142, 98, 0.4);
        background: rgba(255, 255, 255, 0.04);
    }

    .select-trigger-left {
        display: flex;
        align-items: center;
        gap: 0.45rem;
        color: #B58E62;
        min-width: 0;
    }

    .select-prefix {
        color: var(--echo-text-2, rgba(255, 255, 255, 0.6));
        font-weight: 400;
    }

    .select-label {
        color: var(--echo-text-1, #FFFFFF);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    :global(.select-trigger svg) {
        transition: transform 0.2s cubic-bezier(0.4, 0, 0.2, 1);
        color: var(--echo-text-2, rgba(255, 255, 255, 0.6));
        flex-shrink: 0;
    }

    :global(.select-trigger svg.rotated) {
        transform: rotate(180deg);
        color: #B58E62;
    }

    .select-backdrop {
        position: fixed;
        inset: 0;
        z-index: 100;
    }

    .custom-select-menu {
        position: absolute;
        top: calc(100% + 0.5rem);
        right: 0;
        min-width: 100%;
        max-width: 320px;
        display: flex;
        flex-direction: column;
        background: var(--echo-surface, #161618);
        border: 1px solid var(--echo-border-medium, rgba(255, 255, 255, 0.12));
        border-radius: 8px;
        padding: 0.35rem;
        z-index: 101;
        box-shadow: 0 10px 30px -10px rgba(0, 0, 0, 0.7);
        animation: selectSlideDown 0.15s cubic-bezier(0.16, 1, 0.3, 1) forwards;
        transform-origin: top center;
    }

    @keyframes selectSlideDown {
        from {
            opacity: 0;
            transform: translateY(-6px) scale(0.98);
        }
        to {
            opacity: 1;
            transform: translateY(0) scale(1);
        }
    }

    .select-option {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 0.75rem;
        padding: 0.5rem 0.65rem;
        border-radius: 6px;
        background: transparent;
        border: none;
        color: var(--echo-text-2, rgba(255, 255, 255, 0.7));
        font-family: inherit;
        font-size: 0.82rem;
        cursor: pointer;
        transition: all 0.12s ease;
        text-align: left;
        white-space: nowrap;
    }

    .select-option:hover {
        background: rgba(255, 255, 255, 0.06);
        color: var(--echo-text-1, #FFFFFF);
    }

    .select-option.selected {
        color: #B58E62;
        background: rgba(181, 142, 98, 0.12);
        font-weight: 600;
    }

    .option-right {
        display: flex;
        align-items: center;
        gap: 0.4rem;
        margin-left: auto;
    }

    :global(.select-option .check-icon) {
        color: #B58E62;
        flex-shrink: 0;
    }

    .option-badge {
        font-size: 0.58rem;
        font-family: ui-monospace, monospace;
        font-weight: 700;
        padding: 0.08rem 0.32rem;
        border-radius: 3px;
        background: rgba(181, 142, 98, 0.15);
        color: #B58E62;
        border: 1px solid rgba(181, 142, 98, 0.3);
    }
</style>
