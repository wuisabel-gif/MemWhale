import { ChevronDown, Search as SearchIcon } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { ago, call } from "../api";
import { AgentPill, Empty, PageHead } from "../components/bits";
import type { PageProps, SearchHit } from "../types";

const SOURCE_LABEL: Record<SearchHit["source"], string> = {
  command: "command",
  note: "lesson",
  session: "session",
  conversation: "conversation",
  document: "document"
};

export function SearchPage({ openRun }: PageProps) {
  const [query, setQuery] = useState("");
  const [hits, setHits] = useState<SearchHit[] | null>(null);
  const [open, setOpen] = useState<number | null>(null);
  const [error, setError] = useState("");
  const input = useRef<HTMLInputElement>(null);

  useEffect(() => input.current?.focus(), []);

  useEffect(() => {
    if (!query.trim()) {
      setHits(null);
      return;
    }
    let stale = false;
    const timer = window.setTimeout(() => {
      call<SearchHit[]>("search", { query, limit: 20 }).then(
        (next) => !stale && (setHits(next), setError("")),
        (e) => !stale && setError(String(e))
      );
    }, 250);
    return () => {
      stale = true;
      window.clearTimeout(timer);
    };
  }, [query]);

  return (
    <>
      <PageHead title="Search">Everything you and your agents ran, saw, or saved, ranked with the reasons why.</PageHead>
      <label className="search-field">
        <SearchIcon size={20} aria-hidden="true" />
        <span className="sr-only">Search memory</span>
        <input
          ref={input}
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="linker cc not found, npm test, a lesson you saved…"
          spellCheck={false}
        />
      </label>
      {error && <p className="error-banner">{error}</p>}
      {hits === null ? (
        <Empty>Ask your memory about an error, a command, or a project. Search runs on this machine only.</Empty>
      ) : hits.length === 0 ? (
        <Empty>Nothing matches “{query}” yet.</Empty>
      ) : (
        <ol className="hits">
          {hits.map((hit) => (
            <li className="hit" key={hit.id}>
              <div className="hit-score" style={{ ["--score" as string]: `${hit.score}%` }}>
                <span>{hit.score}</span>
              </div>
              <div className="hit-body">
                <div className="hit-meta">
                  <span className={`pill source source-${hit.source}`}>{SOURCE_LABEL[hit.source]}</span>
                  <AgentPill agent={hit.agent} />
                  <span className="run-meta">{ago(hit.created_at)}</span>
                </div>
                <pre className="hit-text">{hit.text}</pre>
                <div className="reasons">
                  {hit.reasons.slice(0, 4).map((reason) => (
                    <span className="reason" key={reason}>
                      {reason}
                    </span>
                  ))}
                </div>
                <div className="hit-actions">
                  {hit.source === "command" && (
                    <button className="link-button" onClick={() => openRun(hit.row_id)} type="button">
                      Open the command
                    </button>
                  )}
                  <button
                    className="link-button"
                    aria-expanded={open === hit.id}
                    onClick={() => setOpen(open === hit.id ? null : hit.id)}
                    type="button"
                  >
                    Why this result <ChevronDown size={15} className={open === hit.id ? "flip" : ""} aria-hidden="true" />
                  </button>
                </div>
                {open === hit.id && (
                  <div className="signals">
                    {hit.signals.map((s) => (
                      <div className={`signal ${s.applicable ? "" : "off"}`} key={s.name}>
                        <span className="signal-name">{s.name}</span>
                        <span className="bar">
                          <span style={{ width: `${Math.round(s.score * 100)}%` }} />
                        </span>
                        <span className="signal-detail">{s.detail}</span>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            </li>
          ))}
        </ol>
      )}
    </>
  );
}
