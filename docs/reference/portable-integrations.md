# Portable client integrations

Connect the same MemoryWhale store to different coding clients, and optionally
install a reviewed response-style skill alongside it. These are **Interfaces**
adapters: no new database schema, model provider, or skill marketplace.

**Availability:** these commands are development-branch functionality, not part
of the published 0.10.0 binaries. Build the current source checkout first:

```sh
cargo build --release --locked -p memorywhale-cli --bins
export MW="$PWD/target/release/mw"
```

No command below downloads a third-party skill, launches a coding client, or
changes that client's approval mode. Connecting an MCP server later lets the
client read memory and potentially send retrieved context to its model provider.

## 1. Connect memory

Rho's existing adapter installs MCP, the bundled memory skill, and its optional
capture hook:

```sh
"$MW" integrate rho
```

The new Codex and Cursor adapters install **MCP access only**:

```sh
"$MW" integrate codex --dry-run
"$MW" integrate codex
"$MW" integrate codex --check

"$MW" integrate cursor --dry-run
"$MW" integrate cursor
"$MW" integrate cursor --check
```

| Client | Default MCP configuration | Other components |
| --- | --- | --- |
| Rho | `$RHO_HOME/config.toml`, otherwise `~/.rho/config.toml` | Existing hook and memory-skill installer; see the Rho guide |
| Codex CLI | `$CODEX_HOME/config.toml`, otherwise `~/.codex/config.toml` | No capture hook or bundled memory skill installed by this adapter |
| Cursor | `~/.cursor/mcp.json` | No capture hook or bundled memory skill installed by this adapter |

The new adapters resolve an absolute local `mw-mcp` executable, preferring the
sibling binary. If `MEMORYWHALE_DATA_DIR` is set, that explicit location is added
to the server's environment. Other environment variables and credentials are
not copied into the entry. Relative data-directory overrides are anchored to
the installer's working directory before saving, so a client launched from a
different directory still reaches the intended store.

For an explicit project configuration or a test fixture:

```sh
"$MW" integrate cursor --config .cursor/mcp.json --dry-run
"$MW" integrate cursor --config .cursor/mcp.json
```

`--config` chooses the file MemoryWhale edits; it does not make an arbitrary file
discoverable by a client. Use a documented client configuration location.
`--check` verifies local configuration and ownership, **not a live connection**.
Reload the client, inspect its MCP tools, and perform a retrieval to test that.
MCP access is not automatic command capture.

## 2. Add optional response guidance

Create and review a local file named `SKILL.md`, for example `style/SKILL.md`:

```markdown
---
name: concise-response
description: Use when the user explicitly requests a concise debugging answer.
license: MIT
---
Lead with the next action and use at most three numbered steps.
Distinguish retrieved evidence, a proposed fix, and observed verification.
Never hide uncertainty or infer health information from a style preference.
```

Then choose a client:

```sh
"$MW" integrate rho --skill ./style/SKILL.md --dry-run
"$MW" integrate rho --skill ./style/SKILL.md
"$MW" integrate rho --skill ./style/SKILL.md --check

"$MW" integrate codex --skill ./style/SKILL.md
"$MW" integrate cursor --skill ./style/SKILL.md
```

`--skill` is **skill-only mode**. It does not also install or remove MCP, capture
hooks, authorization files, or the bundled `memorywhale` skill. `--http`,
`--token`, and MCP `--config` cannot be combined with it.

| Client | Default loose-skill directory |
| --- | --- |
| Rho | `~/.rho/skills` |
| Codex CLI | `~/.agents/skills` |
| Cursor | `~/.cursor/skills` |

The shared `.agents/skills` layout is also useful for project-local setup:

```sh
"$MW" integrate rho --skill ./style/SKILL.md --skills-dir .agents/skills
"$MW" integrate rho --skill ./style/SKILL.md --skills-dir .agents/skills --check
```

**Rho 2.9.1 compatibility:** the live test found that a custom `RHO_HOME` changes
configuration/state locations but does not redirect loose user-skill discovery.
The optional-skill adapter therefore requires `--skills-dir` when `RHO_HOME`
points somewhere other than `~/.rho`. A project `.agents/skills` directory was
verified with the running client. This does not change the legacy Rho setup
command; check skill discovery separately when using a custom Rho home.

A custom root must be a directory the client actually discovers. User-level or
built-in skills can take precedence over a project copy with the same name.
Installation success is not proof of discovery, invocation, or model compliance.

### Third-party skills

Review the source and license yourself before supplying a file. For example,
[ayghri/i-have-adhd](https://github.com/ayghri/i-have-adhd) uses optional metadata
and requests manual invocation. Its instructions are not medical guidance
endorsed by MemoryWhale, and selecting it is not evidence of a diagnosis.

The installer preserves accepted metadata verbatim:

- Required string `name` and `description`; nonempty Markdown body.
- Optional string `license`, `compatibility`, and string-to-string `metadata`.
- Boolean `disable-model-invocation` for Rho and Cursor. The Codex adapter rejects
  this extension rather than silently discard an invocation restriction.

Unknown fields, tool-permission declarations, and malformed types are rejected.
Do not remove a safety-related field just to make validation pass: choose a
compatible client or review a deliberate client-specific adaptation instead.
Auxiliary scripts, assets, and reference files are not copied; this command is
for self-contained instruction files, not arbitrary skill packages.

## 3. Remove only what MemoryWhale owns

Keep the original reviewed skill source. Remove it with the same target root:

```sh
"$MW" integrate rho --skill ./style/SKILL.md --skills-dir .agents/skills --revert
"$MW" integrate codex --revert
"$MW" integrate cursor --revert
```

The first command removes only the optional skill. The other two remove only
the MCP entry installed by the respective adapter. They do not delete memory.
Existing plain `mw integrate rho --revert` remains separate from optional skills.

Skill installation uses exclusive creation and a private exact-content ownership
snapshot. Reinstallation is a no-op only when source, installed file, and snapshot
match. Unowned or edited files are never silently adopted or overwritten.
Cooperating skill mutations hold a per-skill lock outside the removable skill
directory; removal rechecks ownership and content at its mutation boundary.
These guards do not synchronize an uncooperative editor writing through an
already-open file descriptor; stop editing an installation while reverting it.

Codex/Cursor MCP setup keeps an entry-only ownership journal adjacent to the
configuration. It preserves unrelated servers/settings and TOML comments, never
backs up the whole potentially secret-bearing configuration, and refuses to
remove a modified or unowned entry. JSON is parsed strictly, including duplicate
key rejection. Errors do not echo configuration contents.

MCP changes use an exclusive lock, guarded atomic file replacement, and a pending
journal state. If interrupted, inspect the configuration and sidecar files before
manual recovery; do not blindly delete the ownership journal or retry over user
changes. There is no cross-file transaction or protection against hostile
concurrent directory replacement.

Both adapters reject symlink paths (including ancestors), nonregular inputs,
parent traversal, oversized files, and invalid text. Use physical paths where
macOS aliases such as `/tmp` would otherwise introduce a symlink. Skills are
limited to 64 KiB; MCP configuration and journal files to 1 MiB.

Restart or open a fresh client session after removing a skill. Removing a file
cannot erase instructions already loaded into an existing conversation.

## Verified behavior and limits

On **September 10, 2026**, local CLI tests covered all three skill roots and both
MCP formats: install/check/dry-run/revert, idempotence, ownership conflicts,
preservation of user settings, metadata validation, and symlink rejection.

A **live native Rho 2.9.1** test with `openai-codex/gpt-6-astra` also verified:

1. A fresh session loaded the bundled memory skill and an installed
   `concise-recall` test skill, then called the real MemoryWhale MCP search tool.
2. It retrieved a synthetic failure and an untested proposal and correctly said
   the fix was **not verified**.
3. The test harness—not the Rho agent—executed the fixed command, observed exit
   0, and recorded the result. Another fresh Rho session retrieved that evidence
   and distinguished it from its own actions.
4. After `--revert`, a fresh Rho session's single skill-load attempt returned
   `unknown skill: concise-recall`; the memory skill remained installed.

The test used a disposable project and memory store, an explicit MCP config with
only read-only memory tools, and Rho's Plan mode. Normal `HOME` was retained for
macOS keychain authentication; this was not an OS sandbox and normal user-level
instructions could still be discovered. No credentials were copied, no approval
mode was weakened, and no Claude process was used for the live test.

The first attempts exposed two real limitations: replacing `HOME` prevented
macOS keychain access, and placing skills under custom `RHO_HOME` did not make
them discoverable. The corrected test used project-local skills. Rho also emitted
a nonfatal attach-output collision warning; direct result JSON and tool-event
logs were used instead of claiming that attachment succeeded.

This verifies one neutral, self-contained style skill—not the third-party
`i-have-adhd` skill, all models, automatic lifecycle recall, or Codex/Cursor live
sessions. Those client adapters have configuration-level tests, not runtime
certification. See the [response-style walkthrough](../guides/response-style-skills.md)
for the conceptual boundary and its separately unverified Claude example.

## Primary references

Checked September 10, 2026:

- [Rho skill discovery](https://matthewyjiang.github.io/rho/skills) and
  [MCP configuration](https://matthewyjiang.github.io/rho/integrations/mcp).
- [Codex MCP configuration](https://learn.chatgpt.com/docs/extend/mcp?surface=cli)
  and [skill authoring](https://learn.chatgpt.com/docs/build-skills), the current
  destinations of the former OpenAI developer documentation URLs.
- [Cursor MCP configuration](https://cursor.com/docs/mcp) and
  [Agent Skills](https://cursor.com/docs/skills).

Cursor can sync some personal skills to cloud clients when its own syncing
features are enabled. MemoryWhale does not enable or disable those settings;
review them before installing private instructions.
