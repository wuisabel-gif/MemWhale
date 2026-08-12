# Configuration reference

MemoryWhale is usable without a configuration file. Runtime behavior is
controlled through CLI flags, environment variables, shell-hook state, and
client-specific MCP configuration.

- CLI flags and commands: [CLI reference](cli.md)
- Environment variables: [Environment variables](environment-variables.md)
- MCP server interface: [MCP reference](mcp.md)
- Client configuration: [Integration matrix](../../integrations/README.md)

Client configuration belongs to the external client and should remain a thin
adapter around the `mw-mcp` stdio interface.
