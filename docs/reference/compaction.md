# Memory compaction

MemoryWhale's compaction policy shrinks bulky evidence without deleting its
row. It is intentionally conservative: failures, error-fingerprinted runs,
and bookmarked runs are protected; large successful output is the first thing
that becomes compactable.

## Preview a plan

`mw memory compact` is a dry run by default:

```bash
mw memory compact
```

It reports sessions and command runs selected by the policy, including the
rule that selected each one. Nothing changes during a dry run.

## Apply a plan

```bash
mw memory compact --apply
```

The default thresholds are:

| Evidence | Default rule |
| --- | --- |
| Session transcript | at least 256 KiB and at least 180 elapsed days old, unless an approved lesson makes it eligible after 45 days |
| Successful command output | combined stdout/stderr over 64 KiB, with no error fingerprint and no bookmark; `--max-output-bytes` must be at least 256 |
| Failed/error-fingerprinted/bookmarked command | always keep |

Tune the thresholds for a preview or apply:

```bash
mw memory compact \
  --min-session-bytes 1048576 \
  --stale-days 365 \
  --max-output-bytes 131072
```

## What compaction preserves

- **Rows are never deleted.** IDs, timestamps, commands, statuses, links, and
  provenance remain searchable.
- **Raw session files are not touched.** A compacted session keeps its
  existing absolute `transcript_path`; only the inline SQLite transcript becomes
  a marker. Sessions whose backing file is unavailable are not selected.
- **Original byte counts remain.** `sessions.byte_count` continues to describe
  the raw transcript, so `mw list` and `mw audit` do not underreport retained
  evidence after compaction.
- **Command output keeps both ends.** Compacted stdout/stderr retain a head and
  tail around a `[COMPACTED: … bytes omitted]` marker.
- **Lessons are not compacted by this command.** A saved lesson is the reason
  evidence may become compactable; it is the durable summary.

Always run the dry-run first. Export or back up the database before applying a
large policy change:

```bash
mw export
mw memory compact
mw memory compact --apply
```

Compaction is local and does not upload data. It is separate from TTL expiry:
TTL can remove lessons from normal retrieval while preserving their rows;
compaction reduces bulky evidence while preserving its rows.
