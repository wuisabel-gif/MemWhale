import { invoke } from "@tauri-apps/api/core";
import { mockCall } from "./mock";

/** Inside the desktop shell; otherwise a browser preview with sample data. */
export const inTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

export function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  return inTauri ? invoke<T>(command, args) : mockCall<T>(command, args);
}

/** Open a link in the system browser, never inside the app window. */
export function openUrl(url: string) {
  if (inTauri) void invoke("plugin:opener|open_url", { url });
  else window.open(url, "_blank", "noopener");
}

const rtf = new Intl.RelativeTimeFormat(undefined, { numeric: "auto" });
const UNITS: [Intl.RelativeTimeFormatUnit, number][] = [
  ["year", 31_536_000],
  ["month", 2_592_000],
  ["week", 604_800],
  ["day", 86_400],
  ["hour", 3_600],
  ["minute", 60]
];

/** "3 hours ago", from an RFC 3339 or SQLite timestamp. */
export function ago(at: string) {
  const time = Date.parse(at.includes("T") ? at : `${at.replace(" ", "T")}Z`);
  if (Number.isNaN(time)) return at;
  const seconds = (time - Date.now()) / 1000;
  for (const [unit, size] of UNITS) {
    if (Math.abs(seconds) >= size) return rtf.format(Math.round(seconds / size), unit);
  }
  return "just now";
}

/** A home directory shown as ~, so long paths fit. */
export function shortPath(path: string | null | undefined) {
  if (!path) return "unknown directory";
  return path.replace(/^\/(?:Users|home)\/[^/]+/, "~");
}

export function count(n: number | undefined) {
  return n === undefined ? "–" : n.toLocaleString();
}
