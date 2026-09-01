<script lang="ts">
    import { settingsStore } from "$lib/stores/settings.svelte";
    import PlaybackTab from "./settings/PlaybackTab.svelte";
    import AppearanceTab from "./settings/AppearanceTab.svelte";
    import ShortcutsTab from "./settings/ShortcutsTab.svelte";
    import DataTab from "./settings/DataTab.svelte";
    import AdvancedTab from "./settings/AdvancedTab.svelte";
    import { onMount } from "svelte";

    let activeTab = $state<"playback" | "appearance" | "shortcuts" | "data" | "advanced">("playback");

    onMount(async () => {
        await settingsStore.init();
    });
</script>

<div class="settings-layout">
    <!-- Left Pane: Navigation -->
    <aside class="settings-nav">
        <h2 class="settings-title">Settings</h2>

        <nav class="nav-list">
            <button
                class="nav-item"
                class:active={activeTab === "playback"}
                onclick={() => (activeTab = "playback")}
            >
                Playback
            </button>
            <button
                class="nav-item"
                class:active={activeTab === "appearance"}
                onclick={() => (activeTab = "appearance")}
            >
                Appearance
            </button>
            <button
                class="nav-item"
                class:active={activeTab === "shortcuts"}
                onclick={() => (activeTab = "shortcuts")}
            >
                Shortcuts & Keybinds
            </button>
            <button
                class="nav-item"
                class:active={activeTab === "data"}
                onclick={() => (activeTab = "data")}
            >
                Data & Privacy
            </button>
            <button
                class="nav-item"
                class:active={activeTab === "advanced"}
                onclick={() => (activeTab = "advanced")}
            >
                Advanced & Debug
            </button>
        </nav>
    </aside>

    <!-- Right Pane: Content -->
    <div class="settings-content">
        <div class="content-container">
            {#if activeTab === "playback"}
                <PlaybackTab />
            {:else if activeTab === "appearance"}
                <AppearanceTab />
            {:else if activeTab === "shortcuts"}
                <ShortcutsTab />
            {:else if activeTab === "data"}
                <DataTab />
            {:else if activeTab === "advanced"}
                <AdvancedTab />
            {/if}
        </div>
    </div>
</div>

<style>
    .settings-layout {
        display: flex;
        height: 100vh;
        width: 100%;
    }

    /* Left Pane */
    .settings-nav {
        width: 250px;
        min-width: 250px;
        border-right: 1px solid var(--echo-border);
        padding: 3rem 1.5rem;
        display: flex;
        flex-direction: column;
        gap: 2rem;
    }

    .settings-title {
        font-family: var(--echo-font-heading);
        font-size: 1.5rem;
        font-weight: 600;
        color: var(--echo-text-1);
        margin: 0;
        padding-left: 0.5rem;
    }

    .nav-list {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .nav-item {
        background: transparent;
        border: none;
        color: var(--echo-text-2);
        font-family: var(--echo-font-body);
        font-size: 0.95rem;
        font-weight: 500;
        text-align: left;
        padding: 0.6rem 0.8rem;
        border-radius: 6px;
        cursor: pointer;
        transition: all 0.2s ease;
        position: relative;
    }

    .nav-item:hover:not(.active) {
        background: rgba(255, 255, 255, 0.05);
        color: var(--echo-text-1);
    }

    .nav-item.active {
        color: var(--echo-text-1);
        background: rgba(226, 169, 115, 0.08); /* subtle primary tint */
    }

    .nav-item.active::before {
        content: "";
        position: absolute;
        left: 0;
        top: 20%;
        bottom: 20%;
        width: 3px;
        background: var(--echo-primary);
        border-radius: 0 4px 4px 0;
    }

    /* Right Pane */
    .settings-content {
        flex: 1;
        overflow-y: auto;
        padding: 3rem 4rem var(--player-clearance, 10rem) 4rem;
        scroll-padding-bottom: var(--player-scroll-padding, 10rem);
        display: flex;
        flex-direction: column;
    }

    .content-container {
        max-width: 650px;
        width: 100%;
    }
</style>
