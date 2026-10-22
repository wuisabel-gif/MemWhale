import { ArrowRight, Bell, Search } from "lucide-react";
import { useEffect, useState } from "react";
import { ago, call, count } from "../api";
import { AgentPill, Empty, Outcome, RunRow } from "../components/bits";
import { OceanHero } from "../components/Ocean";
import type { PageProps, RecallEvent, Run } from "../types";

export function Home({ overview, refreshKey, go, openRun }: PageProps) {
  const [events, setEvents] = useState<RecallEvent[]>([]);
  const [failures, setFailures] = useState<Run[]>([]);

  useEffect(() => {
    call<RecallEvent[]>("recall_history", { limit: 4 }).then(setEvents, () => setEvents([]));
    call<Run[]>("list_runs", { failuresOnly: true, limit: 5 }).then(setFailures, () => setFailures([]));
  }, [refreshKey]);

  const recall = overview?.recall;
  const agents = overview?.agents ?? [];
  const busiest = Math.max(1, ...agents.map(([, n]) => n));

  return (
    <>
      <OceanHero label="Overview">
        <div className="hero-grid">
          <div className="hero-copy">
            <span className="release-badge">
              <span className="live-dot" aria-hidden="true" /> Remembering on this machine
            </span>
            <h1 className="script hero-title">MemoryWhale</h1>
            <p className="hero-headline">
              Your terminal <span className="hero-accent">remembers</span>.
            </p>
            <p className="hero-lead">
              Every command, error, and fix you and your agents ran stays here, on this machine. When a failure you fixed
              before comes back, the agent gets a note.
            </p>
            <ul className="hero-stats">
              <li>
                <strong>{count(overview?.commands)}</strong> commands remembered
              </li>
              <li>
                <strong>{count(overview?.failures)}</strong> failures on record
              </li>
              <li>
                <strong>{count(overview?.lessons)}</strong> lessons saved
              </li>
            </ul>
            <div className="cta-row">
              <button className="button primary" onClick={() => go("search")} type="button">
                <Search size={17} aria-hidden="true" /> Search memory
              </button>
              <button className="button glass" onClick={() => go("recall")} type="button">
                What agents were told
              </button>
            </div>
          </div>
          <LatestNote event={events[0]} />
        </div>
      </OceanHero>

      <section className="score-grid" aria-label="How recall is doing">
        <article className="score-card">
          <h2>Notes after a failure</h2>
          <div className="score-row">
            <Score n={recall?.shown} label="shown" />
            <Score n={recall?.followed} label="followed" />
            <Score n={recall?.worked} label="worked" tone="good" />
          </div>
          <p className="muted">Read back from the runs that followed, not from anyone's claim.</p>
        </article>
        <article className="score-card">
          <h2>Warnings before a command</h2>
          <div className="score-row">
            <Score n={recall?.warnings} label="warned" />
            <Score n={recall?.warned_then_passed} label="then passed" tone="good" />
          </div>
          <p className="muted">Only when that program failed here and a fix has evidence.</p>
        </article>
        <article className="score-card">
          <h2>Who ran what</h2>
          {overview && agents.length === 0 ? (
            <p className="muted">No commands yet. Run `mw integrate claude` to start remembering.</p>
          ) : (
            <ul className="agent-bars">
              {agents.slice(0, 4).map(([agent, n]) => (
                <li key={agent}>
                  <AgentPill agent={agent} />
                  <span className="bar">
                    <span style={{ width: `${(100 * n) / busiest}%` }} />
                  </span>
                  <span className="bar-count">{count(n)}</span>
                </li>
              ))}
            </ul>
          )}
        </article>
      </section>

      <div className="two-col">
        <section className="card">
          <header className="card-head">
            <h2>Recent failures</h2>
            <button className="link-button" onClick={() => go("commands")} type="button">
              All commands <ArrowRight size={15} aria-hidden="true" />
            </button>
          </header>
          {failures.length === 0 ? (
            <Empty>No failures on record. Calm seas.</Empty>
          ) : (
            <div className="run-list">
              {failures.map((run) => (
                <RunRow key={run.id} run={run} onOpen={openRun} />
              ))}
            </div>
          )}
        </section>
        <section className="card">
          <header className="card-head">
            <h2>Latest from your agents</h2>
            <button className="link-button" onClick={() => go("recall")} type="button">
              Recall history <ArrowRight size={15} aria-hidden="true" />
            </button>
          </header>
          {events.length === 0 ? (
            <Empty>No notes or warnings yet. They appear when a failure with a known fix comes back.</Empty>
          ) : (
            <ul className="event-mini">
              {events.map((e) => (
                <li key={e.id}>
                  <Bell size={15} className={e.kind === "warning" ? "warn-icon" : "note-icon"} aria-hidden="true" />
                  <span className="event-mini-main">
                    <code title={e.command}>{e.stem || e.command}</code>
                    <span className="run-meta">
                      {e.kind === "warning" ? "warning" : "note"} · {ago(e.at)}
                    </span>
                  </span>
                  <AgentPill agent={e.agent} />
                  <Outcome text={e.outcome} />
                </li>
              ))}
            </ul>
          )}
        </section>
      </div>
    </>
  );
}

function Score({ n, label, tone }: { n: number | undefined; label: string; tone?: "good" }) {
  return (
    <div className={`score ${tone ?? ""}`}>
      <strong>{count(n)}</strong>
      <span>{label}</span>
    </div>
  );
}

/** The beige device from the website, showing the last thing an agent was told. */
function LatestNote({ event }: { event?: RecallEvent }) {
  return (
    <figure className="device">
      <div className="device-grille" aria-hidden="true">
        <i />
        <i />
        <i />
        <i />
        <i />
      </div>
      <div className="terminal-bar">
        <span className="terminal-dots" aria-hidden="true">
          <i />
          <i />
          <i />
        </span>
        <span>{event ? `${event.kind === "warning" ? "Warning" : "Note"} to ${event.agent ?? "you"}, ${ago(event.at)}` : "Latest note"}</span>
      </div>
      <pre className="screen">
        {event ? (
          <>
            <span className="prompt">$</span> <span title={event.command}>{event.stem || event.command}</span>
            {"\n\n"}
            <span className={event.kind === "warning" ? "warn" : "mint"}>
              {event.text ?? "(the exact text is kept from 0.23 on)"}
            </span>
          </>
        ) : (
          <>
            <span className="prompt">$</span> mw recall
            {"\n\n"}
            <span className="dim">
              No notes yet. When an agent repeats a failure you have fixed before, the note it saw shows up here.
            </span>
          </>
        )}
      </pre>
      <figcaption className="device-note">
        {event ? (
          <>
            Outcome: <Outcome text={event.outcome} />
          </>
        ) : (
          "A note needs evidence: a fix that worked here before."
        )}
      </figcaption>
    </figure>
  );
}
