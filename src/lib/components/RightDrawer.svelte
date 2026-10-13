<script lang="ts">
    import { fade } from "svelte/transition";
    import { X, MagnifyingGlass } from "phosphor-svelte";
    import { lyricsStore } from "$lib/stores/lyrics.svelte";
    import TabPills, { type TabItem } from "$lib/components/common/TabPills.svelte";

    let { 
        title, 
        isOpen, 
        onClose, 
        tabs = [], 
        activeTab = $bindable(), 
        children 
    } = $props<{
        title: string;
        isOpen: boolean;
        onClose: () => void;
        tabs?: TabItem[];
        activeTab?: string;
        children?: import("svelte").Snippet;
    }>();
</script>

<aside class="right-drawer" class:is-open={isOpen}>
    <div class="drawer-content">
        <div class="drawer-header">
            {#if tabs && tabs.length > 0}
                <TabPills
                    tabs={tabs}
                    bind:activeTab={activeTab}
                    size="sm"
                    ariaLabel="Right drawer tabs"
                />
            {:else}
                <h2 class="drawer-title">{title}</h2>
            {/if}
            <div class="drawer-actions">
                {#if tabs && tabs.length > 0 && activeTab === "lyrics"}
                    <button
                        type="button"
                        class="header-action-btn"
                        class:active={lyricsStore.isSearchOpen}
                        onclick={() => {
                            if (lyricsStore.isSearchOpen) {
                                lyricsStore.closeSearch();
                            } else {
                                lyricsStore.openSearch();
                            }
                        }}
                        title={lyricsStore.isSearchOpen ? "Close search (Esc)" : "Search other lyrics"}
                        aria-label="Search lyrics"
                    >
                        <MagnifyingGlass size={17} weight={lyricsStore.isSearchOpen ? "bold" : "regular"} />
                    </button>
                {/if}
                <button class="close-btn" onclick={onClose} aria-label="Close">
                    <X size={20} weight="bold" />
                </button>
            </div>
        </div>

        <div class="drawer-body">
            {#key activeTab || title}
                <div 
                    class="tab-pane"
                    in:fade={{ duration: 220, delay: 50 }} 
                    out:fade={{ duration: 160 }}
                >
                    {@render children?.()}
                </div>
            {/key}
        </div>

    </div>
</aside>

<style>
    .right-drawer {
        width: 0px;
        height: 100%;
        background-color: #08080a;
        border-left: 1px solid transparent;
        flex-shrink: 0;
        position: relative;
        z-index: 20;
        overflow: hidden;
    }
    .right-drawer.is-open {
        width: 400px;
        border-left: 1px solid rgba(255, 255, 255, 0.05); /* border-white/5 */
    }

    .drawer-content {
        width: 400px; 
        height: 100%;
        display: flex;
        flex-direction: column;
    }

    .drawer-header {
        padding: 1.5rem; /* px-6 py-6 */
        display: flex;
        align-items: center;
        justify-content: space-between;
        border-bottom: 1px solid rgba(255, 255, 255, 0.05);
        flex-shrink: 0;
    }

    .drawer-title {
        font-size: 1.125rem; /* text-lg */
        font-family: var(--echo-font-body);
        font-weight: 500;
        letter-spacing: 0.025em; /* tracking-wide */
        text-transform: uppercase;
        color: var(--echo-primary-dark);
        margin: 0;
    }

    .drawer-actions {
        display: flex;
        align-items: center;
        gap: 0.35rem;
    }

    .header-action-btn {
        width: 32px;
        height: 32px;
        border-radius: 9999px;
        background: transparent;
        border: 1px solid transparent;
        display: flex;
        align-items: center;
        justify-content: center;
        color: var(--echo-text-2, #888);
        cursor: pointer;
        transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
        padding: 0;
    }

    .header-action-btn:hover {
        background-color: rgba(255, 255, 255, 0.08);
        color: var(--echo-text-1, #eee);
    }

    .header-action-btn.active {
        background-color: rgba(255, 255, 255, 0.15);
        border-color: rgba(255, 255, 255, 0.2);
        color: #ffffff;
    }

    .close-btn {
        width: 32px; /* w-8 */
        height: 32px; /* h-8 */
        border-radius: 9999px; /* rounded-full */
        background: transparent;
        border: none;
        display: flex;
        align-items: center;
        justify-content: center;
        color: var(--echo-text-2); /* text-muted */
        cursor: pointer;
        transition: all 0.2s;
        padding: 0;
    }

    .close-btn:hover {
        background-color: rgba(255, 255, 255, 0.05); /* hover:bg-white/5 */
        color: var(--echo-text-1); /* hover:text-text-main */
    }

    .drawer-body {
        flex: 1;
        position: relative;
        overflow: hidden;
    }

    .tab-pane {
        position: absolute;
        inset: 0;
        width: 100%;
        height: 100%;
        display: flex;
        flex-direction: column;
    }
</style>
