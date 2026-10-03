# Nexees User Manual

**Status: pre-implementation draft.** This describes the agreed workflow, not a tested released interface. Before release, replace pending details with verified instructions/screenshots for the actual software. Nexees version, supported platforms, manual revision and review date must be filled from release metadata, not guessed here.

## 1. What Nexees does
Nexees keeps projects, tools, tasks and their progress independently of the chosen LLM. You can work manually or allow an agent to execute permitted tasks. Switching models does not by itself create a new project or move execution to another device. LCL is an optional way to configure project descriptions, rules and ordered work.

## 2. Installation and first access
Use the verified Desktop package or Android installer for the release. Exact package names, installation commands and update instructions are pending implementation. Desktop must launch its own installed local application, not depend on a hosted UI website. The renderer/toolkit choice is still an explicit design decision.

Sign in with email or GitHub, or create an account and complete the supported verification/2FA flow. Keep recovery material outside project files and AI chat. Account access, local device unlock, provider credentials and PC pairing are separate. After authorized setup, losing PC/internet access must not lock away permitted local editing; first-time network-based signup is not promised offline.

## 3. Finding your project
Choose the foreground workspace. Its file tree, tabs and editor state become visible. The thin Desktop title bar contains the agreed side-panel controls; Android uses the corresponding compact responsive navigation. The current project/device label matters more than whichever provider name is shown.

An agent in project A can keep working while you open LCL project B. That agent remains bound to A and must not force your tree back to A. Check the agent panel's workspace, device and task before issuing a command.

## 4. Providers and model limits
Configure a supported API, explicitly permitted subscription-backed integration or local endpoint. Provider availability, credentials, model capabilities, context limits and charges depend on the actual selected integration. A subscription is not automatically a general-purpose API, and Nexees does not make paid providers unlimited.

A local endpoint on the PC is not available to the phone when the PC is off. With phone internet, a supported phone-accessible provider may still be used by the phone-hosted agent. Completely offline LLM inference requires a separately supported installed on-device model; it is not guaranteed by local file editing.

## 5. Starting and following work
Select the intended workspace/device, specification mode and autonomy policy. Review the task and permissions, then start. The agent panel shows current actions, tests, blockers and progress. Task completion depends on required evidence, not a model saying Done. Default coding-task quality includes readable explanatory comments, scoped cleanup and verification after cleanup.

Build failures, missing mandatory information, permission needs and unresolved command outcomes may pause work. A notification should explain the blocked operation without switching your foreground workspace. Read-only browsing in an unrelated workspace can continue.

## 6. Switching a model mid-session
When a model reaches its limit or you choose another, use the session's model selector. Nexees pauses/fences old work, protects the current checkpoint and checks the replacement's capabilities. It must not ask the exhausted model for a last response to preserve progress.

**Read-only orientation after model switch is ON by default.** Nexees sends a short kickoff directive and lets the replacement read the required current state/files before allowing edits. The runtime restricts its tools during this step. Successful orientation resumes the unfinished task within your normal autonomy policy; missing files, limits or stale state produce a visible pause instead.

The setting is available in Models & Providers and at session switching, with a visible session override. OFF skips the extra orientation, but Nexees still supplies current-state/policy context and enforces permissions. It does not erase task state or allow unsafe execution. The short kickoff and subsequent reads can consume provider usage; do not expect guaranteed comprehension from a single small message.

## 7. Working with LCL and imports
Open the LCL workspace within Nexees. Import `.lcl` / `.lcl.txt` files or a ZIP of LCL files. Choose the target, review the proposed structure and diagnostics, resolve collisions, and confirm. Valid existing folders/references should be preserved. Nexees asks about ambiguous roles rather than inventing instructions.

Imported content is not automatically run. Invalid sets can remain inactive drafts. Changing a running session's rules requires validation and explicit adoption at a safe checkpoint. See the LCL manual for modes, read order and validation.

## 8. Android without a PC
Open/create phone-local workspaces, browse/edit/save files and prepare/validate LCL. After authorized setup those local operations and both manuals work without PC connectivity; core local validation also works in airplane mode. Supported phone-hosted agents use only the phone's actual available tools. Missing desktop-only compilers/extensions are reported rather than marked verified.

Android may stop background processes. Saved checkpoints/local edits must survive restart; continuous unlimited background execution is not promised. Exact device/OS requirements and tested limitations belong in the implemented release manual.

## 9. Remote control and synchronization
Pair only with the intended trusted Desktop device. Remote commands target a specific device/workspace/agent and require acknowledgement; no acknowledgement means unknown/pending, not confirmed success. Revoke a lost device's trust through verified controls.

Select what to synchronize. Sync copies approved content and portable state, not secrets, permission grants or executable approvals. Review conflicts, including conflicting deletes, before replacement. Unsaved/local-only changes are preserved. Syncing files does not move the execution owner or automatically adopt new LCL rules.

## 10. Editing, terminal, Git and extensions
Use the existing editor/terminal/Git surfaces within the selected workspace. Confirm command location and review destructive actions. Compatible extensions come through the approved source with their actual license/platform restrictions; neither every VS Code extension nor desktop extensions on Android are guaranteed. An extension does not automatically receive Nexees agent authority.

## 11. Security and Developer Settings
Keep permissions as narrow as the current project requires. Reauthentication may protect sensitive account/security changes. Debug logs must redact credentials. Advanced settings do not bypass the fixed workspace, host or read-only boundaries. Never paste recovery codes or provider tokens into tasks or comments.

## 12. Recovery, cleanup and updates
On a failure, inspect the saved task/checkpoint, actual file state and last confirmed operation before retrying. Do not assume an interrupted shell command did nothing. Restoring a checkpoint must protect unrelated user changes. Cleanup removes task-owned obsolete/temporary work, not arbitrary untracked files or shared caches. Test again after cleanup.

Use only the verified update workflow and preserve unsynced phone data. Required migration/recovery steps and exact errors are to be documented from implementation tests.

## 13. Troubleshooting and help
Wrong tree: check foreground workspace versus the agent's bound workspace. No remote control: check pairing, connectivity and target acknowledgement; local Android work should remain available. Quota reached: choose a supported model and inspect orientation status. Invalid import: inspect paths, roles, version and missing references. Conflicts: retain both versions until a reviewed decision. Missing test backend: choose a supported execution environment explicitly, not by switching only the model.

Both manuals must be available from in-app Help with contents, search and accessible text size. Exact final labels, screenshots and installation/provider-specific steps remain pending verification; release cannot ship this draft as a finished manual.


## Planned Devices & Remote Access (v0.5, verify against implementation)
Remote receiving, startup/login/resume and keeping access after window close are explicit settings. Fresh devices receive no control requests until enabled and paired. Each direction has independent grants. A phone can request a permitted PC app launch with the PC window closed only while a supported receiver is available; a PC can request permitted phone-local Nexees actions. Android may require foreground/user interaction. Stop access, revoke devices, inspect normalized targets and audit outcomes. Launching HopToDesk does not itself establish an unattended screen session. The governing requirements are RC-01–RC-26 and the acceptance scenarios RC-T01–RC-T18 of the specification frozen in `docs/charter/`. This section describes required behavior, not an already delivered feature.
