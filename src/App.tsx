import {
  BookOpen,
  Database,
  FilePlus2,
  Filter,
  Network,
  RefreshCcw,
  Search,
  Sparkles,
  Terminal
} from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { DetailsPanel } from "./components/DetailsPanel";
import { Galaxy } from "./components/Galaxy";
import type {
  CommandRun,
  Document,
  GraphNode,
  GraphPayload,
  Lesson,
  RecallHit,
  SearchResult,
  SignalView,
  TerminalMemory
} from "./types";

const emptyGraph: GraphPayload = {
  documents: [],
  concepts: [],
  quotes: [],
  nodes: [],
  links: []
};

const demoContent = `# MemoryWhale MVP

Rust and Tauri make a fast local desktop app. SQLite stores documents,
concepts, quotes, and links without cloud sync.

Graph visualization helps connect source notes, transcript ideas, and
recurring topics like Rust, robotics, inference, batching, and knowledge graphs.
`;

function isTauriRuntime() {
  return "__TAURI_INTERNALS__" in window;
}

async function callBackend<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (isTauriRuntime()) {
    return invoke<T>(command, args);
  }
  return mockBackend<T>(command, args);
}

function App() {
  const [graph, setGraph] = useState<GraphPayload>(emptyGraph);
  const [terminalMemory, setTerminalMemory] = useState<TerminalMemory>({ runs: [], arguments: [] });
  const [recallQuery, setRecallQuery] = useState("");
  const [recallHits, setRecallHits] = useState<RecallHit[]>([]);
  const [openHit, setOpenHit] = useState<number | null>(null);
  const recallTimer = useRef<number | null>(null);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [query, setQuery] = useState("");
  const [searchResult, setSearchResult] = useState<SearchResult | null>(null);
  const [terminalResult, setTerminalResult] = useState<TerminalMemory | null>(null);
  const [title, setTitle] = useState("");
  const [sourceType, setSourceType] = useState("note");
  const [content, setContent] = useState(demoContent);
  const [commandLine, setCommandLine] = useState("cargo check --manifest-path MemoryWhale/src-tauri/Cargo.toml");
  const [cwd, setCwd] = useState("MemoryWhale");
  const [exitCode, setExitCode] = useState("127");
  const [stdout, setStdout] = useState("");
  const [stderr, setStderr] = useState("zsh:1: command not found: cargo");
  const [commandNotes, setCommandNotes] = useState("Terminal could not verify Rust because cargo is missing.");
  const [status, setStatus] = useState("Ready");
  const [lessons, setLessons] = useState<Lesson[]>([]);
  const [agentOnly, setAgentOnly] = useState(false);

  useEffect(() => {
    void refreshGraph();
  }, []);

  useEffect(() => {
    void refreshLessons();
  }, [agentOnly]);

  async function refreshLessons() {
    try {
      const next = await callBackend<Lesson[]>("list_lessons", { agentOnly });
      setLessons(next);
    } catch {
      setLessons([]);
    }
  }

  async function deleteLesson(id: number) {
    await callBackend("delete_lesson", { id });
    await refreshLessons();
  }

  async function approveLesson(id: number) {
    await callBackend("approve_lesson", { id });
    await refreshLessons();
  }

  const selected = useMemo(() => {
    if (!selectedId) return null;
    if (selectedId.startsWith("document:")) {
      const id = Number(selectedId.split(":")[1]);
      return {
        kind: "document" as const,
        value: graph.documents.find((doc) => doc.id === id)
      };
    }
    if (selectedId.startsWith("command:")) {
      const id = Number(selectedId.split(":")[1]);
      return {
        kind: "command" as const,
        value: terminalMemory.runs.find((run) => run.id === id)
      };
    }
    const id = Number(selectedId.split(":")[1]);
    return {
      kind: "concept" as const,
      value: graph.concepts.find((concept) => concept.id === id)
    };
  }, [graph, selectedId, terminalMemory]);

  const connectedDocuments = useMemo(() => {
    if (!selectedId) return [];
    const neighbors = new Set<string>();
    graph.links.forEach((link) => {
      if (link.source === selectedId) neighbors.add(link.target);
      if (link.target === selectedId) neighbors.add(link.source);
    });
    if (selectedId.startsWith("document:")) {
      neighbors.add(selectedId);
    }
    return graph.documents.filter((doc) => neighbors.has(`document:${doc.id}`));
  }, [graph, selectedId]);

  const topConcepts = useMemo(() => {
    const weights = new Map(graph.nodes.map((node) => [node.id, node.weight]));
    return [...graph.concepts]
      .sort((a, b) => (weights.get(`concept:${b.id}`) ?? 0) - (weights.get(`concept:${a.id}`) ?? 0))
      .slice(0, 12);
  }, [graph]);

  async function refreshGraph() {
    const next = await callBackend<GraphPayload>("get_graph");
    const terminal = await callBackend<TerminalMemory>("list_terminal_memory");
    setGraph(next);
    setTerminalMemory(terminal);
    if (!selectedId && next.nodes.length > 0) {
      setSelectedId(next.nodes[0].id);
    }
    setStatus(isTauriRuntime() ? "SQLite connected" : "Browser demo store");
  }

  async function importNote() {
    if (!content.trim()) {
      setStatus("Add text before importing");
      return;
    }
    await callBackend<Document>("import_text", {
      request: {
        title: title.trim() || null,
        source_type: sourceType,
        content
      }
    });
    setTitle("");
    setContent("");
    setStatus("Imported memory and rebuilt graph");
    await refreshGraph();
  }

  async function importFiles(files: FileList | null) {
    if (!files?.length) return;
    for (const file of Array.from(files)) {
      const text = await file.text();
      await callBackend<Document>("import_text", {
        request: {
          title: file.name.replace(/\.[^.]+$/, ""),
          source_type: file.name.endsWith(".md") ? "markdown" : "text",
          content: text
        }
      });
    }
    setStatus(`Imported ${files.length} file${files.length === 1 ? "" : "s"}`);
    await refreshGraph();
  }

  async function runSearch(nextQuery = query) {
    setQuery(nextQuery);
    if (!nextQuery.trim()) {
      setSearchResult(null);
      setTerminalResult(null);
      return;
    }
    const result = await callBackend<SearchResult>("search_memory", { query: nextQuery });
    const terminal = await callBackend<TerminalMemory>("list_terminal_memory", { query: nextQuery });
    setSearchResult(result);
    setTerminalResult(terminal);
  }

  // Debounced so semantic mode doesn't re-embed on every keystroke.
  function runRecall(nextQuery = recallQuery) {
    setRecallQuery(nextQuery);
    setOpenHit(null);
    if (recallTimer.current) window.clearTimeout(recallTimer.current);
    if (!nextQuery.trim()) {
      setRecallHits([]);
      return;
    }
    recallTimer.current = window.setTimeout(() => {
      void callBackend<RecallHit[]>("recall_memories", { query: nextQuery, limit: 8 }).then(setRecallHits);
    }, 350);
  }

  async function rememberCommand() {
    if (!commandLine.trim()) {
      setStatus("Add a command before saving terminal memory");
      return;
    }
    await callBackend<CommandRun>("remember_command_run", {
      request: {
        command_line: commandLine,
        cwd: cwd.trim() || null,
        exit_code: exitCode.trim() === "" ? null : Number(exitCode),
        stdout,
        stderr,
        notes: commandNotes
      }
    });
    setStatus("Remembered terminal command, arguments, and error log");
    setStdout("");
    setStderr("");
    setCommandNotes("");
    await refreshGraph();
  }

  async function resetDemoData() {
    const next = await callBackend<GraphPayload>("reset_demo_data");
    const terminal = await callBackend<TerminalMemory>("list_terminal_memory");
    setGraph(next);
    setTerminalMemory(terminal);
    setSelectedId(next.nodes[0]?.id ?? null);
    setSearchResult(null);
    setTerminalResult(null);
    setStatus("Loaded demo knowledge galaxy");
  }

  return (
    <main className="shell">
      <aside className="sidebar" aria-label="Sources and import">
        <div className="brand">
          <div className="brand-mark" role="img" aria-label="MemoryWhale whale mark">
            <span className="whale-body" />
            <span className="whale-tail" />
          </div>
          <div>
            <h1>MemoryWhale</h1>
            <p>Local knowledge galaxy</p>
          </div>
        </div>

        <section className="panel import-panel">
          <div className="panel-title">
            <FilePlus2 size={17} />
            <span>Import</span>
          </div>
          <input
            className="text-input"
            value={title}
            onChange={(event) => setTitle(event.target.value)}
            placeholder="Title"
          />
          <select
            className="text-input"
            value={sourceType}
            onChange={(event) => setSourceType(event.target.value)}
            aria-label="Source type"
          >
            <option value="note">Note</option>
            <option value="markdown">Markdown</option>
            <option value="youtube_transcript">YouTube transcript</option>
            <option value="web_article">Web article text</option>
            <option value="text">Plain text</option>
          </select>
          <textarea
            value={content}
            onChange={(event) => setContent(event.target.value)}
            placeholder="Paste notes, transcripts, or article text"
          />
          <div className="button-row">
            <button className="primary-button" onClick={importNote} type="button">
              <Sparkles size={16} />
              Import
            </button>
            <label className="icon-button" title="Import .txt or .md files">
              <BookOpen size={17} />
              <input
                type="file"
                accept=".txt,.md,.markdown,text/plain,text/markdown"
                multiple
                onChange={(event) => void importFiles(event.currentTarget.files)}
              />
            </label>
          </div>
        </section>

        <section className="panel terminal-panel">
          <div className="panel-title">
            <Terminal size={17} />
            <span>Terminal Memory</span>
          </div>
          <input
            className="text-input mono-input"
            value={commandLine}
            onChange={(event) => setCommandLine(event.target.value)}
            placeholder="cargo check --manifest-path app/Cargo.toml"
          />
          <div className="terminal-grid">
            <input
              className="text-input mono-input"
              value={cwd}
              onChange={(event) => setCwd(event.target.value)}
              placeholder="cwd"
            />
            <input
              className="text-input mono-input"
              value={exitCode}
              onChange={(event) => setExitCode(event.target.value)}
              placeholder="exit"
              inputMode="numeric"
            />
          </div>
          <textarea
            className="terminal-textarea"
            value={stderr}
            onChange={(event) => setStderr(event.target.value)}
            placeholder="stderr / error log"
          />
          <textarea
            className="terminal-textarea"
            value={stdout}
            onChange={(event) => setStdout(event.target.value)}
            placeholder="stdout"
          />
          <textarea
            className="terminal-textarea"
            value={commandNotes}
            onChange={(event) => setCommandNotes(event.target.value)}
            placeholder="What should MemoryWhale remember?"
          />
          <button className="primary-button" onClick={() => void rememberCommand()} type="button">
            <Terminal size={16} />
            Remember Command
          </button>
        </section>

        <section className="panel">
          <div className="panel-title">
            <Search size={17} />
            <span>Search</span>
          </div>
          <div className="search-box">
            <Search size={16} />
            <input
              value={query}
              onChange={(event) => void runSearch(event.target.value)}
              placeholder="Rust, robotics, graphs..."
            />
          </div>
          <div className="result-list">
            {(searchResult?.concepts ?? topConcepts).slice(0, 8).map((concept) => (
              <button
                className="source-item"
                key={`concept-${concept.id}`}
                onClick={() => setSelectedId(`concept:${concept.id}`)}
                type="button"
              >
                <span className="dot concept-dot" />
                <span>{concept.name}</span>
              </button>
            ))}
            {(searchResult?.documents ?? graph.documents).slice(0, 8).map((doc) => (
              <button
                className="source-item"
                key={`doc-${doc.id}`}
                onClick={() => setSelectedId(`document:${doc.id}`)}
                type="button"
              >
                <span className="dot document-dot" />
                <span>{doc.title}</span>
              </button>
            ))}
            {(terminalResult?.runs ?? terminalMemory.runs).slice(0, 8).map((run) => (
              <button
                className="source-item"
                key={`run-${run.id}`}
                onClick={() => setSelectedId(`command:${run.id}`)}
                type="button"
              >
                <span className={`dot ${run.exit_code === 0 ? "command-dot" : "error-dot"}`} />
                <span>{run.command}</span>
              </button>
            ))}
          </div>
        </section>

        <section className="panel">
          <div className="panel-title">
            <span>Remembered lessons</span>
            <label style={{ marginLeft: "auto", fontSize: "0.8rem", display: "flex", gap: "0.35rem", alignItems: "center" }}>
              <input
                type="checkbox"
                checked={agentOnly}
                onChange={(event) => setAgentOnly(event.target.checked)}
              />
              Agent-written only
            </label>
          </div>
          <div className="result-list">
            {lessons.length === 0 && <span className="source-item">No lessons yet.</span>}
            {lessons.map((lesson) => (
              <div className="source-item" key={`lesson-${lesson.id}`} style={{ flexDirection: "column", alignItems: "stretch", gap: "0.25rem" }}>
                <span>{lesson.label}{lesson.approved ? "" : " (pending review)"}</span>
                <span style={{ fontSize: "0.75rem", opacity: 0.7 }}>{lesson.provenance}</span>
                <span style={{ display: "flex", gap: "0.5rem" }}>
                  {!lesson.approved && (
                    <button type="button" onClick={() => void approveLesson(lesson.id)}>Approve</button>
                  )}
                  <button type="button" onClick={() => void deleteLesson(lesson.id)}>Delete</button>
                </span>
              </div>
            ))}
          </div>
        </section>

        <RecallPanel
          query={recallQuery}
          hits={recallHits}
          openId={openHit}
          onQuery={(q) => void runRecall(q)}
          onToggle={(id) => setOpenHit(openHit === id ? null : id)}
        />
      </aside>

      <section className="workspace" aria-label="Knowledge galaxy">
        <header className="topbar">
          <div className="metrics">
            <Metric icon={<Database size={16} />} label="Documents" value={graph.documents.length} />
            <Metric icon={<Network size={16} />} label="Concepts" value={graph.concepts.length} />
            <Metric icon={<Filter size={16} />} label="Links" value={graph.links.length} />
            <Metric icon={<Terminal size={16} />} label="Commands" value={terminalMemory.runs.length} />
          </div>
          <div className="actions">
            <span className="status">{status}</span>
            <button className="icon-button" onClick={() => void refreshGraph()} title="Refresh graph" type="button">
              <RefreshCcw size={17} />
            </button>
            <button className="secondary-button" onClick={() => void resetDemoData()} type="button">
              Demo Data
            </button>
          </div>
        </header>

        <div className="galaxy-layout">
          <Galaxy graph={graph} selectedId={selectedId} onSelect={setSelectedId} />
          <DetailsPanel
            selected={selected}
            documents={connectedDocuments}
            quotes={graph.quotes}
            terminalArguments={terminalMemory.arguments}
            links={graph.links.filter((link) => link.source === selectedId || link.target === selectedId)}
          />
        </div>
      </section>
    </main>
  );
}

function RecallPanel({
  query,
  hits,
  openId,
  onQuery,
  onToggle
}: {
  query: string;
  hits: RecallHit[];
  openId: number | null;
  onQuery: (q: string) => void;
  onToggle: (id: number) => void;
}) {
  return (
    <section className="panel recall-panel">
      <div className="panel-title">
        <Sparkles size={17} />
        <span>Recall</span>
      </div>
      <div className="search-box">
        <Search size={16} />
        <input value={query} onChange={(e) => onQuery(e.target.value)} placeholder="Ask your memory…" />
      </div>
      <div className="recall-list">
        {hits.map((h) => (
          <div className="recall-hit" key={h.id}>
            <button className="recall-head" onClick={() => onToggle(h.id)} type="button">
              <span className="recall-score">{h.score}%</span>
              <span className="recall-text">{h.text}</span>
            </button>
            <div className="recall-reasons">
              {h.reasons.slice(0, 4).map((r, i) => (
                <span className="reason" key={i}>
                  {r}
                </span>
              ))}
            </div>
            {openId === h.id && (
              <div className="recall-explain">
                {h.signals.map((s) => (
                  <div className={`sig ${s.applicable ? "" : "sig-off"}`} key={s.name}>
                    <span className="sig-name">{s.name}</span>
                    <span className="sig-bar">
                      <span style={{ width: `${Math.round(s.score * 100)}%` }} />
                    </span>
                    <span className="sig-detail">{s.detail}</span>
                  </div>
                ))}
                <div className="sig-meta">
                  mentioned {h.mentions}× · importance {h.importance.toFixed(2)}
                  {h.tags.length ? ` · ${h.tags.join(", ")}` : ""}
                </div>
              </div>
            )}
          </div>
        ))}
        {query.trim() !== "" && hits.length === 0 && (
          <p className="muted">No matches — import notes or save terminal commands, then recall them here.</p>
        )}
      </div>
    </section>
  );
}

function Metric({ icon, label, value }: { icon: React.ReactNode; label: string; value: number }) {
  return (
    <div className="metric">
      {icon}
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

async function mockBackend<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  await new Promise((resolve) => window.setTimeout(resolve, 80));
  const store = loadMockStore();
  const terminal = loadMockTerminal();

  if (command === "get_graph") return graphWithCommands(store, terminal) as T;
  if (command === "list_terminal_memory") {
    const query = String(args?.query ?? "").toLowerCase();
    if (!query) return terminal as T;
    return {
      runs: terminal.runs.filter((run) =>
        `${run.command} ${run.argv_json} ${run.cwd ?? ""} ${run.stdout} ${run.stderr} ${run.notes}`.toLowerCase().includes(query)
      ),
      arguments: terminal.arguments.filter((argument) => argument.value.toLowerCase().includes(query))
    } as T;
  }
  if (command === "reset_demo_data") {
    localStorage.removeItem("memorywhale-demo");
    localStorage.removeItem("memorywhale-terminal-demo");
    seedMockTerminal();
    return graphWithCommands(seedMockStore(), loadMockTerminal()) as T;
  }
  if (command === "search_memory") {
    const query = String(args?.query ?? "").toLowerCase();
    return {
      documents: store.documents.filter((doc) => `${doc.title} ${doc.content} ${doc.summary}`.toLowerCase().includes(query)),
      concepts: store.concepts.filter((concept) => concept.name.toLowerCase().includes(query))
    } as T;
  }
  if (command === "import_text") {
    const request = args?.request as { title?: string | null; source_type: string; content: string };
    const next = addMockDocument(store, request.title || inferTitle(request.content), request.source_type, request.content);
    saveMockStore(next);
    return next.documents[0] as T;
  }
  if (command === "remember_command_run") {
    const request = args?.request as {
      command_line: string;
      cwd?: string | null;
      exit_code?: number | null;
      stdout?: string;
      stderr?: string;
      notes?: string;
    };
    const next = addMockCommand(terminal, request);
    saveMockTerminal(next);
    return next.runs[0] as T;
  }
  if (command === "recall_memories") {
    return demoRecallHits() as T;
  }
  if (command === "explain_memory") {
    const id = Number(args?.id);
    return (demoRecallHits().find((h) => h.id === id) ?? null) as T;
  }
  throw new Error(`Mock backend does not implement ${command}`);
}

function demoRecallHits(): RecallHit[] {
  const sig = (
    name: string,
    weight: number,
    score: number,
    applicable: boolean,
    detail: string
  ): SignalView => ({ name, weight, score, applicable, contribution: applicable ? weight * score : 0, detail });
  return [
    {
      id: 1000000002,
      text: "cargo check --manifest-path MemoryWhale/src-tauri/Cargo.toml zsh:1: command not found: cargo",
      score: 68,
      reasons: ["62% term overlap (lexical)", "mentioned 4×", "importance 0.65"],
      signals: [
        sig("similarity", 0.4, 0.62, true, "62% term overlap (lexical)"),
        sig("recency", 0.2, 0.55, true, "last used 2 days ago"),
        sig("importance", 0.15, 0.65, true, "importance 0.65"),
        sig("reinforcement", 0.1, 0.5, true, "mentioned 4×"),
        sig("task", 0.15, 0, false, "no task context")
      ],
      created_at: "",
      last_used: "",
      mentions: 4,
      importance: 0.65,
      tags: ["command", "error"]
    },
    {
      id: 1,
      text: "Rust Desktop Systems. Rust, Tauri, and SQLite make local-first desktop software feel fast.",
      score: 41,
      reasons: ["38% term overlap (lexical)", "importance 0.50", "last used today"],
      signals: [
        sig("similarity", 0.4, 0.38, true, "38% term overlap (lexical)"),
        sig("recency", 0.2, 1.0, true, "last used today"),
        sig("importance", 0.15, 0.5, true, "importance 0.50"),
        sig("reinforcement", 0.1, 0.2, true, "mentioned 1×"),
        sig("task", 0.15, 0, false, "no task context")
      ],
      created_at: "",
      last_used: "",
      mentions: 1,
      importance: 0.5,
      tags: ["document", "markdown"]
    }
  ];
}

function loadMockStore() {
  const stored = localStorage.getItem("memorywhale-demo");
  if (stored) return JSON.parse(stored) as GraphPayload;
  return seedMockStore();
}

function seedMockStore() {
  let graph = { ...emptyGraph };
  graph = addMockDocument(graph, "Rust Desktop Systems", "markdown", demoContent);
  graph = addMockDocument(
    graph,
    "Transcript: Knowledge Galaxy",
    "youtube_transcript",
    "NotebookLM, Obsidian, and Roam show how connected notes help people think. MemoryWhale connects concepts, transcripts, quotes, and documents in a zoomable graph."
  );
  saveMockStore(graph);
  return graph;
}

function loadMockTerminal() {
  const stored = localStorage.getItem("memorywhale-terminal-demo");
  if (stored) return JSON.parse(stored) as TerminalMemory;
  return seedMockTerminal();
}

function seedMockTerminal() {
  let terminal: TerminalMemory = { runs: [], arguments: [] };
  terminal = addMockCommand(terminal, {
    command_line: "npm run build",
    cwd: "MemoryWhale",
    exit_code: 0,
    stdout: "tsc && vite build completed successfully",
    stderr: "",
    notes: "Frontend build passed."
  });
  terminal = addMockCommand(terminal, {
    command_line: "cargo check --manifest-path MemoryWhale/src-tauri/Cargo.toml",
    cwd: "MemoryWhale",
    exit_code: 127,
    stdout: "",
    stderr: "zsh:1: command not found: cargo",
    notes: "Rust verification needs cargo installed."
  });
  saveMockTerminal(terminal);
  return terminal;
}

function saveMockStore(graph: GraphPayload) {
  localStorage.setItem("memorywhale-demo", JSON.stringify(graph));
}

function saveMockTerminal(terminal: TerminalMemory) {
  localStorage.setItem("memorywhale-terminal-demo", JSON.stringify(terminal));
}

function addMockCommand(
  terminal: TerminalMemory,
  request: {
    command_line: string;
    cwd?: string | null;
    exit_code?: number | null;
    stdout?: string;
    stderr?: string;
    notes?: string;
  }
): TerminalMemory {
  const runId = Math.max(0, ...terminal.runs.map((run) => run.id)) + 1;
  const argv = splitMockCommand(request.command_line);
  const run: CommandRun = {
    id: runId,
    command: argv[0] ?? request.command_line,
    argv_json: JSON.stringify(argv),
    cwd: request.cwd,
    exit_code: request.exit_code,
    stdout: request.stdout ?? "",
    stderr: request.stderr ?? "",
    notes: request.notes ?? "",
    created_at: new Date().toISOString()
  };
  const nextArgId = Math.max(0, ...terminal.arguments.map((argument) => argument.id)) + 1;
  const args = argv.map((value, index) => ({
    id: nextArgId + index,
    command_run_id: runId,
    position: index,
    value
  }));
  return {
    runs: [run, ...terminal.runs],
    arguments: [...args, ...terminal.arguments]
  };
}

function graphWithCommands(graph: GraphPayload, terminal: TerminalMemory): GraphPayload {
  const commandNodes = terminal.runs.map((run) => ({
    id: `command:${run.id}`,
    label: `${run.command} (${run.exit_code === 0 ? "ok" : "error"})`,
    node_type: "command",
    weight: run.exit_code === 0 ? 2 : 4
  }));
  return {
    ...graph,
    nodes: [...graph.nodes, ...commandNodes]
  };
}

function addMockDocument(graph: GraphPayload, title: string, sourceType: string, content: string): GraphPayload {
  const docId = Math.max(0, ...graph.documents.map((doc) => doc.id)) + 1;
  const doc: Document = {
    id: docId,
    title,
    source_type: sourceType,
    content,
    summary: content.replace(/\s+/g, " ").slice(0, 260),
    created_at: new Date().toISOString()
  };
  const conceptNames = extractMockKeywords(content);
  const concepts = [...graph.concepts];
  const links = [...graph.links];
  const docNode = `document:${docId}`;

  for (const name of conceptNames) {
    let concept = concepts.find((item) => item.name === name);
    if (!concept) {
      concept = { id: Math.max(0, ...concepts.map((item) => item.id)) + 1, name, description: `Recurring idea extracted from local sources: ${name}` };
      concepts.push(concept);
    }
    links.push({ source: docNode, target: `concept:${concept.id}`, relation: "mentions", weight: 1 });
  }

  const docs = [doc, ...graph.documents];
  const nodes: GraphNode[] = [
    ...docs.map((item) => ({ id: `document:${item.id}`, label: item.title, node_type: "document", weight: 2 })),
    ...concepts.map((item) => ({
      id: `concept:${item.id}`,
      label: item.name,
      node_type: "concept",
      weight: links.filter((link) => link.source === `concept:${item.id}` || link.target === `concept:${item.id}`).length + 1
    }))
  ];

  return {
    documents: docs,
    concepts,
    quotes: graph.quotes,
    links,
    nodes
  };
}

function splitMockCommand(commandLine: string) {
  return commandLine.match(/(?:[^\s"']+|"[^"]*"|'[^']*')+/g)?.map((part) => part.replace(/^['"]|['"]$/g, "")) ?? [];
}

function inferTitle(content: string) {
  return content
    .split(/\n/)
    .map((line) => line.replace(/^#+/, "").trim())
    .find(Boolean) || "Untitled memory";
}

function extractMockKeywords(content: string) {
  const stop = new Set(["and", "the", "with", "that", "this", "from", "into", "your", "notes", "local", "first"]);
  const counts = new Map<string, number>();
  content
    .toLowerCase()
    .match(/[a-z][a-z0-9_-]{2,}/g)
    ?.forEach((word) => {
      if (!stop.has(word)) counts.set(word, (counts.get(word) ?? 0) + 1);
    });
  return [...counts.entries()]
    .sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0]))
    .slice(0, 10)
    .map(([word]) => word);
}

export default App;
