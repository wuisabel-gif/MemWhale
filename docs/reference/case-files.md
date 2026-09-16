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
