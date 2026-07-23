export type Document = {
  id: number;
  title: string;
  source_type: string;
  content: string;
  summary: string;
  created_at: string;
};

export type Concept = {
  id: number;
  name: string;
  description?: string | null;
};

export type Quote = {
  id: number;
  document_id: number;
  text: string;
};

export type GraphNode = {
  id: string;
  label: string;
  node_type: "document" | "concept" | string;
  weight: number;
};

export type GraphLink = {
  source: string;
  target: string;
  relation: string;
  weight: number;
};

export type GraphPayload = {
  documents: Document[];
  concepts: Concept[];
  quotes: Quote[];
  nodes: GraphNode[];
  links: GraphLink[];
};

export type CommandRun = {
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

export type CommandArgument = {
  id: number;
  command_run_id: number;
  position: number;
  value: string;
};

export type TerminalMemory = {
  runs: CommandRun[];
  arguments: CommandArgument[];
};

export type SearchResult = {
  documents: Document[];
  concepts: Concept[];
};

export type Lesson = {
  id: number;
  label: string;
  created_at: string;
  author_kind: string;
  author_name: string | null;
  source_session_id: number | null;
  approved: boolean;
  provenance: string;
};

export type SignalView = {
  name: string;
  weight: number;
  score: number;
  applicable: boolean;
  contribution: number;
  detail: string;
};

export type RecallHit = {
  id: number;
  text: string;
  score: number;
  reasons: string[];
  signals: SignalView[];
  created_at: string;
  last_used: string;
  mentions: number;
  importance: number;
  tags: string[];
};
