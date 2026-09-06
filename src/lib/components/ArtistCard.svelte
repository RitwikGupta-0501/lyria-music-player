<script lang="ts">
    import { resolveCoverArt } from "$lib/utils/media";

    interface Props {
        artist: {
            id?: string;
            name?: string;
            artist?: string;
            avatar_url?: string;
            total_plays?: number;
            subtitle?: string;
            subscribers?: string;
        } | any;
        onclick?: () => void;
        size?: number;
        avatarSize?: number;
        fluid?: boolean;
    }

    let {
        artist,
        onclick,
        size = 176,
        avatarSize = 160,
        fluid = false,
    }: Props = $props();

    let displayName = $derived(artist?.name || artist?.artist || "Unknown Artist");
    let initial = $derived((displayName.charAt(0) || "A").toUpperCase());
    let avatarSrc = $derived(resolveCoverArt(artist?.avatar_url));

    let statText = $derived.by(() => {
        if (artist?.subtitle) return artist.subtitle;
        if (artist?.subscribers) return artist.subscribers;
        if (typeof artist?.total_plays === "number") {
            if (artist.total_plays > 0) {
                return `${artist.total_plays} ${artist.total_plays === 1 ? "play" : "plays"}`;
            }
            return "Discovered Seed";
        }
        return "Artist";
    });
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div 
    class="artist-card" 
    class:is-fluid={fluid}
    style="--card-size: {fluid ? '100%' : `${size}px`}; --avatar-size: {fluid ? '100%' : `${avatarSize}px`};"
    {onclick}
    role="button"
    tabindex="0"
    onkeydown={(e) => { if (e.key === "Enter" && onclick) onclick(); }}
>
    <div class="avatar-container" class:is-fluid={fluid}>
        {#if avatarSrc}
            <img 
                src={avatarSrc} 
                alt={displayName} 
                class="avatar-img" 
                loading="lazy" 
            />
        {:else}
            <div class="avatar-placeholder">
                <span class="avatar-letter">{initial}</span>
            </div>
        {/if}
    </div>

    <div class="artist-info">
        <span class="artist-name" title={displayName}>{displayName}</span>
        {#if statText}
            <span class="artist-stat">{statText}</span>
        {/if}
    </div>
</div>

<style>
    .artist-card {
        flex: 0 0 var(--card-size, 176px);
        width: var(--card-size, 176px);
        min-width: var(--card-size, 176px);
        display: flex;
        flex-direction: column;
        align-items: center;
        text-align: center;
        cursor: pointer;
        scroll-snap-align: start;
        user-select: none;
        align-self: flex-start;
        box-sizing: border-box;
    }

    .artist-card.is-fluid {
        flex: 1 1 0;
        width: 100%;
        min-width: 0;
    }

    .avatar-container {
        width: var(--avatar-size, 160px);
        height: var(--avatar-size, 160px);
        border-radius: 50%;
        overflow: hidden;
        position: relative;
        background: #141416;
        border: 1px solid rgba(255, 255, 255, 0.08);
        box-shadow: 0 8px 20px -4px rgba(0, 0, 0, 0.5);
        margin-bottom: 0.75rem;
        transform: translateZ(0);
        backface-visibility: hidden;
        transition: border-color 0.2s ease, box-shadow 0.2s ease;
    }

    .avatar-container.is-fluid {
        width: 100%;
        aspect-ratio: 1 / 1;
        height: auto;
    }

    .artist-card:hover .avatar-container {
        border-color: rgba(181, 142, 98, 0.35);
        box-shadow: 0 10px 24px -6px rgba(0, 0, 0, 0.6);
    }

    .avatar-img {
        width: 100%;
        height: 100%;
        object-fit: cover;
        display: block;
        transform: translateZ(0);
        backface-visibility: hidden;
        will-change: transform;
        transition: transform 0.55s cubic-bezier(0.05, 0.75, 0.15, 1);
    }

    .artist-card:hover .avatar-img {
        transform: scale(1.05);
    }

    .avatar-placeholder {
        width: 100%;
        height: 100%;
        display: flex;
        align-items: center;
        justify-content: center;
        background: radial-gradient(circle at center, #1c1c22 0%, #101014 100%);
    }

    .avatar-letter {
        font-family: var(--echo-font-heading, 'Newsreader', serif);
        font-size: 2.2rem;
        font-weight: 700;
        color: #B58E62;
    }

    .artist-info {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 0.2rem;
        width: 100%;
        min-width: 0;
    }

    .artist-name {
        font-family: var(--echo-font-body, system-ui, sans-serif);
        font-size: 0.88rem;
        font-weight: 600;
        color: #FFFFFF;
        max-width: 100%;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        display: block;
    }

    .artist-stat {
        font-family: var(--lyria-font-mono);
        font-size: 0.72rem;
        color: rgba(255, 255, 255, 0.45);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        display: block;
    }
</style>
