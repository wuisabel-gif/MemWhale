"""Minimal OpenHands Software Agent SDK config wiring MemoryWhale over local stdio.

Placeholders only. Replace the two absolute paths before running. Verified against
the OpenHands software-agent-sdk example `examples/01_standalone_sdk/07_mcp_integration.py`
at revision bd88f050259276978dc31541d8099d98e4994428 (see VERIFICATION.md).

Scope: this wires a *local* mw-mcp binary reachable from the process that runs
the agent. If OpenHands executes the agent in a Docker workspace, a VM, or a
remote Agent Server, the paths below must resolve *inside that runtime*, not on
your host. That path is not verified here.
"""

from pydantic import SecretStr

from openhands.sdk import LLM, Agent, Conversation
from openhands.sdk.mcp import MCPServer

# `mcp_config` is a dict of name -> MCPServer. MemoryWhale runs as a local stdio
# child process: an absolute path to the built `mw-mcp`, no args, and an isolated
# store selected with MEMORYWHALE_DATA_DIR. Note the SDK types `env` values as
# pydantic `SecretStr`, so the store path is wrapped even though it is not secret.
mcp_config = {
    "memorywhale": MCPServer(
        command="/absolute/path/to/mw-mcp",
        args=[],
        env={
            "MEMORYWHALE_DATA_DIR": SecretStr("/absolute/path/to/a-memorywhale-store"),
        },
    ),
}

# Configure your own model/provider. MemoryWhale never needs these credentials.
llm = LLM(
    usage_id="agent",
    model="<your-model>",
    api_key=SecretStr("<your-provider-key>"),
)

# The agent exposes MemoryWhale's six tools through the SDK's MCP registry:
# recent_errors, search_memory, get_context, remember, similar_failures, stats.
agent = Agent(llm=llm, tools=[], mcp_config=mcp_config)

conversation = Conversation(agent=agent, workspace=".")
conversation.send_message(
    "Use MemoryWhale (search_memory) to check whether I hit this error before. "
    "Report only recorded evidence; do not save a lesson unless I ask."
)
conversation.run()
