<script lang="ts">
    import { onMount } from 'svelte';
    import { call, type Snapshot, type Review } from './api';
    import { l } from './i18n';
    let {
        snapshotView,
        review,
        busy,
        run,
        onclose,
        error = '',
    }: {
        snapshotView: Snapshot;
        review: Review;
        busy: boolean;
        run: (p: { action: () => Promise<unknown>; success?: string }) => Promise<void>;
        onclose: () => void;
        error?: string;
    } = $props();
    let dialog: HTMLDialogElement;
    let approved = $state(false),
        broadcastApproved = $state(false),
        password = $state('');
    let passwords = $state<Record<string, string>>({}),
        selected = $state<Record<string, string>>({});
    const response = $derived(snapshotView.responses.find((r) => r.requestId === review.inspection.requestId));
    const sign = () =>
        run({
            action: async () => {
                try {
                    await call({
                        command: 'approve_signing',
                        args: {
                            token: review.token,
                            confirmed: approved,
                            password,
                            keyPasswords: passwords,
                            selectedAccounts: Object.values(selected),
                        },
                    });
                    onclose();
                } finally {
                    password = '';
                    passwords = {};
                    approved = false;
                }
            },
            success: l.signed,
        });
    const broadcast = () =>
        run({
            action: async () => {
                try {
                    await call({
                        command: 'broadcast_response',
                        args: { id: review.inspection.requestId, password, confirmed: broadcastApproved },
                    });
                    onclose();
                } finally {
                    password = '';
                    broadcastApproved = false;
                }
            },
            success: l.broadcasted,
        });
    onMount(() => {
        dialog.showModal();
    });
</script>

<dialog
    bind:this={dialog!}
    oncancel={(event) => {
        event.preventDefault();
        if (!busy) onclose();
    }}
>
    <div class="dialog-heading">
        <div>
            <div class="eyebrow">{l.review}</div>
            <h2>{l.native} / {review.inspection.network}</h2>
        </div>
        <button onclick={onclose} disabled={busy}>{l.close}</button>
    </div>
    <p>{l.reviewHelp}</p>
    {#if error}<div class="alert danger" role="alert">{error}</div>{/if}
    <div class="alert warning">{l.external}. UTXO spend status cannot be established offline.</div>
    <h3>{l.outputs}</h3>
    {#each review.inspection.outputs as output, index}<article class="review-output">
            <div class="section-heading">
                <span>{index + 1}. {output.knownAccount ? l.known : l.destination}</span><strong
                    >{output.satoshis.toLocaleString()} {l.sats}</strong
                >
            </div>
            <code class="address">{output.address}</code>
        </article>{/each}
    <div class="stats compact">
        <article><small>{l.totalInput}</small><strong>{review.inspection.totalInputSats.toLocaleString()}</strong></article>
        <article><small>{l.networkFee}</small><strong>{review.inspection.feeSats.toLocaleString()}</strong></article>
        <article><small>{l.feeRate}</small><strong>{review.inspection.feeRate.toFixed(2)}</strong></article>
    </div>
    <h3>{l.inputs}</h3>
    {#each review.inspection.inputs as input}
        {@const matches = snapshotView.accounts.filter((a) => input.accountIds.includes(a.id) && !a.watchOnly)}
        <article class="review-input">
            <code class="address">{input.address}</code><strong>{input.satoshis.toLocaleString()} {l.sats}</strong>
            {#if matches.length === 1}<p>{l.signer}: {matches[0].label} · {matches[0].path}</p>
            {:else if matches.length > 1}<label
                    >{l.selectSigner}<select bind:value={selected[input.outpoint]}
                        ><option value="">{l.select}</option>{#each matches as account}<option value={account.id}
                                >{account.label} · {account.path}</option
                            >{/each}</select
                    ></label
                >
            {:else}<p class="muted">{l.noSigner}</p>{/if}
            <details>
                <summary>{l.advanced}</summary>
                <dl>
                    <dt>{l.outpoint}</dt>
                    <dd><code class="wrap">{input.outpoint}</code></dd>
                    <dt>{l.sequence}</dt>
                    <dd>{input.sequence}</dd>
                    <dt>{l.sighash}</dt>
                    <dd>{input.sighash}</dd>
                </dl>
            </details>
        </article>
    {/each}
    <details>
        <summary>{l.advanced}</summary>
        <dl>
            <dt>{l.digest}</dt>
            <dd><code class="wrap">{review.inspection.digest}</code></dd>
            <dt>{l.txVersion}</dt>
            <dd>{review.inspection.version}</dd>
            <dt>{l.lockTime}</dt>
            <dd>{review.inspection.lockTime}</dd>
        </dl>
        <label
            >{l.rawRequest}<textarea
                readonly
                value={snapshotView.pending.find((r) => r.requestId === review.inspection.requestId)?.payload.data}
            ></textarea></label
        >
    </details>
    {#if response}<div class="alert mid">{l.response}<code class="wrap">{response.txid}</code></div>{/if}
    {#if !response && snapshotView.settings?.mode !== 'proxy'}
        <p class="hint">{l.signHelp}</p>
        <label>{l.password}<input type="password" bind:value={password} autocomplete="off" /></label>
        {#each snapshotView.roots.filter((r) => r.customPassword && review.inspection.inputs.some( (i) => snapshotView.accounts.some((a) => i.accountIds.includes(a.id) && snapshotView.identities.some((id) => id.id === a.identityId && id.rootId === r.id)) )) as root}<label
                >{root.label} / {l.reauth}<input type="password" bind:value={passwords[root.id]} autocomplete="off" /></label
            >{/each}
        <label class="check"><input type="checkbox" bind:checked={approved} />{l.approveCheck}</label><button
            class="primary"
            disabled={busy || !approved || !password}
            onclick={() => void sign()}>{l.approve}</button
        >
    {:else if response && snapshotView.settings?.mode !== 'offline'}
        <label>{l.password}<input type="password" bind:value={password} autocomplete="off" /></label><label class="check"
            ><input type="checkbox" bind:checked={broadcastApproved} />{l.broadcastCheck}</label
        ><button
            class="primary"
            disabled={busy || !broadcastApproved || !password || !snapshotView.settings?.rpcUrl}
            onclick={() => void broadcast()}>{l.broadcast}</button
        >
    {/if}
</dialog>
