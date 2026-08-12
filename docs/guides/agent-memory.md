# Connect a coding agent

MemoryWhale exposes local memory through the stdio MCP server `mw-mcp`. Use the
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
