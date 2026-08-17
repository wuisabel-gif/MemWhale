# Pullfrog + MemoryWhale

[Pullfrog](https://pullfrog.com) is a GitHub App and GitHub Actions-based
agent orchestration service. It can review pull requests, respond to reviews,
triage issues, and run coding workflows with a selected model. Pullfrog is an
external PR workflow client; it is not currently a verified MemoryWhale MCP
host.

## Status

Verified against Pullfrog's official documentation in August 2026 and tested
against a repository owned by the MemWhale organization. The GitHub App,
Codex authentication, model selection, and review automation are separate
account- and repository-level settings.

Pullfrog's review automation remains configured in the Pullfrog console. This
guide does not claim that automatic review is enabled for every PR.

Official references:

- [Getting started](https://docs.pullfrog.com/getting-started)
- [Codex subscription](https://docs.pullfrog.com/codex-auth)
- [Model selection](https://docs.pullfrog.com/models)
- [PR reviews](https://docs.pullfrog.com/pr-reviews)
- [Pullfrog tools](https://docs.pullfrog.com/tools)
- [Pullfrog security](https://docs.pullfrog.com/security)

## Requirements

- A GitHub account with permission to install GitHub Apps on the repository.
- Pullfrog installed with access limited to the repositories that need it.
- Node.js and `npx` for Codex authentication.
- A ChatGPT/Codex subscription or another provider configured in Pullfrog.
- A deliberate review-only policy if Pullfrog must not push commits.

## Capabilities

| Capability | Available |
| --- | --- |
| MCP memory access | No — external `mw-mcp` attachment is not documented or verified |
| Automatic execution capture | No |
| Memory-use guidance | No verified MemoryWhale-specific guidance |
| PR review automation | Yes, when enabled in the Pullfrog console |

## Setup

### 1. Install the GitHub App

Install [Pullfrog for GitHub](https://github.com/apps/pullfrog) and select
`wuisabel-gif/MemWhale` during repository access configuration. Prefer
repository-only access instead of granting access to every repository.

The App installation is account-level, while Pullfrog's triggers, models, and
review settings are configured per repository in the Pullfrog console.

### 2. Add the Pullfrog workflow

Pullfrog dispatches agent runs through a repository workflow. The Pullfrog
console can add this file automatically. The documented minimal workflow is:

```yaml
name: Pullfrog
run-name: ${{ inputs.name || github.workflow }}

on:
  workflow_dispatch:
    inputs:
      prompt:
        type: string
        description: Agent prompt
      name:
        type: string
        description: Run name

permissions:
  contents: read

jobs:
  pullfrog:
    runs-on: ubuntu-latest
    permissions:
      id-token: write
      contents: read
    steps:
      - name: Checkout code
        uses: actions/checkout@d23441a48e516b6c34aea4fa41551a30e30af803
        with:
          fetch-depth: 1
          persist-credentials: false
      - name: Run agent
        uses: pullfrog/pullfrog@0657d542f2e34565c6254d5c84581313e631cd90
        with:
          prompt: ${{ inputs.prompt }}
```

When Codex credentials are stored in Pullfrog's account secret store, this
workflow does not need an `env:` block containing provider keys. Do not add
credentials to the workflow. The Pullfrog console uses this workflow when its
configured automations dispatch a run.

### 3. Authenticate Codex

From the root of this repository, run:

```bash
npx pullfrog auth codex
```

The command detects the current GitHub repository, starts Codex device
authorization, and stores the resulting Codex credential in Pullfrog's account
secret store. Complete the browser authorization yourself; never put the
printed device code or resulting credential in a file, commit, issue, or
MemoryWhale note.

### 4. Choose the model

In the Pullfrog console, select:

```text
Agent → Model
```

Choose an OpenAI/Codex model and its reasoning effort. Pullfrog's model names
are rolling aliases; the exact available choices depend on the account,
organization, and provider access. Codex authentication authorizes OpenAI
models; it does not select a model automatically.

### 5. Enable review-only automation

In the repository's **Reviews** settings, use a policy like:

```text
Auto-review new PRs: enabled
Trigger: every new PR (or every PR ready for review)
Re-review on new commits: enabled
Address reviews / Auto-address reviews on Pullfrog PRs: disabled
Allow Pullfrog to approve PRs: disabled
Auto-merge approved PRs: disabled
```

This configuration makes automatic Pullfrog reviews comment-only for normal
review automation. Pullfrog can still respond to a deliberate manual
`@pullfrog` request, so review-only is a repository policy rather than a
technical guarantee against every manually triggered action. Confirm the
resulting policy in the Pullfrog console before relying on it as a required
review gate.

## Verify

The setup checks can be performed without exposing credentials:

```bash
gh repo view wuisabel-gif/MemWhale --json nameWithOwner --jq .nameWithOwner
npx pullfrog auth codex
```

The authentication command should report that it detected the repository, the
Pullfrog App is installed, Codex authentication succeeded, and the credential
was saved to the Pullfrog account secret store. Do not copy or print the
credential itself.

For review automation, open a test PR and verify in GitHub that Pullfrog adds a
review or a Pullfrog status/check. A manual `@pullfrog` comment is a separate
interactive trigger and does not prove that automatic review-on-PR is enabled.

## How to use

Use Pullfrog for GitHub-native work:

- review a PR using the selected model;
- summarize CI failures;
- inspect GitHub review threads;
- plan or implement issue work when those modes are deliberately enabled.

Use MemoryWhale separately for local terminal history and durable debugging
memory:

```bash
mw context --last-error
mw search "linker error"
mw remember "the linker failed because ..."
```

## Example prompt

For a review-only Pullfrog run, configure review instructions such as:

> Review this pull request for correctness, security, data-integrity, and test
> coverage. Comment with actionable findings only. Do not modify files, push
> commits, open branches, or implement fixes.

This prompt does not grant Pullfrog access to MemoryWhale's local database.

## Pullfrog event archive

The repository also contains `.github/workflows/pullfrog-memory.yml`. After a
workflow named `Pullfrog` completes, it records an allowlisted metadata summary
in a temporary MemoryWhale store and uploads a 14-day GitHub Actions artifact.
The captured event summary is limited to repository, workflow/run ID and URL,
event, status/conclusion, branch, commit SHA, actor, and a PR number only when
Pullfrog's workflow event explicitly provides one. The MemoryWhale command record also necessarily contains
the fixed command identifier and run ID in argv, the temporary runner cwd,
exit code, creation timestamp, and automatic `os:`/`runtime:` capture tags.

It deliberately does **not** capture Pullfrog prompts, review bodies, comments,
diffs, logs, environment variables, or credentials. The workflow checks out the
trusted default branch rather than executing code from the completed PR.

To bring an artifact into a local MemoryWhale store:

1. Download and extract the `pullfrog-memory-<run-id>` artifact from GitHub.
2. Import the extracted bundle directory:

   ```bash
   mw import /path/to/downloaded/project-pullfrog-*
   ```

3. Search the imported event:

   ```bash
   mw search "project:pullfrog"
   ```

The workflow-run event often has no direct PR association because Pullfrog
dispatches its workflow on the default branch. In that case the PR field stays
empty rather than guessing from a default-branch commit; the run URL and commit
SHA are the authoritative links.

Artifacts are snapshots, not synchronization. They do not automatically update
the developer's local SQLite database, and each artifact has a 14-day retention
period. Review GitHub artifact access and retention policies before treating the
archive as long-term history.

## Automatic capture

Pullfrog does not automatically capture local MemoryWhale terminal sessions.
Its GitHub Actions runs can inspect the repository, pull requests, and CI
through Pullfrog's built-in tools, but those tools are not the six MemoryWhale
MCP tools and do not provide access to the developer's local
`memorywhale.sqlite3`.

## MemoryWhale MCP integration status

Pullfrog's public tools documentation describes its built-in GitHub and CI MCP
tools. It does not document a configuration field for arbitrary external MCP
servers such as `mw-mcp`. Therefore this is not a verified configuration:

```text
Pullfrog agent → external mw-mcp → local MemoryWhale database
```

Do not add an invented `mcpServers`, `MCP_SERVERS`, or similar setting to a
Pullfrog workflow. A future supported custom-MCP feature could make this a thin
integration; update this guide only after verifying the official configuration.

## Limitations and privacy

- Pullfrog runs through GitHub and the selected model provider. Code, diffs,
  issue text, and review context may leave the local machine according to
  Pullfrog and provider policies.
- A GitHub Actions runner does not automatically contain the developer's local
  MemoryWhale database.
- Passing a MemoryWhale export or `mw context` output into a Pullfrog prompt
  would be a separate data-transfer workflow and requires explicit review of
  secrets, retention, and provider access.
- Codex usage is subject to the ChatGPT/Codex plan's limits.
- Review-only configuration is a policy setting, not a technical guarantee
  against every possible workflow or manually triggered action; keep workflow
  permissions least-privilege.

## Troubleshooting

- Run `command -v npx`, `node --version`, and `npx pullfrog --help` if the CLI
  is unavailable.
- Run `npx pullfrog auth codex` from the repository root so Pullfrog detects the
  intended repository.
- If Codex device authentication exits early, enable device-code auth in the
  ChatGPT security settings and retry.
- If the App is not detected, reinstall or update the Pullfrog GitHub App and
  check that `wuisabel-gif/MemWhale` is selected.
- If reviews do not appear, check the repository's Pullfrog **Reviews**
  automation settings, trigger mode, model access, and usage limits.
- Pullfrog review status and MemoryWhale MCP status are independent; a healthy
  Pullfrog review does not prove `mw-mcp` is connected.

## Remove integration

Disable review automations in the Pullfrog console, then uninstall the Pullfrog
GitHub App or remove `wuisabel-gif/MemWhale` from its repository access list.
Revoke or rotate the Codex credential from the Pullfrog account secret store
according to Pullfrog's account controls. No MemoryWhale database is deleted by
removing Pullfrog.
