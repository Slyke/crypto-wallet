import { test, expect, type Page, type APIRequestContext } from '@playwright/test';
const bridge = async ({ request, command, args = {} }: { request: APIRequestContext; command: string; args?: object }) => {
    const response = await request.post('/__airvault_test', { data: { command, args } });
    const body = await response.json();
    expect(body.ok).toBe(true);
    return body.value;
};
const create = async ({ page }: { page: Page }) => {
    await page.getByLabel('Vault password', { exact: true }).fill('public-test-password');
    await page.getByLabel('Confirm password', { exact: true }).fill('public-test-password');
    await page.getByRole('button', { name: 'Create vault', exact: true }).click();
    await expect(page.getByRole('button', { name: 'Lock vault' })).toBeVisible();
};
test.beforeEach(async ({ page, request }) => {
    await bridge({ request, command: 'test_reset' });
    await page.goto('/tests/ui/index.html');
});
test('three styleguide palettes, both modes, persistence and offline instructions', async ({ page }) => {
    await page.getByText('Appearance', { exact: true }).click();
    for (const palette of ['terminal', 'paper', 'ocean']) {
        await page.getByRole('combobox', { name: 'Theme', exact: true }).selectOption(palette);
        for (const mode of ['light', 'dark']) {
            await page.getByRole('combobox', { name: 'Mode', exact: true }).selectOption(mode);
            await expect(page.locator('html')).toHaveAttribute('data-palette', palette);
            await expect(page.locator('html')).toHaveAttribute('data-theme', mode);
        }
    }
    await page.reload();
    await expect(page.locator('html')).toHaveAttribute('data-palette', 'ocean');
    await page.getByText('Instructions', { exact: true }).click();
    await expect(page.getByRole('heading', { name: 'How to verify your dice-generated BIP39 words manually' })).toBeVisible();
    await page.getByRole('button', { name: 'Advanced instructions', exact: true }).click();
    await expect(page.getByText(/This preview signs single-key/)).toBeVisible();
    await page.getByText('Bundled offline wordlist', { exact: true }).click();
    await page.getByLabel('Find a word or printed position').fill('2048');
    await expect(page.getByRole('cell', { name: 'zoo', exact: true })).toBeVisible();
});
test('dice prefix review, undo, reset, one-time phrase and clearing on confirmation', async ({ page }) => {
    await create({ page });
    await page.getByRole('button', { name: 'Wallets & identities', exact: true }).click();
    await page.getByLabel('Wallet label', { exact: true }).fill('Disposable dice wallet');
    await page.getByLabel('Manual prefix words', { exact: true }).fill('1');
    await page.getByLabel('Manual prefix words', { exact: true }).blur();
    await page.getByRole('button', { name: 'Roll 6', exact: true }).click();
    await page.getByRole('button', { name: 'Undo last roll', exact: true }).click();
    await expect(page.getByRole('button', { name: 'Undo last roll', exact: true })).toBeDisabled();
    await page.getByRole('button', { name: 'Roll 2', exact: true }).click();
    await page.getByRole('button', { name: 'Reset current word', exact: true }).click();
    for (let i = 0; i < 10; i++) await page.getByRole('button', { name: 'Roll 1', exact: true }).click();
    await page.getByRole('button', { name: 'Roll 6', exact: true }).click();
    await page.getByText('(i) How was this word calculated?', { exact: true }).click();
    await expect(page.locator('.dice-entry pre')).toContainText('Zero-based BIP39 index: 1');
    await expect(page.locator('.dice-entry pre')).toContainText('BIP39 English word: ability');
    await page.getByRole('button', { name: 'Add this word', exact: true }).click();
    await page.getByLabel('I verified this word', { exact: true }).check();
    await expect(page.getByText('1 of 1 manually generated words verified.', { exact: true })).toBeVisible();
    await page.getByRole('button', { name: 'Generate recovery phrase', exact: true }).click();
    await expect(page.locator('.phrase-grid li')).toHaveCount(24);
    const words = await page.locator('.phrase-grid li').allTextContents();
    const phrase = words.map((word) => word.replace(/^\d+/, '').trim()).join(' ');
    expect(phrase.split(' ')).toHaveLength(24);
    expect(phrase.startsWith('ability ')).toBe(true);
    await page.getByRole('button', { name: 'I recorded it — hide and verify', exact: true }).click();
    await expect(page.locator('.phrase-grid')).toHaveCount(0);
    await expect(page.locator('.dice-review')).toBeVisible();
    await page.getByLabel('Re-enter all recovery words', { exact: true }).fill(phrase);
    await page.getByRole('button', { name: 'Confirm and save wallet', exact: true }).click();
    await expect(page.getByRole('heading', { name: 'Disposable dice wallet', exact: true })).toBeVisible();
    await expect(page.locator('.dice-review')).toHaveCount(1); // Help contains the public wordlist; actual entry review is gone.
    await expect(page.getByLabel('I verified this word', { exact: true })).toHaveCount(0);
    expect(await page.evaluate(() => JSON.stringify(localStorage))).not.toContain(phrase);
    await page.getByRole('button', { name: 'Lock vault', exact: true }).click();
    await expect(page.getByRole('heading', { name: 'Open your vault' })).toBeVisible();
    await expect(page.getByRole('heading', { name: 'Disposable dice wallet', exact: true })).toHaveCount(0);
});
test('all physical dice bits create a checksummed 12-word phrase', async ({ page }) => {
    await create({ page });
    await page.getByRole('button', { name: 'Wallets & identities', exact: true }).click();
    await page.getByLabel('Wallet label', { exact: true }).fill('Raw dice test');
    await page.getByRole('combobox', { name: 'Phrase length', exact: true }).selectOption('12');
    await page.getByRole('combobox', { name: 'Entropy source', exact: true }).selectOption('raw');
    for (let i = 0; i < 128; i++) await page.getByRole('button', { name: 'Roll 3', exact: true }).click();
    await page.getByRole('button', { name: 'Generate recovery phrase', exact: true }).click();
    await expect(page.locator('.phrase-grid li').last()).toHaveText('12about');
    await page.getByRole('button', { name: 'Cancel', exact: true }).click();
    await expect(page.locator('.phrase-grid')).toHaveCount(0);
    await expect(page.getByText('0 / 128 rolls entered', { exact: true })).toBeVisible();
});
test('actual native signing requires review, password and explicit checkbox', async ({ page, request }) => {
    await bridge({ request, command: 'test_seed' });
    await bridge({ request, command: 'test_fund' });
    await page.reload();
    await page.getByRole('button', { name: 'Signing requests', exact: true }).click();
    await page.getByRole('button', { name: 'Review exact transaction', exact: true }).click();
    const dialog = page.getByRole('dialog');
    await expect(dialog).toContainText('40,000');
    await expect(dialog.getByRole('button', { name: 'Approve & sign' })).toBeDisabled();
    await dialog.getByLabel('Vault password', { exact: true }).fill('wrong-password');
    await dialog.getByLabel('I verified the network, every recipient, the amounts, and the fee.', { exact: true }).check();
    await dialog.getByRole('button', { name: 'Approve & sign', exact: true }).click();
    await expect(dialog.getByRole('alert')).toBeVisible();
    let snapshot = await bridge({ request, command: 'snapshot' });
    expect(snapshot.responses).toHaveLength(0);
    await dialog.getByRole('button', { name: 'Close review', exact: true }).click();
    await page.getByRole('button', { name: 'Review exact transaction', exact: true }).click();
    await dialog.getByLabel('Vault password', { exact: true }).fill('public-test-password');
    await dialog.getByLabel('I verified the network, every recipient, the amounts, and the fee.', { exact: true }).check();
    await dialog.getByRole('button', { name: 'Approve & sign', exact: true }).click();
    await expect(page.getByText('Signed and independently verified.', { exact: true })).toBeVisible();
    snapshot = await bridge({ request, command: 'snapshot' });
    expect(snapshot.responses).toHaveLength(1);
    expect(snapshot.history[0].action).toBe('signed');
    await page.getByRole('button', { name: 'Lock vault', exact: true }).click();
    snapshot = await bridge({ request, command: 'snapshot' });
    expect(snapshot.accounts).toHaveLength(0);
});
