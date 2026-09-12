# Antigravity CLI compatibility assessment

Research for issue #277, dated **September 12, 2026**. Baseline release:
**1.2.2**, published **2026-09-12 03:51:08 UTC**, neither draft nor prerelease
[1]. Findings below are documentation/release claims, **not runtime
verification**. Current documentation navigation labels the CLI **1.2.0**;
shared documentation is not version-pinned. No client was installed or run,
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
