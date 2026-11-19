import { fireEvent, screen, within } from "@testing-library/react";
import type { Overview, Run } from "../types";
import { invoke } from "./tauri";

export const overview: Overview = {
  commands: 12,
  failures: 3,
  sessions: 2,
  lessons: 4,
  pending_lessons: 1,
  agents: [["claude", 9], ["terminal", 3]],
  recall: { shown: 7, followed: 4, worked: 3, warnings: 2, warned_then_passed: 1 },
  store: "/home/ada/.local/share/MemoryWhale/memorywhale.sqlite3"
};

export const noRuns: Run[] = [];

/** Answer each Tauri command from a lookup; anything unexpected fails loud. */
export function respond(map: Record<string, unknown>) {
  invoke.mockImplementation(async (command: string) => {
    if (command in map) return map[command];
    throw new Error(`unexpected Tauri command: ${command}`);
  });
}

/** Click a section in the sidebar, not any same-named button elsewhere. */
export function goTo(name: string) {
  fireEvent.click(within(screen.getByRole("navigation", { name: "Sections" })).getByRole("button", { name }));
}
