import { Check, Copy, Download, EyeOff, Trash2, X } from "lucide-react";
import { useEffect, useState } from "react";
import { call, openUrl } from "../api";
import { Empty, PageHead } from "../components/bits";
import type { Knowledge as Statement, KnowledgeImport, PageProps } from "../types";

// The only step the chat products do not let a program do for you.
const EXPORT_HELP = {
  ChatGPT: "https://help.openai.com/en/articles/7260999-how-do-i-export-my-chatgpt-history-and-data",
  Claude: "https://support.claude.com/en/articles/9450526-export-your-claude-data"
};

const SOURCE: Record<string, string> = {
  chatgpt: "ChatGPT",
  codex: "Codex",
  claude_chat: "Claude",
  claude_code: "Claude Code",
  manual: "you"
};

export function Knowledge({ refreshKey }: PageProps) {
  const [pending, setPending] = useState<Statement[] | null>(null);
  const [approved, setApproved] = useState<Statement[]>([]);
  const [result, setResult] = useState<KnowledgeImport | null>(null);
  const [busy, setBusy] = useState(false);
  const [copied, setCopied] = useState(false);
  const [error, setError] = useState("");

  const load = () =>
    Promise.all([
      call<Statement[]>("knowledge_list", { status: "proposed" }),
      call<Statement[]>("knowledge_list", { status: "approved" })
    ]).then(
      ([p, a]) => {
        setPending(p);
        setApproved(a);
      },
      (e) => setError(String(e))
    );
  useEffect(() => void load(), [refreshKey]);

  async function act(command: string, args?: Record<string, unknown>) {
    try {
      const value = await call<unknown>(command, args);
      setError("");
      await load();
      return value;
    } catch (e) {
      setError(String(e));
      return undefined;
    }
  }

  async function importAll() {
    setBusy(true);
    const report = (await act("knowledge_import")) as KnowledgeImport | undefined;
    if (report) setResult(report);
    setBusy(false);
  }

  async function copyPack() {
    const pack = (await act("knowledge_pack")) as string | undefined;
    if (pack === undefined) return;
    await navigator.clipboard.writeText(pack);
    setCopied(true);
  }

  return (
    <>
      <PageHead title="Knowledge">
        What you have told your assistants, in one place. Nothing here is used until you approve it.
      </PageHead>
      <section className="composer card">
        <div className="composer-foot">
          <span className="muted">
            Reads your Claude Code and Codex conversations on this machine, and any ChatGPT or Claude export in Downloads.
            Nothing leaves this machine.
          </span>
          <button className="button primary" disabled={busy} onClick={() => void importAll()} type="button">
            <Download size={16} aria-hidden="true" /> {busy ? "Reading…" : "Import my conversations"}
          </button>
        </div>
        {result && (
          <p className="muted" role="status">
            Read {result.files.toLocaleString()} files and {result.messages.toLocaleString()} messages you wrote.{" "}
            {result.proposed.toLocaleString()} new statements are waiting below; {result.already_known.toLocaleString()}{" "}
            were already known.
            {!result.export_found && " No ChatGPT or Claude export was found in Downloads."}
          </p>
        )}
        <p className="muted">
          ChatGPT and Claude only hand over conversations as a download you request yourself:{" "}
          {Object.entries(EXPORT_HELP).map(([name, url]) => (
            <button className="button small" key={name} onClick={() => openUrl(url)} type="button">
              Get my {name} export
            </button>
          ))}{" "}
          Save it to Downloads, then import again.
        </p>
      </section>
      {error && <p className="error-banner">{error}</p>}

      <h2 className="day-label">Waiting for you ({pending?.length ?? 0})</h2>
      {pending !== null && pending.length === 0 && <Empty>Nothing to review. Import to find statements.</Empty>}
      {!!pending?.length && (
        <p>
          <button className="button small" onClick={() => void act("knowledge_decide_all", { decision: "reject" })} type="button">
            Reject all {pending.length}
          </button>
        </p>
      )}
      <ul className="lessons">
        {pending?.map((s) => (
          <li className="lesson card pending" key={s.id}>
            <p className="lesson-text">{s.text}</p>
            <div className="lesson-foot">
              <span className="run-meta">
                {s.kind} · from {SOURCE[s.platform] ?? s.platform}
                {s.project ? ` · project ${s.project}` : ""}
              </span>
              <span className="lesson-actions">
                <button className="button small good" onClick={() => void act("knowledge_decide", { id: s.id, decision: "approve" })} type="button">
                  <Check size={15} aria-hidden="true" /> Approve
                </button>
                <button
                  className="button small"
                  onClick={() => void act("knowledge_decide", { id: s.id, decision: "keep" })}
                  title="Use it, but do not show it to agents"
                  type="button"
                >
                  <EyeOff size={15} aria-hidden="true" /> Keep private
                </button>
                <button className="button small danger" onClick={() => void act("knowledge_decide", { id: s.id, decision: "reject" })} type="button">
                  <X size={15} aria-hidden="true" /> Reject
                </button>
              </span>
            </div>
          </li>
        ))}
      </ul>

      <h2 className="day-label">Approved ({approved.length})</h2>
      {approved.length > 0 && (
        <p>
          <button className="button small" onClick={() => void copyPack()} type="button">
            <Copy size={15} aria-hidden="true" /> {copied ? "Copied" : "Copy as context for an assistant"}
          </button>{" "}
          <span className="muted">Paste it into ChatGPT, Claude, or an AGENTS.md file. Private statements are left out.</span>
        </p>
      )}
      <ul className="lessons">
        {approved.map((s) => (
          <li className="lesson card" key={s.id}>
            <p className="lesson-text">{s.text}</p>
            <div className="lesson-foot">
              <span className="run-meta">
                {s.kind} · from {SOURCE[s.platform] ?? s.platform}
                {s.sensitivity === "sensitive" ? " · private" : ""}
                {s.project ? ` · project ${s.project}` : ""}
              </span>
              <button className="icon-button" onClick={() => void act("knowledge_delete", { id: s.id })} type="button" aria-label={`Delete statement ${s.id}`}>
                <Trash2 size={16} />
              </button>
            </div>
          </li>
        ))}
      </ul>
      <p className="muted">
        Project and private limits decide what an assistant is shown. They are not a security boundary: any program
        running as you can read the store file.
      </p>
    </>
  );
}
