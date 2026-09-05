<script lang="ts">
    import { ArrowLeft } from "phosphor-svelte";

    let {
        label = "Back",
        title = "Back (Esc)",
        onclick
    } = $props<{
        label?: string;
        title?: string;
        onclick: () => void;
    }>();
</script>

<button 
    type="button"
    class="echo-back-btn" 
    {onclick} 
    {title}
    aria-label={title}
>
    <ArrowLeft size={16} weight="bold" />
    <span>{label}</span>
</button>

<style>
    .echo-back-btn {
        display: inline-flex;
        align-items: center;
        gap: 0.5rem;
        background: transparent;
        border: 1px solid transparent;
        color: var(--echo-text-2, rgba(255, 255, 255, 0.7));
        font-family: inherit;
        font-size: 0.82rem;
        font-weight: 600;
        padding: 0.45rem 0.85rem;
        border-radius: 8px;
        cursor: pointer;
        width: fit-content;
        transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
    }

    .echo-back-btn:hover {
        color: var(--echo-primary, #e2a973);
        background: rgba(226, 169, 115, 0.1);
        border-color: rgba(226, 169, 115, 0.2);
    }

    .echo-back-btn:active {
        transform: scale(0.97);
    }
</style>
