// The Nexees About dialog (apps/desktop/src/shell/about_dialog): the logo, the application's name
// and version, and the foundation it is built on. It replaces Theia's dialog, whose details are
// about VS Code extension compatibility and link to a website; Nexees runs no such extensions
// (SA-08) and its About dialog opens nothing outside the computer.
//
// The logo is the icon derived from the bound source (B1), which the build places in the
// application's resources (scripts/build/build_desktop.py); the window's page reads it from
// there, as a file of the installation.

import * as React from '@theia/core/shared/react';
import { injectable } from '@theia/core/shared/inversify';
import { AboutDialog } from '@theia/core/lib/browser/about-dialog';
import { FrontendApplicationConfigProvider } from '@theia/core/lib/browser/frontend-application-config-provider';
import { TOKENS } from './design_tokens';

/** The logo in the application's resources, relative to the window's page (lib/frontend). */
export const LOGO = '../../resources/branding/nexees-128.png';
/** The side of the logo in the dialog; the file has twice as many pixels, for dense screens. */
const LOGO_SIDE = 64;
/** The package of the foundation whose version the dialog names. */
const FOUNDATION = '@theia/core';

@injectable()
export class NexeesAboutDialog extends AboutDialog {

    protected override render(): React.ReactNode {
        const foundation = this.extensionsInfos.find(extension => extension.name === FOUNDATION);
        const gap = 2 * TOKENS.space.unit;
        return React.createElement('div', { className: 'nexees-about', style: { display: 'flex', alignItems: 'center', gap, padding: gap } },
            React.createElement('img', { className: 'nexees-about-logo', src: LOGO, alt: '', width: LOGO_SIDE, height: LOGO_SIDE }),
            React.createElement('div', undefined,
                React.createElement('h2', { className: 'nexees-about-name', style: { margin: 0 } },
                    FrontendApplicationConfigProvider.get().applicationName),
                this.applicationInfo && React.createElement('p', { className: 'nexees-about-version' },
                    `Version ${this.applicationInfo.version}`),
                foundation && React.createElement('p', { className: 'nexees-about-foundation' },
                    `Built on Eclipse Theia ${foundation.version}`)));
    }
}
