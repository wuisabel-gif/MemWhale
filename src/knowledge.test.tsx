import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import App from "./App";
import { goTo, overview } from "./test/helpers";
import { invoke } from "./test/tauri";
import type { Knowledge } from "./types";

const statement = (id: number, text: string, status: string): Knowledge => ({
  id,
  text,
  kind: "preference",
  project: null,
  sensitivity: status === "approved" ? "internal" : "sensitive",
  status,
  platform: "claude_code"
});

let store: Knowledge[];

beforeEach(() => {
  store = [statement(2, "I prefer small pull requests", "proposed"), statement(1, "Never use em dashes", "approved")];
  invoke.mockReset();
  invoke.mockImplementation(async (command: string, args: Record<string, unknown> = {}) => {
    switch (command) {
      case "overview":
        return overview;
      case "recall_history":
      case "list_runs":
        return [];
      case "knowledge_list":
        return store.filter((s) => s.status === args.status);
      case "knowledge_import":
        store = [statement(3, "From now on run the tests first", "proposed"), ...store];
        return { files: 4, messages: 20, proposed: 1, already_known: 2, export_found: false };
      case "knowledge_decide":
        store = store.map((s) => (s.id === args.id ? { ...s, status: args.decision === "reject" ? "rejected" : "approved" } : s));
        return null;
      default:
        throw new Error(`unexpected Tauri command: ${command}`);
    }
  });
});
afterEach(cleanup);

describe("Knowledge page", () => {
  it("imports with one button and moves a statement on approval", async () => {
    render(<App />);
    goTo("Knowledge");
    await screen.findByText("I prefer small pull requests");

    fireEvent.click(screen.getByRole("button", { name: /Import my conversations/ }));
    const status = await screen.findByRole("status");
    expect(status.textContent).toContain("1 new statements are waiting below");
    expect(status.textContent).toContain("No ChatGPT or Claude export was found");
    expect(invoke).toHaveBeenCalledWith("knowledge_import", undefined);

    const row = (await screen.findByText("From now on run the tests first")).closest("li")!;
    fireEvent.click(within(row).getByRole("button", { name: /Approve/ }));
    await waitFor(() => expect(invoke).toHaveBeenCalledWith("knowledge_decide", { id: 3, decision: "approve" }));
    await screen.findByText("Approved (2)");
    expect(screen.getByText("Waiting for you (1)")).toBeTruthy();
  });
});
