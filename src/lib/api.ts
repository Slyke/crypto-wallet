import { invoke } from '@tauri-apps/api/core';
export type Mode = 'offline' | 'proxy' | 'combined';
export type Network = 'bitcoin' | 'testnet' | 'testnet4' | 'signet' | 'regtest';
export interface Root {
    id: string;
    label: string;
    kind: string;
    customPassword: boolean;
    createdAt: number;
}
export interface Identity {
    id: string;
    rootId: string;
    label: string;
    passphraseRecord: string | null;
}
export interface Account {
    id: string;
    identityId: string | null;
    label: string;
    network: Network;
    path: string;
    address: string;
    publicKey: string;
    fingerprint: string;
    watchOnly: boolean;
}
export interface Request {
    requestId: string;
    network: Network;
    payloadSha256: string;
    description: string;
    payload: { encoding: string; data: string };
}
export interface Response {
    requestId: string;
    txid: string;
    requestDigest: string;
}
export interface Settings {
    mode: Mode;
    timeoutSeconds: number;
    lockOnBlur: boolean;
    rpcUrl: string;
}
export interface History {
    id: string;
    requestId: string;
    action: string;
    at: number;
    digest: string;
    txid: string | null;
}
export interface Snapshot {
    initialized: boolean;
    unlocked: boolean;
    networkBuild: boolean;
    settings: Settings | null;
    roots: Root[];
    identities: Identity[];
    accounts: Account[];
    history: History[];
    pending: Request[];
    responses: Response[];
}
export interface Inspection {
    requestId: string;
    digest: string;
    network: Network;
    inputs: { outpoint: string; address: string; satoshis: number; accountIds: string[]; sequence: number; sighash: string }[];
    outputs: { address: string; satoshis: number; knownAccount: boolean }[];
    feeSats: number;
    feeRate: number;
    totalInputSats: number;
    lockTime: number;
    version: number;
}
export interface Review {
    token: string;
    inspection: Inspection;
}
export interface Capability {
    network: string;
    derivation: string;
    signing: string;
    messages: string;
    note: string;
}
export const emptySnapshot: Snapshot = {
    initialized: false,
    unlocked: false,
    networkBuild: true,
    settings: null,
    roots: [],
    identities: [],
    accounts: [],
    history: [],
    pending: [],
    responses: [],
};
// Error ownership belongs to the view's run() boundary. Never log IPC parameters.
export const call = async <T>({ command, args = {} }: { command: string; args?: Record<string, unknown> }): Promise<T> =>
    invoke<T>(command, args);
export const errorText = ({ error }: { error: unknown }): string => {
    if (error && typeof error === 'object' && 'message' in error) return String(error.message);
    if (error && typeof error === 'object' && 'code' in error) return String(error.code);
    return 'The operation failed. Check your input and retry.';
};
export const satoshis = ({ value }: { value: string }): number => {
    if (!/^(0|[1-9][0-9]*)$/.test(value)) throw new Error('Enter a whole number of satoshis.');
    const amount = Number(value);
    if (!Number.isSafeInteger(amount) || amount > 2100000000000000) throw new Error('Amount is outside the Bitcoin range.');
    return amount;
};
