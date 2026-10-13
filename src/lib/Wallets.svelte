<script lang="ts">
    import { onDestroy } from 'svelte';
    import { call, type Snapshot, type Network } from './api';
    import { l } from './i18n';
    import DiceEntry from './DiceEntry.svelte';
    import DiceReview from './DiceReview.svelte';
    import DiceHelp from './DiceHelp.svelte';
    import { rollBit, type DiceWord } from './dice';
    let entries = $state<DiceWord[]>([]),
        rawRolls = $state<number[]>([]);
    let prefixMode = $state('dice'),
        rawMode = $state('dice');
    const clearEntropy = () => {
        entries = [];
        rawRolls = [];
        prefix = '';
        bits = '';
    };

    let {
        snapshotView,
        busy,
        run,
    }: {
        snapshotView: Snapshot;
        busy: boolean;
        run: (p: { action: () => Promise<unknown>; success?: string }) => Promise<void>;
    } = $props();
    let action = $state('generate'),
        label = $state(''),
        phrase = $state(''),
        keyPassword = $state('');
    let words = $state(24),
        method = $state('mixed'),
        count = $state(11),
        prefix = $state(''),
        bits = $state('');
    let token = $state(''),
        shown = $state(''),
        confirmation = $state('');
    let rootId = $state(''),
        identityLabel = $state(''),
        passphrase = $state(''),
        rootPassword = $state(''),
        verifyPhrase = $state(''),
        verifyResult = $state('');
    let network = $state<Network>('testnet4');
    const clearSecrets = () => {
        clearEntropy();
        phrase = '';
        keyPassword = '';
        prefix = '';
        bits = '';
        passphrase = '';
        rootPassword = '';
        verifyPhrase = '';
        confirmation = '';
    };
    const createPhrase = () =>
        run({
            action: async () => {
                const suppliedPrefix = prefixMode === 'dice' ? entries.map((entry) => entry.word).join(' ') : prefix;
                const suppliedBits = rawMode === 'dice' ? rawRolls.map((roll) => rollBit({ roll })).join('') : bits;
                if (method === 'mixed' && suppliedPrefix.trim().split(/\s+/).filter(Boolean).length !== count)
                    throw new Error(l.requiredPrefix);
                const result = await call<{ token: string; phrase: string }>({
                    command: 'generate_phrase',
                    args: {
                        words,
                        prefix: method === 'mixed' ? suppliedPrefix : '',
                        bits: method === 'raw' ? suppliedBits : null,
                    },
                });
                token = result.token;
                shown = result.phrase;
                prefix = '';
                bits = '';
            },
        });
    const cancel = () =>
        run({
            action: async () => {
                shown = '';
                token = '';
                clearSecrets();
                await call({ command: 'cancel_ceremony' });
            },
        });
    onDestroy(() => {
        clearSecrets();
        shown = '';
        if (token)
            void call({ command: 'cancel_ceremony' }).catch(() => {
                /* Native ceremony also expires or is cleared by locking. */
            });
    });
    const finalize = () =>
        run({
            action: async () => {
                try {
                    await call({
                        command: 'confirm_phrase',
                        args: { token, confirmation, label, keyPassword: keyPassword || null },
                    });
                    label = '';
                } finally {
                    token = '';
                    shown = '';
                    clearSecrets();
                }
            },
            success: l.saved,
        });
    const importSecret = () =>
        run({
            action: async () => {
                try {
                    await call({
                        command: action === 'wif' ? 'import_wif' : 'import_phrase',
                        args: { label, phrase, wif: phrase, network, keyPassword: keyPassword || null },
                    });
                    label = '';
                } finally {
                    clearSecrets();
                }
            },
            success: l.saved,
        });
    const addIdentity = () =>
        run({
            action: async () => {
                try {
                    await call({
                        command: 'add_identity',
                        args: { rootId, label: identityLabel, passphrase, keyPassword: rootPassword || null },
                    });
                    identityLabel = '';
                } finally {
                    clearSecrets();
                }
            },
            success: l.saved,
        });
    const verify = () =>
        run({
            action: async () => {
                try {
                    const matched = await call<boolean>({
                        command: 'verify_recovery',
                        args: { rootId, phrase: verifyPhrase, keyPassword: rootPassword || null },
                    });
                    verifyResult = matched ? l.verified : l.notMatched;
                } finally {
                    clearSecrets();
                }
            },
        });
</script>

<div class="two-columns align-start">
    <section>
        <div class="tabs">
            <button
                disabled={!!token}
                class:active={action === 'generate'}
                onclick={() => {
                    action = 'generate';
                    clearSecrets();
                }}>{l.newWallet}</button
            ><button
                disabled={!!token}
                class:active={action === 'import'}
                onclick={() => {
                    action = 'import';
                    clearSecrets();
                }}>{l.importWallet}</button
            ><button
                disabled={!!token}
                class:active={action === 'wif'}
                onclick={() => {
                    action = 'wif';
                    clearSecrets();
                }}>{l.importWif}</button
            >
        </div>
        {#if !token}
            <label>{l.rootLabel}<input bind:value={label} maxlength="120" /></label>
            <label>{l.keyPassword}<input type="password" bind:value={keyPassword} autocomplete="off" maxlength="1024" /></label>
            <p class="hint">{l.keyPasswordHelp}</p>
            {#if action === 'generate'}
                <div class="two-columns">
                    <label
                        >{l.phraseLength}<select
                            bind:value={words}
                            onchange={() => {
                                count = Math.min(count, words - 1);
                                clearEntropy();
                            }}><option value={24}>{l.words24}</option><option value={12}>{l.words12}</option></select
                        ></label
                    ><label
                        >{l.entropyMethod}<select bind:value={method} onchange={clearEntropy}
                            ><option value="mixed">{l.mixed}</option><option value="os">{l.os}</option><option value="raw"
                                >{l.raw}</option
                            ></select
                        ></label
                    >
                </div>
                <details>
                    <summary>{l.basicInstructions}</summary>
                    <p>{l.generatorBasic}</p>
                    <DiceHelp />
                </details>
                {#if method === 'mixed'}
                    <label
                        >{l.manualCount}<input
                            type="number"
                            min="0"
                            max={words - 1}
                            step="1"
                            bind:value={count}
                            onchange={clearEntropy}
                        /></label
                    >
                    <label
                        >{l.prefixEntry}<select bind:value={prefixMode} onchange={clearEntropy}
                            ><option value="dice">{l.guidedDice}</option><option value="manual">{l.manualEntry}</option></select
                        ></label
                    >
                    {#if prefixMode === 'dice'}
                        <DiceEntry bind:entries bind:rawRolls {count} bitCount={words === 24 ? 256 : 128} disabled={busy} />
                    {:else}<label
                            >{l.manualWords}<textarea bind:value={prefix} spellcheck="false" autocomplete="off"></textarea></label
                        >{/if}
                {/if}
                {#if method === 'raw'}
                    <label
                        >{l.entropyEntry}<select bind:value={rawMode} onchange={clearEntropy}
                            ><option value="dice">{l.guidedDice}</option><option value="bits">{l.rawBits}</option></select
                        ></label
                    >
                    {#if rawMode === 'dice'}<DiceEntry
                            bind:entries
                            bind:rawRolls
                            {count}
                            bitCount={words === 24 ? 256 : 128}
                            raw
                            disabled={busy}
                        />
                    {:else}<label>{l.rawBits}<textarea bind:value={bits} spellcheck="false" autocomplete="off"></textarea></label
                        >{/if}
                    <p class="hint">{l.rawHelp}</p>
                {:else}<div class="entropy-meter">
                        <span>{l.manualBits}: {method === 'mixed' ? count * 11 : 0}</span><span
                            >{l.osBits}: {(words === 24 ? 256 : 128) - (method === 'mixed' ? count * 11 : 0)}</span
                        >
                    </div>
                    <details>
                        <summary>{l.entropyMethod} ⓘ</summary>
                        <p>{l.entropyHelp}</p>
                    </details>{/if}
                <button
                    class="primary"
                    disabled={busy ||
                        !label ||
                        (method === 'mixed' &&
                            (!Number.isInteger(count) ||
                                count < 0 ||
                                count >= words ||
                                (prefixMode === 'dice' && entries.length !== count))) ||
                        (method === 'raw' && rawMode === 'dice' && rawRolls.length !== (words === 24 ? 256 : 128))}
                    onclick={() => void createPhrase()}>{l.generate}</button
                >
            {:else}
                <label
                    >{action === 'wif' ? l.wif : l.phrase}<textarea bind:value={phrase} autocomplete="off" spellcheck="false"
                    ></textarea></label
                >
                {#if action === 'wif'}<label
                        >{l.network}<select bind:value={network}
                            ><option value="bitcoin">Bitcoin</option><option value="testnet4">Testnet4</option><option
                                value="testnet">Testnet3</option
                            ><option value="signet">Signet</option><option value="regtest">Regtest</option></select
                        ></label
                    >{/if}
                <button class="primary" disabled={busy || !label || !phrase} onclick={() => void importSecret()}>{l.save}</button>
            {/if}
        {:else}
            <div class="eyebrow">{l.oneTime}</div>
            <h2>{l.ceremony}</h2>
            <p>{l.ceremonyHelp}</p>
            {#if shown}<ol class="phrase-grid">
                    {#each shown.split(' ') as word, index}<li><span>{index + 1}</span>{word}</li>{/each}
                </ol>
                <button
                    class="primary"
                    onclick={() => {
                        shown = '';
                    }}>{l.hidePhrase}</button
                >
            {:else}<label
                    >{l.confirmation}<textarea bind:value={confirmation} autocomplete="off" spellcheck="false"></textarea></label
                ><button class="primary" disabled={busy || !confirmation} onclick={() => void finalize()}
                    >{l.confirmPhrase}</button
                >{/if}
            <button disabled={busy} onclick={() => void cancel()}>{l.cancel}</button>
        {/if}
        {#if entries.length}<DiceReview bind:entries {count} />{/if}
    </section>
    <div>
        <section>
            <h2>{l.wallets}</h2>
            {#if !snapshotView.roots.length}<p class="empty">{l.emptyWallets}</p>{/if}
            {#each snapshotView.roots as root}<article class="wallet-card">
                    <div class="section-heading">
                        <h3>{root.label}</h3>
                        <span class="badge">{root.customPassword ? l.protected : l.vaultProtected}</span>
                    </div>
                    <small>{root.kind} · {new Date(root.createdAt * 1000).toLocaleDateString()}</small
                    >{#each snapshotView.identities.filter((i) => i.rootId === root.id) as identity}<div class="identity">
                            <span>↳ {identity.label}</span><small
                                >{snapshotView.accounts.filter((a) => a.identityId === identity.id).length}
                                {l.accounts.toLowerCase()}</small
                            >
                        </div>{/each}
                </article>{/each}
        </section>
        <section>
            <h2>{l.addIdentity}</h2>
            <p class="hint">{l.passphraseHelp}</p>
            <label
                >{l.root}<select bind:value={rootId}
                    ><option value="">{l.select}</option
                    >{#each snapshotView.roots.filter((r) => r.kind === 'bip39') as root}<option value={root.id}
                            >{root.label}</option
                        >{/each}</select
                ></label
            ><label>{l.name}<input bind:value={identityLabel} maxlength="120" /></label><label
                >{l.passphrase}<input type="password" bind:value={passphrase} autocomplete="off" maxlength="1024" /></label
            ><label>{l.reauth}<input type="password" bind:value={rootPassword} autocomplete="off" /></label><button
                disabled={busy || !rootId || !passphrase || !identityLabel}
                onclick={() => void addIdentity()}>{l.addIdentity}</button
            >
        </section>
        <section>
            <h2>{l.verifyRecovery}</h2>
            <label
                >{l.root}<select bind:value={rootId}
                    ><option value="">{l.select}</option
                    >{#each snapshotView.roots.filter((r) => r.kind === 'bip39') as root}<option value={root.id}
                            >{root.label}</option
                        >{/each}</select
                ></label
            ><label>{l.phrase}<textarea bind:value={verifyPhrase} autocomplete="off" spellcheck="false"></textarea></label><label
                >{l.reauth}<input type="password" bind:value={rootPassword} autocomplete="off" /></label
            ><button disabled={busy || !rootId || !verifyPhrase} onclick={() => void verify()}>{l.verifyRecovery}</button
            >{#if verifyResult}<p role="status">{verifyResult}</p>{/if}
        </section>
    </div>
</div>
