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

import { ContainerModule } from '@theia/core/shared/inversify';
import { app } from '@theia/core/electron-shared/electron';
import * as path from 'node:path';

/** Chromium's host resolver rules: every name is unknown, except this machine's. */
const RESOLVE_THIS_MACHINE_ONLY = 'MAP * ~NOTFOUND , EXCLUDE localhost';

export default new ContainerModule(() => {
    // Read when Electron starts its network service, so it must be set before Electron is ready.
    app.commandLine.appendSwitch('host-resolver-rules', RESOLVE_THIS_MACHINE_ONLY);
    // Set before Theia starts its backend, which inherits it. A folder the user chose stays.
    process.env.THEIA_CONFIG_DIR ??= path.join(app.getPath('userData'), 'theia');
});
