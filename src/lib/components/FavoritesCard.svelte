<script lang="ts">
    import { libraryStore } from "$lib/stores/library.svelte";
    import { exploreStore } from "$lib/stores/explore.svelte";
    import { audioStore } from "$lib/stores/audio.svelte";
    import CardPlayButton from "$lib/components/common/CardPlayButton.svelte";
    import { Play, Heart } from "phosphor-svelte";

    let {
        selected = false,
        onclick = () => exploreStore.openFavorites()
    } = $props<{
        selected?: boolean;
        onclick?: () => void;
    }>();

    let songCount = $derived(libraryStore.likedSongs.length);
    let subtitle = $derived(`${songCount} ${songCount === 1 ? 'Favorite Song' : 'Favorite Songs'}`);

    function handlePlayClick(e: MouseEvent) {
        e.stopPropagation();
        if (libraryStore.likedSongs.length > 0) {
            const queueTracks = libraryStore.likedSongs.map(s => ({
                id: s.local_track_id || s.canonical_key,
                remote_track_id: s.last_source_id || undefined,
                title: s.title,
                artist: s.artist || "Unknown Artist",
                album: s.album || undefined,
                cover_art_url: s.cover_art_url || undefined,
                file_path: s.file_path || s.local_file_path || undefined,
                is_local: !!s.local_file_path,
                provider_id: s.last_provider_id || "youtube-wasm",
                duration_ms: s.duration_ms || 210000,
            }));
            audioStore.setQueue(queueTracks, 0);
        }
    }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div 
    class="favorites-card group {selected ? 'selected' : ''}" 
    {onclick}
>
    <div class="art-container">
        <div class="favorites-art-gradient">
            <Heart size={56} weight="fill" color="#D4A86E" class="favorites-heart-icon" />
        </div>

        <div class="play-overlay">
            <CardPlayButton 
                size="lg"
                onclick={handlePlayClick}
                title="Play favorites"
            />
        </div>
    </div>

    <h3 class="card-title">Liked Songs</h3>
    <p class="card-artist">{subtitle}</p>
</div>

<style>
    .favorites-card {
        cursor: pointer;
        display: flex;
        flex-direction: column;
        width: 100%;
        min-width: 0;
    }

    .favorites-card.selected .art-container {
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

    .favorites-card:hover .art-container {
        border-color: rgba(212, 168, 110, 0.45);
        box-shadow: 0 14px 30px -4px rgba(212, 168, 110, 0.25), 0 10px 12px -5px rgba(0, 0, 0, 0.6);
    }

    .favorites-art-gradient {
        width: 100%;
        height: 100%;
        display: flex;
        align-items: center;
        justify-content: center;
        background: linear-gradient(135deg, #1f1a14 0%, #3a2e1d 50%, #15120c 100%);
        position: relative;
    }

    .favorites-art-gradient::before {
        content: "";
        position: absolute;
        inset: 0;
        background: radial-gradient(circle at 30% 30%, rgba(212, 168, 110, 0.25) 0%, transparent 70%);
        pointer-events: none;
    }

    :global(.favorites-heart-icon) {
        filter: drop-shadow(0 4px 12px rgba(212, 168, 110, 0.4));
        transition: transform 0.4s cubic-bezier(0.34, 1.56, 0.64, 1);
    }

    .favorites-card:hover :global(.favorites-heart-icon) {
        transform: scale(1.1) rotate(5deg);
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

    .favorites-card:hover .play-overlay {
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
