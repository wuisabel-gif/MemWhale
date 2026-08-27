# Jan Desktop + MemoryWhale

## Status

Jan supports local MCP servers through its desktop interface. This guide is
based on Jan's official MCP documentation, which describes the setup path as
**Settings → MCP Servers**. Verify the labels against the Jan version you use;
desktop UI details may change.

- [Jan MCP documentation](https://github.com/janhq/jan/blob/dev/docs/src/pages/docs/desktop/mcp.mdx)
- [MemoryWhale MCP reference](../../docs/reference/mcp.md)

## Requirements

- MemoryWhale installed with `mw-mcp` available on `PATH`.
- Jan Desktop with MCP support enabled.
- A Jan model with tool-calling support. Jan's documentation notes that local
  models may need tool calling enabled in their model capabilities.
- Optional: a deliberate `MEMORYWHALE_DATA_DIR` value if Jan should use a
  non-default store.

## Capabilities

| Capability | Available |
| --- | --- |
| MCP memory access | Yes |
| Automatic execution capture | No |
| Memory-use guidance | No built-in Jan-specific guidance; use the example prompt below |

## Setup

1. Confirm that the server works in the same environment Jan will use:

   ```bash
   command -v mw-mcp
   printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"server/discover","params":{"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientCapabilities":{}}}}' | mw-mcp
   ```

2. Open **Settings → MCP Servers** in Jan.
3. Choose **+** to add a server.
4. Enter:

   ```text
   Server name: memorywhale
   Command: mw-mcp
   Arguments: leave empty
   Environment variables: leave empty for the default store
   ```

   To select another store, add:

   ```text
   MEMORYWHALE_DATA_DIR=/path/to/store
   ```

5. Save or enable the server and restart Jan if it does not appear immediately.
6. Review Jan's MCP permission setting. Jan documents per-tool approval and an
   optional **Allow All MCP Tool Permissions** setting; approving all tools is a
   deliberate trust decision.

## Verify

Ask Jan:

> Use MemoryWhale to check whether I have seen this failure before. Search for
> `openssl` and explain which saved evidence is relevant before suggesting a fix.

The MemoryWhale server exposes six tools:

- `recent_errors`
- `search_memory`
- `get_context`
- `remember`
- `similar_failures`
- `stats`

If Jan shows the server but does not call tools, select a model with tool-calling
support and check Jan's MCP permissions. An empty MemoryWhale store is valid;
`stats` should report zero records rather than a connection failure.

## How to use

Use retrieval when a build, test, or deployment failure may have happened
before. Once the cause or fix is confirmed, use `remember` to save the lesson so
future sessions can find it.

## Automatic capture

Jan's MCP connection does not automatically record Jan prompts, responses, or
shell commands. MCP provides access to MemoryWhale's local memory and an
explicit `remember` tool. Capture commands separately with normal terminal
capture or an explicit wrapper such as:

```bash
mw-run -- cargo test
```

## Security and limitations

- `mw-mcp` is a local stdio process. Jan and the selected model can access the
  MemoryWhale store through its tools, so only connect stores you intend Jan to
  use.
- Review tool arguments and results before enabling broad approval settings.
- Secret redaction reduces accidental retention but is not a security boundary.
- MCP access does not make the database remotely accessible and does not add
  synchronization.
- The selected model must support tool calling; not every local model does.
- Read the canonical [local stdio trust model](../../docs/reference/mcp.md#trust-model)
  before connecting a sensitive store.

## Troubleshooting

- Run `command -v mw-mcp` from the environment used to launch Jan.
- Use the absolute path to `mw-mcp` if Jan cannot find your shell's `PATH`.
- Set `MEMORYWHALE_DATA_DIR` in Jan's server entry, not only in an unrelated
  terminal session.
- Restart Jan after changing the MCP server entry.
- Run `mw doctor` and the direct JSON-RPC discovery command above.
- Check that the model has tool calling enabled in Jan's model capabilities.

## Remove integration

Remove the `memorywhale` entry from **Settings → MCP Servers** and restart Jan.
Removing the entry does not delete the MemoryWhale database or its captured
data.
