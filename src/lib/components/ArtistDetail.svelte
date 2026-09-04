<script lang="ts">
    import { exploreStore, type ArtistDetailResult } from "$lib/stores/explore.svelte";
    import PlaylistCard from "$lib/components/PlaylistCard.svelte";
    import { audioStore } from "$lib/stores/audio.svelte";
    import { settingsStore } from "$lib/stores/settings.svelte";
    import {
        ArrowLeft,
        Play,
        Pause,
        Shuffle,
        Sparkle,
        Disc,
        MicrophoneStage,
        Users,
        CaretRight,
        Video,
        Playlist,
    } from "phosphor-svelte";

    let { onBack } = $props<{ onBack: () => void }>();

    let artist = $derived(exploreStore.selectedArtist);

    let showAllTopSongs = $state(false);
    let showAllAlbums = $state(false);
    let showAllSingles = $state(false);
    let showAllVideos = $state(false);
    let showAllFeatured = $state(false);
    let showAllSimilar = $state(false);

    function isCurrentTrack(track: any): boolean {
        return audioStore.currentQueueTrack?.title === track.title;
    }

    function playTopTracks() {
        if (!artist || artist.top_tracks.length === 0) return;
        const pId = artist.provider_id || "youtube-wasm";
        const queueTracks = artist.top_tracks.map(t => ({
            id: t.id,
            title: t.title,
            artist: t.artist || artist?.name || "Unknown Artist",
            album: undefined,
            remote_track_id: t.id,
            provider_id: t.provider_id || pId,
            cover_art_url: t.cover_art_url || artist?.avatar_url,
            duration_ms: t.duration_ms,
        }));
        audioStore.setQueue(queueTracks as any, 0);
    }

    function shuffleTopTracks() {
        if (!artist || artist.top_tracks.length === 0) return;
        const pId = artist.provider_id || "youtube-wasm";
        const shuffled = [...artist.top_tracks].sort(() => Math.random() - 0.5);
        const queueTracks = shuffled.map(t => ({
            id: t.id,
            title: t.title,
            artist: t.artist || artist?.name || "Unknown Artist",
            album: undefined,
            remote_track_id: t.id,
            provider_id: t.provider_id || pId,
            cover_art_url: t.cover_art_url || artist?.avatar_url,
            duration_ms: t.duration_ms,
        }));
        audioStore.setQueue(queueTracks as any, 0);
    }

    function formatDuration(ms?: number | null): string {
        if (!ms) return "--:--";
        const totalSeconds = Math.floor(ms / 1000);
        const minutes = Math.floor(totalSeconds / 60);
        const seconds = totalSeconds % 60;
        return `${minutes}:${seconds.toString().padStart(2, '0')}`;
    }

    function getInitial(name: string): string {
        return name ? name.charAt(0).toUpperCase() : "?";
    }
</script>

<div class="artist-detail-container">
    <!-- Navigation Topbar -->
    <div class="artist-topbar">
        <button class="back-btn" onclick={onBack}>
            <ArrowLeft size={16} weight="bold" />
            <span>Back</span>
        </button>
    </div>

    {#if exploreStore.isLoadingArtist}
        <div class="artist-loading-state">
            <div class="artist-skeleton-header">
                <div class="skeleton-avatar"></div>
                <div class="skeleton-meta">
                    <div class="skeleton-line lg"></div>
                    <div class="skeleton-line sm"></div>
                </div>
            </div>
        </div>
    {:else if artist}
        <!-- Hero Header -->
        <header class="artist-hero-header">
            <div class="artist-avatar-wrapper">
                {#if artist.avatar_url}
                    <img src={artist.avatar_url} alt={artist.name} class="artist-avatar-img" />
                {:else}
                    <div class="artist-avatar-fallback">
                        <span>{getInitial(artist.name)}</span>
                    </div>
                {/if}
            </div>

            <div class="artist-header-info">

                <h1 class="artist-name-title">{artist.name}</h1>
                
                {#if artist.subscribers}
                    <div class="artist-subscribers-badge">
                        <Users size={13} weight="bold" />
                        <span>{artist.subscribers}</span>
                    </div>
                {/if}

                {#if artist.bio}
                    <p class="artist-bio-text">{artist.bio}</p>
                {/if}

                <div class="artist-actions-row">
                    <button class="echo-primary-btn" onclick={playTopTracks}>
                        <Play size={16} weight="fill" />
                        <span>Play Top Songs</span>
                    </button>
                    <button class="echo-secondary-btn" onclick={shuffleTopTracks}>
                        <Shuffle size={16} weight="bold" />
                        <span>Shuffle</span>
                    </button>
                </div>
            </div>
        </header>

        <!-- Top Tracks Shelf -->
        {#if artist.top_tracks.length > 0}
            <section class="artist-section">
                <div class="section-title-row">
                    <h2>Top Songs</h2>
                    {#if artist.top_tracks.length >= 5}
                        <button class="see-more-btn" onclick={() => showAllTopSongs = !showAllTopSongs}>
                            <span>{showAllTopSongs ? "Show Less" : `See More (${artist.top_tracks.length})`}</span>
                            <CaretRight size={13} weight="bold" class={showAllTopSongs ? "rotate-90" : ""} />
                        </button>
                    {/if}
                </div>
                <div class="top-tracks-ledger">
                    {#each (showAllTopSongs ? artist.top_tracks : artist.top_tracks.slice(0, 5)) as track, index}
                        <div 
                            class="ledger-row"
                            class:active-track={isCurrentTrack(track)}
                            role="button"
                            tabindex="0"
                            ondblclick={() => exploreStore.playTrack(track, track.provider_id || artist.provider_id)}
                            onkeydown={(e) => { if (e.key === 'Enter') exploreStore.playTrack(track, track.provider_id || artist.provider_id); }}
                        >
                            <span class="track-index">{index + 1}</span>

                            <div class="track-art-wrapper">
                                {#if track.cover_art_url}
                                    <img src={track.cover_art_url} alt={track.title} class="track-squircle" />
                                {:else}
                                    <div class="typographic-art-squircle">
                                        <span>{getInitial(track.title)}</span>
                                    </div>
                                {/if}
                                <button class="play-overlay-btn" onclick={() => exploreStore.playTrack(track, track.provider_id || artist.provider_id)}>
                                    {#if isCurrentTrack(track) && audioStore.playbackState === "Playing"}
                                        <Pause size={13} weight="fill" />
                                    {:else}
                                        <Play size={13} weight="fill" />
                                    {/if}
                                </button>
                            </div>

                            <div class="track-meta">
                                <span class="track-title">{track.title}</span>
                                {#if track.album}
                                    <span class="track-album-sub">{track.album}</span>
                                {/if}
                            </div>

                            <div class="track-duration-cell">
                                <span class="duration-text">{formatDuration(track.duration_ms)}</span>
                            </div>
                        </div>
                    {/each}
                </div>
            </section>
        {/if}

        <!-- Discography Albums Grid -->
        {#if artist.albums.length > 0}
            <section class="artist-section">
                <div class="section-title-row">
                    <h2>Albums</h2>
                    {#if artist.albums.length > 6}
                        <button class="see-more-btn" onclick={() => showAllAlbums = !showAllAlbums}>
                            <span>{showAllAlbums ? "Show Less" : `See More (${artist.albums.length})`}</span>
                            <CaretRight size={13} weight="bold" class={showAllAlbums ? "rotate-90" : ""} />
                        </button>
                    {/if}
                </div>
                <div class="artist-albums-grid">
                    {#each (showAllAlbums ? artist.albums : artist.albums.slice(0, 6)) as album}
                        <div 
                            class="album-card"
                            role="button"
                            tabindex="0"
                            onclick={() => exploreStore.openAlbum(album)}
                            onkeydown={(e) => { if (e.key === 'Enter') exploreStore.openAlbum(album); }}
                        >
                            <div class="album-art-wrapper">
                                {#if album.cover_art_url}
                                    <img src={album.cover_art_url} alt={album.title} class="album-art" loading="lazy" />
                                {:else}
                                    <div class="typographic-art-squircle large">
                                        <span>{getInitial(album.title)}</span>
                                    </div>
                                {/if}
                                <button 
                                    class="album-play-btn" 
                                    onclick={(e) => { 
                                        e.stopPropagation(); 
                                        exploreStore.playAlbum({
                                            id: album.id,
                                            title: album.title,
                                            artist: artist.name,
                                            cover_art_url: album.cover_art_url,
                                            provider_id: artist.provider_id || "youtube-wasm",
                                        }); 
                                    }}
                                >
                                    <Play size={16} weight="fill" />
                                </button>
                            </div>
                            <div class="album-info">
                                <span class="album-title">{album.title}</span>
                                {#if album.year}
                                    <span class="album-year">{album.year}</span>
                                {/if}
                            </div>
                        </div>
                    {/each}
                </div>
            </section>
        {/if}

        <!-- Singles & EPs Grid -->
        {#if artist.singles.length > 0}
            <section class="artist-section">
                <div class="section-title-row">
                    <h2>Singles & EPs</h2>
                    {#if artist.singles.length > 6}
                        <button class="see-more-btn" onclick={() => showAllSingles = !showAllSingles}>
                            <span>{showAllSingles ? "Show Less" : `See More (${artist.singles.length})`}</span>
                            <CaretRight size={13} weight="bold" class={showAllSingles ? "rotate-90" : ""} />
                        </button>
                    {/if}
                </div>
                <div class="artist-albums-grid">
                    {#each (showAllSingles ? artist.singles : artist.singles.slice(0, 6)) as single}
                        <div 
                            class="album-card"
                            role="button"
                            tabindex="0"
                            onclick={() => exploreStore.openAlbum(single)}
                            onkeydown={(e) => { if (e.key === 'Enter') exploreStore.openAlbum(single); }}
                        >
                            <div class="album-art-wrapper">
                                {#if single.cover_art_url}
                                    <img src={single.cover_art_url} alt={single.title} class="album-art" loading="lazy" />
                                {:else}
                                    <div class="typographic-art-squircle large">
                                        <span>{getInitial(single.title)}</span>
                                    </div>
                                {/if}
                                <button 
                                    class="album-play-btn" 
                                    onclick={(e) => { 
                                        e.stopPropagation(); 
                                        exploreStore.playAlbum({
                                            id: single.id,
                                            title: single.title,
                                            artist: artist.name,
                                            cover_art_url: single.cover_art_url,
                                            provider_id: artist.provider_id || "youtube-wasm",
                                        }); 
                                    }}
                                >
                                    <Play size={16} weight="fill" />
                                </button>
                            </div>
                            <div class="album-info">
                                <span class="album-title">{single.title}</span>
                                {#if single.year}
                                    <span class="album-year">{single.year}</span>
                                {/if}
                            </div>
                        </div>
                    {/each}
                </div>
            </section>
        {/if}

        <!-- Music Videos Shelf -->
        {#if artist.videos && artist.videos.length > 0}
            <section class="artist-section">
                <div class="section-title-row">
                    <h2>Music Videos</h2>
                    {#if artist.videos.length > 4}
                        <button class="see-more-btn" onclick={() => showAllVideos = !showAllVideos}>
                            <span>{showAllVideos ? "Show Less" : `See More (${artist.videos.length})`}</span>
                            <CaretRight size={13} weight="bold" class={showAllVideos ? "rotate-90" : ""} />
                        </button>
                    {/if}
                </div>
                <div class="artist-videos-grid">
                    {#each (showAllVideos ? artist.videos : artist.videos.slice(0, 4)) as video}
                        <div 
                            class="video-card"
                            role="button"
                            tabindex="0"
                            ondblclick={() => exploreStore.playTrack(video, video.provider_id || artist.provider_id)}
                            onkeydown={(e) => { if (e.key === 'Enter') exploreStore.playTrack(video, video.provider_id || artist.provider_id); }}
                        >
                            <div class="video-art-wrapper">
                                {#if video.cover_art_url}
                                    <img src={video.cover_art_url} alt={video.title} class="video-art" loading="lazy" />
                                {:else}
                                    <div class="typographic-art-squircle large">
                                        <Video size={24} />
                                    </div>
                                {/if}
                                <button 
                                    class="video-play-btn" 
                                    onclick={(e) => { 
                                        e.stopPropagation(); 
                                        exploreStore.playTrack(video, video.provider_id || artist.provider_id); 
                                    }}
                                >
                                    <Play size={16} weight="fill" />
                                </button>
                            </div>
                            <div class="album-info">
                                <span class="album-title" title={video.title}>{video.title}</span>
                                <span class="album-year">Official Music Video</span>
                            </div>
                        </div>
                    {/each}
                </div>
            </section>
        {/if}

        <!-- Featured On Playlists Shelf -->
        {#if artist.featured_on && artist.featured_on.length > 0}
            <section class="artist-section">
                <div class="section-title-row">
                    <h2>Featured On</h2>
                    {#if artist.featured_on.length > 6}
                        <button class="see-more-btn" onclick={() => showAllFeatured = !showAllFeatured}>
                            <span>{showAllFeatured ? "Show Less" : `See More (${artist.featured_on.length})`}</span>
                            <CaretRight size={13} weight="bold" class={showAllFeatured ? "rotate-90" : ""} />
                        </button>
                    {/if}
                </div>
                <div class="artist-albums-grid">
                    {#each (showAllFeatured ? artist.featured_on : artist.featured_on.slice(0, 6)) as playlist}
                        <PlaylistCard 
                            playlist={playlist}
                            onclick={() => exploreStore.openPlaylist({ ...playlist, author: playlist.author || undefined })}
                        />
                    {/each}
                </div>
            </section>
        {/if}

        <!-- Fans Also Like / Similar Artists Row -->
        {#if artist.similar_artists && artist.similar_artists.length > 0}
            <section class="artist-section">
                <div class="section-title-row">
                    <h2>Fans Also Like</h2>
                    {#if artist.similar_artists.length > 6}
                        <button class="see-more-btn" onclick={() => showAllSimilar = !showAllSimilar}>
                            <span>{showAllSimilar ? "Show Less" : `See More (${artist.similar_artists.length})`}</span>
                            <CaretRight size={13} weight="bold" class={showAllSimilar ? "rotate-90" : ""} />
                        </button>
                    {/if}
                </div>
                <div class="similar-artists-grid">
                    {#each (showAllSimilar ? artist.similar_artists : artist.similar_artists.slice(0, 6)) as simArtist}
                        <div 
                            class="artist-avatar-card"
                            role="button"
                            tabindex="0"
                            onclick={() => exploreStore.openArtist(simArtist)}
                            onkeydown={(e) => { if (e.key === 'Enter') exploreStore.openArtist(simArtist); }}
                        >
                            <div class="artist-avatar-wrapper">
                                {#if simArtist.avatar_url}
                                    <img src={simArtist.avatar_url} alt={simArtist.name} class="artist-avatar-img" />
                                {:else}
                                    <div class="artist-avatar-fallback">
                                        <span>{getInitial(simArtist.name)}</span>
                                    </div>
                                {/if}
                            </div>
                            <span class="similar-artist-name" title={simArtist.name}>{simArtist.name}</span>
                            <span class="similar-artist-sub">{simArtist.subscribers || "Artist"}</span>
                        </div>
                    {/each}
                </div>
            </section>
        {/if}
    {/if}
</div>

<style>
    .artist-detail-container {
        display: flex;
        flex-direction: column;
        gap: 2rem;
        padding: 1.5rem 2rem var(--player-clearance, 10rem) 2rem;
        scroll-padding-bottom: var(--player-scroll-padding, 10rem);
        max-width: 1200px;
        margin: 0 auto;
        width: 100%;
        box-sizing: border-box;
    }

    .artist-topbar {
        display: flex;
        align-items: center;
    }

    .back-btn {
        display: inline-flex;
        align-items: center;
        gap: 0.5rem;
        background: rgba(255, 255, 255, 0.05);
        border: 1px solid rgba(255, 255, 255, 0.08);
        color: rgba(255, 255, 255, 0.85);
        font-family: inherit;
        font-size: 0.82rem;
        font-weight: 600;
        padding: 0.45rem 0.9rem;
        border-radius: 8px;
        cursor: pointer;
        transition: all 0.18s ease;
    }

    .back-btn:hover {
        background: rgba(255, 255, 255, 0.1);
        border-color: #B58E62;
        color: #B58E62;
    }

    .artist-hero-header {
        display: flex;
        gap: 2.2rem;
        align-items: center;
        background: linear-gradient(135deg, rgba(255, 255, 255, 0.03) 0%, rgba(255, 255, 255, 0.01) 100%);
        border: 1px solid rgba(255, 255, 255, 0.07);
        border-radius: 16px;
        padding: 2.2rem;
    }

    .artist-avatar-wrapper {
        flex-shrink: 0;
        width: 170px;
        height: 170px;
        border-radius: 50%;
        overflow: hidden;
        box-shadow: 0 12px 32px rgba(0, 0, 0, 0.5);
        border: 2px solid rgba(255, 255, 255, 0.1);
    }

    .artist-avatar-img {
        width: 100%;
        height: 100%;
        object-fit: cover;
    }

    .artist-avatar-fallback {
        width: 100%;
        height: 100%;
        display: flex;
        align-items: center;
        justify-content: center;
        background: #222;
        color: #B58E62;
        font-size: 3.5rem;
        font-weight: 700;
    }

    .artist-header-info {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
        flex: 1;
    }



    .artist-name-title {
        font-size: 2.4rem;
        font-weight: 800;
        letter-spacing: -0.03em;
        margin: 0;
        color: #fff;
        line-height: 1.1;
    }

    .artist-subscribers-badge {
        display: inline-flex;
        align-items: center;
        gap: 0.4rem;
        font-family: ui-monospace, SFMono-Regular, monospace;
        font-size: 0.78rem;
        color: rgba(255, 255, 255, 0.6);
    }

    .artist-bio-text {
        font-size: 0.85rem;
        line-height: 1.45;
        color: rgba(255, 255, 255, 0.7);
        max-width: 650px;
        margin: 0.4rem 0 0.8rem 0;
        display: -webkit-box;
        line-clamp: 2;
        -webkit-line-clamp: 2;
        -webkit-box-orient: vertical;
        overflow: hidden;
    }

    .artist-actions-row {
        display: flex;
        gap: 0.8rem;
        margin-top: 0.4rem;
    }

    .echo-primary-btn {
        display: inline-flex;
        align-items: center;
        gap: 0.5rem;
        background: #B58E62;
        color: #121212;
        font-family: inherit;
        font-size: 0.88rem;
        font-weight: 700;
        padding: 0.65rem 1.4rem;
        border-radius: 24px;
        border: none;
        cursor: pointer;
        transition: transform 0.15s ease, background 0.15s ease;
    }

    .echo-primary-btn:hover {
        background: #c9a073;
        transform: scale(1.03);
    }

    .echo-secondary-btn {
        display: inline-flex;
        align-items: center;
        gap: 0.5rem;
        background: rgba(255, 255, 255, 0.06);
        border: 1px solid rgba(255, 255, 255, 0.12);
        color: #fff;
        font-family: inherit;
        font-size: 0.88rem;
        font-weight: 600;
        padding: 0.65rem 1.2rem;
        border-radius: 24px;
        cursor: pointer;
        transition: all 0.15s ease;
    }

    .echo-secondary-btn:hover {
        background: rgba(255, 255, 255, 0.12);
        border-color: rgba(255, 255, 255, 0.25);
    }

    .artist-section {
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }

    .section-title-row {
        display: flex;
        align-items: center;
        justify-content: space-between;
    }

    .section-title-row h2 {
        font-size: 1.25rem;
        font-weight: 700;
        color: #fff;
        margin: 0;
    }

    .see-more-btn {
        display: inline-flex;
        align-items: center;
        gap: 0.35rem;
        background: none;
        border: none;
        color: #B58E62;
        font-family: inherit;
        font-size: 0.775rem;
        font-weight: 600;
        cursor: pointer;
        padding: 0.25rem 0.5rem;
        border-radius: 4px;
        transition: all 0.15s ease;
    }

    .see-more-btn:hover {
        background: rgba(181, 142, 98, 0.12);
        color: #fff;
    }

    :global(.see-more-btn .rotate-90) {
        transform: rotate(90deg);
        transition: transform 0.2s ease;
    }

    .artist-videos-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
        gap: 1.25rem;
    }

    .video-card {
        display: flex;
        flex-direction: column;
        gap: 0.6rem;
        cursor: pointer;
        transition: transform 0.18s ease;
    }

    .video-card:hover {
        transform: translateY(-3px);
    }

    .video-art-wrapper {
        position: relative;
        width: 100%;
        aspect-ratio: 16 / 9;
        border-radius: 8px;
        overflow: hidden;
        background: rgba(255, 255, 255, 0.05);
    }

    .video-art {
        width: 100%;
        height: 100%;
        object-fit: cover;
    }

    .video-play-btn {
        position: absolute;
        bottom: 0.6rem;
        right: 0.6rem;
        width: 34px;
        height: 34px;
        border-radius: 50%;
        background: #B58E62;
        color: #121212;
        border: none;
        display: flex;
        align-items: center;
        justify-content: center;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
        opacity: 0;
        transform: translateY(6px);
        transition: all 0.2s ease;
        cursor: pointer;
    }

    .video-card:hover .video-play-btn {
        opacity: 1;
        transform: translateY(0);
    }

    .similar-artists-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(135px, 1fr));
        gap: 1.8rem 1.4rem;
        padding: 0.5rem 0;
    }

    .artist-avatar-card {
        display: flex;
        flex-direction: column;
        align-items: center;
        text-align: center;
        gap: 0.5rem;
        cursor: pointer;
        padding: 0.75rem 0.5rem;
        border-radius: 14px;
        transition: background 0.18s ease, transform 0.18s ease;
    }

    .artist-avatar-card .artist-avatar-wrapper {
        width: 100px;
        height: 100px;
    }

    .artist-avatar-card:hover {
        background: rgba(255, 255, 255, 0.04);
        transform: translateY(-3px);
    }

    .similar-artist-name {
        font-size: 0.825rem;
        font-weight: 600;
        color: #fff;
        max-width: 120px;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .similar-artist-sub {
        font-size: 0.72rem;
        color: rgba(255, 255, 255, 0.45);
    }

    /* Top tracks ledger */
    .top-tracks-ledger {
        display: flex;
        flex-direction: column;
        gap: 0.25rem;
    }

    .ledger-row {
        display: grid;
        grid-template-columns: 28px 44px 1fr 60px;
        align-items: center;
        gap: 0.9rem;
        padding: 0.45rem 0.8rem;
        border-radius: 8px;
        cursor: pointer;
        transition: background 0.15s ease;
    }

    .ledger-row:hover {
        background: rgba(255, 255, 255, 0.05);
    }

    .ledger-row.active-track {
        background: rgba(181, 142, 98, 0.12);
    }

    .track-index {
        font-family: ui-monospace, monospace;
        font-size: 0.8rem;
        color: rgba(255, 255, 255, 0.4);
        text-align: center;
    }

    .track-art-wrapper {
        position: relative;
        width: 44px;
        height: 44px;
        border-radius: 6px;
        overflow: hidden;
    }

    .track-squircle {
        width: 100%;
        height: 100%;
        object-fit: cover;
    }

    .play-overlay-btn {
        position: absolute;
        inset: 0;
        background: rgba(0, 0, 0, 0.55);
        display: flex;
        align-items: center;
        justify-content: center;
        color: #fff;
        border: none;
        opacity: 0;
        cursor: pointer;
        transition: opacity 0.15s ease;
    }

    .ledger-row:hover .play-overlay-btn,
    .ledger-row.active-track .play-overlay-btn {
        opacity: 1;
    }

    .track-meta {
        display: flex;
        flex-direction: column;
        gap: 0.15rem;
        overflow: hidden;
    }

    .track-title {
        font-size: 0.88rem;
        font-weight: 600;
        color: #fff;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .track-album-sub {
        font-size: 0.75rem;
        color: rgba(255, 255, 255, 0.5);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .track-duration-cell {
        font-family: ui-monospace, monospace;
        font-size: 0.75rem;
        color: rgba(255, 255, 255, 0.45);
        text-align: right;
    }

    /* Albums grid */
    .artist-albums-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(170px, 1fr));
        gap: 1.2rem;
    }

    .album-card {
        display: flex;
        flex-direction: column;
        gap: 0.6rem;
        cursor: pointer;
    }

    .album-art-wrapper {
        position: relative;
        aspect-ratio: 1 / 1;
        border-radius: 10px;
        overflow: hidden;
        background: rgba(255, 255, 255, 0.04);
        box-shadow: 0 4px 16px rgba(0, 0, 0, 0.3);
    }

    .album-art {
        width: 100%;
        height: 100%;
        object-fit: cover;
        transition: transform 0.25s ease;
    }

    .album-card:hover .album-art {
        transform: scale(1.04);
    }

    .album-play-btn {
        position: absolute;
        bottom: 10px;
        right: 10px;
        width: 38px;
        height: 38px;
        border-radius: 50%;
        background: #B58E62;
        color: #121212;
        border: none;
        display: flex;
        align-items: center;
        justify-content: center;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
        opacity: 0;
        transform: translateY(6px);
        transition: all 0.2s ease;
        cursor: pointer;
    }

    .album-card:hover .album-play-btn {
        opacity: 1;
        transform: translateY(0);
    }

    .album-info {
        display: flex;
        flex-direction: column;
        gap: 0.15rem;
    }

    .album-title {
        font-size: 0.85rem;
        font-weight: 600;
        color: #fff;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .album-year {
        font-family: ui-monospace, monospace;
        font-size: 0.72rem;
        color: rgba(255, 255, 255, 0.45);
    }

    .typographic-art-squircle {
        width: 100%;
        height: 100%;
        display: flex;
        align-items: center;
        justify-content: center;
        background: rgba(255, 255, 255, 0.08);
        color: #B58E62;
        font-weight: 700;
    }
</style>
