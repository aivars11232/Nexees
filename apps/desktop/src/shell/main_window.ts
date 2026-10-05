// The Nexees main window (apps/desktop/src/shell/main_window): what Nexees adds to Theia's
// workbench, bound here in one place. That is the Nexees look (theme), the About dialog
// (about_dialog) and the approved layout: the title row with the sidebar toggles (title_bar,
// panel_controls), the arrangement of the regions (panel_layout) and the right sidebar with its
// areas (right_sidebar). The window's view of the Nexees host in the status bar is in this file.
//
// The entry shows what the window's backend reports and nothing else: attached, with the
// host's device; attaching; or unavailable, with the reason (RC-06). When the host is
// unavailable, clicking the entry asks the backend to try again. The entry decides nothing
// itself: the host's state is the backend's, and the backend's is the host's.

import { ContainerModule, inject, injectable } from '@theia/core/shared/inversify';
import { AboutDialog } from '@theia/core/lib/browser/about-dialog';
import { FrontendApplicationContribution } from '@theia/core/lib/browser/frontend-application-contribution';
import { ServiceConnectionProvider } from '@theia/core/lib/browser/messaging/service-connection-provider';
import { ApplicationShell, ApplicationShellOptions } from '@theia/core/lib/browser/shell/application-shell';
import { SidePanelHandler } from '@theia/core/lib/browser/shell/side-panel-handler';
import { StatusBar, StatusBarAlignment, StatusBarEntry } from '@theia/core/lib/browser/status-bar/status-bar-types';
import { StylingParticipant } from '@theia/core/lib/browser/styling-service';
import { WidgetFactory } from '@theia/core/lib/browser/widget-manager';
import { CommandContribution, CommandRegistry } from '@theia/core/lib/common/command';
import { Emitter, Event } from '@theia/core/lib/common/event';
import { CustomTitleWidget, ElectronMenuContribution } from '@theia/core/lib/electron-browser/menu/electron-menu-contribution';
import { HOST_CONNECTION_PATH, HostConnectionClient, HostConnectionService, HostState } from '../main.protocol';
import { NexeesAboutDialog } from './about_dialog';
import { ICONS } from './design_tokens';
import { SidebarToggles } from './panel_controls';
import { NexeesShell, PANEL_SHARES, PanelLayout } from './panel_layout';
import { AREA_VIEWS, RightSidebar, SidePanels } from './right_sidebar';
import { NexeesTheme } from './theme';
import { TitleBar, WindowTitle } from './title_bar';

/** The status bar entry's ID. */
const ENTRY = 'nexees-host';
/** The command behind the entry. */
const RETRY = { id: 'nexees.host.retry', label: 'Nexees: Attach to the Nexees Host Again' };

/** Receives the backend's notifications and passes them on. */
@injectable()
export class HostStateRelay implements HostConnectionClient {

    protected readonly changed = new Emitter<HostState>();
    readonly onChanged: Event<HostState> = this.changed.event;

    onStateChanged(state: HostState): void {
        this.changed.fire(state);
    }
}

/** The status bar entry, and the command that asks for a new attempt. */
@injectable()
export class MainWindow implements FrontendApplicationContribution, CommandContribution {

    constructor(
        @inject(StatusBar) protected readonly statusBar: StatusBar,
        @inject(HostConnectionService) protected readonly host: HostConnectionService,
        @inject(HostStateRelay) protected readonly relay: HostStateRelay,
    ) { }

    async onStart(): Promise<void> {
        this.relay.onChanged(state => this.show(state));
        this.show(await this.host.getState());
    }

    registerCommands(commands: CommandRegistry): void {
        commands.registerCommand(RETRY, {
            execute: async () => this.show(await this.host.retry()),
        });
    }

    protected show(state: HostState): void {
        void this.statusBar.setElement(ENTRY, entry(state));
    }
}

/** The status bar entry for `state`. */
export function entry(state: HostState): StatusBarEntry {
    const base = { alignment: StatusBarAlignment.LEFT, priority: 1000 };
    switch (state.kind) {
        case 'attached':
            return { ...base, text: `$(${ICONS.host_attached}) Nexees host`, tooltip: `Attached to the Nexees host on ${state.device}` };
        case 'attaching':
            return { ...base, text: `$(${ICONS.host_attaching}~spin) Nexees host`, tooltip: 'Attaching to the Nexees host' };
        case 'unavailable':
            return {
                ...base,
                text: `$(${ICONS.host_unavailable}) Nexees host unavailable`,
                tooltip: `The Nexees host is unavailable: ${state.reason}. Click to try again.`,
                command: RETRY.id,
            };
    }
}

export default new ContainerModule((bind, _unbind, _isBound, rebind) => {
    bind(NexeesTheme).toSelf().inSingletonScope();
    bind(FrontendApplicationContribution).toService(NexeesTheme);
    bind(StylingParticipant).toService(NexeesTheme);
    rebind(AboutDialog).to(NexeesAboutDialog).inSingletonScope();

    // The approved layout: Theia's shell, side panels and title bar, each in its Nexees arrangement.
    rebind(ApplicationShellOptions).toConstantValue(PANEL_SHARES);
    rebind(ApplicationShell).to(NexeesShell).inSingletonScope();
    rebind(SidePanelHandler).to(SidePanels);
    bind(WidgetFactory).toConstantValue(AREA_VIEWS);
    bind(RightSidebar).toSelf().inSingletonScope();
    bind(FrontendApplicationContribution).toService(RightSidebar);
    bind(StylingParticipant).toService(RightSidebar);
    bind(PanelLayout).toSelf().inSingletonScope();
    bind(FrontendApplicationContribution).toService(PanelLayout);
    bind(SidebarToggles).toSelf().inSingletonScope();
    bind(TitleBar).toSelf().inSingletonScope();
    rebind(ElectronMenuContribution).toService(TitleBar);
    bind(StylingParticipant).toService(TitleBar);
    rebind(CustomTitleWidget).to(WindowTitle).inSingletonScope();

    bind(HostStateRelay).toSelf().inSingletonScope();
    // The host connection stays in this window's own backend, never a remote one.
    bind(HostConnectionService).toDynamicValue(context => ServiceConnectionProvider.createLocalProxy<HostConnectionService>(
        context.container, HOST_CONNECTION_PATH, context.container.get(HostStateRelay))).inSingletonScope();
    bind(MainWindow).toSelf().inSingletonScope();
    bind(FrontendApplicationContribution).toService(MainWindow);
    bind(CommandContribution).toService(MainWindow);
});
