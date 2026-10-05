// The Nexees title row (apps/desktop/src/shell/title_bar): the one thin row at the top of the
// window (Desktop UI contract, "Required regions"). From left to right it holds the logo and the
// application's name, the menu, the window's title, the two sidebar toggles (panel_controls) and,
// immediately after them, the window's own controls. Nothing else is added above or below it.
//
// The row is Theia's own title bar for a window without the system's frame, which
// apps/desktop/package.json makes the default (`window.titleBarStyle`). Theia builds it and keeps
// the menu, the title and the window controls working; this file adds the logo with the name and
// the toggles, and gives the row its compact arrangement. A user who sets the title bar style to
// `native` gets the system's frame and menu instead of this row; the sidebars then answer to the
// commands of the command palette.
//
// The logo is the icon derived from the bound source (B1), read as a file of the installation
// like the About dialog's (about_dialog).

import { inject, injectable } from '@theia/core/shared/inversify';
import { Widget } from '@theia/core/shared/@lumino/widgets';
import { FrontendApplication } from '@theia/core/lib/browser/frontend-application';
import { FrontendApplicationConfigProvider } from '@theia/core/lib/browser/frontend-application-config-provider';
import { ColorTheme, CssStyleCollector, StylingParticipant } from '@theia/core/lib/browser/styling-service';
import { CustomTitleWidget, ElectronMenuContribution } from '@theia/core/lib/electron-browser/menu/electron-menu-contribution';
import { LOGO } from './about_dialog';
import { TOKENS } from './design_tokens';
import { SidebarToggles } from './panel_controls';

/** The ID of the logo with the application's name, at the left end of the row. */
const BRAND = 'nexees-title-brand';

/**
 * The row's arrangement, as rules for the window's style sheet. The row is a line of items in the
 * order Theia and this file add them. Theia places the title and the window controls by fixed
 * positions that assume its own row; here they are items of the line like the others, so the
 * title takes the room between the menu and the toggles, and the toggles end where the window
 * controls begin. The row's height is the title bar token, which the theme gives Theia (theme).
 */
function rowRules(): string[] {
    const gap = TOKENS.space.unit;
    // The row's last pixel is its border, so what stands in it is one pixel lower than the row.
    const inner = TOKENS.desktop.title_bar_height - 1;
    return [
        // In a window too narrow for the whole row, the name and the menu give way: the toggles and
        // the window controls stay in view.
        '#theia-top-panel > .lm-MenuBar { flex: 0 1 auto; min-width: 0; overflow: hidden; }',
        // The logo and the name move the window when dragged, as the title does.
        `#theia-top-panel > #${BRAND} { display: flex; flex: 0 1 auto; min-width: 0; overflow: hidden; align-items: center; gap: ${gap}px;`
        + ` padding: 0 ${gap}px 0 ${2 * gap}px; color: var(--theia-textLink-foreground); font-weight: 700; text-transform: uppercase;`
        + ' -webkit-app-region: drag; }',
        `#${BRAND} > img { flex: none; width: ${TOKENS.desktop.title_logo_size}px; height: ${TOKENS.desktop.title_logo_size}px; }`,
        `#theia-custom-title { position: static; flex: 1 1 0; min-width: 0; margin: 0; padding: 0 ${gap}px; transform: none;`
        + ` text-align: center; line-height: ${inner}px; font-size: ${TOKENS.font.small}px; }`,
        '#nexees-sidebar-toggles { display: flex; flex: none; }',
        `.nexees-sidebar-toggle { width: ${TOKENS.desktop.sidebar_toggle_width}px; padding: 0; border: none; background: none;`
        + ' color: var(--theia-titleBar-activeForeground); cursor: pointer; }',
        '.nexees-sidebar-toggle:hover:enabled { background: var(--theia-toolbar-hoverBackground); }',
        '.nexees-sidebar-toggle:disabled { opacity: var(--theia-mod-disabled-opacity); cursor: default; }',
        `#window-controls { position: static; flex: none; height: auto; grid-template-columns: repeat(3, ${TOKENS.desktop.window_control_width}px); }`,
        `#window-controls .control-button { line-height: ${inner}px; }`,
    ];
}

/** Theia's title bar, with what Nexees adds to it. */
@injectable()
export class TitleBar extends ElectronMenuContribution implements StylingParticipant {

    @inject(SidebarToggles)
    protected readonly toggles!: SidebarToggles;

    /** The logo and the application's name, where Theia's menu bar keeps a place for a logo. */
    protected override createLogo(): Widget {
        const brand = new Widget();
        brand.id = BRAND;
        const logo = document.createElement('img');
        logo.src = LOGO;
        logo.alt = '';
        const name = document.createElement('span');
        name.textContent = FrontendApplicationConfigProvider.get().applicationName;
        brand.node.append(logo, name);
        return brand;
    }

    /** Theia adds the window controls right after the title, so the toggles added here stand immediately before them. */
    protected override createCustomTitleWidget(app: FrontendApplication): void {
        super.createCustomTitleWidget(app);
        app.shell.addWidget(this.toggles, { area: 'top' });
    }

    registerThemeStyle(_theme: ColorTheme, collector: CssStyleCollector): void {
        rowRules().forEach(rule => collector.addRule(rule));
    }
}

/**
 * The window's title in the row. Theia centres it on the window by setting a position computed
 * from the widths of its own row; in this row the style places it (rowRules), so nothing is computed.
 */
@injectable()
export class WindowTitle extends CustomTitleWidget {

    protected override adjustTitleToCenter(): void {
        // Placed by the row's style.
    }
}
