# Connect a coding agent

For a deterministic Claude-to-Rho handoff walkthrough, see [Cross-agent handoff](cross-agent-handoff.md).

MemoryWhale exposes local memory through the stdio MCP server `mw-mcp`, and
through `POST /mcp` on `mw-serve` (one JSON-RPC object per request). Use the
[generic MCP contract](../../integrations/generic-mcp/README.md) or a verified
client guide from the [integration matrix](../../integrations/README.md).

After setup, ask the client:

> Use MemoryWhale to check whether I encountered a similar build failure
> before.

MCP gives the agent retrieval and explicit memory-writing tools. It does not
capture every command the agent runs. Automatic execution capture requires a
verified client-specific hook and is called out separately in the matrix.

Agent-authored lessons can influence later retrieval. Review provenance and
confirmation state before treating a conclusion as established evidence.


## Roadmap: an autonomous failure-to-fix loop

The current integration provides MCP tools, a Rho skill, and a limited capture
hook. This section is a shared roadmap, not a claim that those surfaces already
automate a Rho turn. The skill can suggest memory use, but it cannot currently
inject memory into a turn, and the observational hook cannot provide a
pre-compaction callback.

MemoryWhale owns the generic evidence boundary: capture the data a client
actually provides, retrieve relevant memories, preserve provenance, persist
observed results, and expose those capabilities through MCP. Rho (or another
client) must own task lifecycle orchestration, model-context injection, failure
notifications, and pre-compaction callbacks. Those client-side seams are
required before the following loop can be automatic:

1. **Retrieve relevant memory when a task starts.** A client-side task-start
   hook should combine the project and the user's request, ask MemoryWhale for a
   few relevant past fixes, and supply them to Rho. Keep a small context budget
   so memory does not overwhelm the task.
2. **Look up failures automatically.** A client-side failure event should let
   MemoryWhale search matching previous failures and return any supported fix.
   Deduplicate repeated errors so the same failure is not searched on every
   retry.
3. **Save lessons after verification.** After a failed command succeeds following
   a change, the client should propose a memory containing the failure, change,
   and successful verification. MemoryWhale can record observed results, but the
   model's explanation must remain unverified unless the evidence supports it.
4. **Preserve useful findings before compaction.** A client-side pre-compaction
   callback should save unresolved problems, verified fixes, and relevant
   evidence before Rho summarizes its conversation, then retrieve those memories
   when needed afterward.

The target flow is:

```text
failure → retrieve prior evidence → verify a fix → remember the result
```

### Integration boundary and prerequisites

| Responsibility | Owner | Current state |
| --- | --- | --- |
| Capture, redaction, provenance, persistence, and retrieval | MemoryWhale | Available through the existing client-specific paths and MCP |
| Task-start context injection | Rho or another client | Requires a lifecycle hook or explicit client orchestration |
| Failure-triggered lookup | Rho or another client | Requires a failure event that can invoke MCP without duplicate retries |
| Verified-fix proposal | Rho/client plus MemoryWhale | Can be built from observed command outcomes and explicit confirmation |
| Pre-compaction preservation | Rho or another client | Requires a documented pre-compaction callback |

Full command and output capture would strengthen the loop. The current Rho hook
payload is intentionally limited, so MemoryWhale must not imply that every Rho
command was captured. MCP access, command capture, and memory-use guidance
remain separate capabilities until the client-side lifecycle boundaries are
available.
