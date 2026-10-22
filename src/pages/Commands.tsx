import { Search as SearchIcon } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { call } from "../api";
import { Empty, PageHead, RunRow } from "../components/bits";
import type { PageProps, Run } from "../types";

const day = new Intl.DateTimeFormat(undefined, { weekday: "long", month: "long", day: "numeric" });

export function Commands({ refreshKey, openRun }: PageProps) {
  const [failuresOnly, setFailuresOnly] = useState(false);
  const [query, setQuery] = useState("");
  const [runs, setRuns] = useState<Run[] | null>(null);
  const [error, setError] = useState("");

  useEffect(() => {
    let stale = false;
    const timer = window.setTimeout(() => {
      call<Run[]>("list_runs", { query, failuresOnly, limit: 300 }).then(
        (next) => !stale && (setRuns(next), setError("")),
        (e) => !stale && setError(String(e))
      );
    }, 150);
    return () => {
      stale = true;
      window.clearTimeout(timer);
    };
  }, [query, failuresOnly, refreshKey]);

  // Group by calendar day, newest first, like the web dashboard.
  const days = useMemo(() => {
    const groups: [string, Run[]][] = [];
    for (const run of runs ?? []) {
      const parsed = new Date(run.created_at.includes("T") ? run.created_at : `${run.created_at.replace(" ", "T")}Z`);
      const label = Number.isNaN(parsed.getTime()) ? "Undated" : day.format(parsed);
      const last = groups[groups.length - 1];
      if (last?.[0] === label) last[1].push(run);
      else groups.push([label, [run]]);
    }
    return groups;
  }, [runs]);

  return (
    <>
      <PageHead title="Commands">What ran, where, by whom, and how it ended. Open one to see its output and what ran next.</PageHead>
      <div className="toolbar">
        <div className="segmented" role="group" aria-label="Show">
          <button aria-pressed={!failuresOnly} onClick={() => setFailuresOnly(false)} type="button">
            All
          </button>
          <button aria-pressed={failuresOnly} onClick={() => setFailuresOnly(true)} type="button">
            Failures
          </button>
        </div>
        <label className="filter-field">
          <SearchIcon size={16} aria-hidden="true" />
          <span className="sr-only">Filter commands</span>
          <input
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Filter by command, error, folder…"
            spellCheck={false}
          />
        </label>
      </div>
      {error && <p className="error-banner">{error}</p>}
      {runs !== null && runs.length === 0 && (
        <Empty>{query || failuresOnly ? "No commands match." : "No commands yet. Run `mw integrate claude` to start."}</Empty>
      )}
      {days.map(([label, group]) => (
        <section className="day" key={label}>
          <h2 className="day-label">
            {label} <span>{group.length}</span>
          </h2>
          <div className="run-list card">
            {group.map((run) => (
              <RunRow key={run.id} run={run} onOpen={openRun} />
            ))}
          </div>
        </section>
      ))}
    </>
  );
}
