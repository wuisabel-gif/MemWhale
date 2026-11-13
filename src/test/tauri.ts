import { vi } from "vitest";

// The desktop-shell marker must be in place BEFORE anything imports ./api:
// `inTauri` is computed once at module load from window.__TAURI_INTERNALS__.
// Setup files run before each test file's imports, so importing this module
// from src/test/setup.ts is early enough.
if (typeof window !== "undefined" && !("__TAURI_INTERNALS__" in window)) {
  (window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__ = {};
}

/** Controllable stand-in for @tauri-apps/api/core#invoke.

    Every `call()` in src/api.ts routes through this stub in tests, so tests
    can resolve, reject, or defer each Tauri command and assert the exact
    command names and arguments the UI sent. Reset it between tests with
    `invoke.mockReset()`. */
export const invoke = vi.fn();
