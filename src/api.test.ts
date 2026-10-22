import { describe, expect, it } from "vitest";
import { ago, shortPath } from "./api";

describe("ago", () => {
  it("reads RFC 3339 and SQLite timestamps", () => {
    const hour = new Date(Date.now() - 3 * 3_600_000);
    expect(ago(hour.toISOString())).toBe("3 hours ago");
    const sqlite = hour.toISOString().slice(0, 19).replace("T", " ");
    expect(ago(sqlite)).toBe("3 hours ago");
    expect(ago(new Date().toISOString())).toBe("just now");
    expect(ago("not a date")).toBe("not a date");
  });
});

describe("shortPath", () => {
  it("shows a home directory as ~", () => {
    expect(shortPath("/Users/ada/code/robot-arm")).toBe("~/code/robot-arm");
    expect(shortPath("/home/ada/w")).toBe("~/w");
    expect(shortPath("/opt/data")).toBe("/opt/data");
    expect(shortPath(null)).toBe("unknown directory");
  });
});
