<!-- start here -->
# Security status

This is an unimplemented source-layout scaffold, not a security-tested application.
The source slots do not provide an existing sandbox, auth system, encrypted transport,
credential store or secure updater.

Implement and verify security alongside every capability. UI workspace selection
must not change an agent's execution device or project authority. Extension content,
repository text and imported LCL are not permission grants. Remote approvals must be
bound to the intended device, workspace, session, action and current state.

Account access, device pairing and model-provider credentials are separate concerns.
Keep secrets, recovery codes, private keys, real conversations and private workspace
data out of the repository, logs, prompts and test fixtures. The packaging signing
configuration slot must never contain private signing material.

A private vulnerability-reporting contact has not been supplied. Establish one before
public release; this archive does not invent a reporting address or service promise.
