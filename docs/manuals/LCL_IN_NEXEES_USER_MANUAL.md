<!-- start here -->
# LCL in Nexees User Manual

**Status: pre-implementation draft.** This is the integrated-workspace guide, not a replacement for all standalone LCL documentation. Before publishing, inspect the real LCL core/manual and validate every executable example. No LCL grammar is invented in this draft.

## 1. Optional LCL workspace
Nexees is usable without LCL. Opening LCL uses the existing Nexees editor, tabs, file tree and project shell; it does not launch another full application. The foreground LCL workspace shows its own files even while an agent works on a different coding project.

## 2. Choosing a specification mode
Standard leaves LCL off. Full Project LCL defines the whole intended system. Task-Decomposed LCL provides ordered task units and dependencies. Hybrid LCL combines global master rules/contracts/architecture with the task sequence. Autonomy (Supervised/Balanced/Autonomous/Custom) is separate: a detailed specification does not grant extra tool permissions.

Choose the mode for the intended workspace. Mode conversion must show affected content and validate the result; it must not discard meaning silently. Exact UI actions and supported source grammar must be verified against the implementation.

## 3. Authoring the specification
Use the supported templates/roles for project description, architecture, rules, contracts, tasks, read order, stop conditions and verification. Global requirements should be reusable rather than duplicated in every task. Make completion criteria observable where possible and identify approvals/missing information explicitly.

The LCL core—not arbitrary filenames or a model's guess—defines valid syntax, references and inheritance. Required example projects and error examples will be copied/adapted from actual verified LCL documentation and tested with the pinned version during implementation.

## 4. Importing files and ZIP packages
Select LCL import and choose one/multiple `.lcl` or `.lcl.txt` files, or a ZIP containing multiple files. Choose an existing destination or a new project. Phone-local import must work without a PC. Imported files are initially inspected in protected staging, not extracted over your active project.

Review the proposed file tree, project grouping, diagnostics and path mapping. An already valid hierarchy should remain intact. When a flat set has verified roles, Nexees can propose the appropriate grouping. Unknown roles, duplicate task IDs, conflicting masters or multiple projects require a visible choice; the application must not invent missing instructions.

Confirm only the intended changes. Existing files and unsaved edits require explicit conflict resolution or a new project/copy. Unsafe/corrupt/unsupported archives are rejected with a reason. Canceling or losing power must preserve the previous active revision. Re-import should not silently duplicate tasks.

## 5. Validation, diagnostics and inactive drafts
Validation identifies what the actual pinned core can check. Review file/line diagnostics and required-reference errors. Unsupported core versions or missing files block activation. You may keep an incomplete import as an inactive draft for correction; it is not a ready-to-run specification.

A green syntax/structural check does not prove all natural-language requirements will be satisfied. Nexees still needs task-specific builds/tests/security evidence. Do not confuse LCL validation with executed project verification.

## 6. Generated instructions and agent use
Where the supported LCL core provides compiled/translated instructions, review the output associated with the validated source revision. The agent receives the relevant authoritative specification through Nexees, not an assumed understanding of a new language. The same current project state is retained when providers switch.

Read-order requirements must be satisfied or reported blocked. The default model-switch orientation rereads the required current context before writes; turning off the extra orientation does not remove LCL authority or security gates.

## 7. Changing rules while work runs
Edit a draft without silently changing the active session. Validate, inspect the changes and explicitly adopt the revision at a safe task boundary. Current task evidence remains linked to the specification under which it was produced. If files change during a new model's orientation, Nexees must recheck that state before allowing it to continue.

You may instead open another LCL workspace and prepare a future project. That workspace's file tree becomes visible; the other agent keeps its original root and rules.

## 8. Task progression and stopping
The task graph/read order determines progression in the selected mode. Required verification and cleanup close each coding task. A triggered stop condition, failed build/test, missing required input or permission need blocks affected work. Nexees must not fake completion to advance to the next task.

Some LCL requirements can be checked mechanically; others require tests or explicit review. Mark unsupported checks honestly and keep the task blocked when they are mandatory.

## 9. Android and synchronization
After authorized setup, create/edit/import/save/validate LCL locally on the phone without PC connectivity and, for local validation, without internet. Use the same pinned semantics/templates as Desktop. A cloud LLM still requires its own supported network access.

Selected sync preserves both sides of conflicting edits and does not transmit secrets or automatically adopt new instructions. PC paths are not Android paths; references must use the approved portable workspace mapping. Reconnect does not replay stale approvals or silently move the running agent.

## 10. Troubleshooting and help
A missing reference requires the correct source file or an explicit valid specification edit, not a guessed placeholder. An ambiguous import requires a mapping choice. A core-version mismatch requires a verified compatible core or validated migration. A blocked run requires resolving the reported condition, not disabling safeguards. A wrong tree usually means the foreground workspace differs from the agent's bound workspace.

The published manual must include validator-tested examples for every supported mode, importing a flat and nested ZIP, invalid drafts, safe adoption, phone-local use and conflict recovery. All exact syntax, screenshots, commands and expected outputs remain pending implementation verification. Help must be offline, searchable and linked to the Nexees manual on both clients.
