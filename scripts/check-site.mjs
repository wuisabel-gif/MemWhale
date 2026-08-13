import { readFile, access } from "node:fs/promises";
import { resolve } from "node:path";

const root = resolve(new URL("..", import.meta.url).pathname);
const html = await readFile(resolve(root, "index.html"), "utf8");
const failures = [];
const expect = (condition, message) => {
  if (!condition) failures.push(message);
};

expect(!html.includes("v0.3.0"), "landing page contains stale v0.3.0 copy");
expect(!/fonts\.(googleapis|gstatic)\.com/.test(html), "landing page loads third-party fonts");
expect(!/break-reminder|break-toast|Notification|bag-charm|Whale swag|Break reminder/i.test(html), "removed break or merchandise content remains");
expect(!html.includes("never leaves your machine"), "landing page makes an absolute data-location claim");
expect(html.includes("prefers-reduced-motion"), "reduced-motion CSS is missing");
expect(html.includes(":focus-visible"), "keyboard focus styling is missing");
expect(html.includes("Capture → memory → retrieval"), "capture-to-recall demo is missing");
expect(html.includes("MEMORYWHALE_DATA_DIR"), "data directory guidance is missing");
expect(html.includes("loopback") && html.includes("requires a token"), "dashboard trust model is missing");

for (const tool of ["recent_errors", "search_memory", "get_context", "remember", "similar_failures", "stats"]) {
  expect(html.includes(tool), `MCP tool missing from landing page: ${tool}`);
}

const localRefs = [...html.matchAll(/(?:href|src)=["']([^"']+)["']/g)]
  .map(([, ref]) => ref)
  .filter((ref) => !/^(?:[a-z]+:|#|\/\/)/i.test(ref));
for (const ref of localRefs) {
  const path = ref.split(/[?#]/, 1)[0];
  if (!path) continue;
  try {
    await access(resolve(root, path));
  } catch {
    failures.push(`missing local site reference: ${ref}`);
  }
}

if (failures.length) {
  console.error(failures.map((failure) => `- ${failure}`).join("\n"));
  process.exit(1);
}
console.log(`site contract checks passed (${localRefs.length} local references checked)`);
