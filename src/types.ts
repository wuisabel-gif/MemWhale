/** Agent labels from `memorywhale_core::provenance::label`; `terminal` is a person at a prompt. */
export type Agent = "claude" | "rho" | "cursor" | "codewhale" | "codex" | "gemini" | "terminal" | "unknown";

export type RecallStats = {
  shown: number;
  followed: number;
  worked: number;
  warnings: number;
  warned_then_passed: number;
};

export type Overview = {
  commands: number;
  failures: number;
  sessions: number;
  lessons: number;
  pending_lessons: number;
  agents: [Agent, number][];
  recall: RecallStats;
  store: string;
};

/** One note (after a failure) or warning (before a command) as the agent saw it. */
export type RecallEvent = {
  id: number;
  kind: "note" | "warning";
  at: string;
  agent: string | null;
  cwd: string | null;
  command: string;
  text: string | null;
  outcome: string;
};

export type Run = {
  id: number;
  command: string;
  cwd: string | null;
  agent: Agent;
  exit_code: number | null;
  created_at: string;
  notes: string;
};

export type RunDetail = {
  run: Run;
  stdout: string;
  stderr: string;
  next: Run[];
};

export type Signal = {
  name: string;
  score: number;
  applicable: boolean;
  detail: string;
};

export type SearchHit = {
  id: number;
  source: "command" | "note" | "session" | "conversation" | "document";
  row_id: number;
  text: string;
  score: number;
  reasons: string[];
  signals: Signal[];
  created_at: string;
  agent: Agent;
};

export type Lesson = {
  id: number;
  label: string;
  created_at: string;
  approved: boolean;
  provenance: string;
};

export type Page = "home" | "search" | "recall" | "commands" | "lessons";

/** What every page gets from the app shell. */
export type PageProps = {
  overview: Overview | null;
  /** Changes whenever the store may have changed; pages reload on it. */
  refreshKey: number;
  refresh: () => void;
  go: (page: Page) => void;
  openRun: (id: number) => void;
};
