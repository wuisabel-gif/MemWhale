import { vi } from "vitest";

// jsdom does not implement scrolling (whatwg/jsdom#2961): App calls
// main.scrollTo(0, 0) when the page changes, so give it a no-op.
Element.prototype.scrollTo ??= () => {};

// jsdom does not implement matchMedia either, and the ocean hero reads it.
// The tests do not need real media queries, only a stable answer.
if (typeof window !== "undefined" && !window.matchMedia) {
  window.matchMedia = ((query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: () => {},
    removeListener: () => {},
    addEventListener: () => {},
    removeEventListener: () => {},
    dispatchEvent: () => false
  })) as unknown as typeof window.matchMedia;
}

// jsdom prints "Not implemented" noise for every canvas getContext call and
// has no WebGL anyway; the ocean hero probes it and bails on null.
HTMLCanvasElement.prototype.getContext = (() => null) as unknown as typeof HTMLCanvasElement.prototype.getContext;

// Mock the Tauri seam, not the components: src/api.ts resolves every `call()`
// through the shared `invoke` stub in tauri.ts, so behavioral tests exercise
// real command names and arguments without a Tauri runtime or local store.
vi.mock("@tauri-apps/api/core", async () => {
  const { invoke } = await import("./tauri");
  return { invoke };
});

// The tray/listen wiring only runs in the desktop shell; jsdom gets a no-op.
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(async () => () => {})
}));
