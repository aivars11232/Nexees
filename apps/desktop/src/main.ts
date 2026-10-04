// The window's side of the Nexees host (apps/desktop/src/main; RC-04, RC-06, RC-08, RC-23).
//
// The window runs no executor of its own. When it starts, its backend asks the platform
// lifecycle to make sure the host runs (`nexees-host start`, which starts one only when none
// answers) and then attaches to it over the local channel: a Unix socket in a folder private to
// the user. Before connecting, it checks that the folder and the socket belong to this user and
// are closed to everyone else; the host checks this process's user by the kernel's account.
//
// Messages are frames of four bytes of big-endian length followed by JSON, at most 4096 bytes,
// as core/protocol defines them. The window opens with its hello, keeps the host only when the
// two share a protocol version, and resyncs to the host's status.
//
// When the channel closes, the window tries again a bounded number of times with growing pauses,
// then shows the host as unavailable until the user asks again (RC-06, RC-08). When the window
// leaves, it detaches by closing the channel; whether the host then stops is the host's decision
// (RC-04).
//
// This backend serves only the window that started it (TH-02, SI-28). Theia's backend compares a
// per-launch token on every request, but admits every request when it was started without one,
// so this module refuses to load without the window's token, and the backend then stops before
// it listens. Once Theia's validator holds the token, the variable that carried it leaves this
// process's environment, so that no terminal or other child process inherits it.

import { ContainerModule, inject, injectable } from '@theia/core/shared/inversify';
import { BackendApplicationContribution } from '@theia/core/lib/node/backend-application';
import { ConnectionHandler, RpcConnectionHandler } from '@theia/core/lib/common/messaging';
import { ElectronSecurityToken } from '@theia/core/lib/electron-common/electron-token';
import { ElectronTokenValidator } from '@theia/core/lib/electron-node/token/electron-token-validator';
import { execFile } from 'node:child_process';
import * as fs from 'node:fs';
import * as net from 'node:net';
import * as path from 'node:path';
import { HOST_CONNECTION_PATH, HostConnectionClient, HostConnectionService, HostState } from './main.protocol';

/** The largest message on the local channel, in bytes (`binding.sec_max_ipc_message`). */
const MAX_MESSAGE_BYTES = 4096;
/** The protocol versions this window speaks, and the minimum secure one (core/protocol). */
const VERSIONS = { min: 1, max: 1 };
const MIN_SECURE_VERSION = 1;
/** The pauses before each new attempt after the channel closed; there are no more (RC-08). */
const RETRY_PAUSES_MS = [500, 1000, 2000, 4000, 8000];
/** How long `nexees-host start` may take, and how long the host has for each answer. */
const START_TIMEOUT_MS = 20_000;
const ANSWER_TIMEOUT_MS = 5_000;

/** Why `nexees-host start` failed, by its exit code (apps/desktop/src/application_host). */
const START_FAILURES: Readonly<Record<number, string>> = {
    4: 'this session gives the host no channel folder private to you; the host runs only inside a login session',
    5: 'the host refuses to run with root privileges',
    6: 'the host could not open its state store',
    7: 'the host did not answer in time',
};

/** A reason the host cannot be reached, worded for the user. */
class Unavailable extends Error { }

/**
 * Fails unless the window started this backend with its token. Theia's validator takes a missing
 * token for a backend without a local window and then admits every request, so a backend started
 * any other way must not come up at all (TH-02).
 */
function requireWindowToken(): void {
    let value: unknown;
    try {
        value = (JSON.parse(process.env[ElectronSecurityToken] ?? 'null') as { value?: unknown } | null)?.value;
    } catch {
        value = undefined;
    }
    if (typeof value !== 'string' || value.length === 0) {
        throw new Error('The Nexees backend serves only the window that started it; it was started without that window\'s token.');
    }
}

/**
 * Keeps the window's token out of every child process (SI-28): terminals and tools inherit this
 * backend's environment. Theia reads the variable again only for requests from the backend to
 * itself, which this shell does not make; without it such a request is refused.
 */
@injectable()
export class WindowToken implements BackendApplicationContribution {

    constructor(
        // Resolved first, so that Theia's validator has read the token before the variable goes.
        @inject(ElectronTokenValidator) protected readonly validator: ElectronTokenValidator,
    ) { }

    initialize(): void {
        delete process.env[ElectronSecurityToken];
    }
}

/** The window's attachment to the host, shared by every frontend connection of this window. */
@injectable()
export class HostAttachment implements BackendApplicationContribution {

    protected state: HostState = { kind: 'attaching' };
    protected readonly listeners = new Set<(state: HostState) => void>();
    protected socket: net.Socket | undefined;
    protected retries = 0;
    protected stopped = false;

    onStart(): void {
        // Not in initialize(): the window's backend does not wait for the host to start.
        void this.attach();
    }

    onStop(): void {
        this.stopped = true;
        this.socket?.destroy();
    }

    getState(): HostState {
        return this.state;
    }

    /** Calls `listener` on every change until the returned function is called. */
    subscribe(listener: (state: HostState) => void): () => void {
        this.listeners.add(listener);
        return () => this.listeners.delete(listener);
    }

    async retry(): Promise<HostState> {
        if (this.state.kind === 'unavailable') {
            this.retries = 0;
            await this.attach();
        }
        return this.state;
    }

    protected setState(state: HostState): void {
        this.state = state;
        for (const listener of this.listeners) {
            listener(state);
        }
    }

    protected async attach(): Promise<void> {
        this.setState({ kind: 'attaching' });
        try {
            await startHost(hostProgram());
            const socket = await connect(path.join(channelFolder(), 'host.sock'));
            const { device, protocol } = await handshake(socket);
            if (this.stopped) {
                socket.destroy();
                return;
            }
            this.socket = socket;
            this.retries = 0;
            socket.once('close', () => this.lost('the channel to the host closed'));
            this.setState({ kind: 'attached', device, protocol });
        } catch (error) {
            this.lost(error instanceof Unavailable ? error.message : `the host could not be reached: ${String(error)}`);
        }
    }

    /** After a failed or closed attachment: try again after the next pause, or give up. */
    protected lost(reason: string): void {
        this.socket = undefined;
        if (this.stopped) {
            return;
        }
        const pause = RETRY_PAUSES_MS[this.retries];
        if (pause === undefined) {
            this.setState({ kind: 'unavailable', reason });
            return;
        }
        this.retries += 1;
        this.setState({ kind: 'attaching' });
        setTimeout(() => void this.attach(), pause);
    }
}

/** The host program, which the installed launcher names; it must be an absolute path to a file. */
function hostProgram(): string {
    const program = process.env.NEXEES_HOST_PROGRAM;
    if (!program || !path.isAbsolute(program) || !fs.statSync(program, { throwIfNoEntry: false })?.isFile()) {
        throw new Unavailable('the Nexees host program is not installed beside this window');
    }
    return program;
}

/** Runs `nexees-host start`, which returns once a host answers; refused with the reason it gives. */
function startHost(program: string): Promise<void> {
    return new Promise((resolve, reject) => {
        execFile(program, ['start'], { timeout: START_TIMEOUT_MS }, error => {
            if (!error) {
                resolve();
                return;
            }
            const code = typeof error.code === 'number' ? error.code : undefined;
            const known = code === undefined ? undefined : START_FAILURES[code];
            reject(new Unavailable(known ?? `nexees-host start failed (${code ?? error.message})`));
        });
    });
}

/** The channel folder, checked to be this user's and closed to everyone else. */
function channelFolder(): string {
    const runtime = process.env.XDG_RUNTIME_DIR;
    if (!runtime || !path.isAbsolute(runtime)) {
        throw new Unavailable('this session has no runtime folder; the host runs only inside a login session');
    }
    const folder = path.join(runtime, 'nexees');
    requirePrivate(folder, stat => stat.isDirectory());
    return folder;
}

/** Fails unless `target` is, by `isKind`, the right kind of entry, owned by this user and closed to others. */
function requirePrivate(target: string, isKind: (stat: fs.Stats) => boolean): void {
    const stat = fs.lstatSync(target, { throwIfNoEntry: false });
    if (!stat || !isKind(stat) || stat.uid !== process.getuid?.() || (stat.mode & 0o077) !== 0) {
        throw new Unavailable(`${target} is not this user's private channel`);
    }
}

/** Connects to the host's socket after checking it. */
function connect(socketPath: string): Promise<net.Socket> {
    requirePrivate(socketPath, stat => stat.isSocket());
    return new Promise((resolve, reject) => {
        const socket = net.createConnection({ path: socketPath });
        socket.once('connect', () => {
            socket.removeAllListeners('error');
            resolve(socket);
        });
        socket.once('error', reject);
    });
}

/** A socket's chunk as bytes: a socket without an encoding delivers bytes, but its type allows text. */
function bytes(chunk: string | Buffer): Buffer {
    return typeof chunk === 'string' ? Buffer.from(chunk, 'utf8') : chunk;
}

/** One message as a frame: four bytes of big-endian length, then the JSON. */
export function frame(message: unknown): Buffer {
    const body = Buffer.from(JSON.stringify(message), 'utf8');
    if (body.length > MAX_MESSAGE_BYTES) {
        throw new Unavailable(`a message of ${body.length} bytes exceeds the channel's limit`);
    }
    const header = Buffer.alloc(4);
    header.writeUInt32BE(body.length);
    return Buffer.concat([header, body]);
}

/**
 * Splits a byte stream into messages. A frame longer than the limit or a body that is not JSON
 * is a fault, reported once; after a fault nothing more is read.
 */
export class FrameReader {

    protected buffer = Buffer.alloc(0);
    protected failed = false;

    constructor(
        protected readonly onMessage: (message: unknown) => void,
        protected readonly onFault: (reason: string) => void,
    ) { }

    push(chunk: Buffer): void {
        if (this.failed) {
            return;
        }
        this.buffer = Buffer.concat([this.buffer, chunk]);
        while (this.buffer.length >= 4) {
            const size = this.buffer.readUInt32BE(0);
            if (size > MAX_MESSAGE_BYTES) {
                return this.fail(`a frame of ${size} bytes exceeds the channel's limit`);
            }
            if (this.buffer.length < 4 + size) {
                return;
            }
            const body = this.buffer.subarray(4, 4 + size).toString('utf8');
            this.buffer = this.buffer.subarray(4 + size);
            let message: unknown;
            try {
                message = JSON.parse(body);
            } catch {
                return this.fail('a frame is not valid JSON');
            }
            this.onMessage(message);
        }
    }

    protected fail(reason: string): void {
        this.failed = true;
        this.onFault(reason);
    }
}

/** The single key of a message object, and its value; undefined for anything else. */
export function variant(message: unknown): [string, unknown] | undefined {
    if (typeof message !== 'object' || message === null || Array.isArray(message)) {
        return undefined;
    }
    const entries = Object.entries(message);
    return entries.length === 1 ? entries[0] : undefined;
}

/** The newest protocol version both sides speak, when it is at least the minimum secure one. */
export function negotiate(theirs: unknown): number | undefined {
    if (typeof theirs !== 'object' || theirs === null) {
        return undefined;
    }
    const { min, max } = theirs as { min?: unknown; max?: unknown };
    if (!Number.isInteger(min) || !Number.isInteger(max)) {
        return undefined;
    }
    const newest = Math.min(VERSIONS.max, max as number);
    return newest >= Math.max(VERSIONS.min, min as number) && newest >= MIN_SECURE_VERSION ? newest : undefined;
}

/**
 * The hellos and the first resync. Resolves with the host's device and the protocol version, or
 * rejects, closing the socket, when the host speaks no common version, sends anything else or
 * does not answer in time.
 */
function handshake(socket: net.Socket): Promise<{ device: string; protocol: number }> {
    return new Promise((resolve, reject) => {
        let protocol: number | undefined;
        const fail = (reason: string) => {
            clearTimeout(timer);
            socket.destroy();
            reject(new Unavailable(reason));
        };
        const timer = setTimeout(() => fail('the host did not answer in time'), ANSWER_TIMEOUT_MS);
        const reader = new FrameReader(message => {
            const [kind, body] = variant(message) ?? [];
            if (protocol === undefined) {
                const versions = kind === 'hello' ? (body as { versions?: unknown }).versions : undefined;
                protocol = negotiate(versions);
                if (protocol === undefined) {
                    return fail('the host speaks another protocol version; update Nexees');
                }
                socket.write(frame({ resync: { after: null } }));
                return;
            }
            const device = kind === 'status' ? (body as { host_device_id?: unknown }).host_device_id : undefined;
            if (typeof device !== 'string') {
                return fail('the host answered the resync with something other than its status');
            }
            clearTimeout(timer);
            socket.removeAllListeners('data');
            socket.removeAllListeners('error');
            socket.off('close', closed);
            // Attached: later messages are checked as frames; the window shows no events yet.
            const later = new FrameReader(() => undefined, () => socket.destroy());
            socket.on('data', chunk => later.push(bytes(chunk)));
            socket.on('error', () => socket.destroy());
            resolve({ device, protocol });
        }, fail);
        const closed = () => fail('the host closed the channel');
        socket.on('data', chunk => reader.push(bytes(chunk)));
        socket.once('error', error => fail(`the channel failed: ${error.message}`));
        socket.once('close', closed);
        socket.write(frame({ hello: { versions: VERSIONS } }));
    });
}

export default new ContainerModule(bind => {
    requireWindowToken();
    bind(WindowToken).toSelf().inSingletonScope();
    bind(BackendApplicationContribution).toService(WindowToken);
    bind(HostAttachment).toSelf().inSingletonScope();
    bind(BackendApplicationContribution).toService(HostAttachment);
    bind(ConnectionHandler).toDynamicValue(context => new RpcConnectionHandler<HostConnectionClient>(HOST_CONNECTION_PATH, client => {
        const attachment = context.container.get(HostAttachment);
        // Each frontend connection hears every change until it closes.
        const unsubscribe = attachment.subscribe(state => client.onStateChanged(state));
        client.onDidCloseConnection(() => unsubscribe());
        const service: HostConnectionService = {
            getState: async () => attachment.getState(),
            retry: () => attachment.retry(),
        };
        return service;
    })).inSingletonScope();
});
