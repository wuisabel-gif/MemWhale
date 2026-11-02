# Latest fix outcome

Fix candidates retain their historical episode counts and now expose the latest
observed rerun of the failing program in the candidate's directory: run id,
command arguments, exit code, timestamp and recorded repository id. Missing exit
codes and provenance are explicitly unknown. The current schema does not capture
commit SHA for these runs: the display says `commit unknown (not captured)`.
It never reads today's git HEAD to invent historical provenance.

A later failed or unknown rerun is rendered as a historical candidate, not a
`confirmed fix:`/`possible fix:` suggestion. A later passing rerun retains those
existing suggestion markers. Consumers that parse the markers consequently do
not auto-select a contradicted candidate. This intentional wording change does
not change stored records, fingerprinting, or historical success counts.

A passing same-program rerun is observational evidence, not proof the candidate
caused the fix; test-suite changes, environment changes and flaky tests remain
possible. Candidates are grouped by program stem as before. Directory matching
is not an authorization boundary or proof that two runs used the same revision.

