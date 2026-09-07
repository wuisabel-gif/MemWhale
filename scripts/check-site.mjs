import { readFile, access } from "node:fs/promises";
import { resolve } from "node:path";
import vm from "node:vm";
import { LOCALES } from "./sync-readme-locales.mjs";

const root = resolve(new URL("..", import.meta.url).pathname);
const html = await readFile(resolve(root, "index.html"), "utf8");
const i18nSource = await readFile(resolve(root, "site-i18n.js"), "utf8");
const searchSource = await readFile(resolve(root, "site-search.js"), "utf8");
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
for (const term of [
  "MCP",
  "mw-mcp",
  "loopback",
  "requires a token",
  "Bearer authentication",
  "HTTP does not encrypt",
  "local data threat model"
]) {
  expect(html.includes(term), `required stable site term is missing: ${term}`);
}
expect(html.includes('<script defer src="site-i18n.js"></script>'), "local deferred site dictionary is missing");
expect(html.includes('<script defer src="site-search.js"></script>'), "local homepage search script is missing");
expect(!/<script\b[^>]*\bsrc=["'](?:https?:)?\/\//i.test(html), "landing page loads a remote script");
expect(!/(?:fonts\.(?:googleapis|gstatic)\.com|unpkg\.com|jsdelivr\.net)/i.test(`${html}\n${i18nSource}`), "landing page loads a third-party resource");

const supportedLanguages = ["en", ...LOCALES.map(({ file }) => file.slice("README.".length, -".md".length))];
const selector = html.match(/<select\b[^>]*id="language-select"[\s\S]*?<\/select>/i)?.[0] ?? "";
expect(selector.includes('name="language"'), "language selector is missing a name");
expect(selector.includes('aria-label="Language"'), "language selector is missing an accessible label");
for (const language of supportedLanguages) {
  expect(
    new RegExp(`<option\\s+value="${language}"(?:\\s+[^>]*)?>`, "i").test(selector),
    `language selector is missing option: ${language}`
  );
}

const dictionarySandbox = { console };
try {
  vm.runInNewContext(i18nSource, dictionarySandbox, { filename: "site-i18n.js" });
  vm.runInNewContext(searchSource, {}, { filename: "site-search.js" });
} catch (error) {
  failures.push(`site dictionary is not executable: ${error.message}`);
}
const dictionaryApi = dictionarySandbox.MEMORYWHALE_I18N;
expect(dictionaryApi, "site dictionary does not expose its local contract");
if (dictionaryApi) {
  expect(
    JSON.stringify(dictionaryApi.supportedLanguages) === JSON.stringify(supportedLanguages),
    "site dictionary language order or values changed"
  );
  const translationKeys = [
    ...new Set([
      ...[...html.matchAll(/data-i18n="([^"]+)"/g)].map(([, key]) => key),
      ...[...html.matchAll(/data-i18n-(?:aria-label|alt|placeholder)="([^"]+)"/g)].map(([, key]) => key),
      "search.empty", "search.results"
    ])
  ];
  for (const language of supportedLanguages) {
    expect(dictionaryApi.translations[language], `site dictionary is missing language: ${language}`);
    if (!dictionaryApi.translations[language]) continue;
    const dictionary = dictionaryApi.translations[language];
    expect((dictionary["hero.title"]?.match(/class="hero-accent"/g) ?? []).length === 1,
      `${language} hero needs exactly one selectively highlighted phrase`);
    for (const key of ["title", "description", "jsonLdDescription"]) {
      expect(typeof dictionary.meta?.[key] === "string" && dictionary.meta[key].trim().length > 0,
        `${language} dictionary is missing metadata: ${key}`);
    }
    for (const key of translationKeys) {
      expect(
        typeof dictionary[key] === "string" && dictionary[key].trim().length > 0,
        `${language} dictionary is missing key: ${key}`
      );
      const english = dictionaryApi.translations.en[key];
      if (typeof english !== "string" || typeof dictionary[key] !== "string") continue;
      for (const [name, pattern] of [
        ["code", /<code\b[^>]*>([\s\S]*?)<\/code>/g],
        ["links", /\bhref="([^"]+)"/g],
      ]) {
        const values = (text) => [...text.matchAll(pattern)].map((match) => match[1]).sort();
        expect(JSON.stringify(values(english)) === JSON.stringify(values(dictionary[key])),
          `${language} changed protected ${name} in ${key}`);
      }
    }
  }
}
expect(i18nSource.includes("localStorage"), "language preference storage is missing");
expect(i18nSource.includes("navigator.languages"), "browser language detection is missing");
expect(i18nSource.includes("history.replaceState"), "language changes do not preserve the current URL without navigation");
expect(i18nSource.includes("window.location.href"), "language query selection is missing");

for (const tool of ["recent_errors", "search_memory", "get_context", "remember", "similar_failures", "stats"]) {
  expect(html.includes(tool), `MCP tool missing from landing page: ${tool}`);
}

// The setup excerpt must match actual CLI messages, not invented success output.
const claudeIntegration = await readFile(resolve(root, "crates/mw-cli/src/integrate/claude/mod.rs"), "utf8");
for (const message of [
  "MemoryWhale installed for Claude Code.",
  "mcp:      memorywhale registered (user scope)",
  "Restart Claude Code to pick up hook and skill changes."
]) {
  expect(html.includes(message) && claudeIntegration.includes(message), `unverified setup output: ${message}`);
}
for (const client of ["claude-code", "rho", "codex", "cursor"]) {
  const guide = `integrations/${client}/README.md`;
  await access(resolve(root, guide));
  expect(html.includes(`blob/main/${guide}`), `missing verified integration guide: ${client}`);
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
