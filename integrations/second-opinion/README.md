# Second-Opinion + MemoryWhale

[Second-Opinion](https://github.com/wuisabel-gif/second-opinion) is a GitHub
Action that posts an agent-agnostic pull-request review. MemoryWhale uses it as
a thin CI interface. The reviewer is not part of Capture, Memory, or Retrieval,
and it does not read `memorywhale.sqlite3`.

## Status

Verified against the pinned action
`wuisabel-gif/second-opinion@e5cce405ea722fb7c7c5cfa28224c5735127cf6d`
(v0.3.0) and this repository's `.github/workflows/second-opinion.yml`.
Reviews run only when the `REVIEW_API_KEY` Actions secret is present. A green
"Agent-agnostic review" check is not proof that a model ran if the secret is
missing; the job then warns and skips.
The review step is advisory (`continue-on-error`): if the provider fails
(for example an exhausted token quota or an outage), the job warns and
passes. Check the job log or the PR for a posted review before relying on it.

This guide does not ship a model. Provider, base URL, and model are repository
variables. Do not put a provider key in the workflow YAML.

## Requirements

- A GitHub repository with Actions enabled and permission to set secrets.
- The workflow file `.github/workflows/second-opinion.yml` (copy from this
  repository or pin the same action SHA).
- `REVIEW_API_KEY` as a repository Actions secret when you want reviews to run.
- Optional repository variables: `REVIEW_PROVIDER`, `REVIEW_BASE_URL`,
  `REVIEW_MODEL`. If `REVIEW_ENDPOINT` is set, the action uses the webhook
  provider instead.
- `gh` authenticated as a user who can read pull requests, if you want to save
  a review into local MemoryWhale with `mw github context`.

## Setup

1. Keep or add `.github/workflows/second-opinion.yml`. It must stay
   `pull_request_target`: the action reads the PR diff through the GitHub API
   and must not check out or execute PR-head code.

2. Set the secret (do not paste the key into chat, issues, or MemoryWhale):

   ```bash
   printf '%s' "$REVIEW_API_KEY" | gh secret set REVIEW_API_KEY --repo <owner>/<repo>
   ```

3. Optionally set variables. This repository uses an OpenAI Responses-compatible
   gateway:

   ```text
   REVIEW_PROVIDER = openai-responses
   REVIEW_BASE_URL = https://api.chr1.com/v1
   REVIEW_MODEL    = gpt-5.6-sol
   ```

   Use only a key issued for that gateway. Official OpenAI or Anthropic keys
   belong only on their own endpoints.

4. Leave MemoryWhale core unchanged. Do not add provider SDKs, keys, or review
   ranking into `memorywhale-core`.

## Verify

```bash
gh secret list --repo <owner>/<repo>
# REVIEW_API_KEY should appear by name only
```

Open or push to a pull request, then:

```bash
gh pr checks <n>
```

Success looks like:

- job step `Run Second-Opinion` ran (not skipped)
- a PR review from `github-actions[bot]` whose body starts with
  `## second-opinion review`

If the job finishes in a few seconds with `REVIEW_API_KEY is not configured`,
the secret is missing. There is no `@second-opinion` comment command. To run on
an existing PR: push a commit, close/reopen, or `gh run rerun <run-id>`.

To print the review locally without saving it:

```bash
mw github context <n>
```

To store a conclusion you accept, copy from that output and run
`mw remember "..."` yourself. The GitHub command does not write the database.

## Available capabilities

| Capability | Available |
| --- | --- |
| MCP memory access | No |
| Automatic execution capture | No |
| Memory-use guidance | No |
| PR review automation | Yes, when `REVIEW_API_KEY` is set |

Second-Opinion reviews GitHub diffs. MemoryWhale stores local debugging
evidence. They meet only when you explicitly paste or `mw remember` a review.

Do not pipe `memorywhale.sqlite3`, `mw export`, or unreviewed `mw context`
output into the review provider.

## Example prompt

Not applicable. Second-Opinion is a GitHub Action, not a chat client. After a
review lands, a typical explicit save is:

```bash
mw github context 301
mw remember "second-opinion on #301: sanitize case show before printing commands"
```

## Troubleshooting

- **Skipped in seconds:** `REVIEW_API_KEY` is unset. Add the secret; rerun.
- **Wrong provider:** empty `REVIEW_PROVIDER` falls back to `anthropic`. Set
  `REVIEW_PROVIDER` and `REVIEW_BASE_URL` together for a gateway.
- **No review on an old PR:** the workflow does not listen for issue comments.
  Push, reopen, or `gh run rerun`.
- **`mw github context` empty reviews:** `gh` must be logged in with `repo`
  scope; native Windows is not supported for that command.

## Uninstall

1. Delete or disable `.github/workflows/second-opinion.yml`.
2. Remove `REVIEW_API_KEY` and any `REVIEW_*` variables from repository
   settings.
3. Optionally dismiss or hide existing `github-actions[bot]` reviews.

This does not delete the local MemoryWhale database or any notes you saved with
`mw remember`.
