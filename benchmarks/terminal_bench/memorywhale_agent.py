"""Harbor agent: Claude Code with MemoryWhale memory carried across tasks.

Run it next to Harbor's stock ``claude-code`` agent on the same Terminal-Bench
tasks, model, and settings. The only difference is MemoryWhale:

* ``mw-mcp`` is registered as an MCP server (the six memory tools).
* Claude Code's Bash ``PostToolUse``/``PostToolUseFailure`` hooks record every
  command into the store, the same capture a real user gets from
  ``mw integrate claude``.
* One store is carried from task to task: it is uploaded before each trial and
  downloaded after it, so later tasks can recall what earlier tasks hit.
* A short system-prompt addition tells the agent the memory exists.

The carried store is a host file, so trials must run one at a time
(``-n 1``) and once per task (``-k 1``). See README.md in this directory.
"""

from __future__ import annotations

import json
import shlex
from pathlib import Path

from pydantic import Field

from harbor.agents.installed.claude_code import ClaudeCode, ClaudeCodeOptions
from harbor.environments.base import BaseEnvironment
from harbor.models.agent.context import AgentContext
from harbor.models.task.config import MCPServerConfig

DATA_DIR = "/opt/memorywhale-data"
STORE = f"{DATA_DIR}/memorywhale.sqlite3"
RELEASES = "https://github.com/wuisabel-gif/MemWhale/releases/download"

MEMORY_PROMPT = (
    "You have a local debugging memory, MemoryWhale, available through the "
    "memorywhale MCP tools. It holds commands, errors, and fixes from earlier "
    "tasks on similar systems. When a command fails, check similar_failures or "
    "search_memory before debugging from scratch. When you find a non-obvious "
    "fix, save it with remember."
)


class MemoryWhaleClaudeCodeOptions(ClaudeCodeOptions):
    store_dir: str = Field(
        default="benchmarks/terminal_bench/store",
        description="Host directory holding the store carried between trials.",
    )
    mw_version: str = Field(default="0.13.0", description="MemoryWhale release to install.")


class MemoryWhaleClaudeCode(ClaudeCode):
    options_model = MemoryWhaleClaudeCodeOptions
    options: MemoryWhaleClaudeCodeOptions

    @staticmethod
    def name() -> str:
        return "memorywhale-claude-code"

    @property
    def _host_store(self) -> Path:
        return Path(self.options.store_dir).resolve()

    async def install(self, environment: BaseEnvironment) -> None:
        await super().install(environment)
        v = self.options.mw_version
        # Install the published release, checksum-verified, and wrap the two
        # binaries Claude Code calls so both always use the carried store.
        await self.exec_as_root(
            environment,
            command=(
                "set -euo pipefail; "
                'case "$(uname -m)" in x86_64|amd64) a=x86_64;; aarch64|arm64) a=aarch64;; '
                '*) echo "unsupported arch $(uname -m)" >&2; exit 1;; esac; '
                f"n=memorywhale-{v}-$a-unknown-linux-gnu; cd /tmp; "
                f'curl -fsSLO "{RELEASES}/v{v}/$n.tar.gz" -O "{RELEASES}/v{v}/$n.tar.gz.sha256"; '
                'sha256sum -c "$n.tar.gz.sha256"; tar xzf "$n.tar.gz"; '
                'install -m 755 "$n"/bin/mw "$n"/bin/mw-mcp "$n"/bin/mw-remember /usr/local/bin/; '
                f"mkdir -p {DATA_DIR} && chmod 777 {DATA_DIR}; "
                "for b in mw-mcp mw-remember; do "
                f"printf '#!/bin/sh\\nMEMORYWHALE_DATA_DIR={DATA_DIR} exec /usr/local/bin/%s \"$@\"\\n' \"$b\" "
                '> "/usr/local/bin/$b-bench"; chmod 755 "/usr/local/bin/$b-bench"; done; '
                # Fail loudly: a "with memory" score where memory never ran is worse than no score.
                f"MEMORYWHALE_DATA_DIR={DATA_DIR} mw --version"
            ),
        )

    def _build_register_mcp_servers_command(self) -> str | None:
        # Harbor writes the MCP servers here; also drop in the capture hooks.
        hook = {"type": "command", "command": "mw-remember-bench --from-hook claude"}
        group = [{"matcher": "Bash", "hooks": [hook]}]
        settings = json.dumps({"hooks": {"PostToolUse": group, "PostToolUseFailure": group}})
        hooks_cmd = f"echo {shlex.quote(settings)} > $CLAUDE_CONFIG_DIR/settings.json"
        mcp_cmd = super()._build_register_mcp_servers_command()
        return f"{mcp_cmd} && {hooks_cmd}" if mcp_cmd else hooks_cmd

    async def run(
        self, instruction: str, environment: BaseEnvironment, context: AgentContext
    ) -> None:
        if not any(s.name == "memorywhale" for s in self.mcp_servers):
            self.mcp_servers.append(
                MCPServerConfig(name="memorywhale", transport="stdio", command="mw-mcp-bench")
            )
        if not self.options.append_system_prompt:
            self.options.append_system_prompt = MEMORY_PROMPT

        # The whole data dir travels, so SQLite's -wal/-shm sidecars come along.
        if (self._host_store / "memorywhale.sqlite3").exists():
            await environment.upload_dir(self._host_store, DATA_DIR)
            await self.exec_as_root(environment, command=f"chmod -R a+rw {DATA_DIR}")
        try:
            await super().run(instruction, environment, context)
        finally:
            # Carry whatever this trial learned into the next one, even on failure.
            try:
                await self.exec_as_root(environment, command=f"test -f {STORE}")
            except RuntimeError:
                return  # nothing was recorded; keep the previous store as-is
            self._host_store.mkdir(parents=True, exist_ok=True)
            await environment.download_dir(DATA_DIR, self._host_store)
