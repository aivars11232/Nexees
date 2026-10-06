// What the Nexees window's backend tells its frontend about the Nexees host: the RPC contract
// between apps/desktop/src/main (backend) and apps/desktop/src (frontend): the state of the
// window's attachment, the panel layout the host keeps for the window (ST-VIEW), and the
// workspaces of the host's device (ST-WORKSPACE). Nothing in it is authority: it is the window's
// view of a host it does not control.

import type { Event } from '@theia/core/lib/common/event';

/** The RPC path of the host connection service, served by the window's own backend. */
export const HOST_CONNECTION_PATH = '/services/nexees/host';

/**
 * Where the window stands with the Nexees host:
 * - `attaching`: starting the host, or attaching to it;
 * - `attached`: the host answered with a compatible protocol version and its status, which names
 *   the device it runs on;
 * - `unavailable`: no host can be reached, for the reason given; nothing the window shows then
 *   comes from a host (RC-06).
 */
export type HostState =
    | { readonly kind: 'attaching' }
    | { readonly kind: 'attached'; readonly device: string; readonly protocol: number }
    | { readonly kind: 'unavailable'; readonly reason: string };

/**
 * One panel of the window as the user last left it (core/domain/client, `PanelState`):
 * - `shown`: whether the panel is on the screen;
 * - `size`: its size when it was last shown, in CSS pixels: the width of a sidebar, the height
 *   of the bottom panel; null only for a panel that has never been shown;
 * - `selected`: the ID of the view selected in it when it was last shown, or null.
 */
export interface PanelState {
    readonly shown: boolean;
    readonly size: number | null;
    readonly selected: string | null;
}

/** The window's three panels (core/domain/client, `PanelLayout`). */
export interface PanelLayout {
    readonly left: PanelState;
    readonly right: PanelState;
    readonly bottom: PanelState;
}

/** The largest size a panel is remembered with, in CSS pixels (core/domain/client). */
export const MAX_PANEL_SIZE = 16_384;
/** What an identifier of core/domain/ids looks like, such as a view's or a workspace's. */
const ID = /^[A-Za-z0-9][A-Za-z0-9_.:-]{0,127}$/;

/** Whether `id` is an identifier the host accepts: a view's in a panel layout, or a workspace's. */
export function isId(id: unknown): id is string {
    return typeof id === 'string' && ID.test(id);
}

function panelState(value: unknown): PanelState | undefined {
    if (typeof value !== 'object' || value === null) {
        return undefined;
    }
    const { shown, size, selected } = value as { shown?: unknown; size?: unknown; selected?: unknown };
    const sized = typeof size === 'number' && Number.isInteger(size) && size >= 1 && size <= MAX_PANEL_SIZE;
    if (typeof shown !== 'boolean' || !(sized || (size === null && !shown))) {
        return undefined;
    }
    if (selected !== null && !isId(selected)) {
        return undefined;
    }
    return { shown, size: sized ? size : null, selected };
}

/**
 * `value` as a panel layout, or undefined when it is none. The rules are the host's
 * (core/domain/client): each size is a whole number from 1 to MAX_PANEL_SIZE, a shown panel has
 * one, and a selected view is named by an identifier. The window checks a layout before it sends
 * it, because the host closes the channel on a message it refuses, and before it applies one,
 * because a stored layout is input. The result holds the known fields only.
 */
export function panelLayout(value: unknown): PanelLayout | undefined {
    if (typeof value !== 'object' || value === null) {
        return undefined;
    }
    const { left, right, bottom } = value as { left?: unknown; right?: unknown; bottom?: unknown };
    const panels = { left: panelState(left), right: panelState(right), bottom: panelState(bottom) };
    return panels.left && panels.right && panels.bottom ? { left: panels.left, right: panels.right, bottom: panels.bottom } : undefined;
}

/** A workspace's kind (core/domain/workspace). */
export type WorkspaceKind = 'code' | 'lcl';
/** Whether a workspace's content is on the host's device (core/domain/workspace). */
export type ContentAvailability = 'local' | 'remote_only' | 'synced_copy';

/** A workspace as the host tells the window about it (core/protocol, `WorkspaceEntry`). */
export interface WorkspaceEntry {
    readonly workspace_id: string;
    readonly kind: WorkspaceKind;
    readonly name: string;
    /** Its root on the host's device; null when only a listing of it is known there. */
    readonly root: string | null;
    readonly availability: ContentAvailability;
}

/** A workspace the window asks the host to create (core/protocol, `NewWorkspace`). */
export interface NewWorkspace {
    readonly kind: WorkspaceKind;
    readonly name: string;
    /** The folder that holds its content, as an absolute path. */
    readonly root: string;
}

/** The workspace the window asks to open: by its ID, or the one whose root is a folder. */
export type WorkspaceTarget = { readonly workspace_id: string } | { readonly root: string };

/** Why the host refused a workspace request (core/protocol, `WorkspaceRefusal`); nothing changed. */
export type WorkspaceRefusal = 'not_found' | 'root_not_absolute' | 'root_missing' | 'root_not_a_folder' | 'root_unusable'
    | 'root_moved' | 'overlaps' | 'not_here' | 'not_open';

/**
 * The host's answer to a workspace request (core/protocol, `WorkspaceAnswer`): a page of the
 * list, the request carried out, or refused. `foreground` is the workspace the window's client
 * shows.
 */
export type WorkspaceAnswer =
    | { readonly kind: 'page'; readonly entries: readonly WorkspaceEntry[]; readonly more: boolean; readonly foreground: string | null }
    | { readonly kind: 'done'; readonly workspace: WorkspaceEntry | null; readonly foreground: string | null }
    | { readonly kind: 'refused'; readonly reason: WorkspaceRefusal; readonly workspace_id: string | null };

/** Every workspace of the host's device, and the one the window's client shows. */
export interface WorkspaceList {
    readonly entries: readonly WorkspaceEntry[];
    readonly foreground: string | null;
}

const KINDS: readonly string[] = ['code', 'lcl'];
const AVAILABILITIES: readonly string[] = ['local', 'remote_only', 'synced_copy'];
const REFUSALS: readonly string[] = ['not_found', 'root_not_absolute', 'root_missing', 'root_not_a_folder', 'root_unusable',
    'root_moved', 'overlaps', 'not_here', 'not_open'];
/** A control character, as Rust's `char::is_control` counts them: the host's rule for names and roots. */
const CONTROL = /[\u0000-\u001f\u007f-\u009f]/;
/** The longest name and root the host's records hold, in bytes (core/domain). */
const MAX_NAME_BYTES = 256;
const MAX_ROOT_BYTES = 4096;

function bytes(text: string): number {
    return new TextEncoder().encode(text).length;
}

/** Whether `value` is a name a workspace can have: one line of at most MAX_NAME_BYTES bytes. */
export function isWorkspaceName(value: unknown): value is string {
    return typeof value === 'string' && value.trim().length > 0 && bytes(value) <= MAX_NAME_BYTES && !CONTROL.test(value);
}

function isRoot(value: unknown): value is string {
    return typeof value === 'string' && value.length > 0 && bytes(value) <= MAX_ROOT_BYTES && !CONTROL.test(value);
}

/** Whether `value` holds exactly the fields `names`, so that no other field rides along. */
function exactly(value: unknown, names: readonly string[]): value is Record<string, unknown> {
    return typeof value === 'object' && value !== null && !Array.isArray(value)
        && Object.keys(value).length === names.length && names.every(name => name in value);
}

function workspaceEntry(value: unknown): WorkspaceEntry | undefined {
    if (!exactly(value, ['workspace_id', 'kind', 'name', 'root', 'availability'])) {
        return undefined;
    }
    const { workspace_id, kind, name, root, availability } = value;
    return isId(workspace_id) && KINDS.includes(kind as string) && isWorkspaceName(name) && (root === null || isRoot(root))
        && AVAILABILITIES.includes(availability as string)
        ? { workspace_id, kind: kind as WorkspaceKind, name, root, availability: availability as ContentAvailability } : undefined;
}

/** `value` when it is an identifier or null; otherwise undefined. */
function idOrNull(value: unknown): string | null | undefined {
    return value === null || isId(value) ? value : undefined;
}

/**
 * The body of the host's `workspaces` message as an answer, or undefined when it is none. The
 * rules are the host's (core/protocol): known fields only, identifiers, names and roots within
 * their bounds. An answer is input, and is checked before the window uses it.
 */
export function workspaceAnswer(value: unknown): WorkspaceAnswer | undefined {
    const [kind, body] = (typeof value === 'object' && value !== null && Object.keys(value).length === 1
        ? Object.entries(value)[0] : undefined) ?? [];
    if (kind === 'page' && exactly(body, ['entries', 'more', 'foreground']) && Array.isArray(body.entries)
        && typeof body.more === 'boolean' && idOrNull(body.foreground) !== undefined) {
        const entries = body.entries.map(workspaceEntry);
        return entries.every(entry => entry !== undefined)
            ? { kind, entries: entries as WorkspaceEntry[], more: body.more, foreground: body.foreground as string | null } : undefined;
    }
    if (kind === 'done' && exactly(body, ['workspace', 'foreground']) && idOrNull(body.foreground) !== undefined) {
        const workspace = body.workspace === null ? null : workspaceEntry(body.workspace);
        return workspace !== undefined ? { kind, workspace, foreground: body.foreground as string | null } : undefined;
    }
    if (kind === 'refused' && exactly(body, ['reason', 'workspace_id']) && REFUSALS.includes(body.reason as string)
        && idOrNull(body.workspace_id) !== undefined) {
        return { kind, reason: body.reason as WorkspaceRefusal, workspace_id: body.workspace_id as string | null };
    }
    return undefined;
}

/** `value` as a request to create a workspace, with the known fields only, or undefined. */
export function newWorkspace(value: unknown): NewWorkspace | undefined {
    if (!exactly(value, ['kind', 'name', 'root'])) {
        return undefined;
    }
    const { kind, name, root } = value;
    return KINDS.includes(kind as string) && isWorkspaceName(name) && isRoot(root) ? { kind: kind as WorkspaceKind, name, root } : undefined;
}

/** `value` as a workspace to open, by ID or by root, or undefined. */
export function workspaceTarget(value: unknown): WorkspaceTarget | undefined {
    if (exactly(value, ['workspace_id'])) {
        return isId(value.workspace_id) ? { workspace_id: value.workspace_id } : undefined;
    }
    if (exactly(value, ['root'])) {
        return isRoot(value.root) ? { root: value.root } : undefined;
    }
    return undefined;
}

/** The frontend's side: told about every change of the state. */
export interface HostConnectionClient {
    onStateChanged(state: HostState): void;
}

/** The changes of the state, as the frontend passes them on to its own parts (apps/desktop/src/shell/main_window). */
export const HostStateEvents = Symbol('HostStateEvents');
export interface HostStateEvents {
    readonly onChanged: Event<HostState>;
}

/** The backend's side. */
export const HostConnectionService = Symbol('HostConnectionService');
export interface HostConnectionService {
    /** The state now. */
    getState(): Promise<HostState>;
    /**
     * Tries to attach again after the window gave up: the user's request. The window retries on
     * its own only a bounded number of times (RC-08).
     */
    retry(): Promise<HostState>;
    /**
     * The panel layout the host keeps for this window. Undefined when it keeps none, when no host
     * is attached once the first attempt to attach has ended, or when the host speaks a protocol
     * version without layouts.
     */
    loadLayout(): Promise<PanelLayout | undefined>;
    /**
     * Gives the host the window's panels as they are now, to keep in place of the layout kept
     * before. Resolves to whether the host has them. While no host is attached the backend holds
     * on to the newest layout and gives it to the next host that attaches.
     */
    storeLayout(layout: PanelLayout): Promise<boolean>;
    /**
     * Every workspace of the host's device and the one the window shows. Undefined when no host
     * is attached once the first attempt to attach has ended, or when it speaks a protocol
     * version without workspaces.
     */
    listWorkspaces(): Promise<WorkspaceList | undefined>;
    /** Asks the host to create a workspace; its answer, or undefined as for `listWorkspaces`. */
    createWorkspace(request: NewWorkspace): Promise<WorkspaceAnswer | undefined>;
    /**
     * Asks the host to make a workspace the one the window shows: its foreground workspace. It
     * binds no agent. The host's answer, or undefined as for `listWorkspaces`.
     */
    openWorkspace(target: WorkspaceTarget): Promise<WorkspaceAnswer | undefined>;
    /** Asks the host to close the workspace the window shows; its answer, or undefined. */
    closeWorkspace(workspaceId: string): Promise<WorkspaceAnswer | undefined>;
}
