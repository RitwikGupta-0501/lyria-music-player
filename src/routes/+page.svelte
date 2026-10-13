<script lang="ts">
    import { audioStore } from "$lib/stores/audio.svelte";
    import { libraryStore } from "$lib/stores/library.svelte";
    import { settingsStore } from "$lib/stores/settings.svelte";
    import { flagsStore } from "$lib/stores/flags.svelte";
    import { exploreStore } from "$lib/stores/explore.svelte";
    import { lyricsStore } from "$lib/stores/lyrics.svelte";
    import { downloadStore } from "$lib/stores/download.svelte";

    import Sidebar from "$lib/components/Sidebar.svelte";
    import PlayerBar from "$lib/components/PlayerBar.svelte";
    import SettingsView from "$lib/components/SettingsView.svelte";
    import QueueSidebar from "$lib/components/QueueSidebar.svelte";
    import KeyboardHandler from "$lib/components/KeyboardHandler.svelte";
    import ToastContainer from "$lib/components/ToastContainer.svelte";
    import ProvidersView from "$lib/components/ProvidersView.svelte";
    import LyricsView from "$lib/components/lyrics/LyricsView.svelte";

    import AlbumGrid from "$lib/components/AlbumGrid.svelte";
    import ArtistDetail from "$lib/components/ArtistDetail.svelte";
    import PlaylistView from "$lib/components/PlaylistView.svelte";
    import CollectionDetail from "$lib/components/CollectionDetail.svelte";

    import { onMount } from "svelte";
    import RightDrawer from "$lib/components/RightDrawer.svelte";
    import FullScreenPlayer from "$lib/components/FullScreenPlayer.svelte";
    import GlobalSearch from "$lib/components/GlobalSearch.svelte";
    import ExploreView from "$lib/components/ExploreView.svelte";
    import HomeView from "$lib/components/HomeView.svelte";
    import DownloadCandidateModal from "$lib/components/DownloadCandidateModal.svelte";
    import DownloadsView from "$lib/components/DownloadsView.svelte";

    let activeView = $state("home");
    let sidebarOpen = $state(false);
    let rightDrawerTab = $state<"queue" | "lyrics">("queue");
    let fullScreenOpen = $state(false);
    let globalSearchOpen = $state(false);

    let drawerOpen = $derived(sidebarOpen || exploreStore.activeDrawerCollection !== null);
    let isQueueActive = $derived(sidebarOpen && rightDrawerTab === "queue" && exploreStore.activeDrawerCollection === null);
    let isLyricsActive = $derived(sidebarOpen && rightDrawerTab === "lyrics" && exploreStore.activeDrawerCollection === null);

    function toggleQueue() {
        if (isQueueActive) {
            sidebarOpen = false;
        } else {
            exploreStore.closeDrawerCollection();
            rightDrawerTab = "queue";
            sidebarOpen = true;
        }
    }

    function toggleLyrics() {
        if (isLyricsActive) {
            sidebarOpen = false;
        } else {
            exploreStore.closeDrawerCollection();
            rightDrawerTab = "lyrics";
            sidebarOpen = true;
        }
    }
    
    function getDrawerTitle() {
        if (exploreStore.activeDrawerCollection?.kind === "album") return "Album Details";
        if (exploreStore.activeDrawerCollection?.id?.toString().startsWith("horizon-")) return "Adjacent Horizon";
        if (exploreStore.activeDrawerCollection?.kind === "playlist") {
            if (exploreStore.activeDrawerCollection?.subtitle?.includes("Radio")) return "Radio Mix";
            return "Playlist Details";
        }
        if (rightDrawerTab === "lyrics") return "Lyrics";
        return "Up Next";
    }

    function closeDrawer() {
        sidebarOpen = false;
        exploreStore.closeDrawerCollection();
    }

    // Automatically fetch lyrics when track changes
    $effect(() => {
        const track = audioStore.currentQueueTrack;
        if (track) {
            lyricsStore.fetchForCurrentTrack();
        }
    });

    // One-time initialization — runs once on mount, never re-runs on state change.
    onMount(() => {
        (async () => {
            await audioStore.init();
            await libraryStore.fetchAlbums();
            await libraryStore.fetchSavedAlbums();
            await libraryStore.fetchPlaylists();
            await libraryStore.fetchSavedPlaylists();
            await settingsStore.init();
            await flagsStore.init();
            await lyricsStore.init();
            await downloadStore.init();

            if (flagsStore.isEnabled("page_explore")) {
                exploreStore.init().catch(err => console.error("Explore prefetch error:", err));
            }
        })();


        const handleSearch = () => { globalSearchOpen = true; };
        const handleNavigateExplore = () => { activeView = "explore"; };
        const handleNavigateArtist = () => { activeView = "artist"; };
        const handleEscape = () => {
            if (globalSearchOpen) {
                globalSearchOpen = false;
            } else if (activeView === "settings" || activeView === "artist") {
                activeView = "explore";
            } else {
                closeDrawer();
            }
        };
        document.addEventListener('echo:search', handleSearch);
        document.addEventListener('echo:navigate-explore', handleNavigateExplore);
        document.addEventListener('echo:navigate-artist', handleNavigateArtist);
        document.addEventListener('echo:escape', handleEscape);
        return () => {
            document.removeEventListener('echo:search', handleSearch);
            document.removeEventListener('echo:navigate-explore', handleNavigateExplore);
            document.removeEventListener('echo:navigate-artist', handleNavigateArtist);
            document.removeEventListener('echo:escape', handleEscape);
        };
    });

    // When a collection is opened, reset sidebarOpen so the drawer purely reflects collection details
    $effect(() => {
        if (exploreStore.activeDrawerCollection !== null) {
            sidebarOpen = false;
        }
    });

    // Fallback activeView if current page is disabled by feature flags
    $effect(() => {
        if (activeView === "home" && flagsStore.loaded && !flagsStore.isEnabled("page_home")) {
            activeView = flagsStore.isEnabled("page_explore") ? "explore" : "albums";
        } else if (activeView === "explore" && flagsStore.loaded && !flagsStore.isEnabled("page_explore")) {
            activeView = flagsStore.isEnabled("page_home") ? "home" : "albums";
        }
    });

    // Reactive: update the CSS variable whenever drawer state changes.
    $effect(() => {
        document.documentElement.style.setProperty('--drawer-w', drawerOpen ? '400px' : '0px');
    });
    // Scroll to top whenever top-level view changes
    $effect(() => {
        if (activeView) {
            const mainContent = document.querySelector('.main-content');
            if (mainContent) {
                mainContent.scrollTop = 0;
            }
        }
    });
</script>

<KeyboardHandler />

<!-- Three-column fluid canvas -->
<div class="app-container">
    <Sidebar bind:activeView />

    <main class="main-content">
        {#if activeView === "home"}
            <HomeView bind:activeView />
        {:else if activeView === "explore"}
            <ExploreView bind:activeView />
        {:else if activeView === "artist"}
            <ArtistDetail onBack={() => { activeView = "explore"; }} />
        {:else if activeView === "albums"}
            <AlbumGrid bind:activeView onSelectAlbum={(a) => { exploreStore.openLocalAlbum(a); sidebarOpen = false; }} selectedAlbumId={exploreStore.activeDrawerCollection?.id} />
        {:else if activeView === "playlists"}
            <PlaylistView bind:activeView onSelectPlaylist={(p) => { exploreStore.openLocalPlaylist(p); sidebarOpen = false; }} />
        {:else if activeView === "downloads"}
            <DownloadsView />
        {:else if activeView === "providers"}
            <ProvidersView />
        {:else if activeView === "settings"}
            <SettingsView />
        {/if}
    </main>

    <RightDrawer 
        title={getDrawerTitle()} 
        isOpen={drawerOpen} 
        onClose={closeDrawer}
        tabs={exploreStore.activeDrawerCollection ? [] : [
            { id: "queue", label: "Up Next" },
            { id: "lyrics", label: "Lyrics" }
        ]}
        bind:activeTab={rightDrawerTab}
    >
        {#if exploreStore.activeDrawerCollection}
            <CollectionDetail 
                collection={exploreStore.activeDrawerCollection} 
                onBack={closeDrawer} 
                onDeleted={closeDrawer}
            />
        {:else if rightDrawerTab === "lyrics"}
            <LyricsView variant="drawer" />
        {:else}
            <QueueSidebar />
        {/if}
    </RightDrawer>
</div>

<!-- Floating player lives outside the grid, fixed bottom center -->
<PlayerBar 
    bind:fullScreenOpen 
    isQueueOpen={isQueueActive}
    isLyricsOpen={isLyricsActive}
    onToggleQueue={toggleQueue}
    onToggleLyrics={toggleLyrics}
/>


<FullScreenPlayer bind:isOpen={fullScreenOpen} onToggleQueue={toggleQueue} />
<GlobalSearch bind:isOpen={globalSearchOpen} />
<DownloadCandidateModal />
<ToastContainer />
