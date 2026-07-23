import { describe, expect, it } from "vitest";
import { splitCommandLine } from "./commandLine";

describe("splitCommandLine", () => {
  it("keeps quoted arguments together", () => {
    expect(splitCommandLine('cargo test --features "sqlite bundled"')).toEqual([
      "cargo",
      "test",
      "--features",
      "sqlite bundled"
    ]);
  });

  it("handles single quotes and empty input", () => {
    expect(splitCommandLine("git commit -m 'fix parser'")).toEqual([
      "git",
      "commit",
      "-m",
      "fix parser"
    ]);
    expect(splitCommandLine("   ")).toEqual([]);
  });
});
