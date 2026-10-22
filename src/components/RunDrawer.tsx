import { X } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { ago, call, shortPath } from "../api";
import type { RunDetail } from "../types";
import { AgentPill, ExitBadge, RunRow } from "./bits";

/** One command run: its output, and what ran next in the same directory. */
export function RunDrawer({ id, onOpen, onClose }: { id: number; onOpen: (id: number) => void; onClose: () => void }) {
  const [detail, setDetail] = useState<RunDetail | null>(null);
  const [error, setError] = useState("");
  const close = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    setDetail(null);
    setError("");
    call<RunDetail>("run_detail", { id }).then(setDetail, (e) => setError(String(e)));
  }, [id]);

  useEffect(() => {
    close.current?.focus();
    const onKey = (e: KeyboardEvent) => e.key === "Escape" && onClose();
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  const run = detail?.run;
  return (
    <div className="drawer-backdrop" onClick={(e) => e.target === e.currentTarget && onClose()}>
      <aside className="drawer" role="dialog" aria-modal="true" aria-label="Command run">
        <div className="drawer-head">
          <span className="eyebrow">Command #{id}</span>
          <button className="icon-button" onClick={onClose} ref={close} type="button" aria-label="Close">
            <X size={18} />
          </button>
        </div>
        {error && <p className="error-banner">{error}</p>}
        {run && detail && (
          <>
            <pre className="drawer-command">
              <span className="prompt">$</span> {run.command}
            </pre>
            <dl className="facts">
              <div>
                <dt>Result</dt>
                <dd>
                  <ExitBadge code={run.exit_code} />
                </dd>
              </div>
              <div>
                <dt>Ran by</dt>
                <dd>
                  <AgentPill agent={run.agent} />
                </dd>
              </div>
              <div>
                <dt>When</dt>
                <dd title={run.created_at}>{ago(run.created_at)}</dd>
              </div>
              <div>
                <dt>Where</dt>
                <dd className="mono" title={run.cwd ?? undefined}>
                  {shortPath(run.cwd)}
                </dd>
              </div>
            </dl>
            {run.notes && <p className="drawer-notes">{run.notes}</p>}
            {detail.stderr && (
              <section>
                <h3>Error output</h3>
                <pre className="output bad">{detail.stderr}</pre>
              </section>
            )}
            {detail.stdout && (
              <section>
                <h3>Output</h3>
                <pre className="output">{detail.stdout}</pre>
              </section>
            )}
            {!detail.stderr && !detail.stdout && <p className="muted">No output was captured for this run.</p>}
            <section>
              <h3>What ran next here</h3>
              {detail.next.length === 0 ? (
                <p className="muted">Nothing ran after it in this directory yet.</p>
              ) : (
                <div className="run-list compact">
                  {detail.next.map((next) => (
                    <RunRow key={next.id} run={next} onOpen={onOpen} />
                  ))}
                </div>
              )}
            </section>
          </>
        )}
      </aside>
    </div>
  );
}
