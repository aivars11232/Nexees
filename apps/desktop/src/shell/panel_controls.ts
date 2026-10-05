// The sidebar toggles (apps/desktop/src/shell/panel_controls): the two controls at the right end of
// the title row, immediately before the window's own controls, that show and hide the left and
// the right sidebar (Desktop UI contract, "Top-right controls"). The title row puts them there
// (title_bar).
//
// A toggle decides nothing itself. A click runs the command Theia already has for that side, the
// one the command palette runs, and the toggle then shows what the sidebar did: its icon and its
// pressed state follow the sidebar, whoever showed or hid it. While a sidebar holds no view there
// is nothing to show, and its toggle is disabled. Showing and hiding a sidebar changes this
// window's view and nothing else: no setting of the host, and no authority.

import { inject, injectable } from '@theia/core/shared/inversify';
import { Message } from '@theia/core/shared/@lumino/messaging';
import { CommonCommands } from '@theia/core/lib/browser/common-commands';
import { ApplicationShell } from '@theia/core/lib/browser/shell/application-shell';
import { SideTabBar } from '@theia/core/lib/browser/shell/tab-bars';
import { BaseWidget, codicon } from '@theia/core/lib/browser/widgets/widget';
import { CommandRegistry } from '@theia/core/lib/common/command';
import { ICONS } from './design_tokens';

/** One sidebar as its toggle needs it: Theia's command for it, its name for the user, and its two icons. */
interface Side {
    readonly area: 'left' | 'right';
    readonly name: string;
    readonly command: string;
    readonly shown: string;
    readonly hidden: string;
}

/** The two sidebars, in the order of their toggles. */
const SIDES: readonly Side[] = [
    {
        area: 'left', name: 'left sidebar', command: CommonCommands.TOGGLE_LEFT_PANEL.id,
        shown: ICONS.sidebar_left_shown, hidden: ICONS.sidebar_left_hidden,
    },
    {
        area: 'right', name: 'right sidebar', command: CommonCommands.TOGGLE_RIGHT_PANEL.id,
        shown: ICONS.sidebar_right_shown, hidden: ICONS.sidebar_right_hidden,
    },
];

/** The two toggles, as one widget of the title row. */
@injectable()
export class SidebarToggles extends BaseWidget {

    /** For each toggle, what brings it up to date with its sidebar. */
    protected readonly updates: Array<() => void> = [];

    constructor(
        @inject(ApplicationShell) protected readonly shell: ApplicationShell,
        @inject(CommandRegistry) protected readonly commands: CommandRegistry,
    ) {
        super();
        this.id = 'nexees-sidebar-toggles';
        for (const side of SIDES) {
            const toggle = document.createElement('button');
            toggle.id = `nexees-toggle-${side.area}-sidebar`;
            toggle.addEventListener('click', () => this.commands.executeCommand(side.command));
            // A click with the mouse leaves the keyboard where it is, in the editor for instance.
            toggle.addEventListener('mousedown', event => event.preventDefault());
            this.node.append(toggle);
            const update = () => this.mirror(toggle, side);
            this.updates.push(update);
            // Selecting a view shows a sidebar and selecting none hides it, so this is every change.
            this.tabsOf(side).currentChanged.connect(update, this);
        }
        const all = () => this.updates.forEach(update => update());
        this.toDispose.pushAll([this.shell.onDidAddWidget(all), this.shell.onDidRemoveWidget(all)]);
    }

    /** The toggles are first shown once the window has started, when Theia's commands exist. */
    protected override onAfterAttach(message: Message): void {
        super.onAfterAttach(message);
        this.updates.forEach(update => update());
    }

    /** The tab bar that holds a sidebar's views; the selected one is the view the sidebar shows. */
    protected tabsOf(side: Side): SideTabBar {
        return (side.area === 'left' ? this.shell.leftPanelHandler : this.shell.rightPanelHandler).tabBar;
    }

    /** Makes `toggle` show whether its sidebar is shown, and whether the sidebar has anything to show. */
    protected mirror(toggle: HTMLButtonElement, side: Side): void {
        const shown = !!this.tabsOf(side).currentTitle;
        toggle.className = `nexees-sidebar-toggle ${codicon(shown ? side.shown : side.hidden)}`;
        toggle.title = `${shown ? 'Hide' : 'Show'} the ${side.name}`;
        toggle.setAttribute('aria-label', toggle.title);
        toggle.setAttribute('aria-pressed', String(shown));
        toggle.disabled = !this.commands.isEnabled(side.command);
    }
}
