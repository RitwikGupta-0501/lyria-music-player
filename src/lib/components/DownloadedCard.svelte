<script lang="ts">
    import { libraryStore } from "$lib/stores/library.svelte";
    import { exploreStore } from "$lib/stores/explore.svelte";
    import CardPlayButton from "$lib/components/common/CardPlayButton.svelte";
    import { DownloadSimple } from "phosphor-svelte";

    let {
        selected = false,
        onclick = () => exploreStore.openDownloaded()
    } = $props<{
        selected?: boolean;
        onclick?: () => void;
    }>();

    let trackCount = $derived(libraryStore.downloadedTracks.length);
    let subtitle = $derived(`${trackCount} ${trackCount === 1 ? 'Track' : 'Tracks'} • Offline`);

    function handlePlayClick(e: MouseEvent) {
        e.stopPropagation();
        libraryStore.playDownloadedTracks();
    }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div 
    class="downloaded-card group {selected ? 'selected' : ''}" 
    {onclick}
>
    <div class="art-container">
        <div class="downloaded-art-gradient">
            <DownloadSimple size={56} weight="bold" color="#D4A86E" class="downloaded-icon" />
        </div>

        <div class="play-overlay">
            <CardPlayButton 
                size="lg"
                onclick={handlePlayClick}
                title="Play downloaded tracks"
            />
        </div>
    </div>

    <h3 class="card-title">Downloaded</h3>
    <p class="card-artist">{subtitle}</p>
</div>

<style>
    .downloaded-card {
        cursor: pointer;
        display: flex;
        flex-direction: column;
        width: 100%;
        min-width: 0;
    }

    .downloaded-card.selected .art-container {
        border-color: var(--echo-primary);
        box-shadow: 0 0 0 2px var(--echo-primary), 0 10px 15px -3px rgba(0, 0, 0, 0.5);
    }

    .art-container {
        width: 100%;
        aspect-ratio: 1;
        border-radius: 1.5rem;
        background-color: #121215;
        border: 1px solid rgba(212, 168, 110, 0.25);
        overflow: hidden;
        box-shadow: 0 10px 25px -5px rgba(212, 168, 110, 0.15), 0 8px 10px -6px rgba(0, 0, 0, 0.5);
        margin-bottom: 0.85rem;
        position: relative;
        isolation: isolate;
        transform: translateZ(0);
        backface-visibility: hidden;
        transition: border-color 0.25s ease, box-shadow 0.25s ease;
    }

    .downloaded-card:hover .art-container {
        border-color: rgba(212, 168, 110, 0.45);
        box-shadow: 0 14px 30px -4px rgba(212, 168, 110, 0.25), 0 10px 12px -5px rgba(0, 0, 0, 0.6);
    }

    .downloaded-art-gradient {
        width: 100%;
        height: 100%;
        display: flex;
        align-items: center;
        justify-content: center;
        background: linear-gradient(135deg, #131c17 0%, #203529 50%, #101913 100%);
        position: relative;
    }

    .downloaded-art-gradient::before {
        content: "";
        position: absolute;
        inset: 0;
        background: radial-gradient(circle at 30% 30%, rgba(212, 168, 110, 0.2) 0%, transparent 70%);
        pointer-events: none;
    }

    :global(.downloaded-icon) {
        filter: drop-shadow(0 4px 12px rgba(212, 168, 110, 0.4));
        transition: transform 0.4s cubic-bezier(0.34, 1.56, 0.64, 1);
    }

    .downloaded-card:hover :global(.downloaded-icon) {
        transform: scale(1.1) translateY(-2px);
    }

    .play-overlay {
        position: absolute;
        inset: 0;
        display: flex;
        align-items: center;
        justify-content: center;
        background: rgba(0, 0, 0, 0.35);
        backdrop-filter: blur(2px);
        opacity: 0;
        transition: opacity 0.25s cubic-bezier(0.4, 0, 0.2, 1);
    }

    .downloaded-card:hover .play-overlay {
        opacity: 1;
    }

    .card-title {
        font-size: 1.1rem;
        font-weight: 600;
        color: #fff;
        margin: 0 0 0.25rem 0;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .card-artist {
        font-size: 0.9rem;
        color: var(--echo-text-2, rgba(255, 255, 255, 0.5));
        margin: 0;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }
</style>
