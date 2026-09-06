<script lang="ts">
    import { getInitial } from "$lib/utils/format";

    let {
        text = "?",
        size = "md",
        shape = "squircle",
        class: extraClass = "",
    } = $props<{
        text?: string | null;
        size?: "sm" | "md" | "lg" | "xl";
        shape?: "squircle" | "circle" | "rounded";
        class?: string;
    }>();

    let initial = $derived(getInitial(text));
</script>

<div class="echo-squircle-art size-{size} shape-{shape} {extraClass}">
    <span>{initial}</span>
</div>

<style>
    .echo-squircle-art {
        background: #232328;
        display: flex;
        align-items: center;
        justify-content: center;
        color: #B58E62;
        font-family: ui-serif, Georgia, serif;
        font-weight: 700;
        user-select: none;
        flex-shrink: 0;
    }

    .shape-squircle {
        border-radius: 8px;
    }

    .shape-circle {
        border-radius: 50%;
    }

    .shape-rounded {
        border-radius: 4px;
    }

    .size-sm {
        width: 32px;
        height: 32px;
        font-size: 0.9rem;
    }

    .size-md {
        width: 40px;
        height: 40px;
        font-size: 1.1rem;
    }

    .size-lg {
        width: 64px;
        height: 64px;
        font-size: 1.6rem;
    }

    .size-xl {
        width: 100%;
        height: 100%;
        font-size: 2.4rem;
    }
</style>
