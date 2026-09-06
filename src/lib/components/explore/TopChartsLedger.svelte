<script lang="ts">
    import { exploreStore } from "$lib/stores/explore.svelte";
    import TrackRow from "$lib/components/TrackRow.svelte";
    import { audioStore } from "$lib/stores/audio.svelte";
    import { formatDuration, getInitial, isCurrentTrack } from "$lib/utils/format";
    import { Play, Pause, ChartLineUp, Flame, GlobeHemisphereWest, MapPin } from "phosphor-svelte";
    import SectionHeaderSkeleton from "$lib/components/common/SectionHeaderSkeleton.svelte";
    import TabPills, { type TabItem } from "$lib/components/common/TabPills.svelte";

    const chartTabs = $derived<TabItem<"global" | "viral" | "regional">[]>([
        { id: "global", label: "Global Top 50", icon: GlobeHemisphereWest },
        { id: "viral", label: "Trending Viral", icon: Flame },
        { id: "regional", label: `Regional (${exploreStore.userRegion.countryName})`, icon: MapPin },
    ]);
</script>

{#if exploreStore.isLoadingChartTab || exploreStore.isLoading || exploreStore.currentChartTracks.length > 0}
<div class="top-charts-ledger">
    <div class="section-header-row">
        <div class="header-title-group">
            <ChartLineUp size={20} weight="bold" class="section-icon" />
            <h2>Top Charts</h2>
        </div>

        <TabPills
            tabs={chartTabs}
            activeTab={exploreStore.activeChartTab}
            onchange={(tab) => exploreStore.setChartTab(tab)}
            ariaLabel="Top Charts tabs"
        />
    </div>

    <div class="ledger-container">
        {#if exploreStore.isLoadingChartTab || (exploreStore.isLoading && exploreStore.currentChartTracks.length === 0)}
            <div class="ledger-loading-skeleton">
                {#each Array(6) as _}
                    <div class="skeleton-row"></div>
                {/each}
            </div>
        {:else if exploreStore.currentChartTracks.length > 0}
            <div class="ledger-rows-stack">
                {#each exploreStore.currentChartTracks.slice(0, 20) as track, i (track.id || i)}
                    <TrackRow 
                        track={track}
                        index={i + 1}
                        variant="chart"
                    />
                {/each}
            </div>
        {:else}
            <div class="empty-ledger">No top chart tracks available.</div>
        {/if}
    </div>
</div>
{/if}

<style>
    .top-charts-ledger {
        display: flex;
        flex-direction: column;
        gap: 1rem;
        height: 100%;
    }

    .section-header-row {
        display: flex;
        align-items: center;
        justify-content: space-between;
        flex-wrap: wrap;
        gap: 0.75rem;
        min-height: 36px;
    }

    .header-title-group {
        display: flex;
        align-items: center;
        gap: 0.55rem;
    }

    :global(.section-icon) {
        color: #B58E62;
    }

    h2 {
        font-family: var(--echo-font-heading, 'Newsreader', serif);
        font-size: 1.35rem;
        font-weight: 600;
        margin: 0;
        color: #fff;
    }



    .ledger-container {
        flex: 1;
        display: flex;
        flex-direction: column;
        overflow-y: auto;
        background: rgba(18, 18, 22, 0.55);
        border: 1px solid rgba(255, 255, 255, 0.07);
        border-radius: 12px;
        padding: 0.6rem;
        box-sizing: border-box;
        min-height: 0;
    }

    .ledger-container::-webkit-scrollbar {
        width: 4px;
    }

    .ledger-container::-webkit-scrollbar-track {
        background: transparent;
    }

    .ledger-container::-webkit-scrollbar-thumb {
        background: rgba(255, 255, 255, 0.15);
        border-radius: 4px;
    }

    .ledger-container::-webkit-scrollbar-thumb:hover {
        background: rgba(212, 168, 110, 0.5);
    }

    .ledger-rows-stack {
        display: flex;
        flex-direction: column;
        gap: 0.35rem;
        flex: 1;
    }

    .ledger-loading-skeleton {
        display: flex;
        flex-direction: column;
        gap: 0.45rem;
    }

    .skeleton-row {
        height: 48px;
        border-radius: 8px;
        background: rgba(255, 255, 255, 0.04);
        animation: pulse 1.5s infinite ease-in-out;
    }

    @keyframes pulse {
        0%, 100% { opacity: 0.3; }
        50% { opacity: 0.7; }
    }



    .empty-ledger {
        padding: 2rem;
        text-align: center;
        color: rgba(255, 255, 255, 0.4);
        font-size: 0.85rem;
    }
</style>
