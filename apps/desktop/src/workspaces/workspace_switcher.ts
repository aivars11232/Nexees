// The window's workspaces (apps/desktop/src/workspaces/workspace_switcher, TASK-013, TASK-014):
// the window shows one workspace of the device's registry, its client's foreground workspace,
// and the user creates, opens, closes and lists workspaces here (C1, C2, C4, B2, ST-WORKSPACE).
//
// - **The foreground decides what the Explorer shows.** When the window starts on a folder, when
//   its folders change, and again whenever it attaches to a host, it asks the host to open the
//   workspace whose root that folder is, and creates a CODE workspace for it, named after the
//   folder, when there is none. Once the host has answered, the Explorer shows the foreground
//   workspace's folder and nothing else (B2, C4, R12). A folder that cannot be a workspace, such
//   as one inside another workspace's root, is not shown: the window closes it, or goes back to
//   the workspace it showed, and says why. So does a window given several folders at once:
//   Theia's commands that add a folder to a window are removed, and a window of several folders
//   is closed the same way, so that a CODE tree and an LCL tree are never one Explorer (B5, B6).
//   A window without a folder shows no workspace.
// - **The title row names it.** The workspace the host confirmed stands in the title row, where
//   the approved layout has it, with its kind; clicking it lists the workspaces. Until a host
//   answers, the row names none and the window shows the folder it started on.
// - **Theia shows it.** Opening another workspace opens its root as Theia's workspace in this
//   window, which then starts again on that folder; closing the workspace closes Theia's. Nexees
//   keeps the identity, the kind and the root; Theia keeps the files, the tree and the editors
//   (reuse first). An LCL workspace's tree is its folder's, as a CODE workspace's is.
// - **Nothing else follows.** Opening, closing or creating a workspace binds no agent and changes
//   no other workspace (C2, R11): it decides what the window shows, nothing more.
// - **Each window is a client of its own** (apps/desktop/src/application_host): its foreground
//   is its own, what another window opens or closes changes nothing here, and when the window
//   goes the host closes the workspace it showed.

import { inject, injectable } from '@theia/core/shared/inversify';
import { Widget } from '@theia/core/shared/@lumino/widgets';
import { FrontendApplicationContribution } from '@theia/core/lib/browser/frontend-application-contribution';
import { Command, CommandContribution, CommandRegistry } from '@theia/core/lib/common/command';
import { FileUri } from '@theia/core/lib/common/file-uri';
import { MessageService } from '@theia/core/lib/common/message-service';
import { QuickInputService } from '@theia/core/lib/common/quick-pick-service';
import { WorkspaceService } from '@theia/workspace/lib/browser/workspace-service';
import { ICONS } from '../shell/design_tokens';
import { HostConnectionService, HostStateEvents, WorkspaceEntry, WorkspaceKind, WorkspaceRefusal, isWorkspaceName } from '../main.protocol';

/** The workspace commands of the command palette. */
export const WORKSPACE_COMMANDS = {
    open: { id: 'nexees.workspace.open', label: 'Nexees: Open Workspace…' },
    create: { id: 'nexees.workspace.create', label: 'Nexees: New Workspace…' },
    close: { id: 'nexees.workspace.close', label: 'Nexees: Close Workspace' },
} satisfies Record<string, Command>;

/** Theia's commands that add a folder to a window: a window shows one workspace, of one folder. */
const ADD_FOLDER_COMMANDS = ['workspace:addFolder', 'navigator.addRootFolder'];

/** The kinds of workspace, as the user chooses among them and as the title row shows them. */
const KINDS: ReadonlyArray<{ kind: WorkspaceKind; label: string; detail: string; icon: string }> = [
    { kind: 'code', label: 'Code', detail: 'A coding project', icon: ICONS.workspace_code },
    { kind: 'lcl', label: 'LCL', detail: 'An LCL project', icon: ICONS.workspace_lcl },
];

/**
 * Where a window keeps what it has to say when it closes a folder it cannot show: the window
 * starts again, and says it then. The session storage of a window lasts as long as the window.
 */
const NOTICE = 'nexees.workspace.notice';

/** Why the window closed the folder it was given, and the workspace it may open instead. */
interface Notice {
    readonly text: string;
    readonly offer?: { readonly workspace_id: string; readonly name: string };
}

/** A refusal of the host, worded for the user; `other` is the workspace a root overlaps. */
export function refusalText(reason: WorkspaceRefusal, other?: WorkspaceEntry): string {
    switch (reason) {
        case 'not_found': return 'There is no such workspace on this device.';
        case 'root_not_absolute': return 'The folder must be given as an absolute path.';
        case 'root_missing': return 'The folder does not exist, or cannot be read.';
        case 'root_not_a_folder': return 'That is a file, not a folder.';
        case 'root_unusable': return 'The folder\'s path is too long, or is not text.';
        case 'root_moved': return 'The workspace\'s folder was moved, or replaced by a link.';
        case 'overlaps': return `The folder shares files with the workspace ${other ? `"${other.name}"` : 'of another folder'}, and a file belongs to one workspace.`;
        case 'not_here': return 'The workspace\'s files are not on this device.';
        case 'not_open': return 'The workspace is not open.';
    }
}

/** The last segment of a path: a folder's own name. */
function folderName(path: string): string {
    return path.split('/').filter(segment => segment.length > 0).at(-1) ?? path;
}

/** The notice the window kept before it started again, once; undefined when there is none. */
function takeNotice(): Notice | undefined {
    try {
        const kept: unknown = JSON.parse(window.sessionStorage.getItem(NOTICE) ?? 'null');
        window.sessionStorage.removeItem(NOTICE);
        if (typeof kept !== 'object' || kept === null || typeof (kept as Notice).text !== 'string') {
            return undefined;
        }
        const { text, offer } = kept as Notice;
        const valid = typeof offer?.workspace_id === 'string' && typeof offer.name === 'string';
        return valid ? { text, offer } : { text };
    } catch {
        return undefined;
    }
}

/**
 * The name of the workspace the window shows, in the title row where the approved layout has it:
 * before the sidebar toggles, with the icon of its kind. Clicking it lists the workspaces.
 */
@injectable()
export class WorkspaceName extends Widget {

    static readonly ID = 'nexees-title-workspace';

    constructor(@inject(CommandRegistry) protected readonly commands: CommandRegistry) {
        super({ node: document.createElement('button') });
        this.id = WorkspaceName.ID;
        this.node.addEventListener('click', () => void this.commands.executeCommand(WORKSPACE_COMMANDS.open.id));
        this.display(undefined);
    }

    /** Names `workspace`, or says that the window shows none. */
    display(workspace: WorkspaceEntry | undefined): void {
        const kind = workspace && KINDS.find(choice => choice.kind === workspace.kind);
        const name = document.createElement('span');
        name.textContent = workspace ? workspace.name : 'No workspace';
        if (kind) {
            const icon = document.createElement('span');
            icon.className = `codicon codicon-${kind.icon}`;
            this.node.replaceChildren(icon, name);
        } else {
            this.node.replaceChildren(name);
        }
        const shown = `nexees-workspace-${workspace?.kind ?? 'none'}`;
        for (const state of ['nexees-workspace-code', 'nexees-workspace-lcl', 'nexees-workspace-none']) {
            this.toggleClass(state, state === shown);
        }
        const said = workspace && kind
            ? `${kind.label} workspace "${workspace.name}", ${workspace.root ?? 'its files on another device'}. Click to open another.`
            : 'This window shows no workspace. Click to open one.';
        this.node.title = said;
        this.node.setAttribute('aria-label', said);
    }
}

/** Keeps the window's folder and its client's foreground workspace one, and offers the workspace commands. */
@injectable()
export class WorkspaceSwitcher implements FrontendApplicationContribution, CommandContribution {

    /** The workspace this window shows, once the host has said which it is. */
    protected shown: WorkspaceEntry | undefined;
    /** The claims in progress, one after the other: two at once could both create the workspace. */
    protected claims: Promise<void> = Promise.resolve();

    constructor(
        @inject(HostConnectionService) protected readonly host: HostConnectionService,
        @inject(HostStateEvents) protected readonly hostEvents: HostStateEvents,
        @inject(WorkspaceService) protected readonly workspaces: WorkspaceService,
        @inject(QuickInputService) protected readonly quickInput: QuickInputService,
        @inject(MessageService) protected readonly messages: MessageService,
        @inject(CommandRegistry) protected readonly commands: CommandRegistry,
        @inject(WorkspaceName) protected readonly title: WorkspaceName,
    ) { }

    onStart(): void {
        const notice = takeNotice();
        if (notice) {
            void this.tell(notice);
        }
        // The window does not wait for the host: the registry follows once a host answers, and
        // every host the window attaches to later is told again, as is every change of its folders.
        this.hostEvents.onChanged(state => {
            if (state.kind === 'attached') {
                this.claimAgain();
            }
        });
        this.workspaces.onWorkspaceChanged(() => this.claimAgain());
        this.claimAgain();
    }

    onDidInitializeLayout(): void {
        // After every contribution has registered its commands, so that Theia's are there to remove.
        ADD_FOLDER_COMMANDS.forEach(id => this.commands.unregisterCommand(id));
    }

    protected claimAgain(): void {
        this.claims = this.claims.then(() => this.claim(), () => this.claim());
    }

    registerCommands(commands: CommandRegistry): void {
        commands.registerCommand(WORKSPACE_COMMANDS.open, { execute: () => this.pickAndShow() });
        commands.registerCommand(WORKSPACE_COMMANDS.create, { execute: () => this.create() });
        commands.registerCommand(WORKSPACE_COMMANDS.close, { execute: () => this.close() });
    }

    /**
     * Makes the folder the window shows its client's foreground workspace, creating the workspace
     * when there is none; a folder that cannot be one is not shown.
     */
    protected async claim(): Promise<void> {
        const roots = await this.workspaces.roots;
        if (roots.length === 0) {
            await this.leave();
            return;
        }
        const only = roots.length === 1 && !this.workspaces.isMultiRootWorkspaceOpened ? roots[0] : undefined;
        if (!only?.isDirectory) {
            await this.refuse({ text: 'A window shows one workspace, and a workspace is one folder. Open each folder in a window of its own.' });
            return;
        }
        const root = FileUri.fsPath(only.resource);
        let answer = await this.host.openWorkspace({ root });
        if (answer?.kind === 'refused' && answer.reason === 'not_found') {
            const name = folderName(root);
            const created = await this.host.createWorkspace({ kind: 'code', name: isWorkspaceName(name) ? name : 'Workspace', root });
            answer = created?.kind === 'done' && created.workspace
                ? await this.host.openWorkspace({ workspace_id: created.workspace.workspace_id }) : created;
        }
        if (answer?.kind === 'done' && answer.workspace) {
            this.display(answer.workspace);
        } else if (answer?.kind === 'refused') {
            const other = await this.entry(answer.workspace_id);
            await this.refuse({
                text: `This folder is not a Nexees workspace. ${refusalText(answer.reason, other)}`,
                offer: other ? { workspace_id: other.workspace_id, name: other.name } : undefined,
            });
        } else {
            // No host answered: the window keeps the folder it started on until one does.
            this.display(undefined);
        }
    }

    /**
     * Does not show the folders the window was given: the window's client shows no workspace,
     * and the window starts again on the workspace it showed before, or on no folder, and then
     * says why.
     */
    protected async refuse(notice: Notice): Promise<void> {
        const before = this.shown;
        await this.leave();
        try {
            window.sessionStorage.setItem(NOTICE, JSON.stringify(notice));
        } catch {
            // Without the notice the window still closes the folder; it only cannot say why.
        }
        if (before?.root && (await this.host.openWorkspace({ workspace_id: before.workspace_id }))?.kind === 'done') {
            this.workspaces.open(FileUri.create(before.root), { preserveWindow: true });
        } else {
            await this.workspaces.close();
        }
    }

    /** Says why the window closed the folder it was given, offering the workspace the folder overlaps. */
    protected async tell(notice: Notice): Promise<void> {
        const offer = notice.offer && `Open "${notice.offer.name}"`;
        const chosen = await this.messages.warn(notice.text, ...(offer ? [offer] : []));
        const target = chosen === offer && notice.offer ? await this.entry(notice.offer.workspace_id) : undefined;
        if (target) {
            await this.show(target);
        }
    }

    /** Closes the workspace the window's client shows, if any: the window then shows none. */
    protected async leave(): Promise<void> {
        const foreground = (await this.host.listWorkspaces())?.foreground;
        if (foreground) {
            await this.host.closeWorkspace(foreground);
        }
        this.display(undefined);
    }

    /** Takes `workspace` as the one the window shows, and names it in the title row. */
    protected display(workspace: WorkspaceEntry | undefined): void {
        this.shown = workspace;
        this.title.display(workspace);
    }

    /** The workspace with the ID `id`, as the host lists it. */
    protected async entry(id: string | null): Promise<WorkspaceEntry | undefined> {
        return id === null ? undefined : (await this.host.listWorkspaces())?.entries.find(entry => entry.workspace_id === id);
    }

    /** Lets the user choose one of the device's workspaces and shows it in this window. */
    protected async pickAndShow(): Promise<void> {
        const list = await this.host.listWorkspaces();
        if (!list) {
            void this.messages.error('The Nexees host is unavailable, and with it the workspaces.');
            return;
        }
        if (list.entries.length === 0) {
            void this.messages.info('This device has no workspace yet. Open a folder, or create one with "Nexees: New Workspace…".');
            return;
        }
        const chosen = await this.quickInput.showQuickPick(list.entries.map(entry => ({
            label: entry.name,
            description: `${entry.kind === 'lcl' ? 'LCL' : 'Code'}${entry.workspace_id === list.foreground ? ' · shown' : ''}`,
            detail: entry.root ?? 'Its files are not on this device.',
            entry,
        })), { placeholder: 'Choose the workspace this window shows' });
        if (chosen) {
            await this.show(chosen.entry);
        }
    }

    /** Opens `entry` with the host, then its root in this window, which starts again on it. */
    protected async show(entry: WorkspaceEntry): Promise<void> {
        if (entry.root === null) {
            void this.messages.error(refusalText('not_here'));
            return;
        }
        if (entry.workspace_id === this.shown?.workspace_id) {
            return;
        }
        // The host opens it first, so a workspace it refuses is never shown.
        const answer = await this.host.openWorkspace({ workspace_id: entry.workspace_id });
        if (answer?.kind !== 'done') {
            void this.messages.error(answer?.kind === 'refused' ? refusalText(answer.reason) : 'The Nexees host is unavailable.');
            return;
        }
        this.workspaces.open(FileUri.create(entry.root), { preserveWindow: true });
    }

    /** Asks for the kind, the folder and the name of a new workspace, creates it and shows it here. */
    protected async create(): Promise<void> {
        const kind = await this.quickInput.showQuickPick(KINDS.map(choice => ({ label: choice.label, detail: choice.detail, kind: choice.kind })),
            { placeholder: 'Choose the kind of the new workspace' });
        if (!kind) {
            return;
        }
        const shownRoot = this.shown?.root;
        const root = await this.quickInput.input({
            prompt: 'The folder of the new workspace, as an absolute path',
            value: shownRoot ? `${shownRoot.slice(0, shownRoot.lastIndexOf('/') + 1)}` : '',
            validateInput: async value => value.startsWith('/') ? undefined : 'Give the folder as an absolute path, which starts with /.',
        });
        if (!root) {
            return;
        }
        const suggested = folderName(root);
        const name = await this.quickInput.input({
            prompt: 'The name of the new workspace',
            value: isWorkspaceName(suggested) ? suggested : '',
            validateInput: async value => isWorkspaceName(value) ? undefined : 'A name is one line of text of at most 256 bytes.',
        });
        if (!name) {
            return;
        }
        const answer = await this.host.createWorkspace({ kind: kind.kind, name, root });
        if (answer?.kind === 'done' && answer.workspace) {
            await this.show(answer.workspace);
            return;
        }
        const other = answer?.kind === 'refused' ? await this.entry(answer.workspace_id) : undefined;
        void this.messages.error(answer?.kind === 'refused'
            ? `The workspace was not created. ${refusalText(answer.reason, other)}` : 'The Nexees host is unavailable.');
    }

    /** Closes the workspace this window shows: its client then shows none, and the window no folder. */
    protected async close(): Promise<void> {
        if (this.shown) {
            await this.host.closeWorkspace(this.shown.workspace_id);
            this.display(undefined);
        }
        await this.workspaces.close();
    }
}
