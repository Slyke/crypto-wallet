<script lang="ts">
    import { untrack } from 'svelte';
    import { save } from '@tauri-apps/plugin-dialog';
    import { call, type Snapshot, type Settings } from './api';
    import { l } from './i18n';
    let {
        snapshotView,
        busy,
        run,
    }: {
        snapshotView: Snapshot;
        busy: boolean;
        run: (p: { action: () => Promise<unknown>; success?: string }) => Promise<void>;
    } = $props();
    let settings = $state<Settings>(untrack(() => ({ ...snapshotView.settings! })));
    let password = $state(''),
        confirmed = $state(false),
        backupConfirmed = $state(false),
        oldPassword = $state(''),
        newPassword = $state(''),
        confirmation = $state('');
    const saveSettings = () =>
        run({
            action: async () => {
                try {
                    await call({ command: 'save_settings', args: { settings, password, confirmed } });
                } finally {
                    password = '';
                    confirmed = false;
                }
            },
            success: l.saved,
        });
    const backup = () =>
        run({
            action: async () => {
                try {
                    const path = await save({
                        defaultPath: 'airvault-backup.avvault',
                        filters: [{ name: 'Encrypted AirVault backup', extensions: ['avvault'] }],
                    });
                    if (path) await call({ command: 'create_backup', args: { path, password, confirmed: backupConfirmed } });
                } finally {
                    password = '';
                    backupConfirmed = false;
                }
            },
            success: l.backupSaved,
        });
    const changePassword = () =>
        run({
            action: async () => {
                try {
                    if (newPassword !== confirmation) throw new Error(l.wrongConfirmation);
                    await call({ command: 'change_password', args: { oldPassword, newPassword } });
                } finally {
                    oldPassword = '';
                    newPassword = '';
                    confirmation = '';
                }
            },
            success: l.saved,
        });
</script>

<div class="two-columns align-start">
    <div>
        <section>
            <h2>{l.security}</h2>
            <label
                >{l.mode}<select bind:value={settings.mode}
                    ><option value="offline">{l.offline}</option><option
                        value="proxy"
                        disabled={!snapshotView.networkBuild || snapshotView.roots.length > 0}>{l.proxy}</option
                    ><option value="combined" disabled={!snapshotView.networkBuild}>{l.combined}</option></select
                ></label
            >
            <p class="hint">{settings.mode === 'combined' ? l.combinedNote : l.offlineNote}</p>
            <label>{l.timeout}<input type="number" min="30" max="3600" bind:value={settings.timeoutSeconds} /></label><label
                class="check"><input type="checkbox" bind:checked={settings.lockOnBlur} />{l.blur}</label
            ><label>{l.provider}<input type="url" bind:value={settings.rpcUrl} placeholder="https://…" /></label>
            <p class="hint">{l.providerHelp}</p>
            <label>{l.password}<input type="password" bind:value={password} autocomplete="off" /></label><label class="check"
                ><input type="checkbox" bind:checked={confirmed} />{l.settingsConfirm}</label
            ><button class="primary" disabled={busy || !confirmed || !password} onclick={() => void saveSettings()}
                >{l.save}</button
            >
        </section>
        <section>
            <h2>{l.mcp}</h2>
            <p>{l.mcpDisabled}</p>
        </section>
    </div>
    <div>
        <section>
            <h2>{l.backup}</h2>
            <p>{l.backupHelp}</p>
            <label>{l.password}<input type="password" bind:value={password} autocomplete="off" /></label><label class="check"
                ><input type="checkbox" bind:checked={backupConfirmed} />{l.backupCheck}</label
            ><button disabled={busy || !backupConfirmed || !password} onclick={() => void backup()}>{l.backupAction}</button>
        </section>
        <section>
            <h2>{l.changePassword}</h2>
            <label>{l.oldPassword}<input type="password" bind:value={oldPassword} autocomplete="off" /></label><label
                >{l.newPassword}<input type="password" bind:value={newPassword} autocomplete="off" minlength="12" /></label
            ><label>{l.passwordConfirm}<input type="password" bind:value={confirmation} autocomplete="off" /></label><button
                disabled={busy || newPassword.length < 12 || !oldPassword || newPassword !== confirmation}
                onclick={() => void changePassword()}>{l.changePassword}</button
            >
        </section>
    </div>
</div>
