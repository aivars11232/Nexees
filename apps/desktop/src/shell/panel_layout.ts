// The arrangement of the window's regions (apps/desktop/src/shell/panel_layout): where the
// approved layout puts the left sidebar, the editor, the right sidebar and the bottom panel, the
// share of the window each takes, and what a new profile shows first (Desktop UI contract,
// "Required regions" and "Compactness").
//
// Every region is Theia's own panel. This file only arranges them, through the three places
// Theia offers for that: the method that assembles the shell's layout, the shell's options, and
// the default layout a contribution gives a profile that has none stored. Theia stores the layout
// the user leaves and restores it; what Nexees keeps of it, and where, is the subject of TASK-012.

import { inject, injectable } from '@theia/core/shared/inversify';
import { Layout } from '@theia/core/shared/@lumino/widgets';
import { animationFrame } from '@theia/core/lib/browser/browser';
import { FrontendApplicationContribution } from '@theia/core/lib/browser/frontend-application-contribution';
import { ApplicationShell } from '@theia/core/lib/browser/shell/application-shell';
import { TheiaSplitPanel } from '@theia/core/lib/browser/shell/theia-split-panel';
import { TerminalWidget } from '@theia/terminal/lib/browser/base/terminal-widget';
import { TOKENS } from './design_tokens';
import { RightSidebar } from './right_sidebar';

/**
 * The share each panel takes when it is first shown, as Theia's shell options. A share is of the
 * space the arrangement below gives the panel's split: the left sidebar, with its activity bar,
 * takes its share of the window's width; the right sidebar of the width beside the left one; and
 * the bottom panel of the height between the title row and the status bar.
 */
export const PANEL_SHARES = {
    leftPanel: { initialSizeRatio: TOKENS.desktop_percent.left_sidebar_width / 100 },
    rightPanel: { initialSizeRatio: TOKENS.desktop_percent.right_sidebar_width / 100 },
    bottomPanel: { initialSizeRatio: TOKENS.desktop_percent.bottom_panel_height / 100 },
};

/** Theia's application shell, with its regions in the approved arrangement. */
@injectable()
export class NexeesShell extends ApplicationShell {

    /**
     * The left sidebar takes the whole height between the title row and the status bar. Beside it
     * the editor and the right sidebar stand over the bottom panel, which so runs under both.
     * Theia's own arrangement puts the bottom panel under the editor alone, between the sidebars.
     */
    protected override createLayout(): Layout {
        const split = { spacing: 0 };
        const editorAndRight = new TheiaSplitPanel({
            layout: this.createSplitLayout([this.mainPanel, this.rightPanelHandler.container], [1, 0], { ...split, orientation: 'horizontal' }),
        });
        const overBottom = new TheiaSplitPanel({
            layout: this.createSplitLayout([editorAndRight, this.bottomPanel], [1, 0], { ...split, orientation: 'vertical' }),
        });
        const besideLeft = new TheiaSplitPanel({
            layout: this.createSplitLayout([this.leftPanelHandler.container, overBottom], [0, 1], { ...split, orientation: 'horizontal' }),
        });
        return this.createBoxLayout([this.topPanel, besideLeft, this.statusBar], [0, 1, 0], { direction: 'top-to-bottom', spacing: 0 });
    }
}

/** What a profile with no stored layout shows: the approved layout, with every region open. */
@injectable()
export class PanelLayout implements FrontendApplicationContribution {

    constructor(
        @inject(ApplicationShell) protected readonly shell: ApplicationShell,
        @inject(RightSidebar) protected readonly rightSidebar: RightSidebar,
    ) { }

    /**
     * Runs after the default layouts of Theia's own views, which the application loads first:
     * they put the Explorer in the left sidebar and the Problems view and a terminal in the bottom
     * panel, all closed.
     */
    async initializeLayout(): Promise<void> {
        // Theia opens its Outline view in the right sidebar. The approved layout keeps that sidebar
        // for the Nexees areas, so such a view starts closed. The View menu still opens it.
        this.shell.getWidgets('right').forEach(view => view.close());
        await this.rightSidebar.addAreas();
        this.shell.expandPanel('bottom');
        // Of the bottom panel's views, the approved layout shows the terminal.
        const terminal = this.shell.getWidgets('bottom').find(view => view instanceof TerminalWidget);
        if (terminal) {
            await this.shell.revealWidget(terminal.id);
        }
        this.shell.expandPanel('left');
        // The right sidebar takes its share of the width the left one leaves, so the left one is laid out first.
        await this.shell.leftPanelHandler.state.pendingUpdate;
        await animationFrame();
        this.shell.expandPanel('right');
    }
}
