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
import * as d3 from "d3";
import { useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

type Document = {
  id: number;
  title: string;
  source_type: string;
  content: string;
  summary: string;
  created_at: string;
};

type Concept = {
  id: number;
  name: string;
  description?: string | null;
};

type Quote = {
  id: number;
  document_id: number;
  text: string;
};

type GraphNode = {
  id: string;
  label: string;
  node_type: "document" | "concept" | string;
  weight: number;
};

type GraphLink = {
  source: string;
  target: string;
  relation: string;
  weight: number;
};

type GraphPayload = {
  documents: Document[];
  concepts: Concept[];
  quotes: Quote[];
  nodes: GraphNode[];
  links: GraphLink[];
};

type CommandRun = {
  id: number;
  command: string;
  argv_json: string;
  cwd?: string | null;
  exit_code?: number | null;
  stdout: string;
  stderr: string;
  notes: string;
  created_at: string;
};

type CommandArgument = {
  id: number;
  command_run_id: number;
  position: number;
  value: string;
};

type TerminalMemory = {
  runs: CommandRun[];
  arguments: CommandArgument[];
};

type SearchResult = {
  documents: Document[];
  concepts: Concept[];
};

type SimNode = GraphNode & d3.SimulationNodeDatum;
type SimLink = GraphLink & d3.SimulationLinkDatum<SimNode>;

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

  useEffect(() => {
    void refreshGraph();
  }, []);

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
          <div className="brand-mark">MW</div>
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

function Metric({ icon, label, value }: { icon: React.ReactNode; label: string; value: number }) {
  return (
    <div className="metric">
      {icon}
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

function Galaxy({
  graph,
  selectedId,
  onSelect
}: {
  graph: GraphPayload;
  selectedId: string | null;
  onSelect: (id: string) => void;
}) {
  const svgRef = useRef<SVGSVGElement | null>(null);

  useEffect(() => {
    const svgEl = svgRef.current;
    if (!svgEl) return;

    const width = svgEl.clientWidth || 720;
    const height = svgEl.clientHeight || 560;
    const nodes: SimNode[] = graph.nodes.map((node) => ({ ...node }));
    const links: SimLink[] = graph.links.map((link) => ({ ...link }));
    const svg = d3.select(svgEl);
    svg.selectAll("*").remove();

    const defs = svg.append("defs");
    const glow = defs.append("filter").attr("id", "node-glow");
    glow.append("feGaussianBlur").attr("stdDeviation", "3.2").attr("result", "coloredBlur");
    const merge = glow.append("feMerge");
    merge.append("feMergeNode").attr("in", "coloredBlur");
    merge.append("feMergeNode").attr("in", "SourceGraphic");

    const root = svg.append("g");
    svg.call(
      d3
        .zoom<SVGSVGElement, unknown>()
        .scaleExtent([0.35, 4])
        .on("zoom", (event) => root.attr("transform", event.transform))
    );

    root
      .append("g")
      .attr("class", "starfield")
      .selectAll("circle")
      .data(d3.range(100))
      .join("circle")
      .attr("cx", () => Math.random() * width)
      .attr("cy", () => Math.random() * height)
      .attr("r", () => Math.random() * 1.25 + 0.25);

    const link = root
      .append("g")
      .attr("class", "links")
      .selectAll<SVGLineElement, SimLink>("line")
      .data(links)
      .join("line")
      .attr("stroke-width", (d) => Math.max(1, Math.min(5, d.weight)));

    const node = root
      .append("g")
      .attr("class", "nodes")
      .selectAll<SVGGElement, SimNode>("g")
      .data(nodes)
      .join("g")
      .attr("class", (d) => `graph-node ${d.node_type} ${d.id === selectedId ? "selected" : ""}`)
      .on("click", (_, d) => onSelect(d.id))
      .on("dblclick", (_, d) => {
        onSelect(d.id);
        svg
          .transition()
          .duration(450)
          .call(
            d3.zoom<SVGSVGElement, unknown>().transform,
            d3.zoomIdentity.translate(width / 2 - (d.x ?? 0) * 1.8, height / 2 - (d.y ?? 0) * 1.8).scale(1.8)
          );
      });

    node
      .append("circle")
      .attr("r", (d) => nodeRadius(d))
      .attr("filter", "url(#node-glow)");

    node
      .append("text")
      .text((d) => d.label)
      .attr("x", (d) => nodeRadius(d) + 8)
      .attr("y", 4);

    const simulation = d3
      .forceSimulation(nodes)
      .force(
        "link",
        d3
          .forceLink<SimNode, SimLink>(links)
          .id((d) => d.id)
          .distance((d) => (d.relation === "co_occurs" ? 82 : 135))
          .strength(0.58)
      )
      .force("charge", d3.forceManyBody().strength(-360))
      .force("center", d3.forceCenter(width / 2, height / 2))
      .force("collision", d3.forceCollide<SimNode>().radius((d) => nodeRadius(d) + 18))
      .on("tick", () => {
        link
          .attr("x1", (d) => (d.source as SimNode).x ?? 0)
          .attr("y1", (d) => (d.source as SimNode).y ?? 0)
          .attr("x2", (d) => (d.target as SimNode).x ?? 0)
          .attr("y2", (d) => (d.target as SimNode).y ?? 0);
        node.attr("transform", (d) => `translate(${d.x ?? 0},${d.y ?? 0})`);
      });

    node.call(
      d3
        .drag<SVGGElement, SimNode>()
        .on("start", (event, d) => {
          if (!event.active) simulation.alphaTarget(0.3).restart();
          d.fx = d.x;
          d.fy = d.y;
        })
        .on("drag", (event, d) => {
          d.fx = event.x;
          d.fy = event.y;
        })
        .on("end", (event, d) => {
          if (!event.active) simulation.alphaTarget(0);
          d.fx = null;
          d.fy = null;
        })
    );

    return () => {
      simulation.stop();
    };
  }, [graph, onSelect, selectedId]);

  if (graph.nodes.length === 0) {
    return (
      <div className="empty-galaxy">
        <Sparkles size={32} />
        <h2>Import a note to form the first constellation</h2>
        <p>MemoryWhale will extract concepts and connect them as local graph nodes.</p>
      </div>
    );
  }

  return <svg ref={svgRef} className="galaxy" role="img" aria-label="Interactive knowledge graph" />;
}

function DetailsPanel({
  selected,
  documents,
  quotes,
  terminalArguments,
  links
}: {
  selected:
    | { kind: "document"; value?: Document }
    | { kind: "concept"; value?: Concept }
    | { kind: "command"; value?: CommandRun }
    | null;
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

function nodeRadius(node: GraphNode) {
  const base = node.node_type === "document" ? 14 : node.node_type === "command" ? 12 : 9;
  return Math.max(base, Math.min(30, base + node.weight * 1.35));
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
  throw new Error(`Mock backend does not implement ${command}`);
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
