// What the Nexees window's backend tells its frontend about the Nexees host: the RPC contract
// between apps/desktop/src/main (backend) and apps/desktop/src/shell (frontend): the state of the
// window's attachment, and the panel layout the host keeps for the window (ST-VIEW).
// Nothing in it is authority: it is the window's view of a host it does not control.

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
/** What a view's ID must look like to be remembered: an identifier of core/domain/ids. */
const VIEW_ID = /^[A-Za-z0-9][A-Za-z0-9_.:-]{0,127}$/;

/** Whether `id` can name a view in a panel layout. */
export function isViewId(id: unknown): id is string {
    return typeof id === 'string' && VIEW_ID.test(id);
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
    if (selected !== null && !isViewId(selected)) {
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

/** The frontend's side: told about every change of the state. */
export interface HostConnectionClient {
    onStateChanged(state: HostState): void;
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
}
