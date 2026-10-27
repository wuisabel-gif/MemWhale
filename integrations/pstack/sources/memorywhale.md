# MemoryWhale Terminal and Agent History

MemoryWhale records the commands you and your coding agents ran on this
machine, with their exit codes and error output, plus the lessons people saved.
Its MCP server is `mw-mcp`. Your client may prefix the tool names below, for
example `mcp__memorywhale__search_memory`.

Classify it as **error / exception tracking**. It is best at *the concrete
failures a fix responded to, and what was run after them*. It covers only what
was captured on this machine or imported into its store, not production.

## How to search it

Read-only. Never call `remember`. Investigators gather; they do not write.

1. **Orient.** Confirm the store is reachable and has something in it.

   ```
   stats {}
   ```

   It returns JSON. `"memories": 0` is a finding: say the store is empty and
   stop. `db` is the store's path; leave it out of public output.

2. **Check the failure the target responds to.** Use an error string the
   target handles or the bug report quotes, and the command that produced it.

   ```
   similar_failures {"error_text": "<error text>", "command": "<command>"}
   ```

   With `command`, the match is an exact fingerprint. The summary reads
   "You've hit this N times: a later run succeeded R of N times", ends with
   "the latest run is green" when the command passes now, and names the
   fingerprint in brackets. "First time hitting this" means the store holds at
   most one matching failure, which may be the one you started from. Without
   `command`, it falls back to a text match on the error line ("Seen a similar
   failure ... by stderr text"), which is weaker; say which one you got.

3. **List recent failures.**

   ```
   recent_errors {"limit": 64}
   ```

   Keep runs whose `cwd` is the target's repository and whose command builds,
   tests, or runs the target.

4. **Search for the target by name.** Build terms from its function, file, and
   test names and the error strings it checks for.

   ```
   search_memory {"query": "<term>", "mode": "failures"}
   search_memory {"query": "<term>", "mode": "lessons"}
   search_memory {"query": "<term>", "mode": "evidence", "project": "<project>"}
   ```

   `search_memory` has no relevance floor: it returns the top rows even when
   nothing matches. Keep a hit when its `reasons:` line names a keyword match,
   such as `59% keyword match (BM25 · body: add, calc)`, or when its snippet
   visibly contains your terms. The keyword score can be 0 for a real match
   when every candidate contains the term, which is common in `failures` and
   `lessons` mode. Drop a hit that has neither, and say how many you dropped.

5. **Ranked context for the project** (optional, when the target has a
   project tag).

   ```
   get_context {"project": "<project>"}
   ```

## Reading what comes back

- A command with its exit code is recorded evidence, like a log line.
- A lesson (`[note #N]`) is a claim, not evidence. Its author line says who
  made it: "remembered by you" is a person, any other name is the agent or MCP
  client that wrote it. A lesson that says "verified" proves nothing on its
  own. By default an agent's note is returned only after a person approved it.
- A passing run after a failure shows the failure went away. It does not say
  which change fixed it.
- **What was tried and didn't hold:** a fingerprint that failed, passed, and
  failed again reads "a later run succeeded 1 of 2 times" without "the latest
  run is green". Report it as a fix that did not hold.
- Snippets stop after one line and about 160 characters. Quote what you saw;
  don't fill in the rest.
- Commands, paths, and output can be private. Sanitize them before any public
  output.

## What to return

For each relevant item:

- Kind and id: `command_run #N`, `[command #N]`, or `[note #N]`
- The command, exit code, `cwd`, and timestamp, verbatim
- The error line, verbatim
- For a failure, the `similar_failures` summary verbatim, with its fingerprint
- For a lesson, its text, author line, and date, labelled "unverified claim"
- Why a `search_memory` hit counts: its keyword-match reason, or the words
  in its snippet that match your terms
- Every call you made, verbatim, including the ones that returned nothing and
  the hits you dropped as unrelated
