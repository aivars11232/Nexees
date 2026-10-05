#!/usr/bin/env node
// The installed Nexees Desktop application, end to end (tests/e2e/desktop/local_application).
//
//     node tests/e2e/desktop/local_application.test.mjs <prefix>
//
// <prefix> holds an installation made by `scripts/build/build_desktop.py install`. The test runs it
// in a private session it creates itself: a nested KWin compositor that draws into memory, on its
// own D-Bus session, with HOME, the XDG folders and the runtime folder all inside a temporary
// profile. No window reaches the user's screen, and the application gets none of the user's
// session bus, keyring, settings or runtime folder. Every process of that session is stopped and
// the profile removed before the test ends. It needs `kwin_wayland`, `dbus-run-session` and
// systemd's XDG autostart generator.
//
// With the installed application only, it proves:
//  1. the launcher starts a local application with its own window, whose page is a file of the
//     installed application; nothing of it listens beyond this machine or connects to another
//     machine, and it keeps its settings in a folder of its own (C20);
//  2. the workbench shows the files of a local workspace;
//  3. the window starts the host and attaches to it: one host process, separate from the
//     window and holding nothing of it, at the user's privilege, with its channel private to
//     the user (RC-04, RC-23);
//  4. the status bar shows the host as attached, as the backend reports it, where the user sees it;
//  5. closing the window detaches it, and a host not asked to keep running then stops (RC-04);
//  6. a host kept running, as start at login keeps it, outlives the window, and three reopened
//     windows each find that same host: exactly one executor (RC-T14);
//  7. start at login is one user-level autostart entry while enabled, which systemd's own
//     generator turns into a unit bound to the graphical session, and which is removed on
//     disable; outside a login session the host reports unsupported (RC-05, RC-24, RC-T04);
//  8. the idle host holds no network socket and uses no processor time while idle (RC-08);
//  9. extensions stay disabled: the window has no Extensions view and runs no extension host
//     (SA-08, SI-22);
// 10. when the host cannot run, the window shows it as unavailable, with the reason on pointing
//     at the entry, and nothing else starts; once the cause is gone, clicking the entry attaches
//     the window (RC-06, RC-T03);
// 11. the hardening of the Desktop runtime (SG-05, TT-26; SI-28, TH-02 to TH-04): the installed
//     Electron binary has the fuses the package definition names; started with
//     ELECTRON_RUN_AS_NODE, NODE_OPTIONS and an inspector argument, it is still the application,
//     loads nothing NODE_OPTIONS names and opens no inspector; the backend refuses to start
//     without the window's token, and no process it starts inherits that token; the page runs in
//     a sandboxed renderer without Node;
// 12. the Nexees look (R18, B1): the window is dark while it loads; it wears the Nexees dark
//     theme, whose every colour Theia knows and applies and whose surfaces, text and sizes are
//     the shared design tokens'; a reopened window wears it again; each icon of the icon mapping
//     exists; the About dialog shows the logo, as a file of the installation that is the icon
//     derived from the bound source, with the name and version and no link; the user can choose
//     another colour theme and return, the choice kept in the user's settings; and the
//     installation carries the icon in every derived size, named by its desktop entry;
// 13. the approved layout (Desktop UI contract, R18): the window has no system frame but one thin
//     title row, with the logo and name, the menu, the title, the two sidebar toggles and,
//     immediately after them, the window controls; under it the activity bar and the left sidebar
//     over the whole height, the editor and the right sidebar over the bottom panel, and the
//     status bar; a new profile shows them in the shares of the design tokens; the right sidebar
//     has the five areas as one row of text tabs, each saying that it is not available and
//     holding no control; each toggle hides and shows its sidebar and follows it; in a window
//     too narrow for the row the toggles and the window controls stay in view; the window
//     controls work, the one that closes the window among them; and a reopened window comes
//     back as it was closed.
//
// It exits 0 when every check passes and prints one line per check.

import { spawn, spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import * as fs from 'node:fs';
import * as os from 'node:os';
import * as path from 'node:path';
import { setTimeout as sleep } from 'node:timers/promises';

const DEFINITION = JSON.parse(fs.readFileSync(new URL('../../../packaging/desktop/package_definition.json', import.meta.url), 'utf8'));
const METADATA = JSON.parse(fs.readFileSync(new URL('../../../apps/desktop/resources/application_metadata.json', import.meta.url), 'utf8'));
// The shared visual system the window must show: the design tokens, the icon mapping and the logo's manifest.
const TOKENS = JSON.parse(fs.readFileSync(new URL('../../../assets/theme/design_tokens.json', import.meta.url), 'utf8'));
const ICONS = JSON.parse(fs.readFileSync(new URL('../../../assets/theme/icon_mapping.json', import.meta.url), 'utf8')).icons;
const BRANDING = new URL('../../../assets/branding/', import.meta.url);
const LOGO = JSON.parse(fs.readFileSync(new URL('manifest.json', BRANDING), 'utf8'));
const THEME = 'nexees-dark'; // the theme's ID (apps/desktop/src/shell/theme)
// The functional areas of the right sidebar, in the order of the Desktop UI contract.
const AREAS = ['AI Agent', 'LCL', 'Tasks', 'Chat', 'Logs'];
// The nested screen. A new window takes two thirds of it each way, which leaves every panel more than its smallest size.
const SCREEN = { width: 1600, height: 1000 };
const GRACE_MS = 5_000; // the host's grace period after its last window (application_host.rs)
const DEBUG_PORT = 9334; // DevTools, on 127.0.0.1 only, for this test
const INSPECT_PORT = 9335; // where a Node inspector would listen if the application honoured --inspect
const AUTOSTART_GENERATOR = '/usr/lib/systemd/user-generators/systemd-xdg-autostart-generator';
// Electron's fuse wire: this marker, the wire's version, the number of fuses, then one byte per
// fuse, '1' for on and '0' for off, in the order of Electron's fuse schema.
const FUSE_MARKER = 'dL7pKGdnNz796PbbjQWNKmHXBZaB9tsX';
const FUSES = ['RunAsNode', 'EnableCookieEncryption', 'EnableNodeOptionsEnvironmentVariable', 'EnableNodeCliInspectArguments',
    'EnableEmbeddedAsarIntegrityValidation', 'OnlyLoadAppFromAsar', 'LoadBrowserProcessSpecificV8Snapshot',
    'GrantFileProtocolExtraPrivileges', 'WasmTrapHandlers'];
// An address of this machine only, as the tables in /proc/net print it: 127.0.0.0/8, ::1, or the
// first mapped into IPv6.
const LOOPBACK = /^(?:[0-9A-F]{6}7F|0{24}01000000|0{16}FFFF0000[0-9A-F]{6}7F)$/;

const failures = [];
function check(name, ok, detail = '') {
    console.log(`${ok ? 'PASS' : 'FAIL'}  ${name}${detail ? `  [${detail}]` : ''}`);
    if (!ok) failures.push(name);
}

async function until(what, probe, timeoutMs) {
    const deadline = Date.now() + timeoutMs;
    let last;
    while (Date.now() < deadline) {
        last = await probe();
        if (last) {
            return last;
        }
        await sleep(250);
    }
    throw new Error(`timed out waiting for ${what}`);
}

// ---------------------------------------------------------------- outside: the private session

if (process.argv[2] !== '--inside') {
    const prefix = path.resolve(process.argv[2] ?? '');
    if (!process.argv[2] || !fs.existsSync(path.join(prefix, 'bin/nexees'))) {
        console.error('usage: local_application.test.mjs <prefix of an installed Nexees>');
        process.exit(2);
    }
    const profile = fs.mkdtempSync(path.join(os.tmpdir(), 'nexees-e2e-'));
    const folder = name => {
        const dir = path.join(profile, name);
        fs.mkdirSync(dir, { recursive: true, mode: 0o700 });
        return dir;
    };
    const env = { ...process.env };
    for (const name of ['DISPLAY', 'WAYLAND_DISPLAY', 'DBUS_SESSION_BUS_ADDRESS', 'ELECTRON_RUN_AS_NODE', 'NODE_OPTIONS', 'XDG_SESSION_TYPE',
        'SESSION_MANAGER', 'NEXEES_HOST_PROGRAM', 'THEIA_CONFIG_DIR', 'THEIA_ELECTRON_TOKEN']) {
        delete env[name];
    }
    Object.assign(env, {
        HOME: folder('home'), XDG_CONFIG_HOME: folder('config'), XDG_DATA_HOME: folder('data'),
        XDG_CACHE_HOME: folder('cache'), XDG_STATE_HOME: folder('state'), XDG_RUNTIME_DIR: folder('run'),
    });
    // The private bus activates no services: no portal, secret service or accessibility bus starts in
    // the session, so none of them can fail or leave anything behind. No process of the session
    // writes a core dump.
    const busConfig = path.join(profile, 'session.conf');
    fs.writeFileSync(busConfig, `<!DOCTYPE busconfig PUBLIC "-//freedesktop//DTD D-Bus Bus Configuration 1.0//EN"
 "http://www.freedesktop.org/standards/dbus/1.0/busconfig.dtd">
<busconfig>
  <type>session</type>
  <listen>unix:tmpdir=${env.XDG_RUNTIME_DIR}</listen>
  <auth>EXTERNAL</auth>
  <policy context="default">
    <allow send_destination="*" eavesdrop="true"/>
    <allow eavesdrop="true"/>
    <allow own="*"/>
  </policy>
</busconfig>
`);
    const inner = spawnSync('sh', ['-c', 'ulimit -c 0 && exec "$@"', 'session', 'dbus-run-session', `--config-file=${busConfig}`, '--',
        process.execPath, process.argv[1], '--inside', fs.realpathSync(prefix), profile], { env, stdio: 'inherit' });
    fs.rmSync(profile, { recursive: true, force: true });
    process.exit(inner.status ?? 1);
}

// ---------------------------------------------------------------- inside: the checks

const [, , , prefix, profile] = process.argv;
const runtime = process.env.XDG_RUNTIME_DIR;
const app = path.join(prefix, 'lib/nexees/app');
const hostProgram = path.join(app, 'bin/nexees-host');
const electron = path.join(app, DEFINITION.window.electron);
const children = [];

function readProc(pid, file) {
    try {
        return fs.readFileSync(`/proc/${pid}/${file}`, 'latin1');
    } catch {
        return undefined; // a process that ended, or one of another user
    }
}

function exeOf(pid) {
    try {
        return fs.readlinkSync(`/proc/${pid}/exe`);
    } catch {
        return undefined;
    }
}

/** The environment a process was started with. */
function environOf(pid) {
    return (readProc(pid, 'environ') ?? '').split('\0');
}

/** A process's command line as one string; Chromium's processes rewrite theirs that way. */
function commandOf(pid) {
    return (readProc(pid, 'cmdline') ?? '').replaceAll('\0', ' ').trim();
}

/** The fields of /proc/<pid>/stat after the command: state, parent, process group and so on. */
function statOf(pid) {
    return (readProc(pid, 'stat') ?? '').split(') ').pop().split(' ');
}

/**
 * Every process of this private session: those with its runtime folder, and those running from the
 * installation under test, since Chromium's processes overwrite their environment with their title.
 */
function sessionProcesses() {
    const found = [];
    for (const name of fs.readdirSync('/proc').filter(entry => /^\d+$/.test(entry))) {
        const pid = Number(name);
        if (pid === process.pid || pid === process.ppid || readProc(pid, 'environ') === undefined) {
            continue;
        }
        if (environOf(pid).includes(`XDG_RUNTIME_DIR=${runtime}`) || (exeOf(pid) ?? '').startsWith(`${prefix}/`)) {
            found.push(pid);
        }
    }
    return found;
}

function hostPids() {
    return sessionProcesses().filter(pid => exeOf(pid) === hostProgram);
}

/** The window's processes of one Chromium kind, such as `renderer`. */
function windowProcesses(kind) {
    return sessionProcesses().filter(pid => exeOf(pid) === electron && ` ${commandOf(pid)} `.includes(` --type=${kind} `));
}

/** The window's backend: Theia's Node process, which the window's main process starts. */
function backendPid() {
    return sessionProcesses().find(pid => commandOf(pid).startsWith(`${electron} ${path.join(app, 'lib/backend/main.js')} `));
}

function cpuTicks(pid) {
    const fields = statOf(pid);
    return Number(fields[11]) + Number(fields[12]); // utime and stime
}

/** What a process holds open, as the kernel names each descriptor. */
function descriptorsOf(pid) {
    const held = [];
    for (const fd of fs.readdirSync(`/proc/${pid}/fd`)) {
        try {
            held.push(fs.readlinkSync(`/proc/${pid}/fd/${fd}`));
        } catch {
            // Closed meanwhile.
        }
    }
    return held;
}

/**
 * The network sockets the given processes hold, by the kernel's tables: for each, whether its own
 * address is this machine's loopback, and whether it is unconnected or connected to the loopback.
 */
function networkSockets(pids) {
    const held = new Set();
    for (const pid of pids) {
        let descriptors = [];
        try {
            descriptors = fs.readdirSync(`/proc/${pid}/fd`);
        } catch {
            // Ended meanwhile.
        }
        for (const fd of descriptors) {
            try {
                const match = /^socket:\[(\d+)\]$/.exec(fs.readlinkSync(`/proc/${pid}/fd/${fd}`));
                if (match) {
                    held.add(match[1]);
                }
            } catch {
                // Closed meanwhile.
            }
        }
    }
    const sockets = [];
    for (const table of ['tcp', 'tcp6', 'udp', 'udp6', 'raw', 'raw6']) {
        for (const line of fs.readFileSync(`/proc/net/${table}`, 'utf8').split('\n').slice(1)) {
            const fields = line.trim().split(/\s+/);
            if (held.has(fields[9])) {
                const [local, remote] = [fields[1], fields[2]].map(field => field.split(':'));
                sockets.push({
                    listening: table.startsWith('tcp') && fields[3] === '0A', port: parseInt(local[1], 16),
                    local: LOOPBACK.test(local[0]), inside: LOOPBACK.test(remote[0]) || /^0+$/.test(remote[0]),
                    text: `${table} ${fields[1]}>${fields[2]}`,
                });
            }
        }
    }
    return sockets;
}

/** The fuses of an Electron binary by name, '1' for on and '0' for off, read from its fuse wire. */
function fusesOf(binary) {
    const data = fs.readFileSync(binary);
    const at = data.indexOf(FUSE_MARKER);
    const first = at + FUSE_MARKER.length + 2;
    if (at < 0 || data.indexOf(FUSE_MARKER, at + 1) >= 0 || data[first - 2] !== 1 || data[first - 1] !== FUSES.length) {
        throw new Error(`${binary} has no fuse wire of the known form`);
    }
    return Object.fromEntries(FUSES.map((name, index) => [name, String.fromCharCode(data[first + index])]));
}

function host(...args) {
    return spawnSync(hostProgram, args, { stdio: 'ignore' }).status;
}

/** The DevTools page of the window, once it answers. */
async function page() {
    try {
        const pages = await (await fetch(`http://127.0.0.1:${DEBUG_PORT}/json`, { signal: AbortSignal.timeout(2000) })).json();
        return pages.find(p => p.type === 'page');
    } catch {
        return undefined;
    }
}

/**
 * Opens a DevTools session with the page. What a command sets up for the page, such as an emulated
 * window size, lasts as long as the session.
 */
async function session(target) {
    const socket = new WebSocket(target.webSocketDebuggerUrl);
    await new Promise((resolve, reject) => {
        socket.onopen = resolve;
        socket.onerror = reject;
    });
    let sent = 0;
    return {
        /** Sends one command and returns its result. */
        send(method, params) {
            const id = sent += 1;
            socket.send(JSON.stringify({ id, method, params }));
            return new Promise(resolve => {
                const listener = event => {
                    const message = JSON.parse(event.data);
                    if (message.id === id) {
                        socket.removeEventListener('message', listener);
                        resolve(message.result);
                    }
                };
                socket.addEventListener('message', listener);
            });
        },
        close: () => socket.close(),
    };
}

/** Sends one DevTools command to the page, in a session of its own, and returns its result. */
async function send(target, method, params) {
    const devtools = await session(target);
    try {
        return await devtools.send(method, params);
    } finally {
        devtools.close();
    }
}

async function evaluate(target, expression) {
    return (await send(target, 'Runtime.evaluate', { expression, returnByValue: true, awaitPromise: true }))?.result?.value;
}

/** Presses and releases one key in the page, as a user's keyboard would. */
async function press(target, key, code, virtualKey) {
    for (const type of ['keyDown', 'keyUp']) {
        await send(target, 'Input.dispatchKeyEvent', { type, key, code, windowsVirtualKeyCode: virtualKey, nativeVirtualKeyCode: virtualKey });
    }
}

/**
 * Types `text` into the list the command palette shows and takes the entry that starts with it, as a user would:
 * only once the list has taken the text and its highlighted entry is that one.
 */
async function choose(target, text) {
    await send(target, 'Input.insertText', { text });
    await until(`"${text}" highlighted in the palette`, async () => (await evaluate(target, `(() => {
        const palette = document.querySelector('.quick-input-widget');
        const focused = palette?.querySelector('.quick-input-list .monaco-list-row.focused');
        return !!palette?.querySelector('input')?.value.endsWith(${JSON.stringify(text)})
            && !!focused?.innerText.trim().startsWith(${JSON.stringify(text)});
    })()`)) || undefined, 15_000);
    await press(target, 'Enter', 'Enter', 13);
}

/**
 * Runs a command from the command palette, opened with F1. A window listens to its keys only once
 * its start is complete, so F1 is pressed again, as a user would, until the palette shows.
 */
async function runCommand(target, command) {
    // Open means shown and holding the keyboard: text typed before that would be lost.
    const open = `(() => {
        const palette = document.querySelector('.quick-input-widget');
        return !!palette && getComputedStyle(palette).display !== 'none' && document.activeElement === palette.querySelector('input');
    })()`;
    await until('the command palette', async () => {
        if (!(await evaluate(target, open))) {
            await press(target, 'F1', 'F1', 112);
            await sleep(500);
        }
        return (await evaluate(target, open)) || undefined;
    }, 30_000);
    await choose(target, command);
}

/**
 * Chooses a colour theme as a user would, with the command Preferences: Color Theme, and waits until the window wears it
 * and the user's settings hold one of the values in `kept`. Returns the value they hold.
 */
async function chooseTheme(target, theme, bodyClass, kept) {
    const userSettings = path.join(process.env.XDG_CONFIG_HOME, 'Nexees/theia/settings.json');
    await runCommand(target, 'Preferences: Color Theme');
    // The command replaces the palette's list by the list of themes; typing waits for that list.
    await until('the list of themes', async () => (await evaluate(target, `(() => {
        const palette = document.querySelector('.quick-input-widget');
        return palette?.querySelector('input')?.value === '' && document.activeElement === palette.querySelector('input')
            && [...palette.querySelectorAll('.quick-input-list .monaco-list-row')].some(row => row.innerText.trim().startsWith('Nexees Dark'));
    })()`)) || undefined, 15_000);
    await choose(target, theme);
    return until(`the theme ${theme}`, async () => {
        await shown(target);
        const setting = fs.existsSync(userSettings) ? JSON.parse(fs.readFileSync(userSettings, 'utf8'))['workbench.colorTheme'] : undefined;
        return (await evaluate(target, `document.body.classList.contains('${bodyClass}')`)) && kept.includes(setting) ? String(setting) : undefined;
    }, 15_000);
}

function sha256(file) {
    return createHash('sha256').update(fs.readFileSync(file)).digest('hex');
}

/** A colour as the page reports or a theme names it, `#rrggbb[aa]` or `rgb[a](...)`, in one comparable form. */
function colour(value) {
    const hex = /^#([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})?$/i.exec(value ?? '');
    if (hex) {
        return [1, 2, 3].map(i => parseInt(hex[i], 16)).concat((hex[4] ? parseInt(hex[4], 16) / 255 : 1).toFixed(2)).join(',');
    }
    const rgb = /^rgba?\((\d+), (\d+), (\d+)(?:, ([\d.]+))?\)$/.exec(value ?? '');
    return rgb ? [rgb[1], rgb[2], rgb[3], Number(rgb[4] ?? 1).toFixed(2)].join(',') : `unreadable: ${value}`;
}

/** The centre of the first visible element `selector` finds, or null. */
async function centreOf(target, selector) {
    return JSON.parse(await evaluate(target, `JSON.stringify((() => {
        const element = [...document.querySelectorAll(${JSON.stringify(selector)})].find(e => e.getBoundingClientRect().width > 0);
        const r = element?.getBoundingClientRect();
        return r ? { x: r.x + r.width / 2, y: r.y + r.height / 2 } : null;
    })())`) ?? 'null');
}

/** Moves the mouse over the first visible element `selector` finds, as a user pointing at it would. */
async function pointAt(target, selector) {
    const where = await centreOf(target, selector);
    if (where) {
        await send(target, 'Input.dispatchMouseEvent', { type: 'mouseMoved', ...where });
    }
    return where !== null;
}

/** Clicks at a point of the page, as a user's mouse would. */
async function clickAt(target, where) {
    for (const type of ['mousePressed', 'mouseReleased']) {
        await send(target, 'Input.dispatchMouseEvent', { type, x: where.x, y: where.y, button: 'left', clickCount: 1 });
    }
}

/** Clicks the centre of the first visible element `selector` finds, as a user's mouse would. */
async function click(target, selector) {
    const where = await centreOf(target, selector);
    if (where) {
        await clickAt(target, where);
    }
    return where !== null;
}

// What the page shows. The host's entry counts only where the user can see it: Theia keeps its
// status bar hidden until its preferences are loaded.
const PROBE = `JSON.stringify((() => {
    const entry = document.getElementById('status-bar-nexees-host');
    const box = entry?.getBoundingClientRect();
    return {
        origin: location.protocol + '//' + location.hostname,
        page: decodeURIComponent(location.pathname),
        shell: !!document.querySelector('#theia-app-shell'),
        loading: !!document.querySelector('.theia-preload'),
        restricted: !!document.getElementById('status-bar-workspace-trust-status'),
        files: [...document.querySelectorAll('.theia-TreeNodeSegment')].map(n => n.textContent),
        host: entry ? {
            visible: box.width > 0 && box.height > 0,
            label: entry.getAttribute('aria-label') ?? '',
            attached: !!entry.querySelector('.codicon-${ICONS.host_attached}'),
        } : null,
        node: [typeof require, typeof process, typeof module, typeof Buffer].filter(kind => kind !== 'undefined').length,
    };
})())`;

// What the window looks like: the theme Theia holds for the Nexees ID, each of its colours beside
// the value the page applies, and the colours and sizes of the regions a user sees.
const LOOK = `(async () => {
    const style = (selector, property) => { const e = document.querySelector(selector); return e ? getComputedStyle(e)[property] : null; };
    const box = selector => document.querySelector(selector)?.getBoundingClientRect();
    const themes = await new Promise((resolve, reject) => {
        const open = indexedDB.open('theia-monaco');
        open.onerror = () => reject(open.error);
        open.onsuccess = () => {
            const all = open.result.transaction('themes').objectStore('themes').getAll();
            all.onerror = () => reject(all.error);
            all.onsuccess = () => resolve(all.result);
        };
    });
    const theme = themes.find(t => t.id === '${THEME}');
    const root = getComputedStyle(document.documentElement);
    const rules = [...document.styleSheets].flatMap(sheet => { try { return [...sheet.cssRules]; } catch { return []; } });
    return JSON.stringify({
        classes: [...document.body.classList],
        prefersDark: matchMedia('(prefers-color-scheme: dark)').matches,
        colours: Object.entries(theme?.data.colors ?? {}).map(([id, value]) => [id, value, root.getPropertyValue('--theia-' + id.replaceAll('.', '-')).trim()]),
        syntax: (theme?.data.rules ?? []).map(rule => rule.token + '=' + rule.foreground),
        page: style('body', 'backgroundColor'),
        statusBar: { background: style('#theia-statusBar', 'backgroundColor'), text: style('#status-bar-nexees-host', 'color'),
            height: box('#theia-statusBar')?.height, font: style('#theia-statusBar .element', 'fontSize') },
        sideBar: style('#theia-left-content-panel .theia-side-panel', 'backgroundColor'),
        row: { height: box('.theia-TreeNode')?.height, font: style('.theia-TreeNode', 'fontSize'), text: style('.theia-TreeNode', 'color') },
        activityBar: box('.theia-app-left .lm-TabBar-tab')?.width,
        tabHeight: root.getPropertyValue('--theia-private-horizontal-tab-height').trim(),
        tab: [...document.querySelectorAll('#theia-bottom-content-panel .lm-TabBar-tab, #theia-main-content-panel .lm-TabBar-tab')]
            .map(tab => tab.getBoundingClientRect().height).find(height => height > 0) ?? null,
        codicons: ${JSON.stringify(Object.values(ICONS))}.filter(name => rules.some(rule => rule.selectorText?.includes('.codicon-' + name + ':'))),
    });
})()`;

// The About dialog as the user sees it.
const ABOUT = `JSON.stringify((() => {
    const about = document.querySelector('.nexees-about');
    const dialog = about?.closest('.dialogBlock');
    const logo = about?.querySelector('img');
    const ok = dialog?.querySelector('.dialogControl .theia-button');
    return about ? {
        title: dialog?.querySelector('.dialogTitle')?.innerText.trim(),
        text: about.innerText,
        links: dialog?.querySelectorAll('a').length,
        logo: logo ? { complete: logo.complete, natural: [logo.naturalWidth, logo.naturalHeight], shown: [logo.width, logo.height],
            file: decodeURIComponent(new URL(logo.src).pathname) } : null,
        corner: ok ? getComputedStyle(ok).borderRadius : null,
    } : null;
})())`;

// The window's layout as the user sees it: the title row's items in their order, each region's
// place, and the right sidebar's tabs with the area they show. A place is that of a visible element.
const LAYOUT = `(async () => {
    const box = element => {
        const r = element?.getBoundingClientRect();
        return r && r.width > 0 && r.height > 0 ? { x: r.x, y: r.y, w: r.width, h: r.height, r: r.right, b: r.bottom } : null;
    };
    const one = selector => box(document.querySelector(selector));
    const logo = document.querySelector('#nexees-title-brand img');
    const area = [...document.querySelectorAll('#theia-right-side-panel > .lm-DockPanel-widget')].find(view => box(view));
    return JSON.stringify({
        window: { w: innerWidth, h: innerHeight, maximized: document.body.classList.contains('maximized') },
        frame: await window.electronTheiaCore.getTitleBarStyleAtStartup(),
        top: one('#theia-top-panel'),
        row: [...document.querySelector('#theia-top-panel')?.children ?? []]
            .filter(item => getComputedStyle(item).position !== 'absolute' && box(item)).map(item => ({ id: item.id, ...box(item) })),
        moves: ['#theia-drag-panel', '#nexees-title-brand', '#theia-custom-title', '#nexees-sidebar-toggles', '#window-controls']
            .map(selector => { const item = document.querySelector(selector); return item ? getComputedStyle(item).webkitAppRegion : 'missing'; }),
        name: document.querySelector('#nexees-title-brand span')?.textContent,
        logo: logo ? { complete: logo.complete, natural: [logo.naturalWidth, logo.naturalHeight], shown: [logo.width, logo.height],
            file: decodeURIComponent(new URL(logo.src).pathname) } : null,
        menus: [...document.querySelectorAll('#theia-top-panel .lm-MenuBar-itemLabel')].map(label => label.textContent),
        title: document.querySelector('#theia-custom-title')?.textContent, documentTitle: document.title,
        toggles: [...document.querySelectorAll('#nexees-sidebar-toggles button')].map(toggle => ({
            ...box(toggle), pressed: toggle.getAttribute('aria-pressed'), label: toggle.getAttribute('aria-label'), tip: toggle.title,
            icon: [...toggle.classList].find(name => name.startsWith('codicon-')), disabled: toggle.disabled })),
        controls: Object.fromEntries(['minimize', 'maximize', 'restore', 'close'].map(name => [name, one('#' + name + '-button')])),
        activity: one('#theia-left-content-panel .theia-app-sidebar-container'), left: one('#theia-left-content-panel'),
        explorer: one('#explorer-view-container'), editor: one('#theia-main-content-panel'), right: one('#theia-right-content-panel'),
        rightIcons: one('#theia-right-content-panel .theia-app-sidebar-container'),
        bottom: one('#theia-bottom-content-panel'), status: one('#theia-statusBar'),
        tabs: [...document.querySelectorAll('#theia-right-content-panel .lm-TabBar-tab')].filter(tab => box(tab)).map(tab => ({
            ...box(tab), label: tab.textContent.trim(), current: tab.classList.contains('lm-mod-current') })),
        area: area ? { text: area.innerText.trim(),
            controls: area.querySelectorAll('button, input, select, textarea, a, [role="button"], [tabindex="0"]').length } : null,
        bottomTab: document.querySelector('#theia-bottom-content-panel .lm-TabBar-tab.lm-mod-current')?.id ?? null,
    });
})()`;

/** The window's layout, read after the page has drawn a frame. */
async function layoutOf(target) {
    await send(target, 'Page.captureScreenshot', { format: 'png', clip: { x: 0, y: 0, width: 1, height: 1, scale: 1 } });
    return JSON.parse(await evaluate(target, LAYOUT));
}

/** Waits until the window's layout is as `wanted` says, and returns it. */
async function layoutWhen(target, what, wanted) {
    return until(what, async () => {
        const layout = await layoutOf(target);
        return wanted(layout) ? layout : undefined;
    }, 20_000);
}

/** Whether two places or sizes of the page are the same, to within a pixel's rounding. */
function near(one, other, by = 0.6) {
    return typeof one === 'number' && typeof other === 'number' && Math.abs(one - other) <= by;
}

/** Clicks the right sidebar's tab named `label`, as a user's mouse would. */
async function clickTab(target, label) {
    const tab = (await layoutOf(target)).tabs.find(candidate => candidate.label === label);
    if (tab) {
        await clickAt(target, { x: tab.x + tab.w / 2, y: tab.y + tab.h / 2 });
    }
    return !!tab;
}

async function shown(target) {
    // Asking for one pixel of the page makes it draw a frame. Theia's start waits for a frame, and
    // the nested compositor draws none while the user's real session is locked: it then locks its
    // own screen too. The test must not depend on whether the developer's screen is locked.
    await send(target, 'Page.captureScreenshot', { format: 'png', clip: { x: 0, y: 0, width: 1, height: 1, scale: 1 } });
    return JSON.parse((await evaluate(target, PROBE)) ?? 'null');
}

/**
 * Opens a window from the installed launcher on `workspace`, as a user would: it waits for the
 * workbench, answers Theia's question whether to trust a folder it has not seen with no (the
 * folder then opens in Restricted Mode), and opens the Explorer where a stored layout has it
 * closed; a new profile shows it open. Returns once the question is answered and the Explorer
 * lists the workspace's files. `extra` adds to the launcher's environment and arguments.
 */
async function openWindow(workspace, extra = { env: {}, args: [] }) {
    const child = spawn(path.join(prefix, 'bin/nexees'),
        [workspace, `--remote-debugging-port=${DEBUG_PORT}`, '--remote-debugging-address=127.0.0.1', ...extra.args],
        { env: { ...process.env, ...extra.env }, detached: true, stdio: 'ignore' });
    children.push(child);
    // What the page prefers and has painted the first time it can be asked, long before its theme is loaded.
    let early;
    const target = await until('the workbench', async () => {
        const found = await page();
        if (found && early === undefined) {
            early = await evaluate(found, `JSON.stringify({ dark: matchMedia('(prefers-color-scheme: dark)').matches,
                painted: document.documentElement.style.getPropertyValue('--theia-editor-background') })`);
        }
        const seen = found ? await shown(found) : null;
        // The shell alone: the host's entry comes when the Nexees part of the window has started,
        // which may be before or after Theia asks the trust question.
        return seen?.shell ? found : undefined;
    }, 120_000);
    let lastClick = 0;
    const seen = await until('the workspace files in the Explorer', async () => {
        // The trust question may come a moment after the workbench; it is answered whenever it shows.
        const answered = await evaluate(target,
            `(() => { const no = [...document.querySelectorAll('button')].find(b => b.innerText.startsWith("No, I don't trust")); no?.click(); return !!no; })()`);
        if (answered) {
            return undefined;
        }
        // Answering no puts the folder in Restricted Mode, which the status bar then shows. Until
        // then the question may still come and cover the window, so the window is not ready. Nor is
        // it while its loading screen, though fading, still lies over the workbench and takes the clicks.
        const now = await shown(target);
        if (now?.files.includes('hello.txt') && now.restricted && !now.loading) {
            return now;
        }
        // Open the Explorer only while it is closed: a click on an open one would close it.
        const open = await evaluate(target, `(document.querySelector('#explorer-view-container')?.getBoundingClientRect().width ?? 0) > 0`);
        if (!open && Date.now() - lastClick > 3_000) {
            lastClick = Date.now();
            await click(target, '#shell-tab-explorer-view-container');
        }
        return undefined;
    }, 60_000);
    return { child, target, seen, early: JSON.parse(early ?? 'null') };
}

/** Waits until the status bar shows, where the user sees it, that the window is attached to a host. */
async function attached(window) {
    return until('the status bar to show the host as attached', async () => {
        const seen = await shown(window.target);
        return seen?.host?.visible && seen.host.attached && /Attached to the Nexees host/.test(seen.host.label) ? seen : undefined;
    }, 30_000);
}

/** Closes the window as a user would, and waits until its processes are gone. */
async function closeWindow(window) {
    const version = await (await fetch(`http://127.0.0.1:${DEBUG_PORT}/json/version`)).json();
    const socket = new WebSocket(version.webSocketDebuggerUrl);
    await new Promise(resolve => {
        socket.onopen = resolve;
    });
    socket.send(JSON.stringify({ id: 1, method: 'Browser.close' }));
    await until('the window to close', async () => window.child.exitCode !== null || window.child.signalCode !== null, 30_000);
    socket.close();
}

let kwin;
try {
    const display = `wayland-nexees-e2e-${process.pid}`;
    kwin = spawn('kwin_wayland', ['--virtual', '--socket', display, '--width', String(SCREEN.width), '--height', String(SCREEN.height)],
        { stdio: 'ignore' });
    await until('the nested compositor', async () => fs.existsSync(path.join(runtime, display)), 20_000);
    process.env.WAYLAND_DISPLAY = display;
    process.env.XDG_SESSION_TYPE = 'wayland';

    const workspace = path.join(profile, 'workspace');
    fs.mkdirSync(path.join(workspace, 'src'), { recursive: true });
    fs.writeFileSync(path.join(workspace, 'hello.txt'), 'Hello from a local workspace.\n');
    fs.writeFileSync(path.join(workspace, 'src/main.rs'), 'fn main() {}\n');

    // 12, before any window: the application's icon in the installation.
    const entryLines = fs.readFileSync(path.join(prefix, DEFINITION.layout.desktop_entry), 'utf8').split('\n');
    const installedIcons = LOGO.derived.map(icon => [icon, path.join(prefix,
        DEFINITION.layout.icons.replaceAll('{size}', icon.size).replace('{icon}', METADATA.icon))]);
    const wrongIcons = installedIcons.filter(([icon, file]) => !fs.existsSync(file) || sha256(file) !== icon.sha256
        || sha256(file) !== sha256(new URL(icon.file, BRANDING))).map(([icon]) => icon.size);
    const validator = spawnSync('desktop-file-validate', [path.join(prefix, DEFINITION.layout.desktop_entry)], { encoding: 'utf8' });
    check('the installation carries the application icon in every size derived from the logo, and its desktop entry names it (B1)',
        installedIcons.length > 0 && wrongIcons.length === 0 && entryLines.includes(`Icon=${METADATA.icon}`)
        && (validator.error?.code === 'ENOENT' || validator.status === 0),
        `${installedIcons.length} sizes; wrong: ${wrongIcons.join(', ') || 'none'}; desktop-file-validate: `
        + (validator.error ? 'not installed' : `exit ${validator.status} ${validator.stdout.trim()}`));

    // 11, before any window: the fuses, and the backend alone.
    const fuses = fusesOf(electron);
    check('the installed Electron binary has the fuses the package definition names (SI-28, TH-03)',
        Object.entries(DEFINITION.window.fuses).every(([name, on]) => fuses[name] === (on ? '1' : '0')),
        FUSES.map(name => `${name}=${fuses[name]}`).join(' '));
    const alone = spawnSync(electron, [path.join(app, 'lib/backend/main.js'), '--port=0'],
        { env: { ...process.env, ELECTRON_RUN_AS_NODE: '1' }, encoding: 'utf8', timeout: 60_000 });
    check('the backend refuses to start without the window\'s token (TH-02)',
        alone.status === 1 && /without that window's token/.test(alone.stderr) && !/listening on/.test(alone.stdout + alone.stderr),
        `exit ${alone.status}`);

    // 1 to 4: start from the installed launcher, with no host running. This first window is started
    // the way a polluted environment or a meddling process would start it (11): told to run as Node,
    // to load a script through NODE_OPTIONS, and to open an inspector.
    const loaded = path.join(profile, 'loaded_through_node_options');
    const intruder = path.join(profile, 'intruder.js');
    fs.writeFileSync(intruder, `require('node:fs').appendFileSync(${JSON.stringify(loaded)}, process.argv.join(' ') + '\\n');\n`);
    check('no host runs before the window starts', hostPids().length === 0 && host('status') === 1);
    const first = await openWindow(workspace, {
        env: { ELECTRON_RUN_AS_NODE: '1', NODE_OPTIONS: `--require ${intruder}` },
        args: [`--inspect=127.0.0.1:${INSPECT_PORT}`],
    });
    check('the window is a local application: its page is a file of the installed application, not a website (C20)',
        first.seen.origin === 'file://' && first.seen.page.startsWith(`${app}/`), first.seen.page);
    check('the workbench shows the local workspace\'s files', first.seen.files.includes('hello.txt') && first.seen.files.includes('src'),
        first.seen.files.join(', '));
    const extensions = await evaluate(first.target, `!!document.querySelector('[id^="shell-tab-vsx-extensions"]')`);
    const pluginHosts = sessionProcesses().filter(pid => commandOf(pid).includes('plugin-host'));
    check('extensions stay disabled: no Extensions view and no extension host process (SA-08, SI-22)',
        extensions === false && pluginHosts.length === 0, `${pluginHosts.length} extension hosts`);
    const status = await attached(first);
    check('the status bar shows the host as attached, where the user sees it', true, status.host.label);
    const hosts = hostPids();
    check('exactly one host process runs, separate from the window', hosts.length === 1 && !hosts.includes(first.child.pid),
        `${hosts.length} host(s)`);
    const hostPid = hosts[0];
    check('the host runs at the user\'s privilege, in its own process group',
        fs.statSync(`/proc/${hostPid}`).uid === process.getuid() && statOf(hostPid)[2] === String(hostPid));
    const channel = path.join(runtime, 'nexees');
    // The backend that started the host leaves descriptors of its own open in every process it starts.
    const held = descriptorsOf(hostPid);
    const own = held.filter(name => name === '/dev/null' || name.startsWith(`${channel}/`)
        || name.startsWith(`${path.join(process.env.XDG_DATA_HOME, 'nexees')}/`));
    const hostChannel = held.filter(name => name.startsWith('socket:'));
    check('the host keeps nothing of the window that started it: it holds only the null device, its lock, its store and its channel (RC-04)',
        hostChannel.length === 2 && own.length + hostChannel.length === held.length, `${held.length} descriptors`);
    check('the channel folder and the socket are private to the user (0700, 0600)',
        (fs.statSync(channel).mode & 0o777) === 0o700 && (fs.lstatSync(path.join(channel, 'host.sock')).mode & 0o777) === 0o600);
    check('nexees-host status finds the running host', host('status') === 0);

    // 8: the idle host, with the window attached.
    const before = cpuTicks(hostPid);
    await sleep(5_000);
    const used = cpuTicks(hostPid) - before;
    const hostSockets = networkSockets([hostPid]);
    check('the idle host uses no processor time and holds no network socket (RC-08)',
        used <= 1 && hostSockets.length === 0, `${used} ticks in 5 s; ${hostSockets.length} network sockets`);

    // 1, once the window has had the time to do whatever it does on its own.
    const sockets = networkSockets(sessionProcesses());
    const listeners = sockets.filter(socket => socket.listening);
    check('nothing of the application listens beyond this machine: its backend and this test\'s DevTools port, on the loopback interface',
        listeners.length === 2 && listeners.some(socket => socket.port === DEBUG_PORT) && listeners.every(socket => socket.local),
        listeners.map(socket => socket.text).join(' '));
    const dictionaries = path.join(process.env.XDG_CONFIG_HOME, 'Nexees/Dictionaries');
    const fetched = fs.existsSync(dictionaries) ? fs.readdirSync(dictionaries) : [];
    const beyond = sockets.filter(socket => !socket.local || !socket.inside);
    check('the application connects to no other machine, and has fetched nothing',
        beyond.length === 0 && fetched.length === 0,
        `${sockets.length - listeners.length} connections, ${beyond.length} beyond the loopback interface; ${fetched.length} fetched dictionaries`);
    const settings = path.join(process.env.XDG_CONFIG_HOME, 'Nexees');
    check('the application keeps its settings in its own folder, not in one shared with other Electron or Theia applications',
        fs.existsSync(path.join(settings, 'theia')) && !fs.existsSync(path.join(process.env.XDG_CONFIG_HOME, 'Electron'))
        && !fs.existsSync(path.join(process.env.HOME, '.theia')), settings);

    // 11, on the running window.
    check('told to run as Node, to load a script through NODE_OPTIONS and to open an inspector, it is still the application and does none of it (TH-03)',
        !fs.existsSync(loaded) && !listeners.some(socket => socket.port === INSPECT_PORT));
    const renderers = windowProcesses('renderer');
    check('the page runs in a sandboxed renderer, without Node (TH-04)',
        renderers.length > 0 && renderers.every(pid => /^Seccomp:\s+2$/m.test(readProc(pid, 'status') ?? '')) && status.node === 0,
        `${renderers.length} renderer(s) under seccomp; ${status.node} Node globals in the page`);
    const backend = backendPid();
    // Theia's default layout starts one terminal, so the backend has a shell among its children.
    const started = await until('a terminal\'s shell', async () => {
        const below = sessionProcesses().filter(pid => statOf(pid)[1] === String(backend));
        return below.some(pid => exeOf(pid) !== electron) ? below : undefined;
    }, 30_000);
    const inheritors = [...started, hostPid].filter(pid => environOf(pid).some(entry => entry.startsWith('THEIA_ELECTRON_TOKEN=')));
    check('no process the backend starts inherits the window\'s token: not its terminals, its file watcher or the host (SI-28)',
        environOf(backend).some(entry => entry.startsWith('THEIA_ELECTRON_TOKEN=')) && inheritors.length === 0,
        `${started.map(pid => path.basename(exeOf(pid) ?? '?')).join(', ')} and the host`);

    // 12: the Nexees look, on the running window.
    const look = JSON.parse(await evaluate(first.target, LOOK));
    const { surface, text } = TOKENS.color;
    check('the window is dark while it loads, before any theme is applied',
        first.early?.dark === true && !/^#f{6}$/i.test(first.early.painted), `first painted ${first.early?.painted || 'nothing yet'}`);
    check('the window wears the Nexees dark theme, and is dark to the desktop too (R18)',
        look.classes.includes(THEME) && look.classes.includes('theia-dark') && look.prefersDark, look.classes.join(' '));
    const unapplied = look.colours.filter(([, named, applied]) => colour(named) !== colour(applied)).map(([id]) => id);
    check('every colour the theme names is one Theia knows and applies',
        look.colours.length > 100 && unapplied.length === 0, `${look.colours.length} colours; not applied: ${unapplied.join(', ') || 'none'}`);
    const surfaces = {
        'page': [look.page, surface.editor], 'side bar': [look.sideBar, surface.window], 'status bar': [look.statusBar.background, surface.raised],
        'status bar text': [look.statusBar.text, text.secondary], 'tree text': [look.row.text, text.primary],
    };
    const offColour = Object.entries(surfaces).filter(([, [seen, token]]) => colour(seen) !== colour(token)).map(([name, [seen]]) => `${name} ${seen}`);
    check('the surfaces and text a user sees have the colours of the shared design tokens (R19)',
        offColour.length === 0, offColour.join('; ') || Object.keys(surfaces).join(', '));
    const { desktop, font } = TOKENS;
    const sizes = {
        'status bar height': [look.statusBar.height, desktop.status_bar_height], 'status bar text': [look.statusBar.font, `${font.small}px`],
        'tree row height': [look.row.height, desktop.row_height], 'tree text': [look.row.font, `${font.base}px`],
        'activity bar width': [look.activityBar, desktop.activity_bar_width], 'tab height': [look.tabHeight, `${desktop.tab_height}px`],
        'a tab in the window': [look.tab ?? desktop.tab_height, desktop.tab_height],
    };
    const offSize = Object.entries(sizes).filter(([, [seen, token]]) => seen !== token).map(([name, [seen, token]]) => `${name} ${seen}, not ${token}`);
    check('the compact sizes are the tokens\': rows, tabs, status bar and activity bar',
        offSize.length === 0, offSize.join('; ') || Object.entries(sizes).map(([name, [seen]]) => `${name} ${seen}`).join(', '));
    const syntax = Object.values(TOKENS.color.syntax).filter(value => !look.syntax.some(rule => rule.endsWith(`=${value}`)));
    check('the theme carries the syntax colours of the tokens', look.syntax.length > 0 && syntax.length === 0, `${look.syntax.length} rules`);
    check('every icon of the icon mapping exists in the window\'s icon set',
        look.codicons.length === Object.keys(ICONS).length, look.codicons.join(', '));

    await runCommand(first.target, 'About');
    const about = await until('the About dialog with its logo', async () => {
        const seen = JSON.parse((await evaluate(first.target, ABOUT)) ?? 'null');
        return seen?.logo?.complete ? seen : undefined;
    }, 15_000);
    const windowIcon = LOGO.derived.find(icon => icon.size === 128);
    check('the About dialog shows the logo: a file of the installation, the icon derived from the bound source (B1)',
        about.logo.file === path.join(app, 'resources/branding/nexees-128.png') && sha256(about.logo.file) === windowIcon.sha256
        && about.logo.natural.join() === '128,128' && about.logo.shown.join() === '64,64', about.logo.file);
    check('the About dialog names Nexees, its version and its foundation, and links to nothing',
        about.title === METADATA.name && about.text.includes(METADATA.name) && /Version \d+\.\d+\.\d+/.test(about.text)
        && about.text.includes('Built on Eclipse Theia') && about.links === 0, about.text.replace(/\s+/g, ' '));
    check('buttons have the corner radius of the tokens', about.corner === `${TOKENS.radius.control}px`, about.corner);
    // Escape closes the dialog once it holds the keyboard, which the closing command palette may still have.
    await until('the About dialog to close', async () => {
        if (await evaluate(first.target, `!document.querySelector('.nexees-about')`)) {
            return true;
        }
        await press(first.target, 'Escape', 'Escape', 27);
        await sleep(500);
        return undefined;
    }, 15_000);

    // 13: the approved layout, on the same window: the first layout of a new profile.
    const fresh = await layoutWhen(first.target, 'every region of the layout',
        l => !!(l.top && l.activity && l.left && l.explorer && l.editor && l.right && l.bottom && l.status && l.area));
    const { top, activity, left, explorer, editor, right, bottom, status: statusBar } = fresh;
    const rowItems = fresh.row.map(item => item.id);
    const inOneLine = fresh.row.every((item, index) => near(item.x, index === 0 ? 0 : fresh.row[index - 1].r) && item.y >= top.y && item.b <= top.b);
    check('the window has no system frame but one thin title row: logo and name, menu, title, sidebar toggles and window controls, in that order (R18)',
        fresh.frame === 'custom' && top.y === 0 && near(top.h, desktop.title_bar_height) && near(top.w, fresh.window.w)
        && rowItems.join() === 'nexees-title-brand,theia:menubar,theia-custom-title,nexees-sidebar-toggles,window-controls' && inOneLine
        && near(fresh.row.at(-1)?.r, fresh.window.w) && near(activity.y, top.b) && fresh.menus.includes('File') && fresh.menus.includes('Help')
        && fresh.title === fresh.documentTitle && fresh.title.includes(path.basename(workspace)),
        `${fresh.frame} title bar, ${top.h} px high: ${rowItems.join(', ')}; menu ${fresh.menus.join(' ')}; title "${fresh.title}"`);
    check('the title row moves the window by its free part, its logo and its title, and leaves the toggles and the window controls to the mouse',
        fresh.moves.join() === 'drag,drag,drag,no-drag,no-drag', fresh.moves.join(' '));
    check('the title row shows the logo, the icon derived from the bound source as a file of the installation, beside the application\'s name (B1)',
        fresh.logo?.complete === true && fresh.logo.file === about.logo.file && fresh.logo.natural.join() === '128,128'
        && fresh.logo.shown.join() === `${desktop.title_logo_size},${desktop.title_logo_size}` && fresh.name === METADATA.name,
        `${fresh.name}; ${fresh.logo?.shown.join('x')} px of ${fresh.logo?.file}`);
    const [leftToggle, rightToggle] = fresh.toggles;
    const controls = ['minimize', 'maximize', 'close'].map(name => fresh.controls[name]);
    check('both sidebar toggles stand in the title row immediately before the window controls, all in the widths of the tokens',
        fresh.toggles.length === 2 && near(leftToggle.r, rightToggle.x) && near(rightToggle.r, controls[0]?.x)
        && fresh.toggles.every(toggle => near(toggle.w, desktop.sidebar_toggle_width) && toggle.y >= top.y && toggle.b <= top.b && !toggle.disabled)
        && controls.every((control, index) => !!control && near(control.w, desktop.window_control_width)
            && near(control.r, fresh.window.w - (2 - index) * desktop.window_control_width) && control.b <= top.b)
        && !fresh.controls.restore,
        `toggles at ${fresh.toggles.map(toggle => `${toggle.x}-${toggle.r}`).join(', ')}; controls at ${controls.map(control => `${control?.x}-${control?.r}`).join(', ')}`);
    check('the regions are arranged as approved: the activity bar and the left sidebar over the whole height, the editor and the right sidebar over the bottom panel, the status bar below',
        activity.x === 0 && near(activity.w, desktop.activity_bar_width) && near(activity.b, statusBar.y) && near(left.y, top.b) && near(left.b, statusBar.y)
        && explorer.x >= activity.r - 0.6 && explorer.r <= left.r + 0.6
        && near(editor.x, left.r) && near(editor.y, top.b) && near(right.x, editor.r) && near(right.y, top.b) && near(right.r, fresh.window.w)
        && near(bottom.x, left.r) && near(bottom.r, fresh.window.w) && near(bottom.y, editor.b) && near(bottom.y, right.b) && near(bottom.b, statusBar.y)
        && statusBar.x === 0 && near(statusBar.w, fresh.window.w) && near(statusBar.b, fresh.window.h),
        Object.entries({ activity, left, editor, right, bottom, 'status bar': statusBar })
            .map(([name, at]) => `${name} ${Math.round(at.x)},${Math.round(at.y)} ${Math.round(at.w)}x${Math.round(at.h)}`).join('; '));
    const share = TOKENS.desktop_percent;
    const shares = {
        'left sidebar': [left.w, fresh.window.w * share.left_sidebar_width / 100],
        'right sidebar': [right.w, (fresh.window.w - left.w) * share.right_sidebar_width / 100],
        'bottom panel': [bottom.h, (statusBar.y - top.b) * share.bottom_panel_height / 100],
    };
    const offShare = Object.entries(shares).filter(([, [seen, wanted]]) => !near(seen, wanted, 1.5));
    check('a new profile shows the panels in the shares of the tokens: the left sidebar of the window\'s width, the right one of the width beside it, the bottom panel of the height',
        offShare.length === 0, Object.entries(shares).map(([name, [seen, wanted]]) => `${name} ${seen.toFixed(1)} px for ${wanted.toFixed(1)}`).join('; '));
    check('the bottom panel of a new profile shows the terminal', fresh.bottomTab?.startsWith('shell-tab-terminal') === true, `${fresh.bottomTab}`);
    check('the right sidebar shows its five areas as one row of text tabs across its top, the first selected, and no bar of icons at the window\'s edge',
        fresh.tabs.map(tab => tab.label).join() === AREAS.join() && fresh.tabs.every(tab => near(tab.y, right.y) && near(tab.h, desktop.tab_height))
        && fresh.tabs.filter(tab => tab.current).length === 1 && fresh.tabs[0].current && !fresh.rightIcons,
        fresh.tabs.map(tab => tab.label + (tab.current ? ' (selected)' : '')).join(', '));
    const unfilled = [];
    for (const label of AREAS) {
        if (!(await clickTab(first.target, label))) {
            unfilled.push(`${label}: no tab`);
            continue;
        }
        const seen = await layoutWhen(first.target, `the ${label} area`, l => l.tabs.find(tab => tab.current)?.label === label && !!l.area);
        unfilled.push(seen.area.text === `The ${label} area is not available in this version of Nexees.` && seen.area.controls === 0
            ? label : `${label}: ${seen.area.text}, ${seen.area.controls} controls`);
    }
    check('each area says that it is not available, and holds no control that could be taken for a working one',
        unfilled.join() === AREAS.join(), unfilled.join(', '));

    // Each toggle hides its sidebar and shows it again as it was: in its width, and with the area that was selected, the last one.
    const selected = (await layoutOf(first.target)).tabs.find(tab => tab.current)?.label;
    await click(first.target, '#nexees-toggle-right-sidebar');
    const noRight = await layoutWhen(first.target, 'the right sidebar to hide', l => !l.right);
    await click(first.target, '#nexees-toggle-right-sidebar');
    const rightAgain = await layoutWhen(first.target, 'the right sidebar to show', l => !!l.right && near(l.right.w, right.w));
    check('the right toggle hides the right sidebar whole, the editor taking its place, and shows it again in its width and with its area',
        near(noRight.editor.r, noRight.window.w) && noRight.tabs.length === 0 && !noRight.area
        && rightToggle.pressed === 'true' && rightToggle.icon === `codicon-${ICONS.sidebar_right_shown}` && rightToggle.label === 'Hide the right sidebar'
        && noRight.toggles[1].pressed === 'false' && noRight.toggles[1].icon === `codicon-${ICONS.sidebar_right_hidden}`
        && noRight.toggles[1].label === 'Show the right sidebar' && noRight.toggles[1].tip === noRight.toggles[1].label
        && rightAgain.toggles[1].pressed === 'true' && rightAgain.tabs.find(tab => tab.current)?.label === selected && selected !== AREAS[0]
        && near(rightAgain.right.x, rightAgain.editor.r),
        `hidden: ${noRight.toggles[1].label} (${noRight.toggles[1].icon}), editor to ${Math.round(noRight.editor.r)} of ${noRight.window.w}; `
        + `shown: ${Math.round(rightAgain.right.w)} px with ${selected}`);
    await click(first.target, '#nexees-toggle-left-sidebar');
    const noLeft = await layoutWhen(first.target, 'the left sidebar to hide', l => !l.explorer);
    await click(first.target, '#nexees-toggle-left-sidebar');
    const leftAgain = await layoutWhen(first.target, 'the left sidebar to show', l => !!l.explorer && near(l.left.w, left.w));
    check('the left toggle hides the left sidebar, leaving the activity bar, and shows it again in its width',
        !!noLeft.activity && near(noLeft.activity.w, desktop.activity_bar_width) && noLeft.left.w < desktop.activity_bar_width + 2 && near(noLeft.editor.x, noLeft.left.r)
        && leftToggle.pressed === 'true' && leftToggle.icon === `codicon-${ICONS.sidebar_left_shown}` && leftToggle.label === 'Hide the left sidebar'
        && noLeft.toggles[0].pressed === 'false' && noLeft.toggles[0].icon === `codicon-${ICONS.sidebar_left_hidden}`
        && noLeft.toggles[0].label === 'Show the left sidebar' && leftAgain.toggles[0].pressed === 'true',
        `hidden: ${noLeft.toggles[0].label}, ${Math.round(noLeft.left.w)} px left; shown: ${Math.round(leftAgain.left.w)} px`);
    // A toggle shows what its sidebar did, whoever did it: here the activity bar's Explorer icon.
    await click(first.target, '#shell-tab-explorer-view-container');
    const byIcon = await layoutWhen(first.target, 'the Explorer to hide', l => !l.explorer);
    await click(first.target, '#shell-tab-explorer-view-container');
    const byIconAgain = await layoutWhen(first.target, 'the Explorer to show', l => !!l.explorer);
    check('a toggle follows its sidebar: hidden and shown from the activity bar, the left sidebar is shown so by its toggle',
        byIcon.toggles[0].pressed === 'false' && byIcon.toggles[0].icon === `codicon-${ICONS.sidebar_left_hidden}`
        && byIconAgain.toggles[0].pressed === 'true' && byIconAgain.toggles[0].icon === `codicon-${ICONS.sidebar_left_shown}`,
        `${byIcon.toggles[0].label}, then ${byIconAgain.toggles[0].label}`);

    // The window controls of the row.
    await click(first.target, '#maximize-button');
    const maximized = await layoutWhen(first.target, 'the window to fill the screen', l => l.window.maximized && l.window.w > fresh.window.w)
        .catch(() => undefined);
    if (maximized) {
        await click(first.target, '#restore-button');
    }
    const restored = await layoutWhen(first.target, 'the window to have its size again', l => !l.window.maximized && l.window.w === fresh.window.w);
    check('the window controls work: maximised, the window fills the screen and offers to restore it; restored, it has its size again',
        maximized?.window.w === SCREEN.width && maximized.window.h === SCREEN.height && !!maximized.controls.restore && !maximized.controls.maximize
        && near(maximized.controls.close.r, SCREEN.width) && restored.window.h === fresh.window.h && !!restored.controls.maximize && !restored.controls.restore,
        `${fresh.window.w}x${fresh.window.h}, maximised ${maximized?.window.w}x${maximized?.window.h}, restored ${restored.window.w}x${restored.window.h}`);

    // A window too narrow for the whole row. The width is emulated: the page lays itself out for it.
    const NARROW = 420;
    const devtools = await session(first.target);
    let squeezed;
    try {
        await devtools.send('Emulation.setDeviceMetricsOverride', { width: NARROW, height: fresh.window.h, deviceScaleFactor: 0, mobile: false });
        squeezed = await until('the narrow layout', async () => {
            await devtools.send('Page.captureScreenshot', { format: 'png', clip: { x: 0, y: 0, width: 1, height: 1, scale: 1 } });
            const layout = JSON.parse((await devtools.send('Runtime.evaluate', { expression: LAYOUT, returnByValue: true, awaitPromise: true })).result.value);
            return layout.window.w === NARROW ? layout : undefined;
        }, 15_000);
        await devtools.send('Emulation.clearDeviceMetricsOverride', {});
    } finally {
        devtools.close();
    }
    const widened = await layoutWhen(first.target, 'the window\'s own width', l => l.window.w === fresh.window.w && !!l.right && near(l.right.w, right.w));
    check('in a window too narrow for the whole title row the name and the menu give way: the toggles and the window controls stay in view, in one thin row',
        near(squeezed.top.h, desktop.title_bar_height) && squeezed.toggles.length === 2 && squeezed.toggles.every(toggle => near(toggle.w, desktop.sidebar_toggle_width))
        && near(squeezed.toggles[1].r, squeezed.controls.minimize?.x) && near(squeezed.controls.close?.r, NARROW)
        && squeezed.row.every((item, index) => item.x >= 0 && near(item.x, index === 0 ? 0 : squeezed.row[index - 1].r))
        && near(widened.left.w, left.w),
        `at ${NARROW} px: ${squeezed.row.map(item => `${item.id} ${Math.round(item.x)}-${Math.round(item.r)}`).join(', ')}`);

    // 13 and 5: the user hides the right sidebar and closes the window with the row's own control.
    await click(first.target, '#nexees-toggle-right-sidebar');
    await layoutWhen(first.target, 'the right sidebar to hide', l => !l.right);
    await click(first.target, '#close-button');
    const closed = await until('the window to close', async () => first.child.exitCode !== null || first.child.signalCode !== null, 30_000)
        .then(() => true, () => false);
    check('the close control of the title row closes the window', closed);
    if (!closed) {
        await closeWindow(first);
    }

    // 5: closing the window stops a host that was not asked to keep running.
    check('closing the window leaves the host running for its grace period', hostPids().includes(hostPid));
    await until('the host to stop after its grace period', async () => hostPids().length === 0, GRACE_MS + 15_000);
    check('without the opt-in, the host stops after its last window closed (RC-04)',
        hostPids().length === 0 && !fs.existsSync(path.join(channel, 'host.sock')));

    // 6: a host kept running outlives its windows, and reopened windows reattach to it.
    const kept = spawn(hostProgram, ['serve', '--keep-running'], { env: process.env, detached: true, stdio: 'ignore' });
    children.push(kept);
    await until('the kept host', async () => host('status') === 0, 15_000);
    const seenHosts = new Set();
    const themed = [];
    for (let round = 1; round <= 3; round += 1) {
        const window = await openWindow(workspace);
        await attached(window);
        hostPids().forEach(pid => seenHosts.add(pid));
        themed.push(await evaluate(window.target, `document.body.classList.contains('${THEME}')`));
        if (round === 1) {
            // The first window was closed with its right sidebar hidden.
            const reopened = await layoutOf(window.target);
            await click(window.target, '#nexees-toggle-right-sidebar');
            const reshown = await layoutWhen(window.target, 'the right sidebar to show', l => !!l.right && l.tabs.some(tab => tab.current) && !!l.area);
            check('a reopened window comes back as it was closed: the title row, the right sidebar hidden and its toggle saying so; shown again, it has its areas and its width',
                reopened.frame === 'custom' && near(reopened.top?.h, desktop.title_bar_height) && !reopened.right && reopened.toggles[1]?.pressed === 'false'
                && reopened.toggles[0]?.pressed === 'true' && reshown.toggles[1].pressed === 'true'
                && reshown.tabs.map(tab => tab.label).join() === AREAS.join() && near(reshown.right.w, right.w, 1.5),
                `reopened: ${reopened.toggles[1]?.label}; shown again: ${reshown.tabs.map(tab => tab.label).join(', ')} at ${reshown.right.w.toFixed(1)} px`);
        }
        if (round === 3) {
            // In the last of them the user chooses another colour theme and returns. Returning to the
            // default needs no entry in the settings: Theia may remove it or write the default.
            const light = await chooseTheme(window.target, 'Light (Theia)', 'theia-light', ['light']);
            const back = await chooseTheme(window.target, 'Nexees Dark', THEME, [THEME, undefined]);
            check('the user can choose another colour theme and return to Nexees Dark, and the choice is kept in the user\'s settings',
                light === 'light', `workbench.colorTheme was ${light}, then ${back}`);
        }
        await closeWindow(window);
    }
    await sleep(GRACE_MS + 2_000);
    check('three reopened windows found one and the same host (RC-T14)',
        seenHosts.size === 1 && seenHosts.has(kept.pid), [...seenHosts].join(', '));
    check('the kept host outlives its windows (RC-04)', hostPids().includes(kept.pid));
    check('each reopened window wears the Nexees theme again, with no choice of the user behind it',
        themed.length === 3 && themed.every(Boolean), themed.join(', '));
    kept.kill('SIGTERM');
    await until('the kept host to stop', async () => hostPids().length === 0, 10_000);

    // 7: start at login, in this profile only.
    const entry = path.join(process.env.XDG_CONFIG_HOME, 'autostart', 'nexees-host.desktop');
    check('start at login is off by default', host('login-start', 'status') === 1 && !fs.existsSync(entry));
    const enabled = host('login-start', 'enable') === 0 && fs.existsSync(entry) ? fs.readFileSync(entry, 'utf8') : '';
    const command = `${fs.realpathSync(hostProgram)}`;
    check('enabling it writes one user-level entry that runs the installed host to keep running (RC-05)',
        enabled.includes(`Exec="${command}" serve --keep-running`) && host('login-start', 'status') === 0);
    // What the desktop makes of the entry at the next login, without logging anyone in: systemd's
    // generator reads this profile's autostart folder only and writes its units into the profile.
    const units = path.join(profile, 'units');
    fs.mkdirSync(units);
    spawnSync(AUTOSTART_GENERATOR, [units, units, units], {
        env: { PATH: process.env.PATH, XDG_CONFIG_HOME: process.env.XDG_CONFIG_HOME, XDG_CONFIG_DIRS: path.join(profile, 'no-system-entries') },
        stdio: 'ignore',
    });
    const generated = fs.readdirSync(units).filter(name => name.endsWith('.service')).map(name => fs.readFileSync(path.join(units, name), 'utf8'));
    const starts = (generated[0] ?? '').split('\n').filter(line => line.startsWith('ExecStart='));
    check('systemd\'s generator turns the entry into one unit that runs the host in the graphical session and stops it with the session (RC-05)',
        generated.length === 1 && starts.length === 1 && starts[0].includes(command) && starts[0].endsWith(' serve --keep-running')
        && generated[0].includes('\nPartOf=graphical-session.target\n') && generated[0].includes('\nAfter=graphical-session.target\n'),
        `${generated.length} unit(s)`);
    check('disabling it removes exactly that entry (RC-24)',
        host('login-start', 'disable') === 0 && !fs.existsSync(entry) && fs.readdirSync(path.dirname(entry)).length === 0);
    const noSession = { ...process.env };
    delete noSession.XDG_RUNTIME_DIR;
    const outside = ['serve', 'start', 'status'].map(action => spawnSync(hostProgram, [action], { env: noSession, stdio: 'ignore', timeout: 30_000 }).status);
    check('outside a login session, without a runtime folder, the host reports unsupported and starts nothing (RC-05)',
        outside.every(code => code === 4) && hostPids().length === 0, `exit codes ${outside.join(', ')}`);

    // 10: a channel folder open to others, so the host refuses to run. The window keeps trying for
    // its bounded number of times, then says the host is unavailable and why.
    fs.chmodSync(channel, 0o755);
    const refused = await openWindow(workspace);
    const unavailable = await until('the host to be reported unavailable', async () => {
        const seen = await shown(refused.target);
        return seen?.host?.visible && /Nexees host unavailable/.test(seen.host.label) ? seen.host : undefined;
    }, 60_000);
    // Pointing at the entry shows the reason, in Theia's hover.
    await pointAt(refused.target, '#status-bar-nexees-host');
    const reason = await until('the reason, on pointing at the entry', async () => {
        const text = await evaluate(refused.target, `document.querySelector('.theia-hover')?.innerText ?? ''`);
        return /no channel folder private to you/.test(text) ? text.trim() : undefined;
    }, 15_000);
    check('a host that cannot run leaves the window unavailable, with the reason on pointing at the entry, and nothing else starts (RC-06, RC-T03)',
        /no channel folder private to you/.test(unavailable.label) && hostPids().length === 0 && (fs.statSync(channel).mode & 0o777) === 0o755,
        reason);
    fs.chmodSync(channel, 0o700);
    await click(refused.target, '#status-bar-nexees-host');
    await attached(refused);
    check('once the cause is gone, clicking the entry attaches the window to a host (RC-06)', hostPids().length === 1);
    await closeWindow(refused);
    await until('the host to stop after its grace period', async () => hostPids().length === 0, GRACE_MS + 15_000);
} catch (error) {
    check(`the run completed: ${error.message}`, false);
} finally {
    for (const child of children) {
        try {
            process.kill(-child.pid, 'SIGKILL');
        } catch {
            // Already gone.
        }
    }
    kwin?.kill('SIGKILL');
    await sleep(1_000);
    // Anything else this private session started, such as D-Bus activated services, stops too.
    for (const pid of sessionProcesses()) {
        try {
            process.kill(pid, 'SIGKILL');
        } catch {
            // Already gone.
        }
    }
}
console.log(failures.length === 0 ? 'ALL CHECKS PASSED' : `${failures.length} CHECK(S) FAILED`);
process.exit(failures.length === 0 ? 0 : 1);
