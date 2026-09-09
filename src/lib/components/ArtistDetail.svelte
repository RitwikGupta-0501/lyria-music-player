<script lang="ts">
    import { onMount } from "svelte";
    import { exploreStore, type ArtistDetailResult } from "$lib/stores/explore.svelte";
    import AlbumCard from "$lib/components/AlbumCard.svelte";
    import PlaylistCard from "$lib/components/PlaylistCard.svelte";
    import VideoCard from "$lib/components/VideoCard.svelte";
    import BackButton from "$lib/components/common/BackButton.svelte";
    import PillButton from "$lib/components/common/PillButton.svelte";
    import SectionHeaderSkeleton from "$lib/components/common/SectionHeaderSkeleton.svelte";
    import { audioStore } from "$lib/stores/audio.svelte";
    import { settingsStore } from "$lib/stores/settings.svelte";
    import TrackRow from "$lib/components/TrackRow.svelte";
    import ArtistCard from "$lib/components/ArtistCard.svelte";
    import { formatDuration, getInitial, isCurrentTrack, getTrackDisplayMetric, sanitizeAlbumName } from "$lib/utils/format";
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
        MusicNotes,
    } from "phosphor-svelte";

    let { onBack } = $props<{ onBack: () => void }>();

    let artist = $derived(exploreStore.selectedArtist);

    let showAllTopSongs = $state(false);
    let showAllAlbums = $state(false);
    let showAllSingles = $state(false);
    let showAllVideos = $state(false);
    let showAllFeatured = $state(false);
    let showAllSimilar = $state(false);

    function playTopTracks() {
        if (!artist || artist.top_tracks.length === 0) return;
        const pId = artist.provider_id || settingsStore.getEffectiveRemoteProvider();
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
        const pId = artist.provider_id || settingsStore.getEffectiveRemoteProvider();
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

    onMount(() => {
        const mainContent = document.querySelector('.main-content');
        if (mainContent) {
            mainContent.scrollTop = 0;
        }
    });
</script>

<div class="artist-detail-container">
    <!-- Navigation Topbar -->
    <div class="artist-topbar">
        <BackButton onclick={onBack} />
    </div>

    {#if exploreStore.isLoadingArtist}
        <!-- Comprehensive Artist Skeleton Loader -->
        <div class="artist-loading-state">
            <!-- Hero Skeleton -->
            <div class="artist-hero-header skeleton-hero">
                <div class="skeleton-avatar skeleton skeleton-box"></div>
                <div class="artist-header-info">
                    <div class="skeleton-line lg skeleton-box"></div>
                    <div class="skeleton-line sm skeleton-box"></div>
                    <div class="skeleton-line bio skeleton-box"></div>
                    <div class="skeleton-btn-row">
                        <div class="skeleton-pill skeleton-box"></div>
                        <div class="skeleton-pill skeleton-box"></div>
                    </div>
                </div>
            </div>

            <!-- Top Songs Skeleton -->
            <section class="artist-section">
                <SectionHeaderSkeleton hasControls={false} titleWidth="140px" />
                <div class="top-tracks-ledger">
                    {#each Array(5) as _}
                        <div class="ledger-row skeleton">
                            <div class="skeleton-num skeleton-box"></div>
                            <div class="track-squircle skeleton-box"></div>
                            <div class="track-meta">
                                <div class="track-title-skeleton skeleton-box"></div>
                                <div class="track-artist-skeleton skeleton-box"></div>
                            </div>
                            <div class="track-duration-skeleton skeleton-box"></div>
                        </div>
                    {/each}
                </div>
            </section>

            <!-- Albums Grid Skeleton -->
            <section class="artist-section">
                <SectionHeaderSkeleton hasControls={false} titleWidth="110px" />
                <div class="artist-albums-grid">
                    {#each Array(6) as _}
                        <div class="skeleton-card skeleton">
                            <div class="card-art-wrapper skeleton-box"></div>
                            <div class="title-skeleton skeleton-box"></div>
                            <div class="subtitle-skeleton skeleton-box"></div>
                        </div>
                    {/each}
                </div>
            </section>

            <!-- Singles & EPs Skeleton -->
            <section class="artist-section">
                <SectionHeaderSkeleton hasControls={false} titleWidth="130px" />
                <div class="artist-albums-grid">
                    {#each Array(6) as _}
                        <div class="skeleton-card skeleton">
                            <div class="card-art-wrapper skeleton-box"></div>
                            <div class="title-skeleton skeleton-box"></div>
                            <div class="subtitle-skeleton skeleton-box"></div>
                        </div>
                    {/each}
                </div>
            </section>
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
                    <PillButton 
                        variant="primary"
                        icon={Play}
                        label="Play Top Songs"
                        onclick={playTopTracks}
                    />
                    <PillButton 
                        variant="secondary"
                        icon={Shuffle}
                        label="Shuffle"
                        onclick={shuffleTopTracks}
                    />
                </div>
            </div>
        </header>

        <!-- Top Tracks Shelf -->
        {#if artist.top_tracks.length > 0}
            <section class="artist-section">
                <div class="section-title-row">
                    <div class="title-group">
                        <MusicNotes size={20} weight="bold" class="section-icon" />
                        <h2>Top Songs</h2>
                    </div>
                    {#if artist.top_tracks.length > 5}
                        <button class="see-more-btn" onclick={() => showAllTopSongs = !showAllTopSongs}>
                            <span>{showAllTopSongs ? "Show Less" : `See More (${artist.top_tracks.length})`}</span>
                            <CaretRight size={13} weight="bold" class={showAllTopSongs ? "rotate-90" : ""} />
                        </button>
                    {/if}
                </div>
                <div class="top-tracks-ledger">
                    {#each (showAllTopSongs ? artist.top_tracks : artist.top_tracks.slice(0, 5)) as track, index}
                        <TrackRow 
                            track={track}
                            index={index + 1}
                            showArtist={false}
                            providerId={track.provider_id || artist.provider_id}
                        />
                    {/each}
                </div>
            </section>
        {/if}

        <!-- Discography Albums Grid (Using our standard AlbumCard component) -->
        {#if artist.albums.length > 0}
            <section class="artist-section">
                <div class="section-title-row">
                    <div class="title-group">
                        <Disc size={20} weight="bold" class="section-icon" />
                        <h2>Albums</h2>
                    </div>
                    {#if artist.albums.length > 6}
                        <button class="see-more-btn" onclick={() => showAllAlbums = !showAllAlbums}>
                            <span>{showAllAlbums ? "Show Less" : `See More (${artist.albums.length})`}</span>
                            <CaretRight size={13} weight="bold" class={showAllAlbums ? "rotate-90" : ""} />
                        </button>
                    {/if}
                </div>
                <div class="artist-albums-grid">
                    {#each (showAllAlbums ? artist.albums : artist.albums.slice(0, 6)) as album}
                        <AlbumCard 
                            album={album}
                            onclick={() => exploreStore.openAlbum(album)}
                        />
                    {/each}
                </div>
            </section>
        {/if}

        <!-- Singles & EPs Grid (Using our standard AlbumCard component) -->
        {#if artist.singles.length > 0}
            <section class="artist-section">
                <div class="section-title-row">
                    <div class="title-group">
                        <Sparkle size={20} weight="bold" class="section-icon" />
                        <h2>Singles & EPs</h2>
                    </div>
                    {#if artist.singles.length > 6}
                        <button class="see-more-btn" onclick={() => showAllSingles = !showAllSingles}>
                            <span>{showAllSingles ? "Show Less" : `See More (${artist.singles.length})`}</span>
                            <CaretRight size={13} weight="bold" class={showAllSingles ? "rotate-90" : ""} />
                        </button>
                    {/if}
                </div>
                <div class="artist-albums-grid">
                    {#each (showAllSingles ? artist.singles : artist.singles.slice(0, 6)) as single}
                        <AlbumCard 
                            album={single}
                            onclick={() => exploreStore.openAlbum(single)}
                        />
                    {/each}
                </div>
            </section>
        {/if}

        <!-- Music Videos Shelf -->
        {#if artist.videos && artist.videos.length > 0}
            <section class="artist-section">
                <div class="section-title-row">
                    <div class="title-group">
                        <Video size={20} weight="bold" class="section-icon" />
                        <h2>Music Videos</h2>
                    </div>
                    {#if artist.videos.length > 4}
                        <button class="see-more-btn" onclick={() => showAllVideos = !showAllVideos}>
                            <span>{showAllVideos ? "Show Less" : `See More (${artist.videos.length})`}</span>
                            <CaretRight size={13} weight="bold" class={showAllVideos ? "rotate-90" : ""} />
                        </button>
                    {/if}
                </div>
                <div class="artist-videos-grid">
                    {#each (showAllVideos ? artist.videos : artist.videos.slice(0, 4)) as video}
                        <VideoCard 
                            video={video}
                            onclick={() => exploreStore.playTrack(video, video.provider_id || artist.provider_id)}
                            onplay={() => exploreStore.playTrack(video, video.provider_id || artist.provider_id)}
                        />
                    {/each}
                </div>
            </section>
        {/if}

        <!-- Featured On Playlists Shelf (Using our standard PlaylistCard component) -->
        {#if artist.featured_on && artist.featured_on.length > 0}
            <section class="artist-section">
                <div class="section-title-row">
                    <div class="title-group">
                        <Playlist size={20} weight="bold" class="section-icon" />
                        <h2>Featured On</h2>
                    </div>
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
                    <div class="title-group">
                        <MicrophoneStage size={20} weight="bold" class="section-icon" />
                        <h2>Fans Also Like</h2>
                    </div>
                    {#if artist.similar_artists.length > 6}
                        <button class="see-more-btn" onclick={() => showAllSimilar = !showAllSimilar}>
                            <span>{showAllSimilar ? "Show Less" : `See More (${artist.similar_artists.length})`}</span>
                            <CaretRight size={13} weight="bold" class={showAllSimilar ? "rotate-90" : ""} />
                        </button>
                    {/if}
                </div>
                <div class="similar-artists-grid">
                    {#each (showAllSimilar ? artist.similar_artists : artist.similar_artists.slice(0, 6)) as simArtist}
                        <ArtistCard 
                            artist={simArtist}
                            onclick={() => exploreStore.openArtist(simArtist)}
                            fluid
                        />
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
        gap: 2.2rem;
        padding: 1.5rem 2rem var(--player-clearance, 10rem) 2rem;
        scroll-padding-bottom: var(--player-scroll-padding, 10rem);
        max-width: 1300px;
        margin: 0 auto;
        width: 100%;
        box-sizing: border-box;
    }

    .artist-topbar {
        display: flex;
        align-items: center;
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


    .artist-section {
        display: flex;
        flex-direction: column;
        gap: 1.1rem;
    }

    .section-title-row {
        display: flex;
        align-items: center;
        justify-content: space-between;
        min-height: 36px;
    }

    .title-group {
        display: flex;
        align-items: center;
        gap: 0.55rem;
    }

    .title-group h2 {
        font-size: 1.25rem;
        font-weight: 700;
        color: #fff;
        margin: 0;
    }

    :global(.section-icon) {
        color: #B58E62;
    }

    .see-more-btn {
        display: inline-flex;
        align-items: center;
        gap: 0.35rem;
        background: rgba(255, 255, 255, 0.04);
        border: 1px solid rgba(255, 255, 255, 0.08);
        color: var(--echo-primary, #d4a86e);
        font-family: var(--lyria-font-mono);
        font-size: 0.68rem;
        font-weight: 600;
        padding: 0.35rem 0.65rem;
        border-radius: 6px;
        cursor: pointer;
        transition: all 0.15s ease;
    }

    .see-more-btn:hover {
        background: rgba(212, 168, 110, 0.12);
        border-color: rgba(212, 168, 110, 0.25);
    }

    :global(.see-more-btn .rotate-90) {
        transform: rotate(90deg);
        transition: transform 0.2s ease;
    }

    /* Top tracks ledger */
    .top-tracks-ledger {
        display: flex;
        flex-direction: column;
        gap: 0.35rem;
    }



    /* Albums & Playlists grid */
    .artist-albums-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(170px, 1fr));
        gap: 1.4rem;
    }

    /* Video cards */
    .artist-videos-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
        gap: 1.25rem;
    }


    /* Similar Artists */
    .similar-artists-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
        gap: 1.6rem 1.2rem;
        padding: 0.5rem 0;
    }

    /* Skeleton Loading System */
    .artist-loading-state {
        display: flex;
        flex-direction: column;
        gap: 2.5rem;
    }

    .skeleton-hero {
        pointer-events: none;
    }

    .skeleton-avatar {
        width: 170px;
        height: 170px;
        border-radius: 50%;
        flex-shrink: 0;
    }

    .skeleton-box {
        background: linear-gradient(90deg, rgba(255, 255, 255, 0.03) 25%, rgba(255, 255, 255, 0.07) 50%, rgba(255, 255, 255, 0.03) 75%);
        background-size: 200% 100%;
        animation: skeleton-pulse 1.8s infinite ease-in-out;
    }

    @keyframes skeleton-pulse {
        0% { background-position: 200% 0; }
        100% { background-position: -200% 0; }
    }

    .skeleton-line {
        border-radius: 4px;
    }

    .skeleton-line.lg {
        height: 32px;
        width: 45%;
    }

    .skeleton-line.sm {
        height: 14px;
        width: 25%;
    }

    .skeleton-line.bio {
        height: 16px;
        width: 60%;
        margin: 0.3rem 0;
    }

    .skeleton-btn-row {
        display: flex;
        gap: 0.8rem;
        margin-top: 0.5rem;
    }

    .skeleton-pill {
        width: 130px;
        height: 38px;
        border-radius: 20px;
    }

    .skeleton-num {
        width: 1.5rem;
        height: 12px;
        border-radius: 3px;
        flex-shrink: 0;
    }

    .track-title-skeleton {
        height: 14px;
        width: 65%;
        border-radius: 3px;
    }

    .track-artist-skeleton {
        height: 11px;
        width: 40%;
        border-radius: 3px;
    }

    .track-duration-skeleton {
        width: 35px;
        height: 12px;
        border-radius: 3px;
        margin-left: auto;
    }

    .skeleton-card {
        display: flex;
        flex-direction: column;
        gap: 0.6rem;
    }

    .card-art-wrapper {
        width: 100%;
        aspect-ratio: 1 / 1;
        border-radius: 10px;
    }

    .title-skeleton {
        height: 13px;
        width: 75%;
        border-radius: 3px;
    }

    .subtitle-skeleton {
        height: 10px;
        width: 50%;
        border-radius: 3px;
    }
</style>
