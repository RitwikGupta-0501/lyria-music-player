<script lang="ts">
    import { exploreStore, type GenreItem } from "$lib/stores/explore.svelte";

    const CATEGORY_COLORS = [
        "#E2A973", // Brass/Copper
        "#639FAB", // Teal/Cyan
        "#C87D55", // Burnt Amber
        "#9B8EB9", // Lavender Mist
        "#6EAA78", // Sage Green
        "#D9777F", // Rose Bronze
        "#D4AF37", // Metallic Gold
        "#78909C", // Slate Blue
    ];
</script>

{#if exploreStore.categoryGrid.length > 0}
    <section class="explore-section">
        <div class="section-header">
            <h2>Browse by Category</h2>
        </div>
        <div class="category-grid">
            {#each exploreStore.categoryGrid as cat, i}
                {@const color = cat.color_hex || CATEGORY_COLORS[i % CATEGORY_COLORS.length]}
                <button 
                    class="category-tile" 
                    style="--cat-accent: {color}"
                    onclick={() => exploreStore.selectCategory(cat)}
                >
                    <div class="tile-gradient-overlay"></div>
                    <span class="tile-label">{cat.title}</span>
                    <div class="tile-accent-bar"></div>
                </button>
            {/each}
        </div>
    </section>
{/if}

<style>
    .explore-section {
        display: flex;
        flex-direction: column;
        gap: 0.9rem;
    }
    .section-header h2 {
        font-size: 1.15rem;
        font-weight: 700;
        letter-spacing: -0.01em;
        margin: 0;
        color: #EAEAEA;
    }
    .category-grid {
        display: grid;
        grid-template-columns: repeat(4, 1fr);
        gap: 1rem;
    }
    @media (max-width: 900px) {
        .category-grid {
            grid-template-columns: repeat(2, 1fr);
        }
    }
    .category-tile {
        position: relative;
        aspect-ratio: 16 / 9;
        background: #1E1E22;
        border: 1px solid rgba(255, 255, 255, 0.06);
        border-radius: 10px;
        overflow: hidden;
        cursor: pointer;
        display: flex;
        align-items: flex-start;
        padding: 1rem;
        text-align: left;
        transition: transform 0.2s ease, border-color 0.2s ease, box-shadow 0.2s ease;
    }
    .category-tile:hover {
        transform: scale(1.02);
        border-color: rgba(255, 255, 255, 0.2);
        box-shadow: 0 8px 24px rgba(0, 0, 0, 0.35);
    }
    .tile-gradient-overlay {
        position: absolute;
        inset: 0;
        background: radial-gradient(circle at 100% 100%, var(--cat-accent) 0%, transparent 65%);
        opacity: 0.18;
        transition: opacity 0.2s ease;
    }
    .category-tile:hover .tile-gradient-overlay {
        opacity: 0.32;
    }
    .tile-label {
        position: relative;
        z-index: 2;
        font-size: 1.05rem;
        font-weight: 700;
        color: #EAEAEA;
    }
    .tile-accent-bar {
        position: absolute;
        bottom: 0;
        left: 0;
        right: 0;
        height: 3px;
        background: var(--cat-accent);
        opacity: 0.7;
    }
</style>
