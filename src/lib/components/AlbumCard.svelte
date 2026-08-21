<script lang="ts">
    import { libraryStore, type Album } from "$lib/stores/library.svelte";
    import { convertFileSrc } from "@tauri-apps/api/core";
    import { Play, Disc } from "phosphor-svelte";

    let { 
        album, 
        selected = false, 
        onclick 
    } = $props<{ 
        album: Album; 
        selected?: boolean; 
        onclick: () => void 
    }>();

    let artUrl = $state<string | null>(null);

    $effect(() => {
        if (album.cover_art_path) {
            artUrl = convertFileSrc(album.cover_art_path);
        } else {
            libraryStore.getAlbumTracks(album.id).then(tracks => {
                if (tracks && tracks.length > 0) {
                    libraryStore.getArtworkUrl(tracks[0].id, tracks[0].file_path).then(url => {
                        artUrl = url;
                    });
                }
            }).catch(() => {});
        }
    });
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="album-card group {selected ? 'selected' : ''}" {onclick}>
    <div class="art-container">
        {#if artUrl}
            <img 
                src={artUrl} 
                alt={album.title}
                class="art-img" 
                loading="lazy"
            />
        {:else}
            <div class="art-placeholder">
                <Disc size={56} weight="thin" color="rgba(255, 255, 255, 0.35)" />
            </div>
        {/if}

        <div class="play-overlay">
            <div class="play-btn">
                <Play weight="fill" size={28} />
            </div>
        </div>
    </div>
    
    <h3 class="card-title">{album.title}</h3>
    <p class="card-artist">{album.artist || "Unknown Artist"}</p>
</div>

<style>
    .album-card {
        cursor: pointer;
        display: flex;
        flex-direction: column;
        width: 100%;
        min-width: 0;
    }

    .album-card.selected .art-container {
        border-color: var(--echo-primary);
        box-shadow: 0 0 0 2px var(--echo-primary), 0 10px 15px -3px rgba(0, 0, 0, 0.5);
    }

    .art-container {
        width: 100%;
        aspect-ratio: 1;
        border-radius: 1.5rem;
        background-color: #27272a;
        border: 1px solid rgba(255, 255, 255, 0.1);
        overflow: hidden;
        box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.5), 0 4px 6px -4px rgba(0, 0, 0, 0.5);
        margin-bottom: 0.85rem;
        position: relative;
    }

    .art-img {
        width: 100%;
        height: 100%;
        object-fit: cover;
        display: block;
        transition: transform 0.6s cubic-bezier(0.4, 0, 0.2, 1);
    }

    .album-card:hover .art-img {
        transform: scale(1.05);
    }

    .art-placeholder {
        width: 100%;
        height: 100%;
        display: flex;
        align-items: center;
        justify-content: center;
        background: #1e1e24;
    }

    .play-overlay {
        position: absolute;
        inset: 0;
        background-color: rgba(0, 0, 0, 0.4);
        backdrop-filter: blur(4px);
        -webkit-backdrop-filter: blur(4px);
        opacity: 0;
        transition: opacity 0.25s ease;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .album-card:hover .play-overlay {
        opacity: 1;
    }

    .play-btn {
        width: 48px;
        height: 48px;
        border-radius: 50%;
        background-color: var(--echo-primary, #B58E62);
        color: #000000;
        display: flex;
        align-items: center;
        justify-content: center;
        box-shadow: 0 4px 14px rgba(0, 0, 0, 0.5);
        transform: translateY(6px);
        transition: transform 0.25s cubic-bezier(0.34, 1.56, 0.64, 1), background-color 0.2s ease;
    }

    .album-card:hover .play-btn {
        transform: translateY(0);
    }

    .play-btn:hover {
        background-color: #c9a276;
    }

    .card-title {
        font-family: var(--echo-font-body, inherit);
        font-size: 0.95rem;
        font-weight: 600;
        color: var(--echo-text-1, #FFFFFF);
        margin: 0;
        line-height: 1.3;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        width: 100%;
    }

    .card-artist {
        font-size: 0.8rem;
        color: var(--echo-text-2, rgba(255, 255, 255, 0.6));
        margin: 0.2rem 0 0 0;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        width: 100%;
    }
</style>
