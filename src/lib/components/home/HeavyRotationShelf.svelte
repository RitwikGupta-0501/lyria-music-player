<script lang="ts">
    import { homeStore, type HeavyRotationAlbumItem, type HeavyRotationArtistItem } from "$lib/stores/home.svelte";
    import { exploreStore } from "$lib/stores/explore.svelte";
    import { libraryStore } from "$lib/stores/library.svelte";
    import { Flame, User, Disc, CaretLeft, CaretRight } from "phosphor-svelte";
    import { resolveCoverArt } from "$lib/utils/media";

    let activeTab = $state<"artists" | "albums">("artists");

    let artistsTrack = $state<HTMLElement | null>(null);
    let albumsTrack = $state<HTMLElement | null>(null);
    let canScrollLeft = $state(false);
    let canScrollRight = $state(true);
    let hasOverflow = $state(false);

    function updateScrollState() {
        const track = activeTab === "artists" ? artistsTrack : albumsTrack;
        if (!track) {
            hasOverflow = false;
            return;
        }
        const { scrollLeft, scrollWidth, clientWidth } = track;
        hasOverflow = scrollWidth > clientWidth + 6;
        canScrollLeft = scrollLeft > 6;
        canScrollRight = scrollLeft + clientWidth < scrollWidth - 6;
    }

    $effect(() => {
        const _tab = activeTab;
        const _artists = homeStore.heavyRotation.artists.length;
        const _albums = homeStore.heavyRotation.albums.length;
        setTimeout(updateScrollState, 50);
    });

    function scrollPrev() {
        const track = activeTab === "artists" ? artistsTrack : albumsTrack;
        if (!track) return;
        const span = activeTab === "artists" ? 144 : 184;
        const pageStep = Math.max(span, track.clientWidth - span);
        track.scrollBy({ left: -pageStep, behavior: "smooth" });
    }

    function scrollNext() {
        const track = activeTab === "artists" ? artistsTrack : albumsTrack;
        if (!track) return;
        const span = activeTab === "artists" ? 144 : 184;
        const pageStep = Math.max(span, track.clientWidth - span);
        track.scrollBy({ left: pageStep, behavior: "smooth" });
    }

    function handleArtistClick(artist: HeavyRotationArtistItem) {
        exploreStore.openArtist({
            id: artist.artist,
            name: artist.artist,
        });
    }

    function formatAlbumDisplay(title: string): string {
        return (title || "")
            .replace(/\s*\((?:Original Motion Picture Soundtrack|Original Soundtrack|Deluxe Edition|Deluxe Version|Expanded Edition|Special Edition|Bonus Track Edition|Remastered|Anniversary Edition)\)/i, "")
            .replace(/\s*\[(?:Original Motion Picture Soundtrack|Original Soundtrack|Deluxe Edition|Deluxe Version|Expanded Edition|Special Edition|Bonus Track Edition|Remastered|Anniversary Edition)\]/i, "")
            .trim();
    }

    function handleAlbumClick(album: HeavyRotationAlbumItem) {
        const localMatch = libraryStore.albums.find(a => 
            a.title.toLowerCase() === album.album_title.toLowerCase() &&
            (!album.artist || !a.artist || a.artist.toLowerCase() === album.artist.toLowerCase())
        );
        if (localMatch) {
            exploreStore.openLocalAlbum(localMatch);
        } else {
            exploreStore.openAlbum({
                id: album.album_title,
                title: album.album_title,
                artist: album.artist,
                cover_art_url: album.cover_art_url,
            });
        }
    }
</script>

<svelte:window onresize={updateScrollState} />

{#if homeStore.heavyRotation.artists.length > 0 || homeStore.heavyRotation.albums.length > 0}
    <section class="heavy-rotation-section">
        <div class="section-title-row">
            <div class="title-group">
                <Flame size={20} weight="fill" class="flame-icon" />
                <h2>Heavy Rotation</h2>
            </div>
            
            <div class="controls-wrapper">
                <div class="tabs-group">
                    <button 
                        class="tab-btn" 
                        class:active={activeTab === "artists"}
                        onclick={() => { activeTab = "artists"; }}
                    >
                        <User size={13} weight="bold" />
                        <span>Top Artists</span>
                    </button>
                    <button 
                        class="tab-btn" 
                        class:active={activeTab === "albums"}
                        onclick={() => { activeTab = "albums"; }}
                    >
                        <Disc size={13} weight="bold" />
                        <span>Top Albums</span>
                    </button>
                </div>

                {#if hasOverflow}
                    <div class="chevron-controls">
                        <button 
                            class="chevron-btn" 
                            onclick={scrollPrev} 
                            disabled={!canScrollLeft}
                            title="Scroll Left"
                            aria-label="Previous items"
                        >
                            <CaretLeft size={16} weight="bold" />
                        </button>
                        <button 
                            class="chevron-btn" 
                            onclick={scrollNext} 
                            disabled={!canScrollRight}
                            title="Scroll Right"
                            aria-label="Next items"
                        >
                            <CaretRight size={16} weight="bold" />
                        </button>
                    </div>
                {/if}
            </div>
        </div>

        {#if activeTab === "artists"}
            <div 
                class="artists-carousel-track"
                bind:this={artistsTrack}
                onscroll={updateScrollState}
            >
                {#each homeStore.heavyRotation.artists as artist}
                    <div 
                        class="artist-circle-card"
                        role="button"
                        tabindex="0"
                        onclick={() => handleArtistClick(artist)}
                        onkeydown={(e) => { if (e.key === "Enter") handleArtistClick(artist); }}
                    >
                        <div class="artist-avatar-wrapper">
                            {#if resolveCoverArt(artist.avatar_url)}
                                <img src={resolveCoverArt(artist.avatar_url)} alt={artist.artist} loading="lazy" />
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
            <div 
                class="albums-carousel-track"
                bind:this={albumsTrack}
                onscroll={updateScrollState}
            >
                {#each homeStore.heavyRotation.albums as album}
                    <div 
                        class="album-card"
                        role="button"
                        tabindex="0"
                        onclick={() => handleAlbumClick(album)}
                        onkeydown={(e) => { if (e.key === "Enter") handleAlbumClick(album); }}
                    >
                        <div class="album-art-wrapper">
                            {#if resolveCoverArt(album.cover_art_url)}
                                <img src={resolveCoverArt(album.cover_art_url)} alt={album.album_title} loading="lazy" />
                            {:else}
                                <div class="placeholder-art"></div>
                            {/if}
                        </div>
                        <span class="album-title" title={album.album_title}>{formatAlbumDisplay(album.album_title)}</span>
                        <span class="album-artist" title={album.artist}>{album.artist}</span>
                        {#if album.total_plays > 0}
                            <span class="album-stat">
                                {album.total_plays} {album.total_plays === 1 ? "play" : "plays"}
                            </span>
                        {/if}
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

    .controls-wrapper {
        display: flex;
        align-items: center;
        gap: 0.75rem;
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

    .chevron-controls {
        display: flex;
        align-items: center;
        gap: 0.35rem;
        background: rgba(18, 18, 22, 0.75);
        border: 1px solid rgba(255, 255, 255, 0.08);
        border-radius: 20px;
        padding: 0.2rem 0.3rem;
        box-shadow: 0 4px 16px rgba(0, 0, 0, 0.35);
    }

    .chevron-btn {
        width: 28px;
        height: 28px;
        border-radius: 50%;
        background: rgba(255, 255, 255, 0.08);
        border: 1px solid rgba(255, 255, 255, 0.12);
        color: #FFFFFF;
        display: flex;
        align-items: center;
        justify-content: center;
        cursor: pointer;
        padding: 0;
        transition: color 0.15s ease, background-color 0.15s ease, border-color 0.15s ease, transform 0.1s ease, opacity 0.2s ease;
    }

    .chevron-btn :global(svg) {
        display: block;
        flex-shrink: 0;
        fill: currentColor;
    }

    .chevron-btn:hover:not(:disabled) {
        color: #B58E62;
        background: rgba(255, 255, 255, 0.16);
        border-color: rgba(181, 142, 98, 0.4);
    }

    .chevron-btn:active:not(:disabled) {
        transform: scale(0.92);
    }

    .chevron-btn:disabled {
        opacity: 0.25;
        pointer-events: none;
        cursor: default;
    }

    .artists-carousel-track, .albums-carousel-track {
        display: flex;
        gap: 1.5rem;
        overflow-x: auto;
        scroll-snap-type: x mandatory;
        scroll-behavior: smooth;
        padding-bottom: 0.5rem;
        scrollbar-width: none;
        will-change: scroll-position;
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
        scroll-snap-align: start;
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
        background: #1C1C20;
    }

    .avatar-letter {
        font-family: var(--echo-font-heading, "Playfair Display", serif);
        font-size: 1.8rem;
        font-weight: 700;
        color: #B58E62;
    }

    .artist-name {
        font-size: 0.82rem;
        font-weight: 600;
        color: #fff;
        max-width: 120px;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .artist-stat {
        font-family: var(--echo-font-mono, monospace);
        font-size: 0.65rem;
        color: rgba(255, 255, 255, 0.4);
    }

    /* Album Card */
    .album-card {
        flex: 0 0 160px;
        display: flex;
        flex-direction: column;
        gap: 0.45rem;
        cursor: pointer;
        scroll-snap-align: start;
        transition: transform 0.2s cubic-bezier(0.16, 1, 0.3, 1);
    }

    .album-card:hover {
        transform: translateY(-3px);
    }

    .album-art-wrapper {
        width: 160px;
        height: 160px;
        border-radius: 8px;
        overflow: hidden;
        background: #141416;
        border: 1px solid rgba(255, 255, 255, 0.08);
        transition: border-color 0.2s ease, box-shadow 0.2s ease;
    }

    .album-card:hover .album-art-wrapper {
        border-color: rgba(181, 142, 98, 0.35);
        box-shadow: 0 8px 24px -4px rgba(0, 0, 0, 0.5);
    }

    .album-art-wrapper img {
        width: 100%;
        height: 100%;
        object-fit: cover;
    }

    .placeholder-art {
        width: 100%;
        height: 100%;
        background: #1C1C20;
    }

    .album-title {
        font-size: 0.85rem;
        font-weight: 600;
        color: #fff;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .album-artist {
        font-size: 0.76rem;
        color: rgba(255, 255, 255, 0.6);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .album-stat {
        font-family: var(--echo-font-mono, monospace);
        font-size: 0.65rem;
        color: rgba(255, 255, 255, 0.4);
    }
</style>
