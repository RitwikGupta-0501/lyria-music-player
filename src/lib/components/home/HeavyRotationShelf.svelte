<script lang="ts">
    import { homeStore } from "$lib/stores/home.svelte";
    import { Flame, User, Disc } from "phosphor-svelte";

    let activeTab = $state<"artists" | "albums">("artists");
</script>

{#if homeStore.heavyRotation.artists.length > 0 || homeStore.heavyRotation.albums.length > 0}
    <section class="heavy-rotation-section">
        <div class="section-title-row">
            <div class="title-group">
                <Flame size={20} weight="fill" class="flame-icon" />
                <h2>Heavy Rotation</h2>
            </div>
            
            <div class="tabs-group">
                <button 
                    class="tab-btn" 
                    class:active={activeTab === "artists"}
                    onclick={() => activeTab = "artists"}
                >
                    <User size={13} weight="bold" />
                    <span>Top Artists</span>
                </button>
                <button 
                    class="tab-btn" 
                    class:active={activeTab === "albums"}
                    onclick={() => activeTab = "albums"}
                >
                    <Disc size={13} weight="bold" />
                    <span>Top Albums</span>
                </button>
            </div>
        </div>

        {#if activeTab === "artists"}
            <div class="artists-carousel-track">
                {#each homeStore.heavyRotation.artists as artist}
                    <div class="artist-circle-card">
                        <div class="artist-avatar-wrapper">
                            {#if artist.avatar_url}
                                <img src={artist.avatar_url.startsWith("/") ? `asset://localhost/${encodeURIComponent(artist.avatar_url)}` : artist.avatar_url} alt={artist.artist} loading="lazy" />
                            {:else}
                                <div class="avatar-placeholder">
                                    <span class="avatar-letter">{artist.artist.charAt(0).toUpperCase()}</span>
                                </div>
                            {/if}
                        </div>
                        <span class="artist-name" title={artist.artist}>{artist.artist}</span>
                        <span class="artist-stat">
                            {#if artist.total_plays > 0}
                                {artist.total_plays} {artist.total_plays === 1 ? "play" : "plays"}
                            {:else}
                                Discovered Seed
                            {/if}
                        </span>
                    </div>
                {/each}
            </div>
        {:else}
            <div class="albums-carousel-track">
                {#each homeStore.heavyRotation.albums as album}
                    <div class="album-card">
                        <div class="album-art-wrapper">
                            {#if album.cover_art_url}
                                <img src={album.cover_art_url.startsWith("/") ? `asset://localhost/${encodeURIComponent(album.cover_art_url)}` : album.cover_art_url} alt={album.album_title} loading="lazy" />
                            {:else}
                                <div class="placeholder-art"></div>
                            {/if}
                        </div>
                        <span class="album-title" title={album.album_title}>{album.album_title}</span>
                        <span class="album-artist" title={album.artist}>{album.artist}</span>
                        <span class="album-stat">
                            {#if album.total_plays > 0}
                                {album.total_plays} {album.total_plays === 1 ? "play" : "plays"}
                            {:else}
                                Discovered Seed
                            {/if}
                        </span>
                    </div>
                {/each}
            </div>
        {/if}
    </section>
{/if}

<style>
    .heavy-rotation-section {
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
        font-family: var(--echo-font-heading, "Playfair Display", serif);
        font-size: 1.35rem;
        font-weight: 600;
        margin: 0;
        letter-spacing: -0.01em;
        color: #fff;
    }

    :global(.flame-icon) {
        color: #B58E62;
    }

    .tabs-group {
        display: flex;
        background: #141416;
        border: 1px solid rgba(255, 255, 255, 0.08);
        border-radius: 6px;
        padding: 3px;
        gap: 3px;
    }

    .tab-btn {
        display: flex;
        align-items: center;
        gap: 0.4rem;
        background: transparent;
        border: none;
        color: rgba(255, 255, 255, 0.6);
        font-size: 0.75rem;
        font-weight: 600;
        padding: 0.3rem 0.7rem;
        border-radius: 4px;
        cursor: pointer;
        transition: all 0.15s ease;
    }

    .tab-btn:hover {
        color: #fff;
    }

    .tab-btn.active {
        background: rgba(181, 142, 98, 0.15);
        color: #D4A86E;
        border: 1px solid rgba(181, 142, 98, 0.25);
    }

    .artists-carousel-track, .albums-carousel-track {
        display: flex;
        gap: 1.5rem;
        overflow-x: auto;
        scroll-snap-type: x mandatory;
        padding-bottom: 0.5rem;
        scrollbar-width: none;
    }

    .artists-carousel-track::-webkit-scrollbar,
    .albums-carousel-track::-webkit-scrollbar {
        display: none;
    }

    .artist-circle-card {
        flex: 0 0 120px;
        display: flex;
        flex-direction: column;
        align-items: center;
        text-align: center;
        gap: 0.45rem;
        cursor: pointer;
        transition: transform 0.2s cubic-bezier(0.16, 1, 0.3, 1);
    }

    .artist-circle-card:hover {
        transform: translateY(-3px);
    }

    .artist-avatar-wrapper {
        width: 100px;
        height: 100px;
        border-radius: 50%;
        overflow: hidden;
        background: #141416;
        border: 1px solid rgba(255, 255, 255, 0.08);
        transition: border-color 0.2s ease;
    }

    .artist-circle-card:hover .artist-avatar-wrapper {
        border-color: rgba(181, 142, 98, 0.35);
    }

    .artist-avatar-wrapper img {
        width: 100%;
        height: 100%;
        object-fit: cover;
    }

    .avatar-placeholder {
        width: 100%;
        height: 100%;
        display: flex;
        align-items: center;
        justify-content: center;
        background: #18181B;
        border: 1px solid rgba(181, 142, 98, 0.2);
        border-radius: 50%;
    }

    .avatar-letter {
        font-family: var(--echo-font-heading, "Playfair Display", serif);
        font-size: 2rem;
        font-weight: 600;
        color: #D4A86E;
    }

    .artist-name {
        font-size: 0.85rem;
        font-weight: 600;
        color: #fff;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        max-width: 100%;
    }

    .artist-stat {
        font-family: var(--echo-font-mono, monospace);
        font-size: 0.68rem;
        color: rgba(255, 255, 255, 0.45);
    }

    .album-card {
        flex: 0 0 140px;
        display: flex;
        flex-direction: column;
        gap: 0.4rem;
        cursor: pointer;
        transition: transform 0.2s cubic-bezier(0.16, 1, 0.3, 1);
    }

    .album-card:hover {
        transform: translateY(-3px);
    }

    .album-art-wrapper {
        width: 140px;
        height: 140px;
        border-radius: 10px;
        overflow: hidden;
        background: #141416;
        border: 1px solid rgba(255, 255, 255, 0.08);
        transition: border-color 0.2s ease;
    }

    .album-card:hover .album-art-wrapper {
        border-color: rgba(181, 142, 98, 0.35);
    }

    .album-art-wrapper img {
        width: 100%;
        height: 100%;
        object-fit: cover;
    }

    .placeholder-art {
        width: 100%;
        height: 100%;
        background: #18181B;
    }

    .album-title {
        font-size: 0.86rem;
        font-weight: 600;
        color: #fff;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .album-artist {
        font-size: 0.76rem;
        color: rgba(255, 255, 255, 0.55);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .album-stat {
        font-family: var(--echo-font-mono, monospace);
        font-size: 0.68rem;
        color: rgba(255, 255, 255, 0.45);
    }
</style>
