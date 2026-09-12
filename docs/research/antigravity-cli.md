# Antigravity CLI compatibility assessment

Research for issue #277, dated **September 12, 2026**. Baseline release:
**1.2.2**, published **2026-09-12 03:51:08 UTC**, neither draft nor prerelease
[1]. Findings below are documentation/release claims, **not runtime
verification**. Current documentation navigation labels the CLI **1.2.0**;
shared documentation is not version-pinned. No Antigravity client was installed or run,
no credentials or user configuration were inspected, and no desktop access
was used.

## Gemini CLI transition

Google's May 19 announcement set June 18, 2026 as the consumer
free/Google AI Pro/Ultra cutoff. Gemini Code Assist Standard/Enterprise licenses
and paid Gemini/Gemini Enterprise Agent Platform API-key access remain
supported: **not all Gemini CLI users migrated** [2]. Migration documentation
provides extension conversion (`agy plugin import gemini`), not interchangeable
manifests [3]. Keep MemoryWhale's existing Gemini guide and distinguish the two
clients.

## Capability matrix

| Area | Documented contract | Consequence for MemoryWhale |
| --- | --- | --- |
| MCP | Local stdio and remote SSE/Streamable HTTP. CLI text also mentions WebSocket without elaborating it in the property table. Entries use `command`, `args`, `env`, `cwd`, or `serverUrl`/`headers`; legacy `url`/`httpUrl` are unsupported. Global configuration is `~/.gemini/config/mcp_config.json`; project configuration is `.agents/mcp_config.json`. OAuth/ADC are documented [4]. | A local stdio smoke test is a plausible first step. Do not copy another client's configuration format or promise every transport. |
| Trust | Unconfigured MCP tools default to Ask; policy patterns include `mcp(server/tool)` and `mcp(server/*)` [4]. Release notes mention workspace-hook reloads after folder trust [9]. | Confirm workspace trust, precedence, and approval behavior in the selected release; do not bypass them. |
| Execution location | The repository documents SSH use and a shared agent engine. Remote Control exposes the running host through a browser while preserving its development environment [7][10]. The announcement describes a server-side harness [2]. | Distinguish where tools/store files live from where model inference runs. A local tool does not imply offline inference. |
| Skills/plugins | CLI documentation describes `.agents/skills/` or `~/.gemini/antigravity-cli/skills/`; bundles have a required root `plugin.json` and optional MCP/hooks/skills/agents/rules. That CLI page also lists the private `~/.gemini/antigravity-cli/plugins/` path [6], but the **tagged 1.2.2 changelog records a 1.0.2 fix moving `plugin` installs to shared `~/.gemini/config/`** [9]. Shared documentation lists `~/.gemini/config/plugins/` and different manifest requirements [8]. | Weight the tagged release history above the conflicting, likely stale private-path page. Shared configuration is the better-supported starting point, but verify the exact discovery path and manifest contract in the pinned runtime before writing an installer. |
| Execution hooks | Shared hooks receive JSON with camelCase `conversationId`, `workspacePaths`, `transcriptPath`; tool events add `toolCall` and `stepIdx`. `run_command` arguments include `CommandLine` and `Cwd`. Post-tool input has optional `error`, but no documented stdout/stderr or numeric process exit field [5]. | Full execution capture is not established. Never manufacture output, exit codes, or snake_case identifiers. Conversation/step correlation still needs an async-execution test. |
| Lifecycle | Documented events include `PreToolUse`, `PostToolUse`, `PreInvocation`, `PostInvocation`, and `Stop`. Stop supplies `executionNum`, `terminationReason`, and `fullyIdle`. Pre-tool hooks can affect approval, invocation hooks inject steps, and Stop can continue execution; the post-tool response is `{}` [5]. | Do not treat all callbacks as passive session boundaries. Any memory adapter should avoid changing permissions or restarting a cancelled/completed task. |
| Privacy/sync | The README describes interaction collection with a settings opt-out and bidirectional preference/permission syncing [10]. Retention periods, deletion guarantees, full sync inventory, and plan-specific treatment were not established. | Never automatically upload the MemoryWhale database. Selected MCP results or injected context can still disclose retrieved data to the agent/provider. |

## Decision and bounded follow-up

**GO:** an opt-in, local `mw-mcp` feasibility test with synthetic data and a
pinned authorized client. First verify configuration loading, process location,
MCP handshake/tool discovery, and one read-only retrieval. Inspect that the
normal store and unrelated client settings remain untouched.

### Required isolation for the future smoke test

**Do not start `mw-mcp` against its default store.** Read-only tools can still
send existing records to an agent/model provider. Use a new temporary store,
seed only synthetic records, and put its absolute `MEMORYWHALE_DATA_DIR` in
the **MCP server's `env`**, not just in the shell that prepares the test.

The following macOS/Linux Bash preparation uses trusted `mw`/`mw-mcp` binaries on PATH
and Python 3. It creates only a disposable project and example configuration;
it does **not** launch or certify Antigravity:

```bash
(
  set -eu
  umask 077
  smoke_root="$(mktemp -d "${TMPDIR:-/tmp}/mw-agy-smoke.XXXXXX")"
  readonly smoke_root
  trap 'rm -rf -- "$smoke_root"' EXIT

  python3 - "$smoke_root" <<'PY'
import json, os, pathlib, shutil, subprocess, sys

root = pathlib.Path(sys.argv[1]).resolve()
store = root / "store"
workspace = root / "project"
config = workspace / ".agents" / "mcp_config.json"
config.parent.mkdir(parents=True)
mw, server = shutil.which("mw"), shutil.which("mw-mcp")
if not mw or not server:
    raise SystemExit("Use trusted, matching mw and mw-mcp binaries on PATH")

scope = {
    "MEMORYWHALE_DATA_DIR": str(store),
    "HOME": str(root / "home"),
    "XDG_CONFIG_HOME": str(root / "config"),
    "XDG_DATA_HOME": str(root / "data"),
}
env = dict(scope, PATH=os.environ.get("PATH", ""))
marker = "MW_AGY_SYNTHETIC_ONLY"
subprocess.run([mw, "remember", marker + ": synthetic smoke-test note"],
               cwd=workspace, env=env, check=True)
config.write_text(json.dumps({"mcpServers": {"memorywhale-smoke": {
    "command": str(pathlib.Path(server).resolve()),
    "args": [],
    "env": scope
}}}, indent=2))
found = subprocess.run([mw, "search", marker, "source:note"],
                       cwd=workspace, env=env, check=True,
                       text=True, capture_output=True)
if marker not in found.stdout:
    raise SystemExit("Synthetic preparation check did not find its marker")
print(found.stdout, end="")
print("Disposable project:", workspace)
print("Example MCP configuration:", config)
print("Explicit synthetic store:", store)
PY

  # Only after verifying the client-isolation requirements below, perform the
  # approved client test while this subshell remains open. Stop the client and
  # its MCP process before continuing to cleanup. No client is started here.
  printf 'After stopping any test client, press Enter to remove the test files: '
  read -r finished
)
```

Before any agent request, inspect the pinned client's effective configuration
and confirm that `memorywhale-smoke` uses that exact server environment and
that **no normal-store MemoryWhale server or other private data source is
available to the test**. A project file alone does not prove that global
configuration was excluded. If the client cannot be isolated or its effective
environment cannot be verified, stop and keep live validation marked pending.

In the authorized client, verify handshake/tool discovery and use only read-only
retrieval against `memorywhale-smoke` for `MW_AGY_SYNTHETIC_ONLY`; the returned
records must be synthetic. The local `mw search` above is a preparation check,
not proof of client MCP retrieval. Do not edit normal client settings or seed
the usual database. Stop the test client/server before leaving the subshell;
its exit trap removes only the newly created test directory. A forced kill can
prevent traps from running, so inspect and clean up that reported directory
after stopping any surviving test process if needed.

**NO-GO for now:** shipping automatic configuration or claiming full command
capture. Resolve the documentation discrepancies first. Shared hooks use
`.agents/hooks.json` / `~/.gemini/config/hooks.json`, while the CLI plugin page
also mentions `settings.json` [5][6]. Separately test output/status availability,
async correlation, trust, and skill discovery before expanding scope.

**Live verification remains pending:** no authorized Antigravity client was
available for this research. The public repository provides documentation,
changelog, and examples, not sufficient runtime source to prove these semantics.
Unavailable privacy/security documentation was not treated as a guarantee.

The next task should be a small MCP-only smoke test, not a full adapter or a
migration of existing Gemini settings. It should report exact client/build,
platform, test data location, and observed behavior before proposing production
setup commands.

## Primary sources

[1]: https://api.github.com/repos/google-antigravity/antigravity-cli/releases/tags/1.2.2
[2]: https://developers.googleblog.com/an-important-update-transitioning-gemini-cli-to-antigravity-cli/
[3]: https://antigravity.google/docs/cli/gcli-migration/
[4]: https://antigravity.google/docs/cli/mcp/
[5]: https://antigravity.google/docs/hooks
[6]: https://antigravity.google/docs/cli/plugins/
[7]: https://antigravity.google/docs/remote-control/
[8]: https://antigravity.google/docs/plugins
[9]: https://raw.githubusercontent.com/google-antigravity/antigravity-cli/1.2.2/CHANGELOG.md
[10]: https://raw.githubusercontent.com/google-antigravity/antigravity-cli/1.2.2/README.md

1. [Release 1.2.2 metadata][1]
2. [Google's transition announcement][2]
3. [CLI migration guide][3]
4. [CLI MCP documentation][4]
5. [Shared hooks documentation][5]
6. [CLI plugins documentation][6]
7. [Remote Control documentation][7]
8. [Shared plugins documentation][8]
9. [Official changelog at tag 1.2.2 (including the 1.0.2 plugin-path fix)][9]
10. [Official repository README at tag 1.2.2][10]
