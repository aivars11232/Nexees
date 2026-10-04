// What the Nexees window's backend tells its frontend about the Nexees host: the RPC contract
// between apps/desktop/src/main (backend) and apps/desktop/src/shell/main_window (frontend).
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
}
