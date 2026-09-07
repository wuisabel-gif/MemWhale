import { chromium } from "playwright";
import { mkdir, readFile } from "node:fs/promises";
import vm from "node:vm";

const baseUrl = process.env.SITE_URL ?? "http://127.0.0.1:4173/index.html";
const artifactDir = process.env.SITE_ARTIFACT_DIR ?? "artifacts/site-smoke";
const sandbox = {};
vm.runInNewContext(await readFile(new URL("../site-i18n.js", import.meta.url), "utf8"), sandbox);
const { supportedLanguages: languages, translations } = sandbox.MEMORYWHALE_I18N;
const viewports = [["mobile", 390, 844], ["desktop", 1440, 900]];
const sampledKeys = [
  "hero.title", "nav.terminal", "release.title", "who.title", "terminal.title",
  "how.title", "agents.title", "demo.title", "data.title", "security.copy", "run.title",
];
await mkdir(artifactDir, { recursive: true });
const failures = [];
const expect = (condition, message) => { if (!condition) failures.push(message); };
const languageUrl = (language) => {
  const url = new URL(baseUrl);
  if (language) url.searchParams.set("lang", language);
  else url.searchParams.delete("lang");
  url.searchParams.set("smoke", "preserve-me");
  url.hash = "demo";
  return url.toString();
};

function trackErrors(page, label) {
  page.on("console", (message) => {
    if (message.type() === "error") failures.push(`${label}: console: ${message.text()}`);
  });
  page.on("pageerror", (error) => failures.push(`${label}: page: ${error.message}`));
}

async function checkLanguage(page, language, label) {
  const dictionary = translations[language];
  expect(await page.locator("html").getAttribute("lang") === language, `${label}: wrong html lang`);
  expect(await page.locator("html").getAttribute("dir") === (language === "ar" ? "rtl" : "ltr"), `${label}: wrong reading direction`);
  expect(await page.locator("#language-select").inputValue() === language, `${label}: wrong selector value`);
  expect(await page.title() === dictionary.meta.title, `${label}: wrong document title`);
  expect(await page.locator("h1").evaluate((element) => {
    const range = document.createRange();
    range.selectNodeContents(element);
    const box = element.getBoundingClientRect();
    return [...range.getClientRects()].every(rect => rect.left >= box.left - 1 && rect.right <= box.right + 1);
  }), `${label}: heading text overflows its box`);
  expect(await page.locator('meta[name="description"]').getAttribute("content") === dictionary.meta.description, `${label}: wrong description`);
  const jsonLd = JSON.parse(await page.locator('script[type="application/ld+json"]').textContent());
  expect(jsonLd.inLanguage === language && jsonLd.description === dictionary.meta.jsonLdDescription, `${label}: wrong JSON-LD`);
  // Social preview metadata is deliberately static English, not localized SEO.
  expect(await page.locator('meta[property="og:title"]').getAttribute("content") === translations.en.meta.title, `${label}: static social metadata changed`);

  for (const key of sampledKeys) {
    const actual = await page.locator(`[data-i18n="${key}"]`).innerText();
    const wanted = await page.evaluate((html) => {
      const fragment = document.createElement("template");
      fragment.innerHTML = html;
      return fragment.content.textContent;
    }, dictionary[key]);
    expect(actual.replace(/\s+/g, " ").trim() === wanted.replace(/\s+/g, " ").trim(), `${label}: untranslated content in ${key}`);
  }
  expect(await page.locator(".client-strip").getAttribute("aria-label") === dictionary["agents.clientsLabel"], `${label}: wrong client-group label`);
  expect(await page.locator("#demo img").getAttribute("alt") === dictionary["demo.imageAlt"], `${label}: wrong image alternative`);

  const picker = page.getByRole("combobox", { name: dictionary["language.label"], exact: true });
  await picker.scrollIntoViewIfNeeded();
  const box = await picker.boundingBox();
  expect(await picker.isVisible() && box && box.width >= 80 && box.height >= 24, `${label}: language selector is not usable`);
  expect(await picker.evaluate((element) => {
    const box = element.getBoundingClientRect();
    const top = document.elementFromPoint(box.x + box.width / 2, box.y + box.height / 2);
    return top === element || element.contains(top);
  }), `${label}: selector is covered`);
  expect(await page.locator("#language-select option").count() === languages.length, `${label}: missing language options`);

  const direction = await page.locator("pre").first().evaluate((element) => {
    const style = getComputedStyle(element);
    return { direction: style.direction, align: style.textAlign };
  });
  expect(direction.direction === "ltr" && direction.align === "left", `${label}: command blocks are not LTR`);
  expect(await page.locator("code").evaluateAll((elements) => elements.every((element) => getComputedStyle(element).direction === "ltr")), `${label}: inline code is not LTR`);
  for (const selector of ["#demo", "#data", "#security", "#run"]) {
    expect(await page.locator(selector).count() === 1, `${label}: missing section ${selector}`);
  }
  expect(await page.locator("a[href^='http']").count() >= 3, `${label}: external links missing`);
  const url = new URL(page.url());
  expect(url.hash === "#demo" && url.searchParams.get("smoke") === "preserve-me", `${label}: URL anchor or unrelated query lost`);
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth + 1), `${label}: horizontal overflow`);
}

const browser = await chromium.launch({ headless: true });
try {
  for (const language of languages) {
    for (const [size, width, height] of viewports) {
      const label = `${language}/${size}`;
      const page = await browser.newPage({ viewport: { width, height } });
      trackErrors(page, label);
      try {
        const response = await page.goto(languageUrl(language), { waitUntil: "networkidle" });
        expect(response?.ok(), `${label}: page load failed`);
        await checkLanguage(page, language, label);
        expect(new URL(page.url()).searchParams.get("lang") === language, `${label}: query language changed`);
        await page.screenshot({ path: `${artifactDir}/${language}-${size}.png`, fullPage: true });
        if (language === "ar" || language === "de") {
          await page.locator(".hero").screenshot({ path: `${artifactDir}/${language}-${size}-hero.png` });
        }
        // Exercise RTL -> LTR -> original selection on mobile as well as desktop.
        if (language === "ar") {
          const examples = await page.locator("pre").allTextContents();
          for (const next of ["de", "en", "ar"]) {
            await page.locator("#language-select").selectOption(next);
            await checkLanguage(page, next, `${label}/switch-${next}`);
            expect(new URL(page.url()).searchParams.get("lang") === next, `${label}: selected locale missing from URL`);
            expect(JSON.stringify(await page.locator("pre").allTextContents()) === JSON.stringify(examples), `${label}: translated executable examples`);
          }
          expect(await page.evaluate(() => localStorage.getItem("memorywhale.language")) === "ar", `${label}: selection not saved`);
          await page.goto(languageUrl(null), { waitUntil: "networkidle" });
          await checkLanguage(page, "ar", `${label}/stored-reload`);
          expect(!new URL(page.url()).searchParams.has("lang"), `${label}: stored preference rewrote URL`);
        }
      } finally {
        await page.close();
      }
    }
  }

  const preferenceCases = [
    { locale: "ar-EG", expected: "ar" },
    { locale: "de-AT", expected: "de" },
    { locale: "zh-Hans", expected: "zh-CN" },
    { locale: "zh-Hant-TW", expected: "zh-TW" },
    { locale: "en-US", stored: "ar", expected: "ar" },
    { locale: "ar-SA", stored: "de", query: "unsupported", expected: "de" },
    { locale: "ar-SA", stored: "ar", query: "de-DE", expected: "de" },
    { locale: "de-DE", blockedStorage: true, expected: "de" },
    { locale: "de-DE", languages: ["unsupported"], expected: "de" },
    { locale: "es-ES", expected: "en" },
  ];
  for (const [index, settings] of preferenceCases.entries()) {
    const label = `preference-${index}/${settings.locale}`;
    const context = await browser.newContext({ locale: settings.locale, viewport: { width: 390, height: 844 } });
    await context.addInitScript(({ stored, blockedStorage, languages }) => {
      if (stored) localStorage.setItem("memorywhale.language", stored);
      if (blockedStorage) {
        Storage.prototype.getItem = () => { throw new DOMException("Blocked", "SecurityError"); };
        Storage.prototype.setItem = () => { throw new DOMException("Blocked", "SecurityError"); };
      }
      if (languages) Object.defineProperty(navigator, "languages", { get: () => languages });
    }, settings);
    try {
      const page = await context.newPage();
      trackErrors(page, label);
      const response = await page.goto(languageUrl(settings.query), { waitUntil: "networkidle" });
      expect(response?.ok(), `${label}: page load failed`);
      await checkLanguage(page, settings.expected, label);
      expect(new URL(page.url()).searchParams.get("lang") === (settings.query ?? null), `${label}: initial URL was rewritten`);
      if (settings.blockedStorage) {
        await page.locator("#language-select").selectOption("ar");
        await checkLanguage(page, "ar", `${label}/blocked-storage-switch`);
      } else if (!settings.stored) {
        expect(await page.evaluate(() => localStorage.getItem("memorywhale.language")) === null, `${label}: initial detection unexpectedly persisted`);
      }
    } finally {
      await context.close();
    }
  }

  const noScriptContext = await browser.newContext({ javaScriptEnabled: false });
  try {
    const page = await noScriptContext.newPage();
    const response = await page.goto(languageUrl("ar"), { waitUntil: "networkidle" });
    expect(response?.ok(), "no-js: page load failed");
    expect(await page.locator("html").getAttribute("lang") === "en", "no-js: fallback is not English");
    expect(await page.locator("html").getAttribute("dir") === "ltr", "no-js: fallback direction is not LTR");
    expect(await page.locator("h1").innerText() === translations.en["hero.title"], "no-js: English heading missing");
    expect(await page.locator(".nav-links a[href='#terminal-memory']").innerText() === translations.en["nav.terminal"], "no-js: English navigation missing");
    expect((await page.locator("pre").first().innerText()).includes("install.sh"), "no-js: command example missing");
  } finally {
    await noScriptContext.close();
  }
} finally {
  await browser.close();
}

if (failures.length) {
  console.error(failures.map((failure) => `- ${failure}`).join("\n"));
  process.exit(1);
}
console.log(`landing-page smoke passed: ${languages.length} languages, mobile/desktop, RTL, preferences, and no-JS fallback`);
