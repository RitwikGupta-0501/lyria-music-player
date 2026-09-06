<script lang="ts">
    import { homeStore, type HeavyRotationAlbumItem, type HeavyRotationArtistItem } from "$lib/stores/home.svelte";
    import { exploreStore } from "$lib/stores/explore.svelte";
    import { libraryStore } from "$lib/stores/library.svelte";
    import AlbumCard from "$lib/components/AlbumCard.svelte";
    import ArtistCard from "$lib/components/ArtistCard.svelte";
    import CarouselControls from "$lib/components/common/CarouselControls.svelte";
    import TabPills, { type TabItem } from "$lib/components/common/TabPills.svelte";
    import { Flame, User, Disc } from "phosphor-svelte";
    import { resolveCoverArt } from "$lib/utils/media";

    let activeTab = $state<"artists" | "albums">("artists");

    const rotationTabs: TabItem<"artists" | "albums">[] = [
        { id: "artists", label: "Top Artists", icon: User },
        { id: "albums", label: "Top Albums", icon: Disc },
    ];

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
        const span = 176 + 24; // 176px card width + 24px gap
        const pageStep = Math.max(span, track.clientWidth - span);
        track.scrollBy({ left: -pageStep, behavior: "smooth" });
    }

    function scrollNext() {
        const track = activeTab === "artists" ? artistsTrack : albumsTrack;
        if (!track) return;
        const span = 176 + 24;
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
                <TabPills
                    tabs={rotationTabs}
                    bind:activeTab
                    ariaLabel="Heavy rotation tabs"
                />

                {#if hasOverflow}
                    <CarouselControls 
                        canPrev={canScrollLeft} 
                        canNext={canScrollRight} 
                        onPrev={scrollPrev} 
                        onNext={scrollNext} 
                        prevLabel="Previous items"
                        nextLabel="Next items"
                    />
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
                    <ArtistCard 
                        artist={artist} 
                        onclick={() => handleArtistClick(artist)} 
                    />
                {/each}
            </div>
        {:else}
            <div 
                class="albums-carousel-track"
                bind:this={albumsTrack}
                onscroll={updateScrollState}
            >
                {#each homeStore.heavyRotation.albums as album}
                    <div class="carousel-album-wrap">
                        <AlbumCard 
                            album={{
                                id: album.album_title,
                                title: formatAlbumDisplay(album.album_title),
                                artist: album.artist,
                                cover_art_url: resolveCoverArt(album.cover_art_url),
                            }}
                            onclick={() => handleAlbumClick(album)}
                        />
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
        font-family: var(--echo-font-heading, 'Newsreader', serif);
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
        gap: 1rem;
    }





    .artists-carousel-track, .albums-carousel-track {
        display: flex;
        gap: 1.5rem;
        overflow-x: auto;
        scroll-snap-type: x mandatory;
        scroll-behavior: smooth;
        padding-top: 8px;
        margin-top: -8px;
        padding-bottom: 0.75rem;
        margin-bottom: -0.25rem;
        scrollbar-width: none;
        will-change: scroll-position;
    }

    .artists-carousel-track::-webkit-scrollbar,
    .albums-carousel-track::-webkit-scrollbar {
        display: none;
    }

    /* Unified Album Card Wrap */
    .carousel-album-wrap {
        flex: 0 0 176px;
        width: 176px;
        min-width: 176px;
        scroll-snap-align: start;
    }
</style>
