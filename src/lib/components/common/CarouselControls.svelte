<script lang="ts">
    import type { Snippet } from "svelte";
    import { CaretLeft, CaretRight } from "phosphor-svelte";
    import { settingsStore } from "$lib/stores/settings.svelte";

    interface Props {
        container?: HTMLElement | null;
        canPrev?: boolean;
        canNext?: boolean;
        onPrev?: () => void;
        onNext?: () => void;
        cardSpan?: number;
        isGlass?: boolean;
        prevLabel?: string;
        nextLabel?: string;
        children?: Snippet;
    }

    let {
        container = null,
        canPrev = $bindable(false),
        canNext = $bindable(true),
        onPrev,
        onNext,
        cardSpan = 196,
        isGlass,
        prevLabel = "Previous tracks",
        nextLabel = "Next tracks",
        children,
    }: Props = $props();

    let effectiveGlass = $derived(isGlass ?? settingsStore.glassyPlayerBar);

    let optimisticCanPrev = $state<boolean | null>(null);
    let optimisticCanNext = $state<boolean | null>(null);

    let effectiveCanPrev = $derived(optimisticCanPrev !== null ? optimisticCanPrev : canPrev);
    let effectiveCanNext = $derived(optimisticCanNext !== null ? optimisticCanNext : canNext);

    $effect(() => {
        canPrev;
        optimisticCanPrev = null;
    });

    $effect(() => {
        canNext;
        optimisticCanNext = null;
    });

    function getStep(): number {
        if (!container) return cardSpan;
        const firstChild = container.firstElementChild as HTMLElement | null;
        const span = firstChild ? firstChild.clientWidth + 20 : cardSpan;
        return Math.max(span, container.clientWidth - span);
    }

    function handlePrev() {
        if (container) {
            const pageStep = getStep();
            const targetLeft = container.scrollLeft - pageStep;
            if (targetLeft <= 6) {
                optimisticCanPrev = false;
                canPrev = false;
            }
            optimisticCanNext = true;
            canNext = true;
            container.scrollBy({ left: -pageStep, behavior: "smooth" });
        }
        onPrev?.();
    }

    function handleNext() {
        if (container) {
            const pageStep = getStep();
            const targetLeft = container.scrollLeft + pageStep;
            const maxScroll = container.scrollWidth - container.clientWidth - 6;
            if (targetLeft >= maxScroll) {
                optimisticCanNext = false;
                canNext = false;
            }
            optimisticCanPrev = true;
            canPrev = true;
            container.scrollBy({ left: pageStep, behavior: "smooth" });
        }
        onNext?.();
    }
</script>

<div class="chevron-controls" class:is-glass={effectiveGlass}>
    <button 
        type="button"
        class="chevron-btn" 
        onclick={handlePrev} 
        disabled={!effectiveCanPrev}
        title={prevLabel}
        aria-label={prevLabel}
    >
        <CaretLeft size={16} weight="bold" />
    </button>
    {#if children}
        {@render children()}
    {/if}
    <button 
        type="button"
        class="chevron-btn" 
        onclick={handleNext} 
        disabled={!effectiveCanNext}
        title={nextLabel}
        aria-label={nextLabel}
    >
        <CaretRight size={16} weight="bold" />
    </button>
</div>

<style>
    .chevron-controls {
        display: flex;
        align-items: center;
        gap: 0.35rem;
        background: rgba(18, 18, 22, 0.75);
        border: 1px solid rgba(255, 255, 255, 0.08);
        border-radius: 20px;
        padding: 0.2rem 0.3rem;
        box-shadow: 0 4px 16px rgba(0, 0, 0, 0.35);
    }

    .chevron-controls.is-glass {
        backdrop-filter: blur(16px) saturate(130%) brightness(1.06);
        -webkit-backdrop-filter: blur(16px) saturate(130%) brightness(1.06);
        background: linear-gradient(180deg, rgba(30, 30, 38, 0.55) 0%, rgba(18, 18, 24, 0.65) 100%);
        border: 1px solid rgba(255, 255, 255, 0.12);
        border-top-color: rgba(255, 255, 255, 0.26);
        border-bottom-color: rgba(255, 255, 255, 0.05);
        box-shadow: 
            inset 0 1px 1.5px rgba(255, 255, 255, 0.15),
            inset 0 -1px 2px rgba(0, 0, 0, 0.4),
            0 6px 20px rgba(0, 0, 0, 0.35);
    }

    .chevron-btn {
        width: 28px;
        height: 28px;
        border-radius: 50%;
        background: rgba(255, 255, 255, 0.08);
        border: 1px solid rgba(255, 255, 255, 0.12);
        color: #ffffff;
        display: flex;
        align-items: center;
        justify-content: center;
        cursor: pointer;
        padding: 0;
        transition:
            color 0.15s ease,
            background 0.15s ease,
            border-color 0.15s ease,
            transform 0.1s ease,
            opacity 0.2s ease,
            box-shadow 0.15s ease;
    }

    .chevron-btn :global(svg) {
        display: block;
        flex-shrink: 0;
    }

    .chevron-btn:hover:not(:disabled) {
        color: #b58e62;
        background: rgba(255, 255, 255, 0.16);
        border-color: rgba(181, 142, 98, 0.4);
    }

    .chevron-btn:active:not(:disabled) {
        transform: scale(0.92);
    }

    .chevron-btn:disabled {
        opacity: 0.25;
        pointer-events: none;
        cursor: default;
    }

    /* Glass states with refraction */
    .chevron-controls.is-glass .chevron-btn {
        background: linear-gradient(180deg, rgba(255, 255, 255, 0.10) 0%, rgba(255, 255, 255, 0.02) 100%);
        border: 1px solid rgba(255, 255, 255, 0.12);
        border-top-color: rgba(255, 255, 255, 0.35);
        border-bottom-color: rgba(255, 255, 255, 0.04);
        box-shadow: 
            inset 0 1px 1px 0 rgba(255, 255, 255, 0.2),
            0 2px 6px rgba(0, 0, 0, 0.25);
    }

    .chevron-controls.is-glass .chevron-btn:hover:not(:disabled) {
        background: linear-gradient(180deg, rgba(255, 255, 255, 0.18) 0%, rgba(255, 255, 255, 0.06) 100%);
        border-color: rgba(226, 169, 115, 0.4);
        border-top-color: rgba(255, 255, 255, 0.5);
        color: #E2A973;
        box-shadow: 
            inset 0 1px 1.5px 0 rgba(255, 255, 255, 0.3),
            0 4px 12px rgba(0, 0, 0, 0.35);
    }
</style>
