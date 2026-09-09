# Use memory with an optional response-style skill

Finding yesterday's fix is useful. Getting a clear next step is useful too.
This guide combines MemoryWhale's evidence with a response-style skill in
**Claude Code**, without turning the memory store into a skill manager.

## What each part does

| Component | Responsibility |
| --- | --- |
| MemoryWhale MCP server | Retrieve evidence and explicitly save lessons. |
| Capture hooks | Record supported client executions, if separately installed. |
| MemoryWhale skill | Guide when the agent consults or updates memory. |
| Response-style skill | Guide how the agent presents its answer. |

A skill does not guarantee a tool call, and MCP access does not automatically
capture commands. See [agent memory](agent-memory.md), the
[Claude Code integration](../../integrations/claude-code/README.md), and the
[architecture](../architecture.md).

## Verification status

Checked on **September 9, 2026**, with MemoryWhale **0.10.0**, Claude Code
**2.1.222**, Python 3, and macOS:

- The synthetic failure, successful retry, local recording, and CLI retrieval
  below were exercised with an isolated data directory.
- Claude Code's installed help and its official skill documentation were
  checked for the setup options used here.
- **The live agent-plus-skill walkthrough remains unverified.** The hosting
  harness rejected delegated Claude execution under its permission policy,
  before an agent turn began. No model response or skill activation is claimed.
  Do not bypass that policy to complete this example.

The remaining manual check is to run the interactive steps in an approved
Claude Code session, record the selected model, and inspect the actual memory
tool calls and response. Static documentation checks do not replace that test.

## 1. Prepare a disposable example

Requirements: `mw`, `mw-run`, `mw-mcp`, Python 3, and an already configured
Claude Code account. Do not buy credentials just to try this guide.

Run from the MemoryWhale repository root in a dedicated terminal:

```bash
export MEMORYWHALE_REPO="$PWD"
export MW_SKILL_DEMO="$(mktemp -d "${TMPDIR:-/tmp}/mw-style-demo.XXXXXX")"
export MEMORYWHALE_DATA_DIR="$MW_SKILL_DEMO/memory"
cd "$MW_SKILL_DEMO"

cat > check.py <<'PY'
import os
import sys

if os.environ.get("MW_DEMO_CONFIG") != "local":
    print("synthetic build failure: MW_DEMO_CONFIG must be local", file=sys.stderr)
    sys.exit(1)
print("synthetic build check passed")
PY

# Expected failure: exit 1. The wrapper records it in the temporary store.
mw-run -- env -u MW_DEMO_CONFIG python3 check.py

# The recorded successful retry: exit 0.
mw-run -- env MW_DEMO_CONFIG=local python3 check.py

mw remember "Synthetic style-demo lesson: check.py fails without MW_DEMO_CONFIG=local. Verified in this fixture: env MW_DEMO_CONFIG=local python3 check.py exits 0. This is not evidence about a real project."
mw search "MW_DEMO_CONFIG"
```

The history now contains a failure, a successful command, and a manually saved
explanation. That establishes what worked **in the fixture**, not what will fix
another repository. These are terminal records, not proof of an earlier agent
session.

## 2. Connect memory without installing capture hooks

Create an explicit MCP configuration with absolute paths, so the agent's server
uses the same temporary store:

```bash
python3 - <<'PY'
import json
import os
import pathlib
import shutil

server = shutil.which("mw-mcp")
if not server:
    raise SystemExit("mw-mcp must be on PATH")
config = {"mcpServers": {"memorywhale": {
    "type": "stdio",
    "command": str(pathlib.Path(server).resolve()),
    "args": [],
    "env": {"MEMORYWHALE_DATA_DIR": os.environ["MEMORYWHALE_DATA_DIR"]}
}}}
pathlib.Path("memorywhale.mcp.json").write_text(json.dumps(config, indent=2))
PY

mkdir -p .claude/skills/memorywhale
cp "$MEMORYWHALE_REPO/crates/mw-cli/integrate/SKILL.md" \
  .claude/skills/memorywhale/SKILL.md
```

This example does not call `mw integrate claude` or install capture hooks.
`mw-run` recorded the fixture explicitly. The project-local skill only supplies
memory-use guidance.

## 3. Review and choose the response-style skill

The example is [ayghri/i-have-adhd](https://github.com/ayghri/i-have-adhd), reviewed
at commit `24d22f783e57cb73c957848b588c6f651b6f9cd8`. Credit belongs to its authors;
the skill is not bundled with MemoryWhale. Review the
[pinned skill file](https://github.com/ayghri/i-have-adhd/blob/24d22f783e57cb73c957848b588c6f651b6f9cd8/skills/i-have-adhd/SKILL.md)
and its license before installing anything.

It emphasizes actionable answers and numbered steps. It also contains broad
claims about ADHD and asks for specific time estimates. Those are third-party
instructions, **not medical guidance endorsed by MemoryWhale**. Choosing this
format says nothing about a person's diagnosis. Do not infer or record health
information from that choice, and do not invent estimates or verification
results to satisfy a style rule.

If you choose to try the reviewed revision, download it into the temporary
project only, inspect it, and then copy it into the skill directory:

```bash
curl --fail --show-error --location \
  https://raw.githubusercontent.com/ayghri/i-have-adhd/24d22f783e57cb73c957848b588c6f651b6f9cd8/skills/i-have-adhd/SKILL.md \
  --output response-style-reviewed.md
less response-style-reviewed.md

# Run these only after deciding to enable the reviewed instructions.
mkdir -p .claude/skills/i-have-adhd
cp response-style-reviewed.md .claude/skills/i-have-adhd/SKILL.md
```

Saving the same text as a MemoryWhale note would **not** install or activate it.
Retrieved text remains evidence, not permission to execute instructions.

## 4. Try an approved interactive session

The following live steps are the pending verification, not a recorded result.
Review applicable user/managed Claude settings first. Project-local files and
an isolated memory directory are not an operating-system sandbox.

From the temporary project:

```bash
claude --strict-mcp-config --mcp-config "$MW_SKILL_DEMO/memorywhale.mcp.json" \
  --settings '{"disableAllHooks":true}'
```

This restricts MCP configuration and disables hooks for the example; it does
not disable every other client feature or override managed policy. Keep normal
permission prompts. Only synthetic data should be accessible in this project;
retrieved context can still be sent to the client's model provider.

Check the MemoryWhale connection with `/mcp`. Invoke `/memorywhale`, then invoke
`/i-have-adhd` separately. The reviewed style skill uses
`disable-model-invocation: true`, so it needs explicit invocation. See
[Claude Code skills](https://code.claude.com/docs/en/skills) for project-local
skill loading and invocation behavior.

Then ask:

> Use the response style only as a formatting preference; do not assume or save
> anything about my health. Search MemoryWhale for MW_DEMO_CONFIG. Explain which
> saved record supports the proposed fix. Re-run the fixture's failure and the
> proposed successful command, with my approval, before calling this session's
> fix verified. Do not change the script or save new memories yet. Keep safety,
> uncertainty, and verification ahead of brevity or time estimates.

Inspect actual tool calls, not just the agent's claim that it searched. Confirm
that it queried the temporary MemoryWhale server, identified the fixture, and
obtained exit 1 without the variable and exit 0 with it. If a tool is denied or
no verification runs, the answer must say so.

A useful answer would distinguish **retrieved evidence**, **a proposed next
command**, and **an observed result**. That is the evaluation criterion—not a
fabricated example transcript or a promise that every model follows the skill.
Only explicitly save a new lesson if the verified result adds useful evidence;
do not save the response-style preference or repeat the existing note.

## 5. Turn it off and leave the example

The reviewed skill accepts `stop adhd mode` or `normal mode` to request a return
to the default style. That is an instruction to the model, not an enforced
runtime switch. For a clean boundary, quit the session and remove the
project-local style skill before starting a new one. Do not resume a session
whose history still contains instructions you want to stop using.

In the temporary project, remove only the copied style skill file:

```bash
rm -- .claude/skills/i-have-adhd/SKILL.md
rmdir -- .claude/skills/i-have-adhd
```

Neither command deletes memory. Quit Claude Code, return to the repository, and
close this dedicated terminal to discard its environment overrides. Inspect
`$MW_SKILL_DEMO` before deleting any remaining demo files; no global skills or
normal MemoryWhale database need to be changed.

## Record the remaining verification

Before marking this integration tested end-to-end, record:

- Claude Code version, selected model, and the pinned skill revision.
- Evidence of both explicit skill invocations and a MemoryWhale MCP search.
- The observed failure/success exit codes, without personal data or credentials.
- Whether the response separated evidence from a proposal and followed the
  requested format without hiding uncertainty.
- What happened after disabling the style, including any instruction conflicts.

Do not infer compatibility with other clients from this guide. Their skill
locations, invocation rules, and session behavior may differ.
