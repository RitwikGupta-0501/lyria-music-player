<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { audioStore } from "$lib/stores/audio.svelte";
    import { libraryStore } from "$lib/stores/library.svelte";
    import { settingsStore } from "$lib/stores/settings.svelte";
    
    import { Play, Pause } from "phosphor-svelte";
    import { toastStore } from "$lib/stores/toast.svelte";
    import { untrack } from "svelte";

    // Models matching Rust
    interface ProviderModule {
        id: string;
        name: string;
        layout: 'grid' | 'list' | 'carousel' | 'unknown';
    }

    interface ModuleItem {
        type: string;
        id: string;
        title: string;
        artist: string;
        album: string | null;
        cover_art_url: string | null;
        stream_url: string | null;
        quality_hint: string | null;
        duration_ms: number | null;
    }

    interface ModuleData {
        items: ModuleItem[];
    }

    // Props
    let { activeView = $bindable() } = $props();

    // State
    let modules = $state<{ providerId: string; providerName: string; module: ProviderModule; data: ModuleItem[] | null; loading: boolean; error: string | null }[]>([]);
    let isFetchingModules = $state(false);

    $effect(() => {
        if (activeView === "explore") {
            untrack(() => loadExplorePage());
        }
    });

    let explorePageLoaded = $state(false);

    async function loadExplorePage() {
        if (explorePageLoaded || isFetchingModules) return; 
        isFetchingModules = true;
        explorePageLoaded = true;

        try {
            // 1. Get enabled providers with "explore" capability
            const providers: any[] = await invoke("get_providers");
            const exploreProviders = providers.filter(p => p.status === 'enabled' && p.capabilities?.includes('explore'));
            
            // 2. Fetch modules for each provider
            for (const p of exploreProviders) {
                try {
                    const providerModules = await invoke<ProviderModule[]>("get_provider_modules", { providerId: p.id });
                    for (const m of providerModules) {
                        modules.push({
                            providerId: p.id,
                            providerName: p.name,
                            module: m,
                            data: null,
                            loading: true,
                            error: null,
                        });
                    }
                } catch (e) {
                    console.error(`Failed to get modules for ${p.id}:`, e);
                }
            }

            // 3. Fetch data for each module asynchronously (stale-while-revalidate pattern is handled here by just updating the state)
            for (let i = 0; i < modules.length; i++) {
                const m = modules[i];
                invoke<ModuleData>("fetch_provider_module", { providerId: m.providerId, moduleId: m.module.id })
                    .then(data => {
                        modules[i].data = data.items;
                        modules[i].loading = false;
                    })
                    .catch(e => {
                        modules[i].error = String(e);
                        modules[i].loading = false;
                    });
            }
        } catch (e) {
            console.error("Failed to load explore page:", e);
        } finally {
            isFetchingModules = false;
        }
    }

    function playItem(item: ModuleItem, providerId: string) {
        if (item.type === "Track") {
            const track = {
                id: `remote-${providerId}-${item.id}`,
                title: item.title,
                artist: item.artist,
                album: item.album,
                file_path: item.stream_url || '',
                provider_id: providerId,
                stream_url: item.stream_url,
                quality_hint: item.quality_hint,
                cover_art_url: item.cover_art_url,
                duration_ms: item.duration_ms,
            };
            audioStore.setQueue([track], 0);
        } else {
            toastStore.show(`Playback of ${item.type} is not supported yet`, "error");
        }
    }

</script>

<div class="search-view">
    <div class="header">
        <h1>Explore</h1>
    </div>
    
    <div class="local-recent section">
        <h2>Recently Added (Local)</h2>
        <div class="grid">
            {#each libraryStore.recentAlbums as album}
                <div class="album-card" role="button" tabindex="0" onclick={() => { /* Navigate to album */ }} onkeydown={(e) => { if(e.key === 'Enter') { /* Navigate to album */ } }}>
                    {#if album.cover_art_path}
                        <!-- svelte-ignore a11y_missing_attribute -->
                        <img src={`asset://localhost/${encodeURIComponent(album.cover_art_path)}`} />
                    {:else}
                        <div class="placeholder-art"></div>
                    {/if}
                    <div class="album-info">
                        <h3>{album.title}</h3>
                        <p>{album.artist}</p>
                    </div>
                </div>
            {/each}
        </div>
    </div>

    {#each modules as m}
        <div class="module-section section">
            <h2>{m.module.name} <span class="provider-badge">{m.providerName}</span></h2>
            
            {#if m.loading}
                <div class="loading-state">Loading...</div>
            {:else if m.error}
                <div class="error-state">Failed to load: {m.error}</div>
            {:else if m.data && m.data.length > 0}
                <div class="module-layout {m.module.layout}">
                    {#each m.data as item}
                        <div class="item-card" role="button" tabindex="0" onclick={() => playItem(item, m.providerId)} onkeydown={(e) => { if(e.key === 'Enter') playItem(item, m.providerId); }}>
                            <div class="item-art">
                                {#if item.cover_art_url}
                                    <img src={item.cover_art_url} alt={item.title} />
                                {:else}
                                    <div class="placeholder-art"></div>
                                {/if}
                                <div class="play-overlay">
                                    <Play weight="fill" />
                                </div>
                            </div>
                            <div class="item-info">
                                <h3>{item.title}</h3>
                                <p>{item.artist}</p>
                            </div>
                        </div>
                    {/each}
                </div>
            {:else}
                <div class="empty-state">No items found.</div>
            {/if}
        </div>
    {/each}
</div>

<style>
    .search-view {
        padding: 2rem;
        height: 100%;
        overflow-y: auto;
    }

    .header {
        margin-bottom: 2rem;
    }

    h1 {
        font-size: 2.5rem;
        font-weight: 700;
        letter-spacing: -0.02em;
        margin: 0;
    }

    h2 {
        font-size: 1.5rem;
        font-weight: 600;
        margin-bottom: 1rem;
        display: flex;
        align-items: center;
        gap: 0.75rem;
    }

    .provider-badge {
        font-size: 0.75rem;
        font-weight: 600;
        padding: 0.2rem 0.6rem;
        background: var(--surface-2, rgba(255, 255, 255, 0.1));
        border-radius: 4px;
        color: var(--text-muted);
        text-transform: uppercase;
    }

    .section {
        margin-bottom: 3rem;
    }

    .grid, .module-layout.grid, .module-layout.carousel, .module-layout.unknown {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
        gap: 1.5rem;
    }

    .module-layout.list {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .module-layout.list .item-card {
        flex-direction: row;
        align-items: center;
        gap: 1rem;
        padding: 0.5rem;
        border-radius: 8px;
        background: transparent;
    }

    .module-layout.list .item-card:hover {
        background: var(--surface-1, rgba(255, 255, 255, 0.05));
    }

    .module-layout.list .item-art {
        width: 48px;
        height: 48px;
        border-radius: 4px;
        margin-bottom: 0;
    }

    .item-card {
        display: flex;
        flex-direction: column;
        cursor: pointer;
        transition: transform 0.2s ease;
    }

    .item-card:hover {
        transform: translateY(-2px);
    }

    .item-card:hover .play-overlay {
        opacity: 1;
    }

    .item-art, .album-card .placeholder-art, .album-card img {
        width: 100%;
        aspect-ratio: 1;
        border-radius: 8px;
        object-fit: cover;
        margin-bottom: 0.75rem;
        background: var(--surface-2, rgba(255,255,255,0.1));
        position: relative;
        overflow: hidden;
    }

    .play-overlay {
        position: absolute;
        inset: 0;
        background: rgba(0, 0, 0, 0.5);
        display: flex;
        align-items: center;
        justify-content: center;
        opacity: 0;
        transition: opacity 0.2s ease;
        color: white;
        font-size: 2rem;
    }

    .item-info h3, .album-info h3 {
        margin: 0;
        font-size: 1rem;
        font-weight: 500;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .item-info p, .album-info p {
        margin: 0;
        font-size: 0.875rem;
        color: var(--text-muted, rgba(255,255,255,0.6));
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .loading-state, .error-state, .empty-state {
        padding: 2rem;
        text-align: center;
        color: var(--text-muted);
        background: var(--surface-1, rgba(255, 255, 255, 0.02));
        border-radius: 8px;
        border: 1px dashed rgba(255, 255, 255, 0.1);
    }

    .error-state {
        color: #ff6b6b;
        border-color: rgba(255, 107, 107, 0.2);
    }
</style>
