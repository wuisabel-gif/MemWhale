// mw-mcp: a Model Context Protocol server over stdio, so an AI agent (Claude
// Code, Codex, Cursor, …) can query your MemoryWhale memory directly instead of
// pasting it in. Speaks newline-delimited JSON-RPC 2.0 on stdin/stdout.
//
// Register with Claude Code:
//   claude mcp add memorywhale -- mw-mcp
//
// Tools exposed: recent_errors, search_memory, get_context, remember,
// similar_failures, stats. HTTP MCP lives on mw-serve at POST /mcp.

fn main() {
    if std::env::args().nth(1).as_deref() == Some("--list-tools") {
        for name in memorywhale_cli::mcp::tool_names() {
            println!("{name}");
        }
        return;
    }
    memorywhale_cli::mcp::serve_stdio(std::io::stdin().lock(), std::io::stdout());
}
