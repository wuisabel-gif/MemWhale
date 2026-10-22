import { Bell, MessageCircle } from "lucide-react";
import { useEffect, useState } from "react";
import { ago, call, count, shortPath } from "../api";
import { AgentPill, Empty, Outcome, PageHead } from "../components/bits";
import type { PageProps, RecallEvent } from "../types";

export function RecallPage({ overview, refreshKey }: PageProps) {
  const [events, setEvents] = useState<RecallEvent[] | null>(null);
  const [error, setError] = useState("");

  useEffect(() => {
    call<RecallEvent[]>("recall_history", { limit: 200 }).then(setEvents, (e) => setError(String(e)));
  }, [refreshKey]);

  const r = overview?.recall;
  return (
    <>
      <PageHead title="Recall">
        Every note and warning an agent was shown, the exact words, and what happened next.
      </PageHead>
      {r && (
        <p className="summary-line">
          <strong>{count(r.shown)}</strong> notes shown, <strong>{count(r.followed)}</strong> followed,{" "}
          <strong className="good">{count(r.worked)}</strong> worked · <strong>{count(r.warnings)}</strong> warnings,{" "}
          <strong className="good">{count(r.warned_then_passed)}</strong> then passed
        </p>
      )}
      {error && <p className="error-banner">{error}</p>}
      {events !== null && events.length === 0 && (
        <Empty>
          Nothing yet. MemoryWhale speaks up only with evidence: a saved lesson, or a fix that made the same program pass
          here before.
        </Empty>
      )}
      <ol className="events">
        {events?.map((e) => (
          <li className={`event ${e.kind}`} key={e.id}>
            <span className="event-icon" aria-hidden="true">
              {e.kind === "warning" ? <Bell size={18} /> : <MessageCircle size={18} />}
            </span>
            <div className="event-body">
              <div className="event-head">
                <strong>
                  {e.kind === "warning" ? "Warning before " : "Note after "}
                  <code>{e.stem || "a command"}</code>
                  {e.kind === "warning" ? "" : " failed"}
                </strong>
                <AgentPill agent={e.agent} />
                <span className="run-meta" title={e.at}>
                  {ago(e.at)}
                </span>
                <Outcome text={e.outcome} />
              </div>
              <pre className="event-command">
                <span className="prompt">$</span> {e.command}
              </pre>
              {e.text ? (
                <blockquote className="event-text">{e.text}</blockquote>
              ) : (
                <p className="muted">The exact text is kept from 0.23 on.</p>
              )}
              <p className="run-meta mono">{shortPath(e.cwd)}</p>
            </div>
          </li>
        ))}
      </ol>
    </>
  );
}
