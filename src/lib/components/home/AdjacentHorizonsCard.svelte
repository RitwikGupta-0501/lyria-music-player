<script lang="ts">
    import { homeStore } from "$lib/stores/home.svelte";
    import { Compass, Play, Sparkle } from "phosphor-svelte";

    let payload = $derived(homeStore.adjacentHorizon);
</script>

{#if payload}
    <section class="adjacent-horizons-section" style="--accent: {payload.accent_color};">
        <div class="horizon-card">
            <div class="horizon-left">
                <div class="horizon-pill">
                    <Compass size={16} weight="fill" />
                    <span>ADJACENT HORIZON • STYLE INVERSION</span>
                </div>
                <h3 class="horizon-tagline">{payload.tagline}</h3>
                <p class="horizon-desc">{payload.description}</p>
                <div class="horizon-actions">
                    <button 
                        class="explore-btn"
                        onclick={() => {
                            if (payload) {
                                homeStore.playRadioMix({
                                    id: `horizon-${payload.suggested_genre.toLowerCase().replace(' ', '-')}`,
                                    title: payload.suggested_genre,
                                    subtitle: payload.tagline,
                                    category: "temporal_mood",
                                    covers: [],
                                    gradient_start: "#1A1A2E",
                                    gradient_end: "#16213E",
                                    seed: payload.seed,
                                });
                            }
                        }}
                    >
                        <Play size={16} weight="fill" />
                        <span>Explore Horizon</span>
                    </button>
                </div>
            </div>

            {#if payload.preview_tracks && payload.preview_tracks.length > 0}
                <div class="horizon-right">
                    <span class="preview-label">Sample Seed Tracks</span>
                    <div class="preview-grid">
                        {#each payload.preview_tracks.slice(0, 3) as track}
                            <div 
                                class="preview-item" 
                                role="button"
                                tabindex="0"
                                onclick={() => homeStore.playFederatedTrack(track)}
                                onkeydown={(e) => { if (e.key === 'Enter') homeStore.playFederatedTrack(track); }}
                            >
                                <div class="preview-art">
                                    {#if track.cover_art_url}
                                        <img src={track.cover_art_url.startsWith('/') ? `asset://localhost/${encodeURIComponent(track.cover_art_url)}` : track.cover_art_url} alt={track.title} />
                                    {:else}
                                        <div class="placeholder-art"></div>
                                    {/if}
                                </div>
                                <div class="preview-info">
                                    <span class="preview-title">{track.title}</span>
                                    <span class="preview-artist">{track.artist}</span>
                                </div>
                            </div>
                        {/each}
                    </div>
                </div>
            {/if}
        </div>
    </section>
{/if}

<style>
    .adjacent-horizons-section {
        display: flex;
        flex-direction: column;
    }

    .horizon-card {
        background: linear-gradient(135deg, rgba(255, 255, 255, 0.05) 0%, rgba(255, 255, 255, 0.015) 100%);
        border: 1px solid rgba(255, 255, 255, 0.08);
        border-left: 3px solid var(--accent, #ffd166);
        border-radius: 14px;
        padding: 1.75rem;
        display: grid;
        grid-template-columns: 1.4fr 1fr;
        gap: 2rem;
        align-items: center;
    }

    @media (max-width: 1024px) {
        .horizon-card {
            grid-template-columns: 1fr;
        }
    }

    .horizon-left {
        display: flex;
        flex-direction: column;
        align-items: flex-start;
        gap: 0.85rem;
    }

    .horizon-pill {
        display: flex;
        align-items: center;
        gap: 0.45rem;
        font-size: 0.72rem;
        font-weight: 700;
        letter-spacing: 0.06em;
        color: var(--accent, #ffd166);
        background: rgba(255, 255, 255, 0.06);
        padding: 0.25rem 0.65rem;
        border-radius: 20px;
    }

    .horizon-tagline {
        font-size: 1.4rem;
        font-weight: 700;
        margin: 0;
        line-height: 1.3;
        letter-spacing: -0.02em;
    }

    .horizon-desc {
        font-size: 0.9rem;
        color: var(--text-muted, rgba(255, 255, 255, 0.65));
        margin: 0;
        line-height: 1.5;
    }

    .horizon-actions {
        margin-top: 0.35rem;
    }

    .explore-btn {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        background: #fff;
        color: #000;
        border: none;
        padding: 0.65rem 1.15rem;
        border-radius: 8px;
        font-size: 0.88rem;
        font-weight: 600;
        cursor: pointer;
        transition: transform 0.15s ease, background 0.2s ease;
    }

    .explore-btn:hover {
        transform: translateY(-2px);
        background: #f0f0f0;
    }

    .horizon-right {
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
        background: rgba(0, 0, 0, 0.2);
        padding: 1.25rem;
        border-radius: 10px;
        border: 1px solid rgba(255, 255, 255, 0.05);
    }

    .preview-label {
        font-size: 0.72rem;
        font-weight: 600;
        text-transform: uppercase;
        letter-spacing: 0.05em;
        color: var(--text-muted, rgba(255, 255, 255, 0.5));
    }

    .preview-grid {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .preview-item {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        padding: 0.4rem 0.5rem;
        border-radius: 6px;
        cursor: pointer;
        transition: background 0.15s ease;
    }

    .preview-item:hover {
        background: rgba(255, 255, 255, 0.06);
    }

    .preview-art {
        width: 36px;
        height: 36px;
        border-radius: 5px;
        overflow: hidden;
        flex-shrink: 0;
        background: rgba(255, 255, 255, 0.05);
    }

    .preview-art img {
        width: 100%;
        height: 100%;
        object-fit: cover;
    }

    .placeholder-art {
        width: 100%;
        height: 100%;
        background: rgba(255, 255, 255, 0.08);
    }

    .preview-info {
        flex: 1;
        min-width: 0;
        display: flex;
        flex-direction: column;
        gap: 0.1rem;
    }

    .preview-title {
        font-size: 0.85rem;
        font-weight: 600;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .preview-artist {
        font-size: 0.75rem;
        color: var(--text-muted, rgba(255, 255, 255, 0.55));
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }
</style>
