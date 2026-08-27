<script lang="ts">
    import { settingsStore } from "$lib/stores/settings.svelte";
    import { List, House, Disc, PuzzlePiece, Gear, Compass } from "phosphor-svelte";
    
    let { activeView = $bindable("albums") } = $props<{ activeView?: string }>();
    
    let sidebarOpen = $state(false);
    let isMoving = $state(false);
    let moveTimeout: ReturnType<typeof setTimeout> | undefined;

    function toggleSidebar() {
        sidebarOpen = !sidebarOpen;
    }

    const primaryNav = [
        { id: "home", label: "Home", icon: House },
        { id: "explore", label: "Explore", icon: Compass },
        { id: "library", label: "Library", icon: Disc },
        { id: "providers", label: "Extensions", icon: PuzzlePiece },
    ];

    let navButtonRefs: HTMLElement[] = $state([]);
    let pillTop = $state(0);
    let prevIndex = $state(2);

    let activeIndex = $derived.by(() => {
        if (activeView === "home") return 0;
        if (activeView === "explore") return 1;
        if (activeView === "albums" || activeView === "playlists") return 2;
        if (activeView === "providers") return 3;
        return -1;
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
    class:is-glass={settingsStore.glassyPlayerBar}
>
    <!-- 3. Sub-surface Caustic Sheen (Specular Corner Glare) -->
    <div class="specular-corner-glare" aria-hidden="true"></div>

    <!-- Top Section: Header & Navigation Core -->
    <div class="sidebar-top">
        <!-- Header / App Identity -->
        <div class="sidebar-header">
            <button 
                class="toggle-btn" 
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
                    class:is-moving={isMoving}
                    style="transform: translateY({pillTop}px) {isMoving ? 'scaleY(1.12) scaleX(0.97)' : 'scale(1)'};"
                >
                    <!-- 1. Internal Refractive Specular Catch-Light -->
                    <div class="specular-top-highlight" aria-hidden="true"></div>
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
                <div class="active-pill settings-pill">
                    <div class="specular-top-highlight" aria-hidden="true"></div>
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
        
        /* 2. Multi-Tiered Depth Blurring & Dispersion */
        background: rgba(13, 14, 18, 0.45);
        backdrop-filter: blur(48px) saturate(1.9) brightness(1.1);
        -webkit-backdrop-filter: blur(48px) saturate(1.9) brightness(1.1);
        
        /* 1. Double-Layer Specular Catch-Lights (Optical Rim & Deep Ambient Shadow) */
        border-right: 1px solid rgba(255, 255, 255, 0.12);
        box-shadow: 
            inset 0 1px 1px rgba(255, 255, 255, 0.15),
            1px 0 30px rgba(0, 0, 0, 0.6);
        
        transition: width 0.5s cubic-bezier(0.16, 1, 0.3, 1), background 0.3s ease;
    }

    .sidebar.open {
        width: 256px; /* Expanded state */
    }

    @media (prefers-reduced-transparency: reduce) {
        .sidebar {
            background: var(--echo-surface, #101014);
            backdrop-filter: none;
            -webkit-backdrop-filter: none;
        }
    }

    /* 3. Sub-surface Caustic Sheen */
    .specular-corner-glare {
        pointer-events: none;
        position: absolute;
        top: -6rem;
        left: -6rem;
        width: 15rem;
        height: 15rem;
        border-radius: 9999px;
        background: radial-gradient(circle at 35% 35%, rgba(255, 255, 255, 0.12) 0%, rgba(181, 142, 98, 0.08) 45%, transparent 75%);
        filter: blur(32px);
        -webkit-filter: blur(32px);
        z-index: 0;
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
        border: 1px solid rgba(255, 255, 255, 0.15);
        background: rgba(255, 255, 255, 0.06);
        color: #e4e4e7;
        box-shadow: inset 0 1px 1px rgba(255, 255, 255, 0.2);
        display: flex;
        align-items: center;
        justify-content: center;
        cursor: pointer;
        transition: all 0.2s ease;
        padding: 0;
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
        max-width: 0;
        opacity: 0;
        transform: translateX(-8px);
        transition: max-width 0.5s cubic-bezier(0.16, 1, 0.3, 1), opacity 0.3s ease, transform 0.5s cubic-bezier(0.16, 1, 0.3, 1);
        white-space: nowrap;
    }

    .brand-wrapper.visible {
        max-width: 180px;
        opacity: 1;
        transform: translateX(0);
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
        
        /* Refractive Amber/Brass Liquid Gradient */
        background: linear-gradient(180deg, rgba(200, 157, 110, 0.25) 0%, rgba(150, 107, 61, 0.15) 100%);
        border: 1px solid rgba(224, 184, 143, 0.40);
        backdrop-filter: blur(24px) saturate(2);
        -webkit-backdrop-filter: blur(24px) saturate(2);
        
        /* Double-Layer Specular Catch-Lights */
        box-shadow: 
            inset 0 1px 1px rgba(255, 255, 255, 0.35),
            0 8px 24px -4px rgba(0, 0, 0, 0.5);
            
        transition: 
            transform 0.4s cubic-bezier(0.16, 1, 0.3, 1),
            width 0.5s cubic-bezier(0.16, 1, 0.3, 1),
            opacity 0.2s ease;
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
        max-width: 0;
        opacity: 0;
        transform: translateX(-6px);
        transition: max-width 0.5s cubic-bezier(0.16, 1, 0.3, 1), opacity 0.28s ease, transform 0.5s cubic-bezier(0.16, 1, 0.3, 1);
        white-space: nowrap;
    }

    .label-wrapper.visible {
        max-width: 160px;
        opacity: 1;
        transform: translateX(0);
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
        background: linear-gradient(180deg, rgba(200, 157, 110, 0.25) 0%, rgba(150, 107, 61, 0.15) 100%);
        border: 1px solid rgba(224, 184, 143, 0.40);
        backdrop-filter: blur(24px) saturate(2);
        -webkit-backdrop-filter: blur(24px) saturate(2);
        box-shadow: 
            inset 0 1px 1px rgba(255, 255, 255, 0.35),
            0 8px 24px -4px rgba(0, 0, 0, 0.5);
        pointer-events: none;
        z-index: -1;
    }

    .settings-icon {
        transition: transform 0.5s cubic-bezier(0.16, 1, 0.3, 1), color 0.2s ease;
    }

    .settings-item:hover .settings-icon {
        transform: rotate(90deg);
    }
</style>
