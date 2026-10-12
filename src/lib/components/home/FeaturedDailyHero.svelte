<script lang="ts">
    import { homeStore, type FederatedTrack } from "$lib/stores/home.svelte";
    import TrackRow from "$lib/components/TrackRow.svelte";
    import { Sparkle, Play, Waves } from "phosphor-svelte";
    import { resolveCoverArt } from "$lib/utils/media";

    let featuredTrack = $derived<FederatedTrack | null>(
        homeStore.dailyDiscover[0] || homeStore.quickPicks[0] || null
    );

    let previewTracks = $derived<FederatedTrack[]>(
        homeStore.dailyDiscover.length > 1
            ? homeStore.dailyDiscover.slice(1, 4)
            : homeStore.quickPicks.slice(1, 4)
    );

    let artUrl = $derived(
        featuredTrack ? resolveCoverArt(featuredTrack.cover_art_url) : null
    );
</script>

{#if featuredTrack}
    <div class="spotlight-bezel-outer">
        <div class="spotlight-card-core">
            <div class="ambient-glow"></div>
            
            <!-- Vinyl Groove Watermark -->
            <div class="vinyl-watermark">
                <svg viewBox="0 0 240 240" xmlns="http://www.w3.org/2000/svg">
                    <circle cx="120" cy="120" r="100" stroke="rgba(255,255,255,0.03)" stroke-width="1.5" fill="none" />
                    <circle cx="120" cy="120" r="75" stroke="rgba(255,255,255,0.035)" stroke-width="1.5" fill="none" />
                    <circle cx="120" cy="120" r="50" stroke="rgba(181,142,98,0.08)" stroke-width="2" fill="none" />
                </svg>
            </div>

            <!-- Left Half: Featured Track Spotlight -->
            <div class="spotlight-left">
                <div class="spotlight-art-and-meta">
                    {#if artUrl}
                        <div class="hero-art-wrapper">
                            <img src={artUrl} alt={featuredTrack.title} />
                        </div>
                    {/if}

                    <div class="meta-content">
                        <div class="spotlight-eyebrow">
                            <Waves size={11} weight="bold" class="eyebrow-icon" />
                            <span>DAILY SPOTLIGHT • SEED FREQUENCY</span>
                        </div>

                        <h2 class="spotlight-title" title={featuredTrack.title}>{featuredTrack.title}</h2>
                        <p class="spotlight-artist">
                            <span>{featuredTrack.artist}</span>
                            {#if featuredTrack.album}
                                <span class="artist-dot">•</span>
                                <span class="spotlight-album">{featuredTrack.album}</span>
                            {/if}
                        </p>
                    </div>
                </div>

                <div class="spotlight-actions">
                    <button 
                        class="play-hero-btn group"
                        onclick={() => {
                            if (featuredTrack) {
                                homeStore.playFederatedTrack(featuredTrack);
                            }
                        }}
                    >
                        <span>Play Spotlight</span>
                        <div class="btn-icon-bubble">
                            <Play size={13} weight="fill" />
                        </div>
                    </button>
                </div>
            </div>

            <!-- Right Half: Soundscape Preview Ledger -->
            {#if previewTracks.length > 0}
                <div class="spotlight-right">
                    <div class="preview-ledger-header">
                        <span class="ledger-label">UPCOMING SEED FREQUENCIES</span>
                    </div>
                    <div class="preview-ledger">
                        {#each previewTracks as track}
                            <TrackRow 
                                track={track}
                                variant="compact"
                                onclick={() => homeStore.playFederatedTrack(track)}
                            />
                        {/each}
                    </div>
                </div>
            {/if}
        </div>
    </div>
{:else if !homeStore.phase1Loaded || homeStore.isLoadingRemote}
    <div class="spotlight-bezel-outer skeleton-stage">
        <div class="spotlight-card-core">
            <div class="spotlight-left">
                <div class="spotlight-art-and-meta">
                    <div class="hero-art-wrapper skeleton-box"></div>
                    <div class="meta-content">
                        <div class="skeleton-pill skeleton-box stage-eyebrow-ph"></div>
                        <div class="skeleton-line skeleton-box stage-title-ph"></div>
                        <div class="skeleton-line skeleton-box stage-sub-ph"></div>
                    </div>
                </div>

                <div class="spotlight-actions">
                    <div class="skeleton-pill skeleton-box stage-btn-ph"></div>
                </div>
            </div>

            <div class="spotlight-right">
                <div class="preview-ledger-header">
                    <div class="skeleton-line skeleton-box stage-header-ph"></div>
                </div>
                <div class="preview-ledger">
                    {#each Array(3) as _}
                        <div class="stage-row-ph">
                            <div class="skeleton-box stage-row-art"></div>
                            <div class="stage-row-info">
                                <div class="skeleton-line skeleton-box stage-row-title"></div>
                                <div class="skeleton-line skeleton-box stage-row-sub"></div>
                            </div>
                        </div>
                    {/each}
                </div>
            </div>
        </div>
    </div>
{/if}

<style>
    /* ─── Double-Bezel (Doppelrand) Outer Shell ─── */
    .spotlight-bezel-outer {
        position: relative;
        padding: 6px;
        border-radius: 20px;
        background: rgba(255, 255, 255, 0.025);
        border: 1px solid rgba(255, 255, 255, 0.07);
        box-shadow: 0 16px 40px -12px rgba(0, 0, 0, 0.7);
        transition: border-color 0.25s ease;
        width: 100%;
    }

    .spotlight-bezel-outer:hover {
        border-color: rgba(181, 142, 98, 0.25);
    }

    /* ─── Inner Core ─── */
    .spotlight-card-core {
        position: relative;
        background: var(--echo-surface-card);
        border-radius: 14px;
        border: 1px solid rgba(255, 255, 255, 0.06);
        padding: 2.25rem;
        display: grid;
        grid-template-columns: minmax(0, 1.25fr) minmax(0, 0.85fr);
        gap: 2.5rem;
        align-items: center;
        overflow: hidden;
        box-shadow: inset 0 1px 0 0 rgba(255, 255, 255, 0.1);
    }

    @media (max-width: 1024px) {
        .spotlight-card-core {
            grid-template-columns: minmax(0, 1fr);
            gap: 1.75rem;
            padding: 1.5rem;
        }
    }

    .ambient-glow {
        position: absolute;
        top: -40%;
        left: -15%;
        width: 65%;
        height: 180%;
        background: radial-gradient(circle, rgba(181, 142, 98, 0.1) 0%, transparent 65%);
        pointer-events: none;
        filter: blur(50px);
    }

    .vinyl-watermark {
        position: absolute;
        right: -30px;
        bottom: -30px;
        width: 240px;
        height: 240px;
        pointer-events: none;
        opacity: 0.55;
    }

    .spotlight-left {
        display: flex;
        flex-direction: column;
        align-items: flex-start;
        gap: 1.25rem;
        z-index: 1;
        min-width: 0;
    }

    .spotlight-art-and-meta {
        display: flex;
        align-items: center;
        gap: 1.5rem;
        width: 100%;
        min-width: 0;
    }

    .hero-art-wrapper {
        width: 120px;
        height: 120px;
        min-width: 120px;
        border-radius: 12px;
        overflow: hidden;
        background: var(--echo-surface-pill);
        border: 1px solid var(--echo-border-medium);
        box-shadow: 0 12px 28px -6px rgba(0, 0, 0, 0.7);
        flex-shrink: 0;
    }

    .hero-art-wrapper img {
        width: 100%;
        height: 100%;
        object-fit: cover;
        display: block;
    }

    .meta-content {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
        min-width: 0;
        flex: 1;
    }

    .spotlight-eyebrow {
        display: inline-flex;
        align-items: center;
        gap: 0.45rem;
        background: rgba(181, 142, 98, 0.1);
        border: 1px solid rgba(181, 142, 98, 0.22);
        padding: 0.22rem 0.65rem;
        border-radius: 9999px;
        width: fit-content;
    }

    :global(.eyebrow-icon) {
        color: var(--echo-primary-dark);
    }

    .spotlight-eyebrow span {
        font-family: var(--lyria-font-mono, monospace);
        font-size: 0.62rem;
        font-weight: 700;
        letter-spacing: 0.14em;
        color: var(--echo-primary);
    }

    .spotlight-title {
        font-family: var(--echo-font-heading, sans-serif);
        font-size: 1.65rem;
        font-weight: 600;
        line-height: 1.2;
        margin: 0;
        color: #fff;
        letter-spacing: -0.025em;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        max-width: 100%;
    }

    .spotlight-artist {
        font-size: 0.92rem;
        line-height: 1.4;
        color: var(--echo-text-2, rgba(255, 255, 255, 0.65));
        margin: 0;
        display: flex;
        align-items: center;
        gap: 0.45rem;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .artist-dot {
        opacity: 0.4;
    }

    .spotlight-album {
        color: var(--echo-text-3, rgba(255, 255, 255, 0.45));
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .spotlight-actions {
        margin-top: 0.25rem;
    }

    /* ─── Button-in-Button Architecture ─── */
    .play-hero-btn {
        display: inline-flex;
        align-items: center;
        gap: 0.75rem;
        background: var(--echo-primary-dark);
        color: #0e0e10;
        border: none;
        padding: 0.45rem 0.55rem 0.45rem 1.25rem;
        border-radius: 9999px;
        font-size: 0.84rem;
        font-weight: 600;
        letter-spacing: 0.01em;
        cursor: pointer;
        box-shadow: 0 4px 14px rgba(181, 142, 98, 0.25);
        transition: background 0.2s ease, transform 0.15s cubic-bezier(0.16, 1, 0.3, 1), box-shadow 0.2s ease;
    }

    .play-hero-btn:hover {
        background: var(--echo-primary);
        transform: translateY(-1px);
        box-shadow: 0 6px 20px rgba(181, 142, 98, 0.35);
    }

    .play-hero-btn:active {
        transform: scale(0.98);
    }

    .btn-icon-bubble {
        width: 28px;
        height: 28px;
        border-radius: 50%;
        background: rgba(0, 0, 0, 0.22);
        color: #0e0e10;
        display: flex;
        align-items: center;
        justify-content: center;
        transition: transform 0.2s cubic-bezier(0.16, 1, 0.3, 1), background 0.2s ease;
    }

    .play-hero-btn:hover .btn-icon-bubble {
        transform: scale(1.08) translateX(1px);
        background: rgba(0, 0, 0, 0.32);
    }

    .spotlight-right {
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
        background: rgba(14, 14, 18, 0.75);
        padding: 1.25rem;
        border-radius: 12px;
        border: 1px solid rgba(255, 255, 255, 0.06);
        z-index: 1;
        box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.05);
    }

    .preview-ledger-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding-bottom: 0.35rem;
        border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    }

    .ledger-label {
        font-family: var(--lyria-font-mono, monospace);
        font-size: 0.65rem;
        font-weight: 600;
        letter-spacing: 0.12em;
        color: var(--echo-text-3, rgba(255, 255, 255, 0.45));
    }

    .preview-ledger {
        display: flex;
        flex-direction: column;
        gap: 0.4rem;
    }

    /* ─── Skeleton Loading State ─── */
    .skeleton-stage {
        pointer-events: none;
        user-select: none;
    }

    .stage-eyebrow-ph {
        width: 140px;
        height: 20px;
        border-radius: 9999px;
    }

    .stage-title-ph {
        width: 75%;
        height: 26px;
        border-radius: 6px;
        margin: 0.35rem 0 0.15rem 0;
    }

    .stage-sub-ph {
        width: 45%;
        height: 14px;
        border-radius: 4px;
    }

    .stage-btn-ph {
        width: 135px;
        height: 36px;
        border-radius: 9999px;
    }

    .stage-header-ph {
        width: 160px;
        height: 12px;
        border-radius: 4px;
    }

    .stage-row-ph {
        display: flex;
        align-items: center;
        gap: 0.85rem;
        padding: 0.5rem 0.6rem;
    }

    .stage-row-art {
        width: 36px;
        height: 36px;
        border-radius: 6px;
        flex-shrink: 0;
    }

    .stage-row-info {
        display: flex;
        flex-direction: column;
        gap: 0.35rem;
        flex: 1;
    }

    .stage-row-title {
        width: 70%;
        height: 12px;
        border-radius: 3px;
    }

    .stage-row-sub {
        width: 40%;
        height: 10px;
        border-radius: 3px;
    }
</style>
