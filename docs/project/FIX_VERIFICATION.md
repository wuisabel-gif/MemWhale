# Latest fix outcome

Fix candidates retain their historical episode counts and now expose the latest
observed rerun of the failing program in the candidate's directory, after the
fix and before the error came back (or before the run being hinted about). The
error recurring is why a hint is shown, so it never counts against the fix on
its own. The rerun records the run id,
command arguments, exit code, timestamp and recorded repository id. Missing exit
codes and provenance are explicitly unknown. The current schema does not capture
commit SHA for these runs: the display says `commit unknown (not captured)`.
It never reads today's git HEAD to invent historical provenance.

A failed or unknown rerun in that window is rendered as a historical candidate
("worked N of M times, but a later rerun failed, exit 2"), not a
`confirmed fix:`/`possible fix:` suggestion. Otherwise the hint keeps its
existing wording and markers, which Delphin's `:fix` reads. Consumers that parse the markers consequently do
not auto-select a contradicted candidate. This intentional wording change does
not change stored records, fingerprinting, or historical success counts.

A passing same-program rerun is observational evidence, not proof the candidate
caused the fix; test-suite changes, environment changes and flaky tests remain
possible. Candidates are grouped by program stem as before. Directory matching
is not an authorization boundary or proof that two runs used the same revision.

