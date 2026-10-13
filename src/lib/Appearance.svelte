<script lang="ts">
    import { untrack } from 'svelte';
    import { applyAppearance, readAppearance, palettes } from './appearance';
    import { l } from './i18n';
    let appearance = $state(untrack(readAppearance));
    const apply = () => applyAppearance({ appearance });
</script>

<div class="theme-controls">
    <label
        >{l.palette}<select bind:value={appearance.palette} onchange={apply}
            >{#each palettes as palette}<option value={palette}>{l[palette]}</option>{/each}</select
        ></label
    >
    <label
        >{l.themeMode}<select bind:value={appearance.theme} onchange={apply}
            ><option value="dark">{l.dark}</option><option value="light">{l.light}</option></select
        ></label
    >
    <label
        >{l.font}<select bind:value={appearance.font} onchange={apply}
            ><option value="system-stack">System monospace</option><option value="consolas">Consolas</option><option
                value="cascadia-mono">Cascadia Mono</option
            ></select
        ></label
    >
</div>
<small role="status">{l[appearance.palette]} / {l[appearance.theme]}</small>
