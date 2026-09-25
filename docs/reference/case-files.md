# Memory case files

Case files are explicit, local, human-authored records assembled from selected command-run IDs. They preserve the distinction between **evidence / observations** and **conclusion (interpretation)**; MemoryWhale never generates an automatic summary.

```sh
mw case create --title "Compiler failure" --command-ids 12,18 \
  --observations "12 failed; 18 passed after dependency change" \
  --conclusion "The dependency was missing" \
  --unresolved "Why was it absent?" --status resolved
mw case list
mw case show 1
mw case export 1
```

Only IDs explicitly supplied to `--command-ids` are linked. Case-file rows and links are stored in the same SQLite database as command records. Status is `open`, `resolved`, or `closed`.

`mw case export <id>` prints JSON with the ordered `command_ids` and redacted `evidence`; `--format markdown` renders captured commands and output in fenced code blocks. `show` and both export formats redact secrets and strip terminal control sequences. Case-linked command runs are retained: `mw delete` refuses them, and `mw prune --older-than` and the desktop demo reset skip them (the prune count excludes them).
