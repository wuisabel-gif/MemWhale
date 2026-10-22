// Sample data for the browser preview (`npm run dev` without the desktop
// shell). Nothing here touches a real store.
import type { Lesson, Overview, RecallEvent, Run, RunDetail, SearchHit } from "./types";

const minutesAgo = (m: number) => new Date(Date.now() - m * 60_000).toISOString();

const runs: Run[] = [
  { id: 412, command: "cargo build --release", cwd: "/Users/you/code/robot-arm", agent: "claude", exit_code: 101, created_at: minutesAgo(9), notes: "" },
  { id: 411, command: "git status --short", cwd: "/Users/you/code/robot-arm", agent: "claude", exit_code: 0, created_at: minutesAgo(11), notes: "" },
  { id: 405, command: "npm test", cwd: "/Users/you/code/site", agent: "codex", exit_code: 1, created_at: minutesAgo(52), notes: "" },
  { id: 404, command: "npm ci", cwd: "/Users/you/code/site", agent: "codex", exit_code: 0, created_at: minutesAgo(55), notes: "" },
  { id: 398, command: "python3 -m pytest tests/test_planner.py", cwd: "/Users/you/code/robot-arm", agent: "cursor", exit_code: 2, created_at: minutesAgo(190), notes: "" },
  { id: 377, command: "xcode-select --install", cwd: "/Users/you/code/robot-arm", agent: "terminal", exit_code: 0, created_at: minutesAgo(1500), notes: "project:robot-arm linker needs the Command Line Tools" },
  { id: 376, command: "cargo build --release", cwd: "/Users/you/code/robot-arm", agent: "terminal", exit_code: 101, created_at: minutesAgo(1504), notes: "" },
  { id: 350, command: "docker compose up -d db", cwd: "/Users/you/code/api", agent: "claude", exit_code: 0, created_at: minutesAgo(2900), notes: "" }
];

const output: Record<number, Pick<RunDetail, "stdout" | "stderr">> = {
  412: { stdout: "   Compiling arm-control v0.4.0", stderr: "error: linker `cc` not found\n  |\n  = note: No such file or directory (os error 2)\n\nerror: could not compile `arm-control` (bin \"arm\") due to 1 previous error" },
  405: { stdout: "FAIL tests/nav.test.ts\n  ● menu › opens on click", stderr: "Expected: true\nReceived: false" },
  398: { stdout: "", stderr: "ModuleNotFoundError: No module named 'numpy'" },
  376: { stdout: "", stderr: "error: linker `cc` not found" }
};

const events: RecallEvent[] = [
  { id: 31, kind: "note", at: minutesAgo(9), agent: "claude", cwd: "/Users/you/code/robot-arm", command: "cargo build --release", text: "MemoryWhale: `cargo build` failed here before (linker `cc` not found); seen 2 times; worked 1 of 1 times, run in a terminal yesterday, possible fix: xcode-select --install", outcome: "followed, worked" },
  { id: 30, kind: "warning", at: minutesAgo(48), agent: "codex", cwd: "/Users/you/code/site", command: "npm test", text: "MemoryWhale: `npm test` failed in this directory 4 min ago and has not passed since. Lesson: run npm ci after switching branches.", outcome: "then passed" },
  { id: 27, kind: "note", at: minutesAgo(190), agent: "cursor", cwd: "/Users/you/code/robot-arm", command: "python3 -m pytest tests/test_planner.py", text: "MemoryWhale: seen 3 times; worked 2 of 3 times, last run by claude 2 days ago, confirmed fix: python3 -m pip install -r requirements.txt", outcome: "not followed" }
];

let lessons: Lesson[] = [
  { id: 9, label: "On this Mac, cargo build needs `xcode-select --install` before the first build.", created_at: minutesAgo(1490), approved: true, provenance: "you, yesterday" },
  { id: 8, label: "The site tests need `npm ci` after switching branches; stale node_modules fail the nav test.", created_at: minutesAgo(4000), approved: true, provenance: "codex, 3 days ago" },
  { id: 7, label: "docker compose needs the db service up before the API tests run.", created_at: minutesAgo(9000), approved: false, provenance: "claude, 6 days ago (pending review)" }
];

const overview = (): Overview => ({
  commands: 1284,
  failures: 97,
  sessions: 12,
  lessons: lessons.length,
  pending_lessons: lessons.filter((l) => !l.approved).length,
  agents: [["claude", 842], ["terminal", 301], ["codex", 96], ["cursor", 45]],
  recall: { shown: 14, followed: 6, worked: 5, warnings: 4, warned_then_passed: 3 },
  store: "/Users/you/Library/Application Support/MemoryWhale/memorywhale.sqlite3"
});

function search(query: string): SearchHit[] {
  const words = query.toLowerCase().split(/\s+/).filter(Boolean);
  const texts = [
    ...runs.map((r) => ({ source: "command" as const, row_id: r.id, text: `${r.command}\n${output[r.id]?.stderr ?? ""}`.trim(), at: r.created_at, agent: r.agent })),
    ...lessons.map((l) => ({ source: "note" as const, row_id: l.id, text: l.label, at: l.created_at, agent: "terminal" as const }))
  ];
  return texts
    .map((t, i) => ({ ...t, hits: words.filter((w) => t.text.toLowerCase().includes(w)).length, i }))
    .filter((t) => t.hits > 0)
    .sort((a, b) => b.hits - a.hits)
    .map((t) => {
      const score = Math.round((60 * t.hits) / words.length + 30);
      return {
        id: t.i,
        source: t.source,
        row_id: t.row_id,
        text: t.text,
        score,
        reasons: [`matched ${t.hits} of ${words.length} words`, t.source === "note" ? "saved lesson" : "recent"],
        signals: [
          { name: "lexical", score: score / 100, applicable: true, detail: `${t.hits}/${words.length} query words` },
          { name: "recency", score: 0.7, applicable: true, detail: "within the last week" },
          { name: "semantic", score: 0, applicable: false, detail: "Ollama not running" }
        ],
        created_at: t.at,
        agent: t.agent
      };
    });
}

export async function mockCall<T>(command: string, args: Record<string, unknown> = {}): Promise<T> {
  const result = ((): unknown => {
    switch (command) {
      case "overview":
        return overview();
      case "recall_history":
        return events.slice(0, Number(args.limit ?? 100));
      case "list_runs": {
        const q = String(args.query ?? "").toLowerCase();
        return runs
          .filter((r) => !args.failuresOnly || r.exit_code !== 0)
          .filter((r) => `${r.command} ${r.cwd} ${output[r.id]?.stderr ?? ""}`.toLowerCase().includes(q))
          .slice(0, Number(args.limit ?? 200));
      }
      case "run_detail": {
        const run = runs.find((r) => r.id === args.id);
        if (!run) throw new Error("command run not found");
        const next = runs.filter((r) => r.id > run.id && r.cwd === run.cwd).reverse().slice(0, 5);
        return { run, stdout: output[run.id]?.stdout ?? "", stderr: output[run.id]?.stderr ?? "", next };
      }
      case "search":
        return search(String(args.query ?? ""));
      case "list_lessons":
        return lessons;
      case "add_lesson": {
        const id = Math.max(0, ...lessons.map((l) => l.id)) + 1;
        lessons = [{ id, label: String(args.text), created_at: new Date().toISOString(), approved: true, provenance: "you, just now" }, ...lessons];
        return id;
      }
      case "approve_lesson":
        lessons = lessons.map((l) => (l.id === args.id ? { ...l, approved: true, provenance: l.provenance.replace(" (pending review)", "") } : l));
        return null;
      case "delete_lesson":
        lessons = lessons.filter((l) => l.id !== args.id);
        return null;
      default:
        throw new Error(`unknown command ${command}`);
    }
  })();
  return result as T;
}
