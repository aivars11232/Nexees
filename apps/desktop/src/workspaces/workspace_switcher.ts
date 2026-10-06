// The window's workspaces (apps/desktop/src/workspaces/workspace_switcher, TASK-013): the window
// shows one workspace of the device's registry, its client's foreground workspace, and the user
// creates, opens, closes and lists workspaces here (C1, C2, B2, ST-WORKSPACE).
//
// - **The folder the window shows is a workspace.** When the window starts on a folder, and again
//   whenever it attaches to a host, it asks the host to open the workspace whose root that folder
//   is, and creates a CODE workspace for it, named after the folder, when there is none. Once the
//   host has answered, the tree the window shows is its client's foreground workspace's (B2, C4),
//   with one exception: a folder that cannot be a workspace, such as one inside another
//   workspace's root, is shown with a warning while the client shows no workspace (TASK-014). A
//   window without a folder shows no workspace either: the one its client showed is closed.
// - **Theia shows it.** Opening another workspace opens its root as Theia's workspace in this
//   window, which then starts again on that folder; closing the workspace closes Theia's. Nexees
//   keeps the identity, the kind and the root; Theia keeps the files, the tree and the editors
//   (reuse first). An LCL workspace's tree is its folder's, as a CODE workspace's is (B5, B6).
// - **Nothing else follows.** Opening, closing or creating a workspace binds no agent and changes
//   no other workspace (C2, R11): it decides what the window shows, nothing more.
// - **One client for now.** Every window of the user is the host's one Desktop client, so the
//   client's foreground is the workspace opened last in any of its windows (TASK-014).

import { inject, injectable } from '@theia/core/shared/inversify';
import { FrontendApplicationContribution } from '@theia/core/lib/browser/frontend-application-contribution';
import { Command, CommandContribution, CommandRegistry } from '@theia/core/lib/common/command';
import { FileUri } from '@theia/core/lib/common/file-uri';
import { MessageService } from '@theia/core/lib/common/message-service';
import { QuickInputService } from '@theia/core/lib/common/quick-pick-service';
import { WorkspaceService } from '@theia/workspace/lib/browser/workspace-service';
import { HostConnectionService, HostStateEvents, WorkspaceEntry, WorkspaceKind, WorkspaceRefusal, isWorkspaceName } from '../main.protocol';

/** The workspace commands of the command palette. */
export const WORKSPACE_COMMANDS = {
    open: { id: 'nexees.workspace.open', label: 'Nexees: Open Workspace…' },
    create: { id: 'nexees.workspace.create', label: 'Nexees: New Workspace…' },
    close: { id: 'nexees.workspace.close', label: 'Nexees: Close Workspace' },
} satisfies Record<string, Command>;

/** The kinds of workspace, as the user chooses among them. */
const KINDS: ReadonlyArray<{ kind: WorkspaceKind; label: string; detail: string }> = [
    { kind: 'code', label: 'Code', detail: 'A coding project' },
    { kind: 'lcl', label: 'LCL', detail: 'An LCL project' },
];

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
    ) { }

    onStart(): void {
        // The window does not wait for the host: the registry follows once a host answers, and
        // every host the window attaches to later is told again.
        this.hostEvents.onChanged(state => {
            if (state.kind === 'attached') {
                this.claimAgain();
            }
        });
        this.claimAgain();
    }

    protected claimAgain(): void {
        this.claims = this.claims.then(() => this.claim(), () => this.claim());
    }

    registerCommands(commands: CommandRegistry): void {
        commands.registerCommand(WORKSPACE_COMMANDS.open, { execute: () => this.pickAndShow() });
        commands.registerCommand(WORKSPACE_COMMANDS.create, { execute: () => this.create() });
        commands.registerCommand(WORKSPACE_COMMANDS.close, { execute: () => this.close() });
    }

    /** Makes the folder the window shows its client's foreground workspace, creating the workspace when there is none. */
    protected async claim(): Promise<void> {
        const roots = await this.workspaces.roots;
        const only = roots.length === 1 && !this.workspaces.isMultiRootWorkspaceOpened ? roots[0] : undefined;
        if (!only?.isDirectory) {
            await this.leave();
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
            this.shown = answer.workspace;
        } else if (answer?.kind === 'refused') {
            const other = await this.entry(answer.workspace_id);
            await this.leave();
            void this.messages.warn(`This folder is not a Nexees workspace. ${refusalText(answer.reason, other)}`);
        }
    }

    /** Closes the workspace the window's client shows, if any: the window then shows none. */
    protected async leave(): Promise<void> {
        const foreground = (await this.host.listWorkspaces())?.foreground;
        if (foreground) {
            await this.host.closeWorkspace(foreground);
        }
        this.shown = undefined;
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
            this.shown = undefined;
        }
        await this.workspaces.close();
    }
}
