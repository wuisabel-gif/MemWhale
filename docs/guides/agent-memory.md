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


## Next step: an autonomous failure-to-fix loop

The current integration provides MCP tools, a Rho skill, and a limited capture
hook. The skill can suggest memory use, but the most useful next step for
MemoryWhale is making that loop reliable and evidence-first:

1. **Retrieve relevant memory when a task starts.** Use the project and the
   user's request to find a few relevant past fixes and supply them to Rho.
   Keep a small context budget so memory does not overwhelm the task.
2. **Look up failures automatically.** When a command fails, search for matching
   previous failures and show Rho any supported fix. Deduplicate repeated errors
   so the same failure is not searched on every retry.
3. **Save lessons after verification.** When a failed command succeeds after a
   change, propose a memory containing the failure, change, and successful
   verification. Record observed results automatically, but mark the model's
   explanation as unverified unless the evidence supports it.
4. **Preserve useful findings before compaction.** Save unresolved problems,
   verified fixes, and relevant evidence before Rho summarizes its conversation.
   Retrieve those memories when needed afterward.

The target flow is:

```text
failure → retrieve prior evidence → verify a fix → remember the result
```

Full command and output capture would strengthen this loop. The current Rho
hook payload is intentionally limited, so MemoryWhale must not imply that every
agent command was captured. MCP access, command capture, and memory-use guidance
remain separate capabilities until a future integration closes that boundary.
