<script lang="ts">
    import { onMount } from 'svelte';
    import { open, save } from '@tauri-apps/plugin-dialog';
    import { call, emptySnapshot, errorText, type Snapshot, type Mode, type Review, type Capability } from './lib/api';
    import { l } from './lib/i18n';
    import Wallets from './lib/Wallets.svelte';
    import Accounts from './lib/Accounts.svelte';
    import Transfer from './lib/Transfer.svelte';
    import RequestReview from './lib/RequestReview.svelte';
    import Help from './lib/Help.svelte';
    import Appearance from './lib/Appearance.svelte';
    import SettingsView from './lib/Settings.svelte';
    let snapshotView = $state<Snapshot>(emptySnapshot);
    let page = $state('overview');
    let busy = $state(false);
    let error = $state('');
    let notice = $state('');
    let ready = $state(false);
    let password = $state('');
    let passwordConfirm = $state('');
    let mode = $state<Mode>('offline');
    let review = $state<Review | null>(null);
    let registry = $state<Capability[]>([]);
    let dryRun = $state(true);
    let lastActivity = 0;
    let wasUnlocked = false;
    const refresh = async () => {
        snapshotView = await call<Snapshot>({ command: 'snapshot' });
        if (wasUnlocked && !snapshotView.unlocked) {
            review = null;
            password = '';
            passwordConfirm = '';
            page = 'overview';
        }
        wasUnlocked = snapshotView.unlocked;
    };
    const run = async ({ action, success = '' }: { action: () => Promise<unknown>; success?: string }) => {
        if (busy) return;
        busy = true;
        error = '';
        notice = '';
        try {
            await action();
            await refresh();
            notice = success;
        } catch (cause) {
            error = errorText({ error: cause });
        } finally {
            busy = false;
        }
    };
    const markActivity = () => {
        if (!snapshotView.unlocked || Date.now() - lastActivity < 10000) return;
        lastActivity = Date.now();
        void call({ command: 'activity' }).catch(() => {
            error = l.locked;
        });
    };
    const authenticate = () =>
        run({
            action: async () => {
                try {
                    if (!snapshotView.initialized && password !== passwordConfirm) throw new Error(l.wrongConfirmation);
                    await call({ command: snapshotView.initialized ? 'unlock_vault' : 'create_vault', args: { password, mode } });
                } finally {
                    password = '';
                    passwordConfirm = '';
                }
            },
        });
    const importFile = () =>
        run({
            action: async () => {
                const path = await open({
                    multiple: false,
                    filters: [{ name: 'AirVault', extensions: ['avpub', 'avreq', 'avsig'] }],
                });
                if (typeof path === 'string') await call({ command: 'import_file', args: { path } });
            },
            success: l.imported,
        });
    const exportFile = ({ kind, id }: { kind: 'public' | 'request' | 'response'; id?: string }) =>
        run({
            action: async () => {
                const ext = { public: 'avpub', request: 'avreq', response: 'avsig' }[kind];
                const path = await save({
                    defaultPath: `airvault-${id ?? 'accounts'}.${ext}`,
                    filters: [{ name: 'AirVault', extensions: [ext] }],
                });
                if (path) await call({ command: `export_${kind}`, args: { path, id } });
            },
            success: l.exportDone,
        });
    const openReview = ({ id }: { id: string }) =>
        run({
            action: async () => {
                review = await call<Review>({ command: 'review_request', args: { id } });
            },
        });
    const restore = () =>
        run({
            action: async () => {
                try {
                    const path = await open({
                        multiple: false,
                        filters: [{ name: 'Encrypted AirVault backup', extensions: ['avvault'] }],
                    });
                    if (typeof path === 'string') await call({ command: 'restore_backup', args: { path, password, dryRun } });
                } finally {
                    password = '';
                }
            },
            success: dryRun ? l.restoreChecked : l.restoreDone,
        });
    onMount(() => {
        void run({
            action: async () => {
                await refresh();
                registry = await call<Capability[]>({ command: 'capabilities' });
                ready = true;
            },
        });
        const timer = setInterval(() => {
            if (!busy)
                void refresh().catch((cause) => {
                    error = errorText({ error: cause });
                });
        }, 2000);
        return () => clearInterval(timer);
    });
    const navigation = ['overview', 'wallets', 'accounts', 'transfer', 'requests', 'history', 'networks', 'settings', 'help'];
</script>

<svelte:window onpointerdown={markActivity} onkeydown={markActivity} />
{#if !snapshotView.unlocked}
    <main class="gate">
        <div class="gate-brand">
            <span class="logo">AV</span>
            <div>
                <h1>{l.title}</h1>
                <p>{l.subtitle}</p>
            </div>
        </div>
        <details><summary>{l.appearance}</summary><Appearance /></details>
        <div class="gate-card">
            <div class="eyebrow">{l.preview}</div>
            <h2>{snapshotView.initialized ? l.startup : l.setup}</h2>
            <p class="muted">{snapshotView.initialized ? l.passwordHelp : l.setupHint}</p>
            {#if error}<div class="alert danger" role="alert">{error}</div>{/if}
            {#if notice}<div class="alert mid" role="status">{notice}</div>{/if}
            <form
                onsubmit={(event) => {
                    event.preventDefault();
                    void authenticate();
                }}
            >
                {#if !snapshotView.initialized}<div class="mode-options">
                        {#each ['offline', 'proxy', 'combined'] as item}
                            <label class:selected={mode === item} class="mode-choice"
                                ><input
                                    type="radio"
                                    name="mode"
                                    value={item}
                                    bind:group={mode}
                                    disabled={!snapshotView.networkBuild && item !== 'offline'}
                                /><span><strong>{l[item]}</strong><small>{l[item + 'Help']}</small></span></label
                            >
                        {/each}
                    </div>{/if}
                <label
                    >{l.password}<input
                        type="password"
                        bind:value={password}
                        autocomplete="off"
                        required
                        minlength={snapshotView.initialized ? 1 : 12}
                        maxlength="1024"
                    /></label
                >
                {#if !snapshotView.initialized}<label
                        >{l.passwordConfirm}<input
                            type="password"
                            bind:value={passwordConfirm}
                            autocomplete="off"
                            required
                            minlength="12"
                            maxlength="1024"
                        /></label
                    >{/if}
                <button class="primary wide" disabled={busy || !ready}
                    >{busy ? l.working : snapshotView.initialized ? l.unlock : l.create}</button
                >
            </form>
            <details>
                <summary>{l.restore}</summary>
                <p>{l.restoreHelp}</p>
                <label class="check"><input type="checkbox" bind:checked={dryRun} />{l.dryRun}</label><button
                    disabled={busy || !password}
                    onclick={() => void restore()}>{l.restoreAction}</button
                >
            </details>
        </div>
        <details><summary>{l.help}</summary><Help /></details>
        <p class="footnote">{l.testing}</p>
    </main>
{:else}
    <div class="app-shell">
        <aside>
            <div class="brand">
                <span class="logo">AV</span>
                <div><strong>{l.title}</strong><small>0.1.0 · preview</small></div>
            </div>
            <div class="vault-state">
                <span class="status-dot"></span>{l.unlocked}<small>{l[snapshotView.settings?.mode ?? 'offline']}</small>
            </div>
            <nav aria-label={l.tools}>
                {#each navigation as item}{#if !(item === 'wallets' && snapshotView.settings?.mode === 'proxy') && !(item === 'transfer' && snapshotView.settings?.mode === 'offline')}
                        <button
                            class:active={page === item}
                            onclick={() => {
                                page = item;
                                error = '';
                                notice = '';
                            }}>{l[item]}</button
                        >
                    {/if}{/each}
            </nav>
            <div class="sidebar-bottom">
                <small>{l.bytesInfo}</small><button onclick={() => void run({ action: () => call({ command: 'lock_vault' }) })}
                    >{l.lock}</button
                >
            </div>
        </aside>
        <div class="workspace">
            <header>
                <div class="breadcrumb">{l.vault} / <strong>{l[page]}</strong></div>
                <div class="toolbar">
                    <span class:warning={snapshotView.settings?.mode === 'combined'} class="badge"
                        >{l[snapshotView.settings?.mode ?? 'offline']}</span
                    ><button onclick={() => void importFile()} disabled={busy}>{l.import}</button>
                </div>
            </header>
            <main class="content">
                <div class="page-title">
                    <div>
                        <div class="eyebrow">{l.title.toUpperCase()} / {l[page].toUpperCase()}</div>
                        <h1>{l[page]}</h1>
                    </div>
                    <span class="badge neutral">{l.preview}</span>
                </div>
                {#if error}<div class="alert danger" role="alert">{error}</div>{/if}
                {#if notice}<div class="alert mid" role="status">{notice}</div>{/if}
                {#if busy}<div class="working" role="status">{l.working}</div>{/if}
                {#if page === 'overview'}
                    <div class="alert" class:warning={snapshotView.settings?.mode === 'combined'}>
                        {snapshotView.settings?.mode === 'combined'
                            ? l.combinedNote
                            : snapshotView.settings?.mode === 'offline'
                              ? l.offlineNote
                              : l.proxyHelp}
                    </div>
                    <div class="stats">
                        <article>
                            <small>{l.walletCount}</small><strong>{snapshotView.roots.length.toString().padStart(2, '0')}</strong>
                        </article>
                        <article>
                            <small>{l.accountCount}</small><strong
                                >{snapshotView.accounts.length.toString().padStart(2, '0')}</strong
                            >
                        </article>
                        <article>
                            <small>{l.pendingCount}</small><strong
                                >{snapshotView.pending.length.toString().padStart(2, '0')}</strong
                            >
                        </article>
                    </div>
                    <section>
                        <h2>{l.workflow}</h2>
                        <div class="three-columns">
                            {#each [1, 2, 3] as step}<article class="step">
                                    <span class="step-number">0{step}</span>
                                    <h3>{l['step' + step]}</h3>
                                    <p>{l['step' + step + 'Help']}</p>
                                </article>{/each}
                        </div>
                    </section>
                    <section>
                        <h2>{l.recent}</h2>
                        {#if snapshotView.history.length === 0}<p class="empty">
                                {l.emptyHistory}
                            </p>{:else}{#each snapshotView.history.slice(-5).reverse() as entry}<div class="list-row">
                                    <span class="badge">{entry.action}</span><code>{entry.txid ?? entry.digest}</code><small
                                        >{new Date(entry.at * 1000).toLocaleString()}</small
                                    >
                                </div>{/each}{/if}
                    </section>
                    <p class="footnote">{l.testing}</p>
                {:else if page === 'wallets'}<Wallets {snapshotView} {busy} {run} />
                {:else if page === 'accounts'}<Accounts
                        {snapshotView}
                        {busy}
                        {run}
                        onexport={() => void exportFile({ kind: 'public' })}
                    />
                {:else if page === 'transfer'}<Transfer
                        {snapshotView}
                        {busy}
                        {run}
                        onprepared={() => {
                            page = 'requests';
                        }}
                    />
                {:else if page === 'requests'}
                    <p>{l.bytesInfo}</p>
                    {#if snapshotView.pending.length === 0}<section class="empty">{l.emptyRequests}</section>{/if}
                    {#each snapshotView.pending as request}
                        {@const response = snapshotView.responses.find((r) => r.requestId === request.requestId)}
                        <section>
                            <div class="section-heading">
                                <h2>{l.native}</h2>
                                <span class="badge">{request.network}</span>
                            </div>
                            <p>{l.context}: {request.description}</p>
                            <code class="wrap">{request.requestId}</code>
                            {#if response}<div class="alert mid">{l.response}<code class="wrap">{response.txid}</code></div>{/if}
                            <div class="actions">
                                <button class="primary" onclick={() => void openReview({ id: request.requestId })} disabled={busy}
                                    >{response ? l.viewResponse : l.review}</button
                                ><button
                                    onclick={() => void exportFile({ kind: 'request', id: request.requestId })}
                                    disabled={busy}>{l.exportRequest}</button
                                >{#if response}<button
                                        onclick={() => void exportFile({ kind: 'response', id: request.requestId })}
                                        disabled={busy}>{l.exportResponse}</button
                                    >{/if}
                            </div>
                        </section>
                    {/each}
                {:else if page === 'history'}
                    <section>
                        {#if !snapshotView.history.length}<p class="empty">{l.emptyHistory}</p>{:else}{#each snapshotView.history
                                .slice()
                                .reverse() as entry}<article class="history-entry">
                                    <div class="section-heading">
                                        <span class="badge">{entry.action}</span><small
                                            >{new Date(entry.at * 1000).toLocaleString()}</small
                                        >
                                    </div>
                                    <code class="wrap">{entry.requestId}</code><code class="wrap"
                                        >{entry.txid ?? entry.digest}</code
                                    >
                                </article>{/each}{/if}
                    </section>
                {:else if page === 'networks'}
                    <p>{l.supportHelp}</p>
                    <section class="table-wrap">
                        <table>
                            <thead
                                ><tr><th>{l.network}</th><th>{l.derivation}</th><th>{l.signing}</th><th>{l.messages}</th></tr
                                ></thead
                            ><tbody
                                >{#each registry as item}<tr
                                        ><td><strong>{item.network}</strong><small>{item.note}</small></td><td
                                            >{item.derivation}</td
                                        ><td>{item.signing}</td><td>{item.messages}</td></tr
                                    >{/each}</tbody
                            >
                        </table>
                    </section>
                {:else if page === 'settings'}
                    <SettingsView {snapshotView} {busy} {run} />
                    <section>
                        <h2>{l.appearance}</h2>
                        <Appearance />
                    </section>
                {:else if page === 'help'}<Help />
                {/if}
            </main>
        </div>
    </div>
{/if}
{#if review && snapshotView.unlocked}<RequestReview
        {error}
        {snapshotView}
        {review}
        {busy}
        {run}
        onclose={() => {
            review = null;
        }}
    />{/if}
