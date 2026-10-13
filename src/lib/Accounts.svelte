<script lang="ts">
    import { call, type Snapshot, type Network } from './api';
    import { l } from './i18n';
    let {
        snapshotView,
        busy,
        run,
        onexport,
    }: {
        snapshotView: Snapshot;
        busy: boolean;
        run: (p: { action: () => Promise<unknown>; success?: string }) => Promise<void>;
        onexport: () => void;
    } = $props();
    let identityId = $state(''),
        label = $state(''),
        path = $state("m/84'/1'/0'/0/0"),
        keyPassword = $state(''),
        search = $state('');
    let network = $state<Network>('testnet4');
    let summary = $state('');
    const derive = () =>
        run({
            action: async () => {
                try {
                    await call({
                        command: 'derive_account',
                        args: { identityId, label, network, path, keyPassword: keyPassword || null },
                    });
                } finally {
                    keyPassword = '';
                }
            },
            success: l.saved,
        });
    const copy = ({ text }: { text: string }) => run({ action: () => navigator.clipboard.writeText(text), success: l.copied });
    const balance = ({ accountId }: { accountId: string }) =>
        run({
            action: async () => {
                summary = JSON.stringify(await call({ command: 'address_summary', args: { accountId } }), null, 2);
            },
        });
</script>

<div class="actions">
    <button onclick={onexport} disabled={busy || !snapshotView.accounts.length}>{l.exportPublic}</button><span class="hint"
        >{l.exportPrivacy}</span
    >
</div>
{#if snapshotView.settings?.mode !== 'proxy'}
    <section>
        <h2>{l.derive}</h2>
        <div class="three-columns">
            <label
                >{l.identity}<select bind:value={identityId}
                    ><option value="">{l.select}</option>{#each snapshotView.identities as identity}<option value={identity.id}
                            >{snapshotView.roots.find((r) => r.id === identity.rootId)?.label} / {identity.label}</option
                        >{/each}</select
                ></label
            ><label>{l.name}<input bind:value={label} maxlength="120" /></label><label
                >{l.network}<select
                    bind:value={network}
                    onchange={() => {
                        path = network === 'bitcoin' ? "m/84'/0'/0'/0/0" : "m/84'/1'/0'/0/0";
                    }}
                    ><option value="bitcoin">Bitcoin</option><option value="testnet4">Testnet4</option><option value="testnet"
                        >Testnet3</option
                    ><option value="signet">Signet</option><option value="regtest">Regtest</option></select
                ></label
            >
        </div>
        <div class="two-columns">
            <label>{l.path}<input bind:value={path} maxlength="120" /></label><label
                >{l.reauth}<input type="password" bind:value={keyPassword} autocomplete="off" /></label
            >
        </div>
        <p class="hint">{l.pathHelp}</p>
        <button class="primary" disabled={busy || !identityId || !label} onclick={() => void derive()}>{l.derive}</button>
    </section>
{/if}
<label>{l.search}<input bind:value={search} type="search" /></label>
{#if !snapshotView.accounts.length}<section class="empty">{l.emptyAccounts}</section>{/if}
{#each snapshotView.accounts.filter((a) => (a.label + a.address + a.path)
        .toLowerCase()
        .includes(search.toLowerCase())) as account}
    <section>
        <div class="section-heading">
            <h2>{account.label}</h2>
            <span class="badge">{account.network} · {account.watchOnly ? l.watchOnly : l.localKey}</span>
        </div>
        <code class="address">{account.address}</code>
        <div class="actions">
            <button onclick={() => void copy({ text: account.address })}>{l.copy}</button
            >{#if snapshotView.settings?.mode !== 'offline'}<button
                    disabled={busy}
                    onclick={() => void balance({ accountId: account.id })}>{l.balance}</button
                >{/if}
        </div>
        <details>
            <summary>{l.details}</summary>
            <dl>
                <dt>{l.path}</dt>
                <dd><code>{account.path}</code></dd>
                <dt>{l.publicKey}</dt>
                <dd><code class="wrap">{account.publicKey}</code></dd>
                <dt>{l.identity}</dt>
                <dd>{snapshotView.identities.find((i) => i.id === account.identityId)?.label ?? l.watchOnly}</dd>
            </dl>
        </details>
    </section>
{/each}
{#if summary}<section>
        <h2>{l.external}</h2>
        <pre>{summary}</pre>
    </section>{/if}
