<script lang="ts">
    import { l } from './i18n';
    import DiceHelp from './DiceHelp.svelte';
    let level = $state('basic');
</script>

<div class="help-page">
    <div class="tabs">
        <button
            class:active={level === 'basic'}
            onclick={() => {
                level = 'basic';
            }}>{l.basicInstructions}</button
        ><button
            class:active={level === 'advanced'}
            onclick={() => {
                level = 'advanced';
            }}>{l.advancedInstructions}</button
        >
    </div>
    <section>
        {#if level === 'basic'}<h2>{l.basicInstructions}</h2>
            <pre>{l.basicVault}</pre>
            <p>{l.basicBackup}</p>{:else}<h2>{l.advancedInstructions}</h2>
            <p>{l.advancedVault}</p>
            <p>{l.advancedProvider}</p>
            <p>{l.secretWorksheet}</p>{/if}
    </section>
    <section><DiceHelp /></section>
</div>
