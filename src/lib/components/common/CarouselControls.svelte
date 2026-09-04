<script lang="ts">
    import type { Snippet } from "svelte";
    import { CaretLeft, CaretRight } from "phosphor-svelte";

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
        isGlass = false,
        prevLabel = "Previous tracks",
        nextLabel = "Next tracks",
        children,
    }: Props = $props();

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

            if (onPrev) {
                onPrev();
            } else {
                container.scrollBy({ left: -pageStep, behavior: "smooth" });
            }
        } else if (onPrev) {
            onPrev();
        }
    }

    function handleNext() {
        if (container) {
            const pageStep = getStep();
            const targetLeft = container.scrollLeft + pageStep;
            if (targetLeft + container.clientWidth >= container.scrollWidth - 6) {
                optimisticCanNext = false;
                canNext = false;
            }
            optimisticCanPrev = true;
            canPrev = true;

            if (onNext) {
                onNext();
            } else {
                container.scrollBy({ left: pageStep, behavior: "smooth" });
            }
        } else if (onNext) {
            onNext();
        }
    }
</script>

<div class="chevron-controls" class:is-glass={isGlass}>
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
        backdrop-filter: blur(12px);
        background: rgba(25, 25, 32, 0.45);
        border-color: rgba(255, 255, 255, 0.12);
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
            background-color 0.15s ease,
            border-color 0.15s ease,
            transform 0.1s ease,
            opacity 0.2s ease;
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
</style>
