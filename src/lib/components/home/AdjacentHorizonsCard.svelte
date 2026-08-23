<script lang="ts">
    import { homeStore } from "$lib/stores/home.svelte";
    import { Compass, Play } from "phosphor-svelte";

    let payload = $derived(homeStore.adjacentHorizon);

    function formatDuration(ms?: number | null): string {
        if (!ms) return "";
        const s = Math.floor(ms / 1000);
        const mins = Math.floor(s / 60);
        const secs = s % 60;
        return `${mins}:${secs < 10 ? "0" : ""}${secs}`;
    }
</script>

{#if payload}
    <section class="adjacent-horizons-section">
        <div class="horizon-card">
            <!-- Left 50%: Editorial Copy & Solid Brass CTA -->
            <div class="horizon-left">
                <div class="horizon-pill">
                    <Compass size={14} weight="fill" />
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
                                    id: `horizon-${payload.suggested_genre.toLowerCase().replace(" ", "-")}`,
                                    title: payload.suggested_genre,
                                    subtitle: payload.tagline,
                                    category: "temporal_mood",
                                    covers: [],
                                    gradient_start: "#141416",
                                    gradient_end: "#18181B",
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

            <!-- Right 50%: Compact Clean Track Ledger -->
            {#if payload.preview_tracks && payload.preview_tracks.length > 0}
                <div class="horizon-right">
                    <span class="preview-label">Sample Seed Tracks</span>
                    <div class="preview-ledger">
                        {#each payload.preview_tracks.slice(0, 3) as track}
                            <div 
                                class="ledger-row" 
                                role="button"
                                tabindex="0"
                                onclick={() => homeStore.playFederatedTrack(track)}
                                onkeydown={(e) => { if (e.key === "Enter") homeStore.playFederatedTrack(track); }}
                            >
                                <div class="ledger-art">
                                    {#if track.cover_art_url}
                                        <img src={track.cover_art_url.startsWith("/") ? `asset://localhost/${encodeURIComponent(track.cover_art_url)}` : track.cover_art_url} alt={track.title} />
                                    {:else}
                                        <div class="placeholder-art"></div>
                                    {/if}
                                    <div class="ledger-hover-overlay">
                                        <Play size={14} weight="fill" color="#fff" />
                                    </div>
                                </div>
                                <div class="ledger-info">
                                    <span class="ledger-title">{track.title}</span>
                                    <span class="ledger-artist">{track.artist}</span>
                                </div>
                                {#if track.duration_ms}
                                    <span class="ledger-time">{formatDuration(track.duration_ms)}</span>
                                {/if}
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
        background: #141416;
        border: 1px solid rgba(255, 255, 255, 0.08);
        border-radius: 14px;
        padding: 2rem;
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 2.5rem;
        align-items: center;
        transition: border-color 0.25s ease, box-shadow 0.25s ease;
    }

    .horizon-card:hover {
        border-color: rgba(181, 142, 98, 0.35);
        box-shadow: 0 12px 32px -8px rgba(0, 0, 0, 0.6);
    }

    @media (max-width: 1024px) {
        .horizon-card {
            grid-template-columns: 1fr;
            gap: 1.75rem;
        }
    }

    .horizon-left {
        display: flex;
        flex-direction: column;
        align-items: flex-start;
        gap: 0.9rem;
    }

    .horizon-pill {
        display: inline-flex;
        align-items: center;
        gap: 0.45rem;
        font-family: var(--echo-font-mono, monospace);
        font-size: 0.65rem;
        font-weight: 700;
        letter-spacing: 0.08em;
        color: #B58E62;
        background: rgba(181, 142, 98, 0.1);
        border: 1px solid rgba(181, 142, 98, 0.2);
        padding: 0.25rem 0.65rem;
        border-radius: 4px;
    }

    .horizon-tagline {
        font-family: var(--echo-font-heading, "Playfair Display", serif);
        font-size: 1.55rem;
        font-weight: 600;
        margin: 0;
        line-height: 1.25;
        color: #fff;
        letter-spacing: -0.01em;
    }

    .horizon-desc {
        font-size: 0.88rem;
        color: rgba(255, 255, 255, 0.65);
        margin: 0;
        line-height: 1.55;
    }

    .horizon-actions {
        margin-top: 0.5rem;
    }

    .explore-btn {
        display: inline-flex;
        align-items: center;
        gap: 0.55rem;
        background: #B58E62;
        color: #0E0E10;
        border: none;
        padding: 0.65rem 1.25rem;
        border-radius: 6px;
        font-size: 0.85rem;
        font-weight: 600;
        letter-spacing: 0.02em;
        cursor: pointer;
        transition: background 0.2s ease, transform 0.15s ease, box-shadow 0.2s ease;
    }

    .explore-btn:hover {
        background: #D4A86E;
        transform: translateY(-1px);
        box-shadow: 0 4px 14px rgba(181, 142, 98, 0.3);
    }

    .horizon-right {
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
        background: #18181B;
        padding: 1.25rem;
        border-radius: 10px;
        border: 1px solid rgba(255, 255, 255, 0.06);
    }

    .preview-label {
        font-family: var(--echo-font-mono, monospace);
        font-size: 0.65rem;
        font-weight: 600;
        text-transform: uppercase;
        letter-spacing: 0.08em;
        color: rgba(255, 255, 255, 0.5);
    }

    .preview-ledger {
        display: flex;
        flex-direction: column;
        gap: 0.4rem;
    }

    .ledger-row {
        display: flex;
        align-items: center;
        gap: 0.85rem;
        padding: 0.45rem 0.6rem;
        border-radius: 6px;
        cursor: pointer;
        transition: background 0.15s ease;
    }

    .ledger-row:hover {
        background: rgba(255, 255, 255, 0.05);
    }

    .ledger-art {
        width: 40px;
        height: 40px;
        border-radius: 8px;
        overflow: hidden;
        flex-shrink: 0;
        position: relative;
        background: rgba(255, 255, 255, 0.05);
        border: 1px solid rgba(255, 255, 255, 0.08);
    }

    .ledger-art img {
        width: 100%;
        height: 100%;
        object-fit: cover;
    }

    .placeholder-art {
        width: 100%;
        height: 100%;
        background: #202024;
    }

    .ledger-hover-overlay {
        position: absolute;
        inset: 0;
        background: rgba(0, 0, 0, 0.5);
        display: flex;
        align-items: center;
        justify-content: center;
        opacity: 0;
        transition: opacity 0.15s ease;
    }

    .ledger-row:hover .ledger-hover-overlay {
        opacity: 1;
    }

    .ledger-info {
        flex: 1;
        min-width: 0;
        display: flex;
        flex-direction: column;
        gap: 0.15rem;
    }

    .ledger-title {
        font-size: 0.85rem;
        font-weight: 600;
        color: #fff;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .ledger-artist {
        font-size: 0.75rem;
        color: rgba(255, 255, 255, 0.55);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .ledger-time {
        font-family: var(--echo-font-mono, monospace);
        font-size: 0.72rem;
        color: rgba(255, 255, 255, 0.4);
        flex-shrink: 0;
    }
</style>
