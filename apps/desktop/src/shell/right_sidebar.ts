// The right sidebar (apps/desktop/src/shell/right_sidebar): the region beside the editor that
// holds the functional areas of the Desktop UI contract: AI Agent, LCL, Tasks, Chat and Logs.
//
// - Each area is an ordinary view of Theia's right side panel. Showing, hiding and sizing the
//   sidebar, selecting an area and restoring all of it with the window's layout are Theia's own.
// - The sidebar shows its views as one row of text tabs across its top, as the approved layout
//   does, where Theia stands a bar of icons at the window's edge. Hidden, it leaves nothing
//   behind: its tabs are part of it.
// - No task has filled an area yet. Until the task that owns an area does, the area says that it
//   is not available and holds no control, so nothing in it can be taken for a working one.

import { inject, injectable } from '@theia/core/shared/inversify';
import { some } from '@theia/core/shared/@lumino/algorithm';
import { Message } from '@theia/core/shared/@lumino/messaging';
import { BoxLayout, BoxPanel, Panel } from '@theia/core/shared/@lumino/widgets';
import { animationFrame } from '@theia/core/lib/browser/browser';
import { FrontendApplicationContribution } from '@theia/core/lib/browser/frontend-application-contribution';
import { ApplicationShell, MAIN_BOTTOM_AREA_CLASS } from '@theia/core/lib/browser/shell/application-shell';
import { SidePanelHandler } from '@theia/core/lib/browser/shell/side-panel-handler';
import { SHELL_TABBAR_CONTEXT_MENU, SideTabBar } from '@theia/core/lib/browser/shell/tab-bars';
import { ColorTheme, CssStyleCollector, StylingParticipant } from '@theia/core/lib/browser/styling-service';
import { WidgetFactory, WidgetManager } from '@theia/core/lib/browser/widget-manager';
import { BaseWidget, codicon } from '@theia/core/lib/browser/widgets/widget';
import { ICONS, TOKENS } from './design_tokens';

/** One functional area: the ID of its view, its name on its tab, and its icon where Theia shows a view by an icon. */
interface Area {
    readonly id: string;
    readonly label: string;
    readonly icon: string;
}

/** The areas of the right sidebar, in the order of their tabs. */
const RIGHT_AREAS: readonly Area[] = [
    { id: 'nexees-area-agent', label: 'AI Agent', icon: ICONS.area_agent },
    { id: 'nexees-area-lcl', label: 'LCL', icon: ICONS.area_lcl },
    { id: 'nexees-area-tasks', label: 'Tasks', icon: ICONS.area_tasks },
    { id: 'nexees-area-chat', label: 'Chat', icon: ICONS.area_chat },
    { id: 'nexees-area-logs', label: 'Logs', icon: ICONS.area_logs },
];

/** The ID under which Theia's widget manager makes, and a stored layout names, the view of an area. */
const AREA_FACTORY = 'nexees-area';
/** The class of the row over the right sidebar's views: its tabs and the selected view's actions. */
const HEADER = 'nexees-right-header';
/** The class of the right sidebar's tabs. */
const TEXT_TABS = 'nexees-text-tabs';

/** The view of an area nothing fills yet: one sentence that says so, and no control. */
class AreaView extends BaseWidget {

    constructor(area: Area) {
        super();
        this.id = area.id;
        this.title.label = area.label;
        this.title.caption = area.label;
        this.title.iconClass = codicon(area.icon);
        this.title.closable = false;
        this.addClass('nexees-area');
        // Focusable, so that selecting the area makes it the window's active view, but not a stop of the Tab key.
        this.node.tabIndex = -1;
        const note = document.createElement('p');
        note.textContent = `The ${area.label} area is not available in this version of Nexees.`;
        this.node.append(note);
    }

    protected override onActivateRequest(message: Message): void {
        super.onActivateRequest(message);
        this.node.focus();
    }
}

/**
 * Makes the view of the area that `options` names. The options of a stored layout come from the
 * window's storage, so an area this version does not know is refused; Theia then restores the
 * layout without it.
 */
export const AREA_VIEWS: WidgetFactory = {
    id: AREA_FACTORY,
    createWidget: (options?: { id?: unknown }) => {
        const area = RIGHT_AREAS.find(known => known.id === options?.id);
        if (!area) {
            throw new Error(`The right sidebar has no area ${JSON.stringify(options?.id)}`);
        }
        return new AreaView(area);
    },
};

/**
 * A side bar's tabs as a row of text. Theia's side tab bar stands its tabs on their side: it
 * renders them twice, to measure them first, and hides those that do not fit its height. A row
 * needs neither. The browser lays it out, and where the sidebar is too narrow for all its tabs
 * the row scrolls (sidebarRules): by the mouse wheel, and to the tab that becomes selected.
 */
class TextTabBar extends SideTabBar {

    constructor(options: ConstructorParameters<typeof SideTabBar>[0]) {
        super(options);
        const row = this.contentContainer;
        row.addEventListener('wheel', event => { row.scrollLeft += event.deltaY; }, { passive: true });
    }

    protected override updateTabs(): void {
        if (this.isAttached) {
            this.renderTabs(this.contentNode, []);
        }
    }

    override async revealTab(index: number): Promise<void> {
        // The tab may be new: it is in the row once the row is drawn again.
        await animationFrame();
        this.contentNode.children[index]?.scrollIntoView({ block: 'nearest', inline: 'nearest' });
    }
}

/**
 * Theia's handler of a side panel, for both sides. The left one is Theia's, unchanged: the
 * activity bar and the views beside it. The right one is arranged as this file's header says.
 */
@injectable()
export class SidePanels extends SidePanelHandler {

    protected override createSideBar(): SideTabBar {
        return this.side === 'right' ? this.createTextTabs() : super.createSideBar();
    }

    /**
     * The right sidebar's tabs, connected as Theia connects a side bar's (SidePanelHandler.createSideBar)
     * with two exceptions. A click on the selected tab does not hide the sidebar, because the tabs
     * would go with it; the toggle in the title row hides it. And Theia's menu of the tabs that do
     * not fit is not connected: such tabs are reached by scrolling the row.
     */
    protected createTextTabs(): SideTabBar {
        const renderer = this.tabBarRendererFactory();
        const tabs = new TextTabBar({
            orientation: 'horizontal', insertBehavior: 'none', removeBehavior: 'select-previous-tab',
            allowDeselect: false, tabsMovable: true, renderer,
        });
        renderer.tabBar = tabs;
        renderer.contextMenuPath = SHELL_TABBAR_CONTEXT_MENU;
        tabs.disposed.connect(() => renderer.dispose());
        // Theia's look of a row of tabs, as in the editor and the bottom panel, and this file's own on top of it.
        tabs.addClass(MAIN_BOTTOM_AREA_CLASS);
        tabs.addClass(TEXT_TABS);
        tabs.tabAdded.connect((_, { title }) => {
            if (!some(this.dockPanel.widgets(), view => view === title.owner)) {
                this.dockPanel.addWidget(title.owner);
            }
        });
        tabs.tabActivateRequested.connect((_, { title }) => title.owner.activate());
        tabs.tabCloseRequested.connect((_, { title }) => title.owner.close());
        tabs.currentChanged.connect(this.onCurrentTabChanged, this);
        tabs.tabDetachRequested.connect(this.onTabDetachRequested, this);
        return tabs;
    }

    /** The right sidebar: the tabs and, beside them, the selected view's actions, in one row over the views. */
    protected override createContainer(): Panel {
        if (this.side !== 'right') {
            return super.createContainer();
        }
        const header = new Panel();
        header.addClass(HEADER);
        header.addWidget(this.tabBar);
        header.addWidget(this.toolBar);
        const layout = new BoxLayout({ direction: 'top-to-bottom', spacing: 0 });
        BoxPanel.setStretch(header, 0);
        layout.addWidget(header);
        BoxPanel.setStretch(this.dockPanel, 1);
        layout.addWidget(this.dockPanel);
        const container = new BoxPanel({ layout });
        container.id = 'theia-right-content-panel';
        return container;
    }

    /** Theia keeps a side panel's tabs in view while its views are hidden; the right sidebar hides whole. */
    override refresh(): void {
        super.refresh();
        if (this.side === 'right') {
            this.container.setHidden(this.dockPanel.isHidden);
        }
    }
}

/** The look of the right sidebar's header and of an unfilled area, as rules for the window's style sheet. */
function sidebarRules(): string[] {
    const gap = TOKENS.space.unit;
    const row = 'var(--theia-horizontal-toolbar-height)';
    return [
        '#theia-right-content-panel { border-left: var(--theia-panel-border-width) solid var(--theia-activityBar-border); }',
        `.${HEADER} { display: flex; min-height: ${row}; max-height: ${row}; background: var(--theia-sideBar-background); }`,
        `.${HEADER} > .${TEXT_TABS} { flex: 1 1 auto; min-width: 0; background: none; }`,
        // The selected tab names the view, so the toolbar keeps only the view's actions.
        `.${HEADER} > .theia-sidepanel-toolbar { flex: none; }`,
        `.${HEADER} .theia-sidepanel-title { display: none; }`,
        // The tabs keep their width; a row too narrow for them scrolls, without a scroll bar of its own.
        `.${TEXT_TABS} .lm-TabBar-content-container { min-width: 0; overflow-x: auto; scrollbar-width: none; }`,
        `.${TEXT_TABS} .lm-TabBar-tab { padding: 0 ${2 * gap}px; color: var(--theia-panelTitle-inactiveForeground); }`,
        `.${TEXT_TABS} .lm-TabBar-tab.lm-mod-current { color: var(--theia-panelTitle-activeForeground);`
        + ' box-shadow: 0 -2px 0 var(--theia-panelTitle-activeBorder) inset; }',
        `.${TEXT_TABS} .lm-TabBar-tab:hover { color: var(--theia-panelTitle-activeForeground); }`,
        // Text tabs: a view's icon is for the places where Theia shows the view by its icon alone.
        `.lm-TabBar.${TEXT_TABS} .lm-TabBar-tab .lm-TabBar-tabIcon { display: none; }`,
        `.nexees-area { display: flex; align-items: center; justify-content: center; padding: ${2 * gap}px; text-align: center;`
        + ' color: var(--theia-descriptionForeground); }',
    ];
}

/** Keeps the areas in the window, and gives the sidebar its look. */
@injectable()
export class RightSidebar implements FrontendApplicationContribution, StylingParticipant {

    constructor(
        @inject(ApplicationShell) protected readonly shell: ApplicationShell,
        @inject(WidgetManager) protected readonly views: WidgetManager,
    ) { }

    /** Whatever layout the window starts with, a stored one of an earlier version too, it holds every area. */
    async onDidInitializeLayout(): Promise<void> {
        await this.addAreas();
    }

    /** Adds each area that the window does not hold to the right sidebar, in the order of RIGHT_AREAS. */
    async addAreas(): Promise<void> {
        for (const [index, area] of RIGHT_AREAS.entries()) {
            const view = await this.views.getOrCreateWidget(AREA_FACTORY, { id: area.id });
            if (!view.isAttached) {
                this.shell.addWidget(view, { area: 'right', rank: index + 1 });
            }
        }
    }

    registerThemeStyle(_theme: ColorTheme, collector: CssStyleCollector): void {
        sidebarRules().forEach(rule => collector.addRule(rule));
    }
}
