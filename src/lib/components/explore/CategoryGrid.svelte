<script lang="ts">
    import { exploreStore } from "$lib/stores/explore.svelte";
    import { SquaresFour, CaretDown, CaretUp } from "phosphor-svelte";
    import SectionHeaderSkeleton from "$lib/components/common/SectionHeaderSkeleton.svelte";

    const CATEGORY_COLORS = [
        "#D4A86E", // Echo Brass
        "#5B8C96", // Deep Teal
        "#C87D55", // Burnt Amber
        "#8B7EA8", // Muted Lavender
        "#5E9E68", // Sage Green
        "#C46B72", // Rose Bronze
        "#C5A059", // Warm Gold
        "#667C8A", // Slate Blue
    ];
</script>

{#if exploreStore.isLoading && exploreStore.categoryGrid.length === 0}
    <section class="explore-category-section">
        <SectionHeaderSkeleton hasControls={true} titleWidth="180px" />
        <div class="category-grid">
            {#each Array(8) as _}
                <div class="category-tile skeleton skeleton-box"></div>
            {/each}
        </div>
    </section>
{:else if exploreStore.categoryGrid.length > 0}
    <section class="explore-category-section">
        <div class="section-header">
            <div class="title-group">
                <SquaresFour size={20} weight="bold" class="section-icon" />
                <h2>Browse Moods & Genres</h2>
            </div>
            {#if exploreStore.categoryGrid.length > 8}
                <button 
                    type="button" 
                    class="see-more-btn"
                    onclick={() => exploreStore.toggleAllCategories()}
                >
                    <span>{exploreStore.showAllCategories ? "See Less" : "See All (" + exploreStore.categoryGrid.length + ")"}</span>
                    {#if exploreStore.showAllCategories}
                        <CaretUp size={13} weight="bold" />
                    {:else}
                        <CaretDown size={13} weight="bold" />
                    {/if}
                </button>
            {/if}
        </div>

        <div class="category-grid">
            {#each exploreStore.displayedCategories as cat, i}
                {@const color = cat.color_hex || CATEGORY_COLORS[i % CATEGORY_COLORS.length]}
                <button 
                    class="category-tile" 
                    style="--cat-accent: {color}"
                    onclick={() => exploreStore.selectCategory(cat)}
                >
                    <div class="tile-ambient-glow"></div>
                    <span class="tile-label">{cat.title}</span>
                    <div class="tile-bottom-accent"></div>
                </button>
            {/each}
        </div>
    </section>
{/if}

<style>
    .explore-category-section {
        display: flex;
        flex-direction: column;
        gap: 1.1rem;
    }

    .section-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        min-height: 36px;
    }

    .title-group {
        display: flex;
        align-items: center;
        gap: 0.55rem;
    }

    :global(.section-icon) {
        color: #B58E62;
    }

    .see-more-btn {
        display: flex;
        align-items: center;
        gap: 0.35rem;
        background: rgba(255, 255, 255, 0.04);
        border: 1px solid rgba(255, 255, 255, 0.08);
        color: var(--echo-primary, #d4a86e);
        font-family: var(--echo-font-mono, monospace);
        font-size: 0.68rem;
        font-weight: 600;
        letter-spacing: 0.04em;
        padding: 0.28rem 0.65rem;
        border-radius: 6px;
        cursor: pointer;
        transition: background-color 0.15s ease, border-color 0.15s ease, color 0.15s ease, transform 0.15s ease;
    }

    .see-more-btn:hover {
        background: rgba(181, 142, 98, 0.15);
        border-color: rgba(181, 142, 98, 0.35);
        color: #fff;
        transform: translateY(-1px);
    }

    h2 {
        font-family: var(--echo-font-heading, "Playfair Display", serif);
        font-size: 1.35rem;
        font-weight: 600;
        margin: 0;
        color: #fff;
    }

    .category-grid {
        display: grid;
        grid-template-columns: repeat(4, 1fr);
        gap: 0.85rem;
    }

    @media (max-width: 900px) {
        .category-grid {
            grid-template-columns: repeat(2, 1fr);
        }
    }

    .category-tile {
        position: relative;
        height: 76px;
        background: rgba(18, 18, 22, 0.55);
        border: 1px solid rgba(255, 255, 255, 0.07);
        border-radius: 10px;
        overflow: hidden;
        cursor: pointer;
        display: flex;
        align-items: center;
        padding: 0 1.25rem;
        text-align: left;
        transform: translateZ(0);
        backface-visibility: hidden;
        transition: transform 0.2s cubic-bezier(0.16, 1, 0.3, 1), border-color 0.2s ease, background 0.2s ease;
    }

    .category-tile:hover {
        transform: translateY(-2px);
        background: rgba(26, 26, 32, 0.75);
        border-color: rgba(181, 142, 98, 0.35);
    }

    .tile-ambient-glow {
        position: absolute;
        inset: 0;
        background: radial-gradient(circle at 100% 50%, var(--cat-accent) 0%, transparent 60%);
        opacity: 0.12;
        transition: opacity 0.25s ease;
    }

    .category-tile:hover .tile-ambient-glow {
        opacity: 0.25;
    }

    .tile-label {
        position: relative;
        z-index: 2;
        font-family: var(--echo-font-heading, "Playfair Display", serif);
        font-size: 1.05rem;
        font-weight: 600;
        color: #FFFFFF;
        letter-spacing: -0.01em;
    }

    .tile-bottom-accent {
        position: absolute;
        bottom: 0;
        left: 0;
        right: 0;
        height: 2px;
        background: var(--cat-accent);
        opacity: 0.5;
        transition: opacity 0.2s ease, height 0.2s ease;
    }

    .category-tile:hover .tile-bottom-accent {
        opacity: 0.9;
        height: 3px;
    }

    .category-tile.skeleton {
        height: 76px;
        border-radius: 10px;
        border: 1px solid rgba(255, 255, 255, 0.06);
        pointer-events: none;
    }

</style>
