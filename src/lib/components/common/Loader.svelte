<script module lang="ts">
    let loaderInstanceId = 0;
</script>

<script lang="ts">
    interface Props {
        size?: number | string;
        strokeWidth?: number;
        color?: "primary" | "current" | "muted" | string;
        trackOpacity?: number;
        label?: string;
        class?: string;
    }

    let {
        size = 16,
        strokeWidth,
        color = "current",
        trackOpacity = 0.15,
        label = "Loading...",
        class: className = ""
    }: Props = $props();

    const instanceId = ++loaderInstanceId;
    const gradientId = `echo-loader-grad-${instanceId}`;

    let computedSize = $derived(typeof size === "number" ? `${size}px` : size);
    let numSize = $derived(typeof size === "number" ? size : parseFloat(size) || 16);
    let computedStroke = $derived(
        strokeWidth ?? (numSize <= 16 ? 2.5 : numSize <= 24 ? 2.2 : 2.0)
    );

    let resolvedColor = $derived.by(() => {
        if (color === "primary") return "var(--echo-primary, #e2a973)";
        if (color === "muted") return "var(--echo-text-3, #6e6c7a)";
        if (color === "current") return "currentColor";
        return color;
    });
</script>

<svg
    class="echo-loader {className}"
    style="width: {computedSize}; height: {computedSize};"
    viewBox="0 0 24 24"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    role="status"
    aria-label={label}
>
    <defs>
        <linearGradient id={gradientId} x1="100%" y1="0%" x2="0%" y2="100%">
            <stop offset="0%" stop-color={resolvedColor} stop-opacity="1" />
            <stop offset="65%" stop-color={resolvedColor} stop-opacity="0.35" />
            <stop offset="100%" stop-color={resolvedColor} stop-opacity="0" />
        </linearGradient>
    </defs>

    <!-- Subtle recessed circular track -->
    <circle
        cx="12"
        cy="12"
        r="9"
        stroke={resolvedColor}
        stroke-width={computedStroke}
        stroke-opacity={trackOpacity}
    />

    <!-- Precision tapered arc -->
    <path
        d="M 12 3 A 9 9 0 0 1 21 12 A 9 9 0 0 1 12 21"
        stroke="url(#{gradientId})"
        stroke-width={computedStroke}
        stroke-linecap="round"
    />
</svg>

<style>
    .echo-loader {
        display: inline-block;
        vertical-align: middle;
        flex-shrink: 0;
        animation: echo-loader-spin 0.8s linear infinite;
        pointer-events: none;
    }

    @keyframes echo-loader-spin {
        from {
            transform: rotate(0deg);
        }
        to {
            transform: rotate(360deg);
        }
    }

    @media (prefers-reduced-motion: reduce) {
        .echo-loader {
            animation-duration: 2s;
        }
    }
</style>
