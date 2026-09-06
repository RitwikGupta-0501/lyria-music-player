<script lang="ts">
    import { settingsStore } from "$lib/stores/settings.svelte";
</script>

<section class="settings-section">
    <h3 class="section-title">Appearance</h3>

    <div class="setting-row">
        <div class="setting-info">
            <p class="setting-label">Glassy Player Bar</p>
            <p class="setting-desc">Enable a sleek, semi-transparent frosted glass effect for the bottom player.</p>
        </div>
        <label class="switch">
            <input
                type="checkbox"
                checked={settingsStore.glassyPlayerBar}
                onchange={(e) => settingsStore.setGlassyPlayerBar(e.currentTarget.checked)}
                disabled={!settingsStore.loaded}
            />
            <span class="slider round"></span>
        </label>
    </div>
</section>

<style>
    .settings-section {
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
        animation: fadeIn 0.2s ease;
    }
    .section-title {
        font-family: var(--lyria-font-body);
        font-size: 1.25rem;
        font-weight: 500;
        color: var(--echo-text-1);
        padding-bottom: 0.5rem;
        border-bottom: 1px solid var(--echo-border);
        margin-bottom: 0.5rem;
    }
    .setting-row {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 1rem 0;
        gap: 1.5rem;
    }
    .setting-info {
        display: flex;
        flex-direction: column;
        gap: 0.3rem;
        flex: 1;
    }
    .setting-label {
        font-size: 0.95rem;
        font-weight: 500;
        color: var(--echo-text-1);
    }
    .setting-desc {
        font-size: 0.85rem;
        color: var(--echo-text-2);
        line-height: 1.4;
    }
    .switch {
        position: relative;
        display: inline-block;
        width: 44px;
        height: 24px;
        flex-shrink: 0;
    }
    .switch input {
        opacity: 0;
        width: 0;
        height: 0;
    }
    .slider {
        position: absolute;
        cursor: pointer;
        inset: 0;
        background-color: var(--echo-raised);
        border: 1px solid var(--echo-border-medium);
        transition: 0.2s cubic-bezier(0.16, 1, 0.3, 1);
        border-radius: 24px;
    }
    .slider:before {
        position: absolute;
        content: "";
        height: 16px;
        width: 16px;
        left: 3px;
        bottom: 3px;
        background-color: var(--echo-text-2);
        transition: 0.2s cubic-bezier(0.16, 1, 0.3, 1);
        border-radius: 50%;
    }
    input:checked + .slider {
        background-color: var(--echo-primary);
        border-color: var(--echo-primary);
    }
    input:checked + .slider:before {
        transform: translateX(20px);
        background-color: var(--echo-void);
    }
</style>
