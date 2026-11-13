import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import App from "./App";
import { invoke } from "./test/tauri";
import type { Overview, Run } from "./types";

const overview: Overview = {
  commands: 12,
  failures: 3,
  sessions: 2,
  lessons: 4,
  pending_lessons: 1,
  agents: [["claude", 9], ["terminal", 3]],
  recall: { shown: 7, followed: 4, worked: 3, warnings: 2, warned_then_passed: 1 },
  store: "/home/ada/.local/share/MemoryWhale/memorywhale.sqlite3"
};

const noRuns: Run[] = [];

/** Answer each Tauri command from a lookup; anything unexpected fails loud. */
function respond(map: Record<string, unknown>) {
  invoke.mockImplementation(async (command: string) => {
    if (command in map) return map[command];
    throw new Error(`unexpected Tauri command: ${command}`);
  });
}

/** Click a section in the sidebar, not any same-named button elsewhere. */
function goTo(name: string) {
  fireEvent.click(within(screen.getByRole("navigation", { name: "Sections" })).getByRole("button", { name }));
}

beforeEach(() => {
  invoke.mockReset();
});

// Vitest globals are off, so @testing-library/react does not auto-clean the
// document between tests.
afterEach(cleanup);

describe("App shell", () => {
  it("renders the overview after the store answers, asking Tauri for the expected commands", async () => {
    respond({ overview, recall_history: [], list_runs: noRuns });

    render(<App />);

    const stat = await screen.findByText(/commands remembered/);
    expect(stat.textContent).toContain("12");
    expect(screen.getByText("Stored only on this machine")).toBeTruthy();
    expect(invoke).toHaveBeenCalledWith("overview", undefined);
    expect(invoke).toHaveBeenCalledWith("recall_history", { limit: 4 });
    expect(invoke).toHaveBeenCalledWith("list_runs", { failuresOnly: true, limit: 5 });
  });

  it("shows the Commands empty state and the exact list_runs arguments", async () => {
    respond({ overview, recall_history: [], list_runs: noRuns });

    render(<App />);
    goTo("Commands");

    expect(await screen.findByText(/No commands yet/)).toBeTruthy();
    expect(invoke).toHaveBeenCalledWith("list_runs", { query: "", failuresOnly: false, limit: 300 });
  });

  it("sends failuresOnly: true when the Failures filter is pressed", async () => {
    respond({ overview, recall_history: [], list_runs: noRuns });

    render(<App />);
    goTo("Commands");
    await screen.findByText(/No commands yet/);
    fireEvent.click(screen.getByRole("button", { name: "Failures" }));

    await waitFor(() => expect(invoke).toHaveBeenLastCalledWith("list_runs", { query: "", failuresOnly: true, limit: 300 }));
  });

  it("stays silent about empty state while list_runs is still pending (loading)", async () => {
    let resolveRuns!: (runs: Run[]) => void;
    invoke.mockImplementation(async (command: string) => {
      if (command === "overview") return overview;
      if (command === "recall_history") return [];
      if (command === "list_runs") return new Promise<Run[]>((resolve) => (resolveRuns = resolve));
      throw new Error(`unexpected Tauri command: ${command}`);
    });

    render(<App />);
    goTo("Commands");

    // Home also asks for list_runs; wait for the Commands page's own call so
    // the captured resolver belongs to it.
    await waitFor(() => expect(invoke).toHaveBeenCalledWith("list_runs", { query: "", failuresOnly: false, limit: 300 }));
    expect(screen.queryByText(/No commands yet/)).toBeNull();

    resolveRuns(noRuns);
    expect(await screen.findByText(/No commands yet/)).toBeTruthy();
  });

  it("shows an error banner and no overview when the store call rejects", async () => {
    invoke.mockRejectedValue(new Error("database locked"));

    render(<App />);

    expect(await screen.findByText(/database locked/)).toBeTruthy();
    // The stat label always renders; without the overview it shows "–".
    expect(screen.queryByText("12")).toBeNull();
  });

  it("clears the error banner after the next refresh succeeds", async () => {
    // The first overview read fails; later reads succeed. Target the command
    // explicitly: child effects run before parent effects, so call order
    // alone cannot identify the overview read.
    let rejected = false;
    invoke.mockImplementation(async (command: string) => {
      if (command === "overview" && !rejected) {
        rejected = true;
        throw new Error("database locked");
      }
      if (command === "overview") return overview;
      if (command === "recall_history") return [];
      if (command === "list_runs") return noRuns;
      throw new Error(`unexpected Tauri command: ${command}`);
    });

    render(<App />);
    expect(await screen.findByText(/database locked/)).toBeTruthy();

    // Other processes write to the store; the app re-reads when the window
    // regains focus. A later read now succeeds.
    window.dispatchEvent(new Event("focus"));

    await waitFor(() => expect(screen.queryByText(/database locked/)).toBeNull());
    expect(await screen.findByText("12")).toBeTruthy();
  });
});
