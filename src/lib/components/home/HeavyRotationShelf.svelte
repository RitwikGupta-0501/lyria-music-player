<script lang="ts">
    import { homeStore } from "$lib/stores/home.svelte";
    import { Flame, Play, User, Disc } from "phosphor-svelte";

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
                    <User size={14} weight="bold" />
                    <span>Top Artists</span>
                </button>
                <button 
                    class="tab-btn" 
                    class:active={activeTab === "albums"}
                    onclick={() => activeTab = "albums"}
                >
                    <Disc size={14} weight="bold" />
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
                                <img src={artist.avatar_url.startsWith('/') ? `asset://localhost/${encodeURIComponent(artist.avatar_url)}` : artist.avatar_url} alt={artist.artist} loading="lazy" />
                            {:else}
                                <div class="avatar-placeholder">
                                    <span>{artist.artist.charAt(0).toUpperCase()}</span>
                                </div>
                            {/if}
                        </div>
                        <span class="artist-name">{artist.artist}</span>
                        <span class="artist-stat">{artist.total_plays} plays</span>
                    </div>
                {/each}
            </div>
        {:else}
            <div class="albums-carousel-track">
                {#each homeStore.heavyRotation.albums as album}
                    <div class="album-card">
                        <div class="album-art-wrapper">
                            {#if album.cover_art_url}
                                <img src={album.cover_art_url.startsWith('/') ? `asset://localhost/${encodeURIComponent(album.cover_art_url)}` : album.cover_art_url} alt={album.album_title} loading="lazy" />
                            {:else}
                                <div class="placeholder-art"></div>
                            {/if}
                        </div>
                        <span class="album-title">{album.album_title}</span>
                        <span class="album-artist">{album.artist}</span>
                        <span class="album-stat">{album.total_plays} plays</span>
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
        font-size: 1.35rem;
        font-weight: 700;
        margin: 0;
        letter-spacing: -0.02em;
    }

    :global(.flame-icon) {
        color: #ef476f;
    }

    .tabs-group {
        display: flex;
        background: var(--surface-1, rgba(255, 255, 255, 0.05));
        border: 1px solid rgba(255, 255, 255, 0.06);
        border-radius: 8px;
        padding: 2px;
        gap: 2px;
    }

    .tab-btn {
        display: flex;
        align-items: center;
        gap: 0.4rem;
        background: transparent;
        border: none;
        color: var(--text-muted, rgba(255, 255, 255, 0.6));
        font-size: 0.78rem;
        font-weight: 600;
        padding: 0.35rem 0.75rem;
        border-radius: 6px;
        cursor: pointer;
        transition: all 0.15s ease;
    }

    .tab-btn:hover {
        color: #fff;
    }

    .tab-btn.active {
        background: rgba(255, 255, 255, 0.12);
        color: #fff;
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
        gap: 0.4rem;
        cursor: pointer;
        transition: transform 0.2s cubic-bezier(0.16, 1, 0.3, 1);
    }

    .artist-circle-card:hover {
        transform: translateY(-4px);
    }

    .artist-avatar-wrapper {
        width: 100px;
        height: 100px;
        border-radius: 50%;
        overflow: hidden;
        background: var(--surface-1, rgba(255, 255, 255, 0.06));
        border: 2px solid rgba(255, 255, 255, 0.08);
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
        font-size: 1.8rem;
        font-weight: 700;
        background: linear-gradient(135deg, rgba(239, 71, 111, 0.3), rgba(17, 138, 178, 0.3));
    }

    .artist-name {
        font-size: 0.86rem;
        font-weight: 600;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        max-width: 100%;
    }

    .artist-stat {
        font-size: 0.72rem;
        color: var(--text-muted, rgba(255, 255, 255, 0.5));
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
        transform: translateY(-4px);
    }

    .album-art-wrapper {
        width: 140px;
        height: 140px;
        border-radius: 10px;
        overflow: hidden;
        background: var(--surface-1, rgba(255, 255, 255, 0.04));
        border: 1px solid rgba(255, 255, 255, 0.06);
    }

    .album-art-wrapper img {
        width: 100%;
        height: 100%;
        object-fit: cover;
    }

    .placeholder-art {
        width: 100%;
        height: 100%;
        background: linear-gradient(135deg, rgba(255, 255, 255, 0.08), rgba(255, 255, 255, 0.02));
    }

    .album-title {
        font-size: 0.88rem;
        font-weight: 600;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .album-artist {
        font-size: 0.76rem;
        color: var(--text-muted, rgba(255, 255, 255, 0.6));
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .album-stat {
        font-size: 0.7rem;
        color: var(--text-muted, rgba(255, 255, 255, 0.5));
    }
</style>
