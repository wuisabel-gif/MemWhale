# Portable debugging handoffs

`mw handoff` creates a local Markdown or JSON file from records explicitly named by ID:

```sh
mw handoff --ids c:42,s:7 --format markdown --output ./handoff.md
```

`c:ID` selects a command run and `s:ID` selects a session. There is no search,
recency, or automatic selection. Output is redacted using the capture sanitizer,
bounded, and includes notices when content is truncated. Each item contains
stored provenance and source IDs, plus unresolved questions. The export makes no
causal claims and performs no upload or network operation. Existing output files
are never overwritten.
