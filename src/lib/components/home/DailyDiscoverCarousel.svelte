<script lang="ts">
    import { homeStore, type FederatedTrack } from "$lib/stores/home.svelte";
    import { Sparkle, Play } from "phosphor-svelte";
</script>

<section class="daily-discover-section">
    <div class="section-title-row">
        <div class="title-group">
            <Sparkle size={20} weight="fill" class="sparkle-icon" />
            <h2>Daily Discover</h2>
        </div>
        {#if homeStore.failedProviders.length > 0}
            <span class="section-tag warning" title="Degraded: {homeStore.failedProviders.join(', ')}">Partial Feed</span>
        {:else}
            <span class="section-tag">Curated Provenance</span>
        {/if}
    </div>

    {#if homeStore.isLoadingRemote && homeStore.dailyDiscover.length === 0}
        <div class="carousel-track">
            {#each Array(6) as _}
                <div class="carousel-card skeleton">
                    <div class="card-art-wrapper skeleton-box"></div>
                    <div class="skeleton-line title"></div>
                    <div class="skeleton-line artist"></div>
                </div>
            {/each}
        </div>
    {:else if homeStore.dailyDiscover.length > 0}
        <div class="carousel-track">
            {#each homeStore.dailyDiscover as item}
                <div 
                    class="carousel-card"
                    role="button"
                    tabindex="0"
                    onclick={() => homeStore.playFederatedTrack(item)}
                    onkeydown={(e) => { if (e.key === 'Enter') homeStore.playFederatedTrack(item); }}
                >
                    <div class="card-art-wrapper">
                        {#if item.cover_art_url}
                            <img src={item.cover_art_url.startsWith('/') ? `asset://localhost/${encodeURIComponent(item.cover_art_url)}` : item.cover_art_url} alt={item.title} loading="lazy" />
                        {:else}
                            <div class="placeholder-art"></div>
                        {/if}
                        <div class="card-overlay">
                            <div class="play-bubble">
                                <Play size={20} weight="fill" />
                            </div>
                        </div>
                    </div>
                    <div class="card-info">
                        <span class="card-title">{item.title}</span>
                        <span class="card-artist">{item.artist}</span>
                        {#if item.seed_provenance}
                            <span class="card-provenance">{item.seed_provenance}</span>
                        {/if}
                    </div>
                </div>
            {/each}
        </div>
    {/if}
</section>

<style>
    .daily-discover-section {
        display: flex;
        flex-direction: column;
        gap: 1.25rem;
    }

    .section-title-row {
        display: flex;
        align-items: center;
        justify-content: space-between;
    }

    .title-group {
        display: flex;
        align-items: center;
        gap: 0.65rem;
    }

    .section-title-row h2 {
        font-size: 1.35rem;
        font-weight: 700;
        margin: 0;
        letter-spacing: -0.02em;
    }

    :global(.sparkle-icon) {
        color: #ffd166;
    }

    .section-tag {
        font-size: 0.72rem;
        font-weight: 600;
        text-transform: uppercase;
        letter-spacing: 0.05em;
        color: var(--text-muted, rgba(255, 255, 255, 0.5));
        background: var(--surface-1, rgba(255, 255, 255, 0.05));
        padding: 0.2rem 0.55rem;
        border-radius: 6px;
        border: 1px solid rgba(255, 255, 255, 0.05);
    }

    .section-tag.warning {
        color: #ff9f1c;
        background: rgba(255, 159, 28, 0.1);
        border-color: rgba(255, 159, 28, 0.2);
    }

    .carousel-track {
        display: flex;
        gap: 1.25rem;
        overflow-x: auto;
        scroll-snap-type: x mandatory;
        padding-bottom: 0.5rem;
        scrollbar-width: none;
    }

    .carousel-track::-webkit-scrollbar {
        display: none;
    }

    .carousel-card {
        flex: 0 0 155px;
        scroll-snap-align: start;
        display: flex;
        flex-direction: column;
        gap: 0.55rem;
        cursor: pointer;
        transition: transform 0.2s cubic-bezier(0.16, 1, 0.3, 1);
    }

    .carousel-card:hover {
        transform: translateY(-4px);
    }

    .card-art-wrapper {
        width: 155px;
        height: 155px;
        border-radius: 10px;
        overflow: hidden;
        position: relative;
        background: var(--surface-1, rgba(255, 255, 255, 0.04));
        border: 1px solid rgba(255, 255, 255, 0.06);
    }

    .card-art-wrapper img {
        width: 100%;
        height: 100%;
        object-fit: cover;
    }

    .placeholder-art {
        width: 100%;
        height: 100%;
        background: linear-gradient(135deg, rgba(255, 255, 255, 0.08), rgba(255, 255, 255, 0.02));
    }

    .card-overlay {
        position: absolute;
        inset: 0;
        background: rgba(0, 0, 0, 0.4);
        display: flex;
        align-items: center;
        justify-content: center;
        opacity: 0;
        transition: opacity 0.2s ease;
    }

    .carousel-card:hover .card-overlay {
        opacity: 1;
    }

    .play-bubble {
        width: 40px;
        height: 40px;
        border-radius: 50%;
        background: #fff;
        color: #000;
        display: flex;
        align-items: center;
        justify-content: center;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.35);
        transform: scale(0.9);
        transition: transform 0.2s ease;
    }

    .carousel-card:hover .play-bubble {
        transform: scale(1);
    }

    .card-info {
        display: flex;
        flex-direction: column;
        gap: 0.18rem;
    }

    .card-title {
        font-size: 0.9rem;
        font-weight: 600;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .card-artist {
        font-size: 0.78rem;
        color: var(--text-muted, rgba(255, 255, 255, 0.6));
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .card-provenance {
        display: inline-block;
        font-size: 0.68rem;
        font-weight: 500;
        color: var(--accent-primary, #ffd166);
        background: rgba(255, 209, 102, 0.1);
        padding: 0.1rem 0.4rem;
        border-radius: 4px;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        max-width: 100%;
        margin-top: 0.1rem;
    }

    /* Skeleton */
    .skeleton-box {
        background: linear-gradient(90deg, rgba(255, 255, 255, 0.03) 25%, rgba(255, 255, 255, 0.07) 50%, rgba(255, 255, 255, 0.03) 75%);
        background-size: 200% 100%;
        animation: shimmer 1.5s infinite;
    }

    .skeleton-line {
        height: 11px;
        border-radius: 4px;
        background: linear-gradient(90deg, rgba(255, 255, 255, 0.03) 25%, rgba(255, 255, 255, 0.07) 50%, rgba(255, 255, 255, 0.03) 75%);
        background-size: 200% 100%;
        animation: shimmer 1.5s infinite;
    }

    .skeleton-line.title { width: 80%; }
    .skeleton-line.artist { width: 50%; }

    @keyframes shimmer {
        0% { background-position: 200% 0; }
        100% { background-position: -200% 0; }
    }
</style>
