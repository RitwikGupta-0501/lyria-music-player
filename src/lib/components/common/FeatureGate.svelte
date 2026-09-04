<script lang="ts">
    import { flagsStore, type FeatureFlagKey } from "$lib/stores/flags.svelte";
    import type { Snippet } from "svelte";

    interface Props {
        flag: FeatureFlagKey | string;
        children?: Snippet;
        fallback?: Snippet;
    }

    let { flag, children, fallback }: Props = $props();
</script>

{#if flagsStore.isEnabled(flag)}
    {@render children?.()}
{:else if fallback}
    {@render fallback()}
{/if}
