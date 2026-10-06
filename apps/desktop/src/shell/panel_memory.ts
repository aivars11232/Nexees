// What the window remembers of its panels (apps/desktop/src/shell/panel_memory): the left
// sidebar, the right sidebar and the bottom panel come back as the user left them, each shown or
// hidden, in its size and with its selected view (ST-VIEW, R13, C5).
//
// - **The host keeps it.** The window gives its panel layout to the Nexees host, which keeps it
//   in the device's state store, and asks the host for it when it starts. So the panels survive
//   the window: closed, stopped unexpectedly, or started on a profile the foundation has stored
//   nothing in (FD-UI-CLIENT). It is one layout for the window; a layout per workspace comes
//   with the workspaces (TASK-016).
// - **Presentation only.** The layout says where panels stand and nothing else. It grants
//   nothing, targets nothing and is checked as input before it is applied (main.protocol).
// - **A change the host has not taken yet** is kept in the window's own profile and given to
//   the host at the next start, where it outranks what the host kept: the window may close, or
//   the host be unreachable, before a change arrives.
// - **The foundation keeps the rest.** Which view stands in which panel, and the open editors,
//   stay in Theia's own stored layout, which Theia writes to the page's storage when the window
//   closes. This module has it written whenever the panels settle too. The browser puts that
//   storage on disk when the window closes, and before that at the earliest five seconds after a
//   change and on average no more than once a minute since the page opened it. So after a window
//   that stops unexpectedly the panels are as the host kept them, but the rest of the layout can
//   be older, by a minute or so in the window's first minutes.

import { inject, injectable } from '@theia/core/shared/inversify';
import { MessageLoop } from '@theia/core/shared/@lumino/messaging';
import { Widget } from '@theia/core/shared/@lumino/widgets';
import { FrontendApplication } from '@theia/core/lib/browser/frontend-application';
import { FrontendApplicationContribution } from '@theia/core/lib/browser/frontend-application-contribution';
import { ApplicationShell } from '@theia/core/lib/browser/shell/application-shell';
import { ShellLayoutRestorer } from '@theia/core/lib/browser/shell/shell-layout-restorer';
import { SidePanelHandler } from '@theia/core/lib/browser/shell/side-panel-handler';
import { LocalStorageService } from '@theia/core/lib/browser/storage-service';
import { HostConnectionService, MAX_PANEL_SIZE, PanelLayout, PanelState, isId, panelLayout } from '../main.protocol';
import { NexeesShell } from './panel_layout';

/** How long the panels must stay as they are before they are stored: a drag is one change. */
const SETTLE_MS = 300;
/** How long the window waits at its start for the layout the host keeps, before it shows without it. */
const LOAD_WAIT_MS = 3000;
/** Where the window's profile keeps a layout the host has not taken yet. */
const UNSAVED = 'nexees.panels.unsaved';
/** The messages of a panel that mean its size or its visibility changed. */
const PANEL_CHANGES: ReadonlySet<string> = new Set(['resize', 'after-show', 'after-hide']);

/** `value`, or undefined when it does not come within `ms`. */
function within<T>(value: Promise<T>, ms: number): Promise<T | undefined> {
    return Promise.race([value, new Promise<undefined>(resolve => setTimeout(resolve, ms))]);
}

/** A measured size as the layout holds it: a whole number of pixels within the bounds, or null. */
function pixels(size: number | undefined): number | null {
    return size !== undefined && size >= 1 ? Math.min(Math.round(size), MAX_PANEL_SIZE) : null;
}

/** Restores the panels when the window starts and stores them as the user changes them. */
@injectable()
export class PanelMemory implements FrontendApplicationContribution {

    protected app: FrontendApplication | undefined;
    /** The last layout sampled, to keep what a hidden panel comes back with. */
    protected last: PanelLayout | undefined;
    /** The layout the host is known to keep, and the one the profile holds for it, as text. */
    protected kept: string | undefined;
    protected unsaved: string | undefined;
    protected settling: number | undefined;

    constructor(
        @inject(ApplicationShell) protected readonly shell: NexeesShell,
        @inject(HostConnectionService) protected readonly host: HostConnectionService,
        @inject(ShellLayoutRestorer) protected readonly restorer: ShellLayoutRestorer,
        @inject(LocalStorageService) protected readonly profile: LocalStorageService,
    ) { }

    /**
     * Runs before the window is revealed, after Theia restored its own layout or made the first
     * one: the panels are in place when the user first sees them.
     */
    async onDidInitializeLayout(app: FrontendApplication): Promise<void> {
        this.app = app;
        try {
            const unsaved = panelLayout(await this.profile.getData(UNSAVED));
            const layout = unsaved ?? await within(this.host.loadLayout(), LOAD_WAIT_MS);
            if (layout) {
                await this.apply(layout);
                this.last = layout;
            }
            if (unsaved) {
                this.unsaved = JSON.stringify(unsaved);
                void this.deliver(unsaved);
            } else if (layout) {
                this.kept = JSON.stringify(layout);
            }
        } catch (error) {
            // The window starts with the layout it has; what it remembers is a convenience.
            console.warn('The panel layout could not be restored.', error);
        }
        this.watch();
    }

    /** The window's last tick: a change that has not settled yet is kept for the next start. */
    onStop(): void {
        if (this.settling !== undefined) {
            window.clearTimeout(this.settling);
            this.store(false);
        }
    }

    /** Stores the panels once they have stayed unchanged for a moment, whatever changed them. */
    protected watch(): void {
        const changed = (): void => {
            window.clearTimeout(this.settling);
            this.settling = window.setTimeout(() => this.store(true), SETTLE_MS);
        };
        const { leftPanelHandler: left, rightPanelHandler: right, bottomPanel: bottom } = this.shell;
        for (const panel of [left.dockPanel, right.dockPanel, bottom]) {
            MessageLoop.installMessageHook(panel, (_, message) => {
                if (PANEL_CHANGES.has(message.type)) {
                    changed();
                }
                return true;
            });
        }
        // Selecting a view of a sidebar shows the sidebar, and selecting none hides it.
        left.tabBar.currentChanged.connect(changed);
        right.tabBar.currentChanged.connect(changed);
        this.shell.onDidChangeCurrentWidget(changed);
        this.shell.onDidAddWidget(changed);
        this.shell.onDidRemoveWidget(changed);
    }

    /**
     * Takes the panels as they are now. Theia's own layout is written with them when the window
     * goes on (`settled`); when it closes, Theia writes it itself.
     */
    protected store(settled: boolean): void {
        this.settling = undefined;
        if (settled && this.app) {
            this.restorer.storeLayout(this.app);
        }
        const layout = this.sample();
        if (!layout) {
            return;
        }
        this.last = layout;
        const text = JSON.stringify(layout);
        if (text === this.kept && this.unsaved === undefined) {
            return;
        }
        if (text !== this.unsaved) {
            this.unsaved = text;
            // The value is in the profile when this call returns; only its promise is pending.
            void this.profile.setData(UNSAVED, layout);
        }
        void this.deliver(layout);
    }

    /** Gives `layout` to the host; once the host has the newest one, the profile's copy goes. */
    protected async deliver(layout: PanelLayout): Promise<void> {
        const text = JSON.stringify(layout);
        if (!await this.host.storeLayout(layout).catch(() => false)) {
            return;
        }
        this.kept = text;
        if (this.unsaved === text) {
            this.unsaved = undefined;
            await this.profile.setData(UNSAVED, undefined);
        }
    }

    /** The panels now, or undefined while they cannot be measured. */
    protected sample(): PanelLayout | undefined {
        const bottom = this.shell.bottomPanelData();
        return panelLayout({
            left: this.sampleSide(this.shell.leftPanelHandler, this.last?.left),
            right: this.sampleSide(this.shell.rightPanelHandler, this.last?.right),
            bottom: this.panel(bottom.shown, bottom.size, this.shell.bottomPanel.currentTitle?.owner, this.last?.bottom),
        });
    }

    /**
     * A sidebar as Theia's own layout data describes it: its size, and the view it shows. A hidden
     * sidebar shows none; its view is then the one Theia noted as its last, which it comes back with.
     */
    protected sampleSide(handler: SidePanelHandler, before: PanelState | undefined): PanelState {
        const { items, size } = handler.getLayoutData();
        const selected = items?.find(item => item.expanded)?.widget;
        const last = handler.tabBar.titles[handler.state.lastActiveTabIndex ?? -1]?.owner;
        return this.panel(selected !== undefined, size, selected ?? last, before);
    }

    /**
     * One panel of the layout; `selected` is the view it shows or, hidden, comes back with. What a
     * hidden panel does not tell, its size or its view, it keeps as it had it `before`; a view
     * whose ID is no identifier is not remembered.
     */
    protected panel(shown: boolean, size: number | undefined, selected: Widget | undefined, before: PanelState | undefined): PanelState {
        const id = selected && isId(selected.id) ? selected.id : null;
        return {
            shown,
            size: pixels(size) ?? before?.size ?? null,
            selected: shown ? id : id ?? before?.selected ?? null,
        };
    }

    /**
     * Puts the panels as `layout` says, one after the other. A panel that is hidden while it is
     * given its size takes the size as the one to come back with, so a panel to be shown gets its
     * size first and a panel to be hidden gets it last.
     */
    protected async apply(layout: PanelLayout): Promise<void> {
        await this.applySide('left', layout.left);
        await this.applySide('right', layout.right);
        const view = this.shell.getWidgets('bottom').find(widget => widget.id === layout.bottom.selected);
        if (view) {
            this.shell.bottomPanel.selectWidget(view);
        }
        if (layout.bottom.shown) {
            this.resize(layout.bottom, 'bottom');
            this.shell.expandPanel('bottom');
        } else {
            await this.shell.collapsePanel('bottom');
            this.resize(layout.bottom, 'bottom');
        }
        await this.shell.bottomPanelUpdate();
    }

    protected async applySide(area: 'left' | 'right', panel: PanelState): Promise<void> {
        const handler = area === 'left' ? this.shell.leftPanelHandler : this.shell.rightPanelHandler;
        const title = handler.tabBar.titles.find(candidate => candidate.owner.id === panel.selected);
        if (panel.shown) {
            this.resize(panel, area);
            // The stored view, or the one Theia shows of itself when the stored one is gone.
            handler.expand(title?.owner.id);
        } else {
            await handler.collapse();
            this.resize(panel, area);
            if (title) {
                // The view the sidebar shows when the user brings it back.
                handler.state.lastActiveTabIndex = handler.tabBar.titles.indexOf(title);
            }
        }
        await handler.state.pendingUpdate;
    }

    protected resize(panel: PanelState, area: 'left' | 'right' | 'bottom'): void {
        if (panel.size !== null) {
            this.shell.resize(panel.size, area);
        }
    }
}
