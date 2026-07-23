import { Sparkles } from "lucide-react";
import type { CommandArgument, CommandRun, Concept, Document, GraphLink, Quote } from "../types";

type Selection =
  | { kind: "document"; value?: Document }
  | { kind: "concept"; value?: Concept }
  | { kind: "command"; value?: CommandRun }
  | null;

export function DetailsPanel({
  selected,
  documents,
  quotes,
  terminalArguments,
  links
}: {
  selected: Selection;
  documents: Document[];
  quotes: Quote[];
  terminalArguments: CommandArgument[];
  links: GraphLink[];
}) {
  const relevantQuotes = quotes.filter((quote) => documents.some((doc) => doc.id === quote.document_id));
  const selectedCommand = selected?.kind === "command" ? selected.value : undefined;
  const selectedArgs = selectedCommand
    ? terminalArguments.filter((argument) => argument.command_run_id === selectedCommand.id)
    : [];

  return (
    <aside className="details" aria-label="Selected memory">
      <div className="panel-title">
        <Sparkles size={17} />
        <span>Selected Idea</span>
      </div>
      {!selected?.value ? (
        <div className="empty-details">Select a node to inspect its connected notes.</div>
      ) : selected.kind === "command" ? (
        <div>
          <p className={selected.value.exit_code === 0 ? "eyebrow success" : "eyebrow danger"}>terminal command</p>
          <h2>{selected.value.command}</h2>
          <p className="summary">
            Exit {selected.value.exit_code ?? "unknown"} · {selected.value.cwd || "cwd unknown"}
          </p>
          {selectedArgs.length > 0 && (
            <div className="arg-list">
              {selectedArgs.map((argument) => (
                <code key={argument.id}>{argument.value}</code>
              ))}
            </div>
          )}
        </div>
      ) : selected.kind === "document" ? (
        <div>
          <p className="eyebrow">{selected.value.source_type}</p>
          <h2>{selected.value.title}</h2>
          <p className="summary">{selected.value.summary || selected.value.content.slice(0, 320)}</p>
        </div>
      ) : (
        <div>
          <p className="eyebrow">concept</p>
          <h2>{selected.value.name}</h2>
          <p className="summary">{selected.value.description}</p>
        </div>
      )}

      <div className="detail-section">
        <h3>Connected Notes</h3>
        <div className="note-list">
          {documents.slice(0, 6).map((doc) => (
            <article className="note-card" key={doc.id}>
              <strong>{doc.title}</strong>
              <p>{doc.summary || doc.content.slice(0, 180)}</p>
            </article>
          ))}
          {documents.length === 0 && <p className="muted">No directly connected notes yet.</p>}
        </div>
      </div>

      {selectedCommand && (
        <div className="detail-section">
          <h3>Terminal Log</h3>
          {selectedCommand.notes && <p className="summary">{selectedCommand.notes}</p>}
          {selectedCommand.stderr && <pre className="terminal-log error-log">{selectedCommand.stderr}</pre>}
          {selectedCommand.stdout && <pre className="terminal-log">{selectedCommand.stdout}</pre>}
        </div>
      )}

      <div className="detail-section">
        <h3>Relationships</h3>
        <div className="link-list">
          {links.slice(0, 10).map((link) => (
            <span className="relationship" key={`${link.source}-${link.target}-${link.relation}`}>
              {link.relation.replace("_", " ")} · {link.weight}
            </span>
          ))}
          {links.length === 0 && <p className="muted">Import another related note to create links.</p>}
        </div>
      </div>

      <div className="detail-section">
        <h3>Quotes</h3>
        {relevantQuotes.slice(0, 4).map((quote) => (
          <blockquote key={quote.id}>{quote.text}</blockquote>
        ))}
        {relevantQuotes.length === 0 && <p className="muted">Quoted lines beginning with &gt; appear here.</p>}
      </div>
    </aside>
  );
}
