import assert from "node:assert/strict";
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

import {
  LOCALES,
  hashContent,
  parseLocalizedReadme,
  translateMarkdown,
  translateStaleLocales,
  validateTranslation,
} from "./sync-readme-locales.mjs";

const source = `<p align="center">
  <img src="assets/logo.png" alt="Logo" />
</p>

<h1 align="center">MemoryWhale</h1>

## Install

Use \`mw-mcp\` and read the [guide](docs/guide.md) and [API](docs/api.md).

Keep \`\`literal \` content\`\` unchanged.

See the [reference][guide-ref] and ![diagram][diagram-ref].

[guide-ref]: docs/reference.md
[diagram-ref]: assets/diagram.png

\`\`\`bash
mw search "linker error"
\`\`\`

\`\`\`\`markdown
\`\`\`bash
mw nested-fence
\`\`\`
\`\`\`\`

    mw context --last-error
`;

test("source markers contain a content hash", () => {
  const hash = hashContent(source);
  const parsed = parseLocalizedReadme(`<!-- README-SOURCE-SHA256: ${hash} -->\n\n${source}`);
  assert.equal(parsed.sourceHash, hash);
  assert.equal(parsed.body, source);
  assert.throws(() => parseLocalizedReadme(source), /must start with/);
});

test("an identity translation preserves protected content and structure", () => {
  assert.doesNotThrow(() => validateTranslation(source, source));
});

test("translated image alt text is allowed", () => {
  assert.doesNotThrow(() => validateTranslation(source, source.replace('alt="Logo"', 'alt="ロゴ"')));
});

test("the English README passes the complete protection contract", async () => {
  const readme = await readFile(new URL("../README.md", import.meta.url), "utf8");
  assert.doesNotThrow(() => validateTranslation(readme, readme));
});

test("validation rejects changes to protected README content", async (t) => {
  await t.test("fenced commands", () => {
    assert.throws(
      () => validateTranslation(source, source.replace("mw search", "mw find")),
      /fenced code blocks and commands/,
    );
  });
  await t.test("inline commands", () => {
    assert.throws(
      () => validateTranslation(source, source.replace("`mw-mcp`", "`mw`")),
      /inline code and commands/,
    );
  });
  await t.test("multi-backtick inline code", () => {
    assert.throws(
      () => validateTranslation(source, source.replace("literal ` content", "changed ` content")),
      /inline code and commands/,
    );
  });
  await t.test("indented commands", () => {
    assert.throws(
      () => validateTranslation(source, source.replace("mw context", "mw recent")),
      /indented code blocks and commands/,
    );
  });
  await t.test("long fenced code delimiters", () => {
    assert.throws(
      () => validateTranslation(source, source.replace("mw nested-fence", "mw changed-fence")),
      /fenced code blocks and commands/,
    );
  });
  await t.test("link targets", () => {
    assert.throws(
      () => validateTranslation(source, source.replace("docs/guide.md", "docs/other.md")),
      /Markdown link and image targets/,
    );
  });
  await t.test("swapped link targets", () => {
    const swapped = source
      .replace("[guide](docs/guide.md)", "[guide](docs/api.md)")
      .replace("[API](docs/api.md)", "[API](docs/guide.md)");
    assert.throws(
      () => validateTranslation(source, swapped),
      /Markdown link and image targets/,
    );
  });
  await t.test("image targets", () => {
    assert.throws(
      () => validateTranslation(source, source.replace("assets/logo.png", "assets/other.png")),
      /HTML link and image targets/,
    );
  });
  await t.test("image presentation attributes", () => {
    assert.throws(
      () => validateTranslation(source, source.replace('alt="Logo" />', 'alt="Logo" width="200" />')),
      /HTML attributes and structure/,
    );
  });
  await t.test("reference-style link targets", () => {
    assert.throws(
      () => validateTranslation(source, source.replace("docs/reference.md", "docs/other.md")),
      /reference-style link and image targets/,
    );
  });
  await t.test("reference-style image identifiers", () => {
    assert.throws(
      () => validateTranslation(source, source.replace("![diagram][diagram-ref]", "![diagram][other-ref]")),
      /reference-style link and image identifiers/,
    );
  });
  await t.test("swapped reference-style identifiers", () => {
    const swapped = source
      .replace("[reference][guide-ref]", "[reference][diagram-ref]")
      .replace("![diagram][diagram-ref]", "![diagram][guide-ref]");
    assert.throws(
      () => validateTranslation(source, swapped),
      /reference-style link and image identifiers/,
    );
  });
  await t.test("heading levels", () => {
    assert.throws(
      () => validateTranslation(source, source.replace("## Install", "### Install")),
      /heading structure|no level-two sections/,
    );
  });
  await t.test("HTML wrappers", () => {
    assert.throws(
      () => validateTranslation(source, source.replace('<p align="center">', "<div>")),
      /HTML (?:attributes and )?structure/,
    );
  });
});

test("the OpenAI-compatible request returns translated Markdown", async () => {
  let request;
  const translated = await translateMarkdown({
    source,
    language: "Test",
    apiKey: "test-key",
    baseUrl: "https://provider.example/v1",
    model: "test-model",
    fetchImpl: async (url, options) => {
      request = {
        authorization: options.headers.Authorization,
        body: JSON.parse(options.body),
        url: url.toString(),
      };
      return new Response(
        JSON.stringify({ choices: [{ message: { content: `\`\`\`markdown\n${source}\n\`\`\`` } }] }),
        { status: 200, headers: { "Content-Type": "application/json" } },
      );
    },
  });

  assert.equal(translated, source.trim());
  assert.equal(request.authorization, "Bearer test-key");
  assert.equal(request.url, "https://provider.example/v1/chat/completions");
  assert.equal(request.body.model, "test-model");
  assert.match(request.body.messages[1].content, /<README_SOURCE>/);
});

test("translation provider URLs must use HTTPS", async () => {
  await assert.rejects(
    translateMarkdown({
      source,
      language: "Test",
      apiKey: "test-key",
      baseUrl: "http://provider.example/v1",
      model: "test-model",
      fetchImpl: async () => assert.fail("an insecure provider must not be contacted"),
    }),
    /must use https/,
  );
});

test("stale locale files are translated and stamped with the current hash", async (t) => {
  const directory = await mkdtemp(join(tmpdir(), "readme-locales-"));
  t.after(() => rm(directory, { recursive: true, force: true }));
  await writeFile(join(directory, "README.md"), source);
  for (const locale of LOCALES) {
    await writeFile(
      join(directory, locale.file),
      `<!-- README-SOURCE-SHA256: ${"0".repeat(64)} -->\n\n${source}`,
    );
  }

  let requests = 0;
  const changed = await translateStaleLocales(directory, {
    apiKey: "test-key",
    baseUrl: "https://provider.example/v1",
    model: "test-model",
    fetchImpl: async () => {
      requests += 1;
      return new Response(JSON.stringify({ choices: [{ message: { content: source } }] }), {
        status: 200,
        headers: { "Content-Type": "application/json" },
      });
    },
  });

  assert.deepEqual(changed, LOCALES.map((locale) => locale.file));
  assert.equal(requests, LOCALES.length);
  for (const locale of LOCALES) {
    const localized = parseLocalizedReadme(await readFile(join(directory, locale.file), "utf8"));
    assert.equal(localized.sourceHash, hashContent(source));
  }
});
