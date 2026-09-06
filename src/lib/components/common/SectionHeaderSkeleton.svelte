<script lang="ts">
    interface Props {
        hasControls?: boolean;
        titleWidth?: string;
        iconSize?: number;
    }

    let { hasControls = false, titleWidth = "140px", iconSize = 20 }: Props = $props();
</script>

<div class="section-header-skeleton" style="--title-width: {titleWidth}; --icon-size: {iconSize}px;">
    <div class="header-left">
        <div class="skeleton-circle icon-placeholder"></div>
        <div class="skeleton-line title-placeholder"></div>
    </div>
    {#if hasControls}
        <div class="skeleton-pill controls-placeholder"></div>
    {/if}
</div>

<style>
    .section-header-skeleton {
        display: flex;
        align-items: center;
        justify-content: space-between;
        min-height: 36px;
        width: 100%;
    }

    .header-left {
        display: flex;
        align-items: center;
        gap: 0.65rem;
    }

    .icon-placeholder {
        width: var(--icon-size, 20px);
        height: var(--icon-size, 20px);
    }

    .title-placeholder {
        width: var(--title-width, 140px);
        height: 18px;
        border-radius: 6px;
    }

    .controls-placeholder {
        width: 68px;
        height: 32px;
        border-radius: 20px;
    }
</style>
