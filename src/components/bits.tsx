import { ChevronRight } from "lucide-react";
import type { ReactNode } from "react";
import { ago, shortPath } from "../api";
import type { Run } from "../types";

export function AgentPill({ agent }: { agent: string | null }) {
  const name = agent ?? "terminal";
  return <span className={`pill agent agent-${name}`}>{name === "terminal" ? "you" : name}</span>;
}

export function ExitBadge({ code }: { code: number | null }) {
  if (code === null) return <span className="exit unknown">exit ?</span>;
  return <span className={`exit ${code === 0 ? "ok" : "bad"}`}>{code === 0 ? "passed" : `exit ${code}`}</span>;
}

/** Outcome of a note or warning, read back from the runs that followed. */
export function Outcome({ text }: { text: string }) {
  const tone = /worked|then passed/.test(text) ? "good" : text === "not followed" ? "quiet" : "wait";
  return <span className={`pill outcome ${tone}`}>{text}</span>;
}

export function PageHead({ title, children }: { title: string; children?: ReactNode }) {
  return (
    <header className="page-head">
      <h1 className="script page-title">{title}</h1>
      {children && <p className="lead">{children}</p>}
    </header>
  );
}

export function Empty({ children }: { children: ReactNode }) {
  return (
    <div className="empty">
      <span className="empty-whale" aria-hidden="true" />
      <p>{children}</p>
    </div>
  );
}

export function RunRow({ run, onOpen }: { run: Run; onOpen: (id: number) => void }) {
  return (
    <button className="run-row" onClick={() => onOpen(run.id)} type="button">
      <ExitBadge code={run.exit_code} />
      <span className="run-main">
        <code className="run-command">{run.command}</code>
        <span className="run-meta">
          {shortPath(run.cwd)} · {ago(run.created_at)}
        </span>
      </span>
      <AgentPill agent={run.agent} />
      <ChevronRight className="run-chevron" size={16} aria-hidden="true" />
    </button>
  );
}
