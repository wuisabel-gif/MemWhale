import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import App from "./App";
import { goTo, noRuns, overview, respond } from "./test/helpers";
import { invoke } from "./test/tauri";
import type { RecallEvent, Run, SearchHit } from "./types";

const minutesAgo = (m: number) => new Date(Date.now() - m * 60_000).toISOString();

const failedRun: Run = {
  id: 412,
  command: "cargo build --release",
  cwd: "/Users/ada/code/robot-arm",
  agent: "claude",
  exit_code: 101,
  created_at: minutesAgo(9),
  notes: "project:robot-arm"
};

const nextRun: Run = {
  id: 413,
  command: "git status --short",
  cwd: "/Users/ada/code/robot-arm",
  agent: "claude",
  exit_code: 0,
  created_at: minutesAgo(8),
  notes: ""
};

const commandHit: SearchHit = {
  id: 1,
  source: "command",
  row_id: 412,
  text: "cargo build --release\nerror: linker `cc` not found",
  score: 88,
  reasons: ["matched 1 of 1 words"],
  signals: [
    { name: "lexical", score: 0.88, applicable: true, detail: "1/1 query words" },
    { name: "recency", score: 0.7, applicable: true, detail: "within the last week" },
    { name: "semantic", score: 0, applicable: false, detail: "Ollama not running" }
  ],
  created_at: minutesAgo(9),
  agent: "claude"
};

const noteHit: SearchHit = {
  id: 2,
  source: "note",
  row_id: 9,
  text: "On this Mac, cargo build needs `xcode-select --install` before the first build.",
  score: 72,
  reasons: ["matched 1 of 1 words", "saved lesson"],
  signals: [{ name: "lexical", score: 0.72, applicable: true, detail: "1/1 query words" }],
  created_at: minutesAgo(1500),
  agent: "terminal"
};

const runDetail = { run: failedRun, stdout: "   Compiling arm-control v0.4.0", stderr: "error: linker `cc` not found", next: [nextRun] };

const warning: RecallEvent = {
  id: 30,
  kind: "warning",
  at: minutesAgo(1),
  agent: "codex",
  cwd: "/Users/ada/code/site",
  command: "npm test",
  stem: "npm test",
  text: "MemoryWhale: `npm test` failed in this directory 4 min ago and has not passed since. Lesson: run npm ci after switching branches.",
  outcome: "then passed"
};

const note: RecallEvent = {
  id: 31,
  kind: "note",
  at: minutesAgo(3),
  agent: "claude",
  cwd: "/Users/ada/code/robot-arm",
  command: "cargo build --release",
  stem: "cargo build",
  text: "MemoryWhale: seen 2 times; worked 1 of 1 times.",
  outcome: "followed, worked"
};

const searchField = () => screen.getByPlaceholderText(/linker cc not found/);

beforeEach(() => {
  invoke.mockReset();
});

// Vitest globals are off, so @testing-library/react does not auto-clean the
// document between tests.
afterEach(cleanup);

describe("Search page", () => {
  it("shows guidance for an empty query without calling search", async () => {
    // No "search" entry: any unexpected Tauri command fails loud in respond().
    respond({ overview, recall_history: [], list_runs: noRuns });

    render(<App />);
    goTo("Search");

    expect(await screen.findByText(/Ask your memory about an error/)).toBeTruthy();
    expect(invoke).not.toHaveBeenCalledWith("search", expect.anything());
  });

  it("invokes search with the typed query and renders ranked hits", async () => {
    const user = userEvent.setup();
    respond({ overview, recall_history: [], list_runs: noRuns, search: [commandHit, noteHit] });

    render(<App />);
    goTo("Search");
    await user.type(searchField(), "linker");

    await waitFor(() => expect(invoke).toHaveBeenLastCalledWith("search", { query: "linker", limit: 20 }));

    const hits = await screen.findAllByText(/linker `cc` not found/);
    expect(hits.length).toBeGreaterThan(0);
    const first = hits[0].closest("li")!;
    expect(within(first).getByText("command")).toBeTruthy();
    expect(within(first).getByText("88")).toBeTruthy();
    expect(within(first).getByText(/matched 1 of 1 words/)).toBeTruthy();
    expect(within(first).getByRole("button", { name: /Open the command/ })).toBeTruthy();

    // The saved lesson renders as a "lesson" source, not a raw "note".
    const second = screen.getByText(/xcode-select --install/).closest("li")!;
    expect(within(second).getByText("lesson")).toBeTruthy();
  });

  it("expands the per-signal breakdown on selection and collapses it again", async () => {
    const user = userEvent.setup();
    respond({ overview, recall_history: [], list_runs: noRuns, search: [commandHit] });

    render(<App />);
    goTo("Search");
    await user.type(searchField(), "linker");
    const first = (await screen.findAllByText(/linker `cc` not found/))[0].closest("li")!;

    const why = within(first).getByRole("button", { name: /Why this result/ });
    expect(why.getAttribute("aria-expanded")).toBe("false");
    expect(within(first).queryByText("Ollama not running")).toBeNull();

    fireEvent.click(why);
    expect(why.getAttribute("aria-expanded")).toBe("true");
    expect(within(first).getByText("lexical")).toBeTruthy();
    expect(within(first).getByText("recency")).toBeTruthy();
    expect(within(first).getByText("Ollama not running")).toBeTruthy();

    fireEvent.click(why);
    expect(why.getAttribute("aria-expanded")).toBe("false");
    expect(within(first).queryByText("Ollama not running")).toBeNull();
  });

  it("keeps the typed query in the nothing-matches empty state", async () => {
    const user = userEvent.setup();
    respond({ overview, recall_history: [], list_runs: noRuns, search: [] });

    render(<App />);
    goTo("Search");
    await user.type(searchField(), "zzz");

    const empty = await screen.findByText(/Nothing matches/);
    expect(empty.textContent).toContain("zzz");
  });
});

describe("Run details drawer", () => {
  it("loads the command's output and what ran next, then closes on Escape", async () => {
    const user = userEvent.setup();
    respond({ overview, recall_history: [], list_runs: noRuns, search: [commandHit], run_detail: runDetail });

    render(<App />);
    goTo("Search");
    await user.type(searchField(), "linker");
    fireEvent.click(await screen.findByRole("button", { name: /Open the command/ }));

    const dialog = await screen.findByRole("dialog", { name: "Command run" });
    expect(invoke).toHaveBeenCalledWith("run_detail", { id: 412 });
    expect(within(dialog).getByText("Command #412")).toBeTruthy();
    expect(within(dialog).getByText("cargo build --release")).toBeTruthy();
    expect(within(dialog).getByText("exit 101")).toBeTruthy();
    expect(within(dialog).getByText("Error output")).toBeTruthy();
    expect(within(dialog).getByText(/error: linker `cc` not found/)).toBeTruthy();
    expect(within(dialog).getByText(/Compiling arm-control/)).toBeTruthy();

    expect(within(dialog).getByText("What ran next here")).toBeTruthy();
    expect(within(dialog).getByText("git status --short")).toBeTruthy();

    fireEvent.keyDown(window, { key: "Escape" });
    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  });
});

describe("Recall page", () => {
  it("renders the event history with the exact agent-facing wording", async () => {
    respond({ overview, recall_history: [warning, note], list_runs: noRuns });

    render(<App />);
    goTo("Recall");

    const summary = await screen.findByText(/notes shown,/);
    expect(summary.textContent).toContain("7");

    expect(invoke).toHaveBeenCalledWith("recall_history", { limit: 200 });
    expect(screen.getByText(/Warning before/)).toBeTruthy();
    expect(screen.getByText(/Note after/)).toBeTruthy();
    expect(screen.getByText(/failed in this directory/)).toBeTruthy();
    expect(screen.getByText("then passed")).toBeTruthy();
    expect(screen.getByText("followed, worked")).toBeTruthy();
    expect(screen.getByText("~/code/site")).toBeTruthy();
  });
});
