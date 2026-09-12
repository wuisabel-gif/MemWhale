# Rho 2.10.0 live compatibility fixtures

Collected September 12, 2026 from native Rho 2.10.0 on macOS arm64, using
synthetic commands, a disposable project/store, and explicitly reviewed project
hooks. Unlike documentation-only examples, the command-result envelopes came
from the running client.

Normalization removes generated event/timestamp/session identities, replaces
temporary paths with `/fixture`, and replaces tool-call IDs with case labels.
Command text, status, failure text, bounds, and observed duration are retained.
Durations are not performance claims. The test wrapper's `case` labels are not
fields supplied by Rho.

- `success`, `nonzero`, and `denied`: observed tool outcomes. The second command
  exited 7, but the hook has no numeric exit-code field; the parser must not infer
  one from failure text.
- `unsaved-success`: observed during an interactive `--no-save` session. The
  separate capture hook still ran and persisted memory. The original event had
  no privacy opt-out signal to suppress that capture.
- `cancel-before` and `cancel-session-failed`: the events observed for a run
  stopped by its deliberate timeout after the synthetic command started. No
  corresponding after-tool event was received. Tests must not fabricate one.

Additional missing/truncated-field cases in `rho_210_compatibility.rs` are
synthetic mutations of these envelopes, not additional live observations.
Replay only passes JSON to `mw-remember`; it never executes fixture commands.

See the [verification report](../../../../../docs/research/rho-2.10-compatibility.md)
for the setup, native fresh/resume/unsaved results, limitations, and pinned
primary references.
