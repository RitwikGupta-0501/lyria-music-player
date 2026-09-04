<script lang="ts">
    import { settingsStore } from "$lib/stores/settings.svelte";
    import { flagsStore } from "$lib/stores/flags.svelte";
    import { List, House, Disc, PuzzlePiece, Gear, Compass } from "phosphor-svelte";
    
    let { activeView = $bindable("albums") } = $props<{ activeView?: string }>();
    
    let sidebarOpen = $state(false);
    let isMoving = $state(false);
    let moveTimeout: ReturnType<typeof setTimeout> | undefined;

    function toggleSidebar() {
        sidebarOpen = !sidebarOpen;
    }

    let primaryNav = $derived.by(() => {
        const items = [];
        if (flagsStore.isEnabled("page_home")) {
            items.push({ id: "home", label: "Home", icon: House });
        }
        if (flagsStore.isEnabled("page_explore")) {
            items.push({ id: "explore", label: "Explore", icon: Compass });
        }
        items.push({ id: "library", label: "Library", icon: Disc });
        items.push({ id: "providers", label: "Extensions", icon: PuzzlePiece });
        return items;
    });

    let navButtonRefs: HTMLElement[] = $state([]);
    let pillTop = $state(0);
    let prevIndex = $state(2);

    let activeIndex = $derived.by(() => {
        return primaryNav.findIndex(item => {
            if (item.id === "library") return activeView === "albums" || activeView === "playlists";
            return activeView === item.id;
        });
    });

    function triggerMove() {
        isMoving = true;
        if (moveTimeout) clearTimeout(moveTimeout);
        moveTimeout = setTimeout(() => {
            isMoving = false;
        }, 400);
    }

    $effect(() => {
        if (activeIndex !== -1) {
            const targetEl = navButtonRefs[activeIndex];
            if (targetEl) {
                pillTop = targetEl.offsetTop;
            } else {
                pillTop = activeIndex * 54;
            }
            if (activeIndex !== prevIndex) {
                triggerMove();
                prevIndex = activeIndex;
            }
        }
    });

    function selectTab(id: string, index: number) {
        if (activeIndex !== index) {
            triggerMove();
        }
        if (id === "library") {
            activeView = "albums";
        } else {
            activeView = id;
        }
    }
</script>

<aside 
    class="sidebar" 
    class:open={sidebarOpen}
>

    <!-- Top Section: Header & Navigation Core -->
    <div class="sidebar-top">
        <!-- Header / App Identity -->
        <div class="sidebar-header">
            <button 
                class="toggle-btn" 
                class:is-glass={settingsStore.glassyPlayerBar}
                onclick={toggleSidebar} 
                title={sidebarOpen ? "Collapse sidebar" : "Expand sidebar"}
                aria-label="Toggle navigation"
            >
                <List size={20} weight="bold" />
            </button>
            
            <div class="brand-wrapper" class:visible={sidebarOpen}>
                <span class="wordmark">Sonic Topography</span>
            </div>
        </div>

        <!-- Navigation Core (Anchor Command Center with Dynamic Offset Indicator) -->
        <nav class="sidebar-nav">
            <!-- Continuous Liquid Glass Capsule with Fluid Squeeze & Spring Physics -->
            {#if activeIndex !== -1}
                <div 
                    class="sliding-glass-pill" 
                    class:is-glass={settingsStore.glassyPlayerBar}
                    class:is-moving={isMoving}
                    style="transform: translateY({pillTop}px) {isMoving ? 'scaleY(1.12) scaleX(0.97)' : 'scale(1)'};"
                >
                    {#if settingsStore.glassyPlayerBar}
                        <!-- 1. Internal Refractive Specular Catch-Light -->
                        <div class="specular-top-highlight" aria-hidden="true"></div>
                    {/if}
                </div>
            {/if}

            {#each primaryNav as item, index}
                {@const isActive = activeIndex === index}
                <button
                    bind:this={navButtonRefs[index]}
                    class="nav-item group"
                    class:active={isActive}
                    onclick={() => selectTab(item.id, index)}
                    title={item.label}
                >
                    <!-- Hover Ambient Sheen (Inactive Items) -->
                    <div class="hover-ghost-glow"></div>

                    <!-- Icon with Dynamic Specular Glow -->
                    <div class="icon-container">
                        <item.icon 
                            size={21} 
                            weight={isActive ? "fill" : "regular"} 
                        />
                    </div>

                    <!-- Label (Fluid Collapse) -->
                    <div class="label-wrapper" class:visible={sidebarOpen}>
                        <span class="label">{item.label}</span>
                    </div>
                </button>
            {/each}
        </nav>
    </div>

    <!-- Bottom Section: Settings Utilities -->
    <div class="sidebar-footer">
        <button
            class="nav-item settings-item group"
            class:active={activeView === "settings"}
            onclick={() => {
                activeView = "settings";
            }}
            title="Settings"
        >
            {#if activeView === "settings"}
                <div class="active-pill settings-pill" class:is-glass={settingsStore.glassyPlayerBar}>
                    {#if settingsStore.glassyPlayerBar}
                        <div class="specular-top-highlight" aria-hidden="true"></div>
                    {/if}
                </div>
            {/if}

            <div class="hover-ghost-glow"></div>

            <div class="icon-container settings-icon">
                <Gear size={21} weight={activeView === "settings" ? "fill" : "regular"} />
            </div>

            <div class="label-wrapper" class:visible={sidebarOpen}>
                <span class="label">Settings</span>
            </div>
        </button>
    </div>
</aside>

<style>
    /* ─────────────────────────────────────────────────────────────
       LIQUID GLASS SIDEBAR CONTAINER (macOS Sonoma / VisionOS Refraction)
    ───────────────────────────────────────────────────────────── */
    .sidebar {
        position: relative;
        height: 100vh;
        width: 80px; /* Collapsed rail width */
        display: flex;
        flex-direction: column;
        justify-content: space-between;
        padding: 1.25rem 0.75rem;
        user-select: none;
        flex-shrink: 0;
        z-index: 30;
        overflow: hidden;
        background: #0d0e11;
        border-right: 1px solid rgba(255, 255, 255, 0.06);
        box-shadow: 1px 0 30px rgba(0, 0, 0, 0.5);
        
        /* Hardware Acceleration & Subtree Containment (In-Flow, Zero Jitter) */
        contain: layout paint;
        will-change: width;
        transform: translateZ(0);
        backface-visibility: hidden;
        
        transition: width 0.18s cubic-bezier(0.16, 1, 0.3, 1);
    }

    .sidebar.open {
        width: 256px; /* Expanded state */
    }

    .sidebar-top {
        display: flex;
        flex-direction: column;
        gap: 2rem;
        width: 100%;
        position: relative;
        z-index: 10;
    }

    .sidebar-header {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        padding: 0 0.25rem;
    }

    /* Top Header Toggle Trigger with Specular Catch-Light */
    .toggle-btn {
        width: 40px;
        height: 40px;
        flex-shrink: 0;
        border-radius: 1rem;
        border: 1px solid rgba(255, 255, 255, 0.08);
        background: rgba(255, 255, 255, 0.04);
        color: #e4e4e7;
        display: flex;
        align-items: center;
        justify-content: center;
        cursor: pointer;
        transition: all 0.2s ease;
        padding: 0;
    }

    .toggle-btn.is-glass {
        border: 1px solid rgba(255, 255, 255, 0.15);
        background: rgba(255, 255, 255, 0.06);
        box-shadow: inset 0 1px 1px rgba(255, 255, 255, 0.2);
    }

    .toggle-btn:hover {
        background: rgba(255, 255, 255, 0.12);
        color: #ffffff;
        border-color: rgba(255, 255, 255, 0.22);
    }

    .toggle-btn:active {
        transform: scale(0.95);
    }

    /* Brand Typography */
    .brand-wrapper {
        display: flex;
        align-items: center;
        overflow: hidden;
        opacity: 0;
        pointer-events: none;
        transform: translateX(-6px) translateZ(0);
        will-change: opacity, transform;
        backface-visibility: hidden;
        transition: opacity 0.08s ease, transform 0.08s ease;
        white-space: nowrap;
    }

    .brand-wrapper.visible {
        opacity: 1;
        pointer-events: auto;
        transform: translateX(0) translateZ(0);
        transition: opacity 0.14s ease 0.04s, transform 0.14s cubic-bezier(0.16, 1, 0.3, 1) 0.04s;
    }

    .wordmark {
        font-family: var(--echo-font-heading, "Newsreader", serif);
        font-size: 1.15rem;
        font-style: italic;
        font-weight: 500;
        color: #f4f4f5;
        white-space: nowrap;
        letter-spacing: -0.01em;
    }

    /* Navigation Core */
    .sidebar-nav {
        position: relative;
        display: flex;
        flex-direction: column;
        gap: 6px;
        width: 100%;
    }

    /* ─────────────────────────────────────────────────────────────
       4. CONTINUOUS LIQUID GLASS CAPSULE & SPRING STRETCH PHYSICS
    ───────────────────────────────────────────────────────────── */
    .sliding-glass-pill {
        position: absolute;
        top: 0;
        left: 0;
        width: 100%;
        height: 48px;
        border-radius: 1rem;
        pointer-events: none;
        z-index: 1;
        background: rgba(226, 169, 115, 0.12);
        border: 1px solid rgba(226, 169, 115, 0.3);
        transition: 
            transform 0.24s cubic-bezier(0.16, 1, 0.3, 1),
            opacity 0.15s ease;
    }

    .sliding-glass-pill.is-glass {
        background: linear-gradient(180deg, rgba(200, 157, 110, 0.22) 0%, rgba(150, 107, 61, 0.12) 100%);
        border: 1px solid rgba(224, 184, 143, 0.35);
        box-shadow: 
            inset 0 1px 1px rgba(255, 255, 255, 0.35),
            0 8px 24px -4px rgba(0, 0, 0, 0.5);
    }

    /* 1. Internal Refractive Specular Catch-Light Line */
    .specular-top-highlight {
        position: absolute;
        inset-inline: 8px;
        top: 0;
        height: 1px;
        background: linear-gradient(90deg, transparent 0%, rgba(255, 255, 255, 0.5) 50%, transparent 100%);
        pointer-events: none;
    }

    /* Nav Item Button */
    .nav-item {
        position: relative;
        width: 100%;
        height: 48px;
        display: flex;
        align-items: center;
        gap: 0.875rem;
        padding: 0 0.875rem;
        border-radius: 1rem;
        border: 1px solid transparent;
        background: transparent;
        color: #a1a1aa;
        cursor: pointer;
        text-align: left;
        overflow: hidden;
        appearance: none;
        -webkit-appearance: none;
        transition: color 0.2s ease, transform 0.12s cubic-bezier(0.16, 1, 0.3, 1);
        z-index: 2;
    }

    .nav-item:active {
        transform: scale(0.97);
    }

    /* Active State Text & Specular Glow */
    .nav-item.active {
        color: #fce9d2;
        font-weight: 500;
    }

    .nav-item.active .icon-container {
        color: #f5cb99;
        filter: drop-shadow(0 2px 8px rgba(245, 203, 153, 0.4));
    }

    .nav-item:hover:not(.active) {
        color: #f4f4f5;
    }

    /* Hover Ambient Sheen (Inactive Items) */
    .hover-ghost-glow {
        position: absolute;
        inset: 0;
        border-radius: 1rem;
        background: rgba(255, 255, 255, 0.04);
        border: 1px solid rgba(255, 255, 255, 0.05);
        opacity: 0;
        pointer-events: none;
        z-index: -1;
        transition: opacity 0.2s ease;
    }

    .nav-item:hover:not(.active) .hover-ghost-glow {
        opacity: 1;
    }

    .icon-container {
        width: 24px;
        height: 24px;
        flex-shrink: 0;
        display: flex;
        align-items: center;
        justify-content: center;
        transition: transform 0.3s cubic-bezier(0.16, 1, 0.3, 1), color 0.2s ease, filter 0.2s ease;
    }

    .nav-item:hover .icon-container {
        transform: scale(1.1);
    }

    /* Label Container with Fluid Collapse */
    .label-wrapper {
        display: flex;
        align-items: center;
        overflow: hidden;
        opacity: 0;
        pointer-events: none;
        transform: translateX(-6px) translateZ(0);
        will-change: opacity, transform;
        backface-visibility: hidden;
        transition: opacity 0.08s ease, transform 0.08s ease;
        white-space: nowrap;
    }

    .label-wrapper.visible {
        opacity: 1;
        pointer-events: auto;
        transform: translateX(0) translateZ(0);
        transition: opacity 0.14s ease 0.04s, transform 0.14s cubic-bezier(0.16, 1, 0.3, 1) 0.04s;
    }

    .label {
        white-space: nowrap;
        font-size: 0.875rem;
        font-weight: 500;
        letter-spacing: 0.01em;
    }

    /* Bottom Utilities: Settings */
    .sidebar-footer {
        width: 100%;
        padding-top: 1rem;
        border-top: 1px solid rgba(255, 255, 255, 0.08);
        position: relative;
        z-index: 10;
    }

    .active-pill.settings-pill {
        position: absolute;
        inset: 0;
        border-radius: 1rem;
        background: rgba(226, 169, 115, 0.12);
        border: 1px solid rgba(226, 169, 115, 0.3);
        pointer-events: none;
        z-index: -1;
    }

    .active-pill.settings-pill.is-glass {
        background: linear-gradient(180deg, rgba(200, 157, 110, 0.22) 0%, rgba(150, 107, 61, 0.12) 100%);
        border: 1px solid rgba(224, 184, 143, 0.35);
        box-shadow: 
            inset 0 1px 1px rgba(255, 255, 255, 0.35),
            0 8px 24px -4px rgba(0, 0, 0, 0.5);
    }

    .settings-icon {
        transition: transform 0.5s cubic-bezier(0.16, 1, 0.3, 1), color 0.2s ease;
    }

    .settings-item:hover .settings-icon {
        transform: rotate(90deg);
    }
</style>
