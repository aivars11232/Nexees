// Starts the TASK-003 Theia Electron build, waits until the workbench has
// rendered, reports what the page shows, then stops the app:
//
//   run_nested.sh <kwin-log> node check_workbench.mjs <theia-app-folder> <workspace-folder>
//
// It runs only inside run_nested.sh's private session, so no window reaches
// the owner's screen and the app gets none of the owner's session bus,
// keyring or settings. The DevTools endpoint binds to 127.0.0.1 only.
import { spawn } from 'node:child_process';
import { setTimeout as sleep } from 'node:timers/promises';

if (process.env.NEXEES_PRIVATE_SESSION !== '1') {
  console.error('run this through run_nested.sh');
  process.exit(2);
}
const [app, workspace] = process.argv.slice(2);
const port = 9333;
// Its own process group, so every helper process stops with it.
const child = spawn(`${app}/node_modules/electron/dist/electron`,
  ['--ozone-platform=wayland', `--remote-debugging-port=${port}`, `${app}/lib/backend/electron-main.js`, workspace],
  { cwd: app, env: { ...process.env, ELECTRON_ENABLE_LOGGING: '1' }, detached: true, stdio: ['ignore', 'pipe', 'pipe'] });
let log = '';
child.stdout.on('data', (d) => { log += d; });
child.stderr.on('data', (d) => { log += d; });

async function pages() {
  try {
    return await (await fetch(`http://127.0.0.1:${port}/json`, { signal: AbortSignal.timeout(3000) })).json();
  } catch {
    return [];
  }
}

function within(ms, promise) {
  return Promise.race([promise, sleep(ms).then(() => { throw new Error(`no answer within ${ms} ms`); })]);
}

async function evaluate(socketUrl, expression) {
  const socket = new WebSocket(socketUrl);
  try {
    await within(3000, new Promise((resolve, reject) => { socket.onopen = resolve; socket.onerror = reject; }));
    socket.send(JSON.stringify({ id: 1, method: 'Runtime.evaluate', params: { expression, returnByValue: true } }));
    const reply = await within(5000, new Promise((resolve) => {
      socket.onmessage = (event) => { const message = JSON.parse(event.data); if (message.id === 1) resolve(message); };
    }));
    return reply.result?.result?.value;
  } finally {
    socket.close();
  }
}

const probe = `JSON.stringify({
  title: document.title,
  shell: !!document.querySelector('#theia-app-shell'),
  statusBar: !!document.querySelector('#theia-statusBar'),
  tabs: [...document.querySelectorAll('.lm-TabBar-tabLabel')].map((n) => n.textContent).filter(Boolean),
  explorerFiles: [...document.querySelectorAll('.theia-TreeNodeSegment')].map((n) => n.textContent).slice(0, 10),
})`;
let rendered = null;
const deadline = Date.now() + 180_000;
while (!rendered && Date.now() < deadline && child.exitCode === null) {
  for (const page of (await pages()).filter((p) => p.type === 'page')) {
    try {
      const value = JSON.parse(await evaluate(page.webSocketDebuggerUrl, probe) ?? 'null');
      if (value?.shell) rendered = { url: page.url, ...value };
    } catch { /* the page is still loading */ }
  }
  if (!rendered) await sleep(1000);
}
// Give the explorer a moment to list the workspace, then read it again.
if (rendered) {
  await sleep(5000);
  const page = (await pages()).find((p) => p.url === rendered.url);
  if (page) rendered = { url: page.url, ...JSON.parse(await evaluate(page.webSocketDebuggerUrl, probe)) };
}
// Quit the way a user closing the app would, so Theia stops its backend,
// which runs in a process group of its own. Kill only as a fallback.
const helpers = (await import('node:child_process')).execFileSync('pgrep', ['-P', String(child.pid)], { encoding: 'utf8' })
  .split('\n').filter(Boolean).map(Number);
try {
  const browser = await (await fetch(`http://127.0.0.1:${port}/json/version`, { signal: AbortSignal.timeout(3000) })).json();
  const socket = new WebSocket(browser.webSocketDebuggerUrl);
  await within(3000, new Promise((resolve, reject) => { socket.onopen = resolve; socket.onerror = reject; }));
  socket.send(JSON.stringify({ id: 1, method: 'Browser.close' }));
} catch { /* fall through to the kill below */ }
for (let i = 0; i < 20 && child.exitCode === null && child.signalCode === null; i++) await sleep(500);
const graceful = child.exitCode !== null;
for (const group of [child.pid, ...helpers]) {
  try { process.kill(-group, 'SIGKILL'); } catch { /* already gone */ }
}
console.log(JSON.stringify({ rendered, graceful, exitCode: child.exitCode, signal: child.signalCode, logTail: log.trim().split('\n').slice(-30) }, null, 1));
process.exit(rendered?.shell ? 0 : 1);
