<script lang="ts">
    import { call, satoshis, type Snapshot } from './api';
    import { l } from './i18n';
    let {
        snapshotView,
        busy,
        run,
        onprepared,
    }: {
        snapshotView: Snapshot;
        busy: boolean;
        run: (p: { action: () => Promise<unknown>; success?: string }) => Promise<void>;
        onprepared: () => void;
    } = $props();
    let accountId = $state(''),
        recipient = $state(''),
        amount = $state('10000'),
        fee = $state('1000'),
        change = $state(''),
        utxos = $state('[]');
    const fetch = () =>
        run({
            action: async () => {
                utxos = JSON.stringify(await call({ command: 'fetch_utxos', args: { accountId } }), null, 2);
            },
        });
    const prepare = () =>
        run({
            action: async () => {
                const source = snapshotView.accounts.find((a) => a.id === accountId);
                if (!source) throw new Error(l.source);
                await call({
                    command: 'prepare_transfer',
                    args: {
                        network: source.network,
                        utxos: JSON.parse(utxos),
                        recipients: [{ address: recipient, satoshis: satoshis({ value: amount }) }],
                        change,
                        feeSats: satoshis({ value: fee }),
                    },
                });
                onprepared();
            },
            success: l.savedRequest,
        });
</script>

<section>
    <h2>{l.transfer}</h2>
    <label
        >{l.source}<select
            bind:value={accountId}
            onchange={() => {
                change = snapshotView.accounts.find((a) => a.id === accountId)?.address ?? '';
            }}
            ><option value="">{l.select}</option>{#each snapshotView.accounts as account}<option value={account.id}
                    >{account.label} · {account.network} · {account.address}</option
                >{/each}</select
        ></label
    ><label>{l.recipient}<input bind:value={recipient} spellcheck="false" /></label>
    <div class="two-columns">
        <label>{l.amount}<input bind:value={amount} inputmode="numeric" /></label><label
            >{l.fee}<input bind:value={fee} inputmode="numeric" /></label
        >
    </div>
    <label
        >{l.change}<select bind:value={change}
            ><option value="">{l.select}</option
            >{#each snapshotView.accounts.filter((a) => a.network === snapshotView.accounts.find((b) => b.id === accountId)?.network) as account}<option
                    value={account.address}>{account.label} · {account.address}</option
                >{/each}</select
        ></label
    >
</section>
<section>
    <div class="section-heading">
        <h2>{l.utxos}</h2>
        <button disabled={busy || !accountId || !snapshotView.settings?.rpcUrl} onclick={() => void fetch()}
            >{l.fetchUtxos}</button
        >
    </div>
    <p class="hint">{l.utxoHelp}</p>
    <label>{l.utxoJson}<textarea class="code-input" bind:value={utxos} spellcheck="false"></textarea></label><code class="hint"
        >[{'{'}"transactionHex": "...", "vout": 0{'}'}]</code
    >
    <div class="actions">
        <button class="primary" disabled={busy || !accountId || !recipient} onclick={() => void prepare()}>{l.prepare}</button>
    </div>
</section>
