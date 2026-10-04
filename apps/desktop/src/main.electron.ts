// The Nexees window's part in Electron's main process (apps/desktop/src/main). Theia's
// main-process application loads this module before Electron is ready, so before it starts its
// backend or opens a window.
//
// - The window connects to nothing beyond this machine (C20). Its own network stack resolves no
//   name but this machine's: nothing in this shell needs another, and Electron would otherwise
//   fetch its spell-checking dictionaries from Google's servers each time a profile starts. A
//   later task that gives the window a reason to reach another machine names it here.
// - The window keeps Theia's settings in its own configuration folder. By default Theia keeps
//   them in a folder of its own name in the user's home folder, which every Theia application
//   on the machine shares.
// - The window is dark from its first frame. Nexees is a dark interface (shell/theme), but until
//   a profile has loaded that theme once, Electron and Theia paint what the desktop prefers,
//   which on a light desktop is a white window. Once a theme is loaded, Theia tells Electron
//   which kind it is, so a user who chooses a light theme gets a light window. Electron takes
//   the setting only once it is ready; this module asks first, so it is set before any window.

import { ContainerModule } from '@theia/core/shared/inversify';
import { app, nativeTheme } from '@theia/core/electron-shared/electron';
import * as path from 'node:path';

/** Chromium's host resolver rules: every name is unknown, except this machine's. */
const RESOLVE_THIS_MACHINE_ONLY = 'MAP * ~NOTFOUND , EXCLUDE localhost';

export default new ContainerModule(() => {
    // Read when Electron starts its network service, so it must be set before Electron is ready.
    app.commandLine.appendSwitch('host-resolver-rules', RESOLVE_THIS_MACHINE_ONLY);
    // Set before Theia starts its backend, which inherits it. A folder the user chose stays.
    process.env.THEIA_CONFIG_DIR ??= path.join(app.getPath('userData'), 'theia');
    void app.whenReady().then(() => { nativeTheme.themeSource = 'dark'; });
});
