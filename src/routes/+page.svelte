<script lang="ts">
    import { audioStore } from "$lib/stores/audio.svelte";
    import { libraryStore } from "$lib/stores/library.svelte";
    import { settingsStore } from "$lib/stores/settings.svelte";
    import { flagsStore } from "$lib/stores/flags.svelte";
    import { exploreStore } from "$lib/stores/explore.svelte";

    import Sidebar from "$lib/components/Sidebar.svelte";
    import PlayerBar from "$lib/components/PlayerBar.svelte";
    import SettingsView from "$lib/components/SettingsView.svelte";
    import QueueSidebar from "$lib/components/QueueSidebar.svelte";
    import KeyboardHandler from "$lib/components/KeyboardHandler.svelte";
    import ToastContainer from "$lib/components/ToastContainer.svelte";
    import ProvidersView from "$lib/components/ProvidersView.svelte";

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

    let activeView = $state("home");
    let queueOpen = $state(false);
    let fullScreenOpen = $state(false);
    let globalSearchOpen = $state(false);

    let drawerOpen = $derived(queueOpen || exploreStore.activeDrawerCollection !== null);
    
    function getDrawerTitle() {
        if (queueOpen) return "Up Next";
        if (exploreStore.activeDrawerCollection?.kind === "album") return "Album Details";
        if (exploreStore.activeDrawerCollection?.id?.toString().startsWith("horizon-")) return "Adjacent Horizon";
        if (exploreStore.activeDrawerCollection?.kind === "playlist") {
            if (exploreStore.activeDrawerCollection?.subtitle?.includes("Radio")) return "Radio Mix";
            return "Playlist Details";
        }
        return "";
    }

    function closeDrawer() {
        queueOpen = false;
        exploreStore.closeDrawerCollection();
    }

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
            exploreStore.init().catch(err => console.error("Explore prefetch error:", err));
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

    // When a collection is opened, close the queue so the drawer shows collection details
    $effect(() => {
        if (exploreStore.activeDrawerCollection !== null) {
            queueOpen = false;
        }
    });

    // When queue is opened, clear any active drawer collection
    $effect(() => {
        if (queueOpen) {
            exploreStore.closeDrawerCollection();
        }
    });

    // Reactive: update the CSS variable whenever drawer state changes.
    $effect(() => {
        document.documentElement.style.setProperty('--drawer-w', drawerOpen ? '400px' : '0px');
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
            <AlbumGrid bind:activeView onSelectAlbum={(a) => { exploreStore.openLocalAlbum(a); queueOpen = false; }} selectedAlbumId={exploreStore.activeDrawerCollection?.id} />
        {:else if activeView === "playlists"}
            <PlaylistView bind:activeView onSelectPlaylist={(p) => { exploreStore.openLocalPlaylist(p); queueOpen = false; }} />
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
    >
        {#if queueOpen}
            <QueueSidebar bind:open={queueOpen} />
        {:else if exploreStore.activeDrawerCollection}
            <CollectionDetail 
                collection={exploreStore.activeDrawerCollection} 
                onBack={closeDrawer} 
                onDeleted={closeDrawer}
            />
        {/if}
    </RightDrawer>
</div>

<!-- Floating player lives outside the grid, fixed bottom center -->
<PlayerBar bind:queueOpen bind:fullScreenOpen />

<FullScreenPlayer bind:isOpen={fullScreenOpen} onToggleQueue={() => { queueOpen = !queueOpen; }} />
<GlobalSearch bind:isOpen={globalSearchOpen} />
<ToastContainer />
