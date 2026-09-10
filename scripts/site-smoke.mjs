import { chromium } from "playwright";
import { mkdir, readFile } from "node:fs/promises";
import vm from "node:vm";

const baseUrl = process.env.SITE_URL ?? "http://127.0.0.1:4173/index.html";
const artifactDir = process.env.SITE_ARTIFACT_DIR ?? "artifacts/site-smoke";
const sandbox = {};
vm.runInNewContext(await readFile(new URL("../site-i18n.js", import.meta.url), "utf8"), sandbox);
const { supportedLanguages: languages, translations } = sandbox.MEMORYWHALE_I18N;
const viewports = [["small-mobile", 320, 900], ["mobile", 390, 844], ["tablet", 768, 1024], ["compact-desktop", 1024, 900], ["desktop", 1440, 900]];
const sampledKeys = [
  "hero.title", "hero.lead", "hero.installCta", "hero.demoCta", "hero.localTitle", "hero.privateTitle", "hero.agentTitle",
  "nav.terminal", "release.title", "who.title", "features.terminalTitle", "features.retrievalTitle", "features.localTitle",
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
  expect(await page.locator("h1").innerText() === "MemoryWhale", `${label}: product name must remain the main heading`);
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
  expect(await page.locator(".hero-headline .hero-accent").count() === 1, `${label}: missing selective headline accent`);
  if (language === "zh-CN") {
    expect(await page.locator(".hero-accent").innerText() === "记住", `${label}: Chinese accent phrase changed`);
  }

  const layout = await page.evaluate(() => {
    const box = (selector) => document.querySelector(selector).getBoundingClientRect();
    const separate = (a, b) => a.right <= b.left + 1 || b.right <= a.left + 1 || a.bottom <= b.top + 1 || b.bottom <= a.top + 1;
    const copy = box(".hero-copy"), visual = box(".hero-visual"), headline = box(".hero-headline");
    const whale = box(".whale-svg"), terminal = box(".hero-terminal");
    const navBrand = box(".nav .brand"), actions = box(".nav-actions");
    const searchButton = box("#site-search-button");
    const wide = innerWidth > 960;
    return {
      separated: separate(copy, visual) && separate(headline, whale) && separate(headline, terminal) && separate(whale, terminal),
      columns: !wide || copy.right <= visual.left + 1 || visual.right <= copy.left + 1,
      header: navBrand.right <= actions.left + 1 && searchButton.width >= 32
        && searchButton.left >= actions.left - 1,
      hierarchy: parseFloat(getComputedStyle(document.querySelector("h1")).fontSize) > parseFloat(getComputedStyle(document.querySelector(".hero-headline")).fontSize),
      terminalWidth: terminal.width,
      sectionOrder: document.querySelector(".hero").nextElementSibling.classList.contains("integration-strip")
        && document.querySelector(".integration-strip").nextElementSibling.id === "terminal-memory",
    };
  });
  expect(layout.separated && layout.columns, `${label}: hero columns, headline, whale, or terminal overlap`);
  expect(layout.header, `${label}: compact header controls exceed their available space`);
  expect(layout.hierarchy, `${label}: headline competes with the product name`);
  expect(layout.sectionOrder, `${label}: integrations and feature cards are out of order`);
  expect(layout.terminalWidth >= 260, `${label}: terminal demo is too small`);
  expect(await page.locator(".hero .cta-row a").count() === 2, `${label}: hero must have exactly two CTAs`);
  expect(await page.locator(".hero .cta-row .primary[href='#run']").count() === 1, `${label}: Quick Start is not the sole primary CTA`);
  expect(await page.locator(".hero .cta-row .secondary[href='#demo']").count() === 1, `${label}: demo CTA missing`);
  expect(await page.locator(".feature-card").count() === 3, `${label}: expected three introductory feature cards`);
  expect(await page.locator(".nav-links a").count() === 3, `${label}: navigation is no longer compact`);
  expect(await page.locator(".nav-actions .github-button").count() === 1, `${label}: GitHub action is not separate from navigation`);

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

async function checkSearch(page, language, label) {
  const dictionary = translations[language];
  await page.getByRole("button", { name: dictionary["search.openLabel"], exact: true }).click();
  const dialog = page.getByRole("dialog", { name: dictionary["search.title"], exact: true });
  expect(await dialog.isVisible(), `${label}: search did not open`);
  const input = page.getByRole("searchbox", { name: dictionary["search.label"], exact: true });
  expect(await input.getAttribute("placeholder") === dictionary["search.placeholder"], `${label}: search placeholder is not localized`);
  await input.fill("MCP");
  expect(await page.locator("#site-search-results a").count() > 0, `${label}: search returned no matching sections`);
  expect(await page.locator("#site-search-status").innerText() === dictionary["search.results"], `${label}: search status is not localized`);
  await input.fill('<img src=x onerror="alert(1)">');
  expect(await page.locator("#site-search-results img").count() === 0, `${label}: search interpreted user input as HTML`);
  expect(await page.locator("#site-search-status").innerText() === dictionary["search.empty"], `${label}: empty result is not localized`);
  await page.keyboard.press("Escape");
  await page.waitForFunction(() => !document.getElementById("site-search").open);
  expect(!await dialog.isVisible(), `${label}: Escape did not close search`);
  expect(await page.evaluate(() => document.activeElement.id) === "site-search-button", `${label}: search focus was not restored`);
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
        await checkSearch(page, language, label);
        if (size === "small-mobile") {
          const spacingOverride = await page.addStyleTag({ content: ".github-button { letter-spacing: 2px !important; }" });
          try {
            await checkLanguage(page, language, `${label}/wide-controls`);
          } finally {
            await spacingOverride.evaluate(element => element.remove());
          }
        }
        expect(new URL(page.url()).searchParams.get("lang") === language, `${label}: query language changed`);
        await page.screenshot({ path: `${artifactDir}/${language}-${size}.png`, fullPage: true });
        if (["en", "ar", "de", "zh-CN"].includes(language)) {
          await page.locator(".hero").screenshot({ path: `${artifactDir}/${language}-${size}-hero.png` });
          if (size === "desktop" && ["ar", "de"].includes(language)) {
            // Exercise the containment fallback independently of the host's
            // installed fonts (CI's Linux fonts differ from macOS).
            const fontOverride = await page.addStyleTag({ content: "h1 { font-family: monospace !important; font-size: 120px !important; }" });
            try {
              await checkLanguage(page, language, `${label}/wide-font`);
            } finally {
              await fontOverride.evaluate(element => element.remove());
            }
          }
        }
        // Exercise RTL -> LTR -> original selection on mobile as well as desktop.
        if (language === "ar") {
          const examples = await page.locator("pre").allTextContents();
          for (const next of ["de", "en", "ar"]) {
            await page.locator("#language-select").selectOption(next);
            await checkLanguage(page, next, `${label}/switch-${next}`);
            await checkSearch(page, next, `${label}/switch-${next}`);
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
    { locale: "ar-EG", expected: "en" },
    { locale: "de-AT", expected: "en" },
    { locale: "zh-Hans", expected: "en" },
    { locale: "zh-Hant-TW", expected: "en" },
    { locale: "ja-JP", expected: "en" },
    { locale: "ko-KR", expected: "en" },
    { locale: "en-US", query: "zh-Hans", expected: "zh-CN" },
    { locale: "en-US", query: "zh-Hant-TW", expected: "zh-TW" },
    { locale: "en-US", stored: "ar", expected: "ar" },
    { locale: "ar-SA", stored: "de", query: "unsupported", expected: "de" },
    { locale: "ar-SA", stored: "ar", query: "de-DE", expected: "de" },
    { locale: "de-DE", blockedStorage: true, expected: "en" },
    { locale: "zh-CN", query: "unsupported", expected: "en" },
    { locale: "de-DE", languages: ["zh-CN", "ar"], expected: "en" },
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

  // Search stays localized even if the interface language changes while open.
  const searchPage = await browser.newPage();
  try {
    trackErrors(searchPage, "search-navigation");
    await searchPage.goto(languageUrl("en"), { waitUntil: "networkidle" });
    await searchPage.keyboard.press("Control+k");
    await searchPage.locator("#site-search-input").fill("MCP");
    const beforeSubmit = searchPage.url();
    await searchPage.keyboard.press("Enter");
    expect(searchPage.url() === beforeSubmit, "search: Enter submitted a network navigation");
    await searchPage.evaluate(() => {
      const picker = document.getElementById("language-select");
      picker.value = "zh-CN";
      picker.dispatchEvent(new Event("change", { bubbles: true }));
    });
    expect(await searchPage.getByRole("dialog", { name: translations["zh-CN"]["search.title"], exact: true }).isVisible(), "search: open dialog title did not switch language");
    expect(await searchPage.locator("#site-search-status").innerText() === translations["zh-CN"]["search.results"], "search: live result status did not switch language");
    const target = searchPage.locator('#site-search-results a[href="#ai-agents"]');
    expect(await target.innerText() === translations["zh-CN"]["agents.title"], "search: result titles were not rebuilt in Chinese");
    await target.click();
    await searchPage.waitForFunction(() => !document.getElementById("site-search").open);
    const destination = new URL(searchPage.url());
    expect(destination.hash === "#ai-agents" && destination.searchParams.get("lang") === "zh-CN", "search: result navigation lost the language or target");
    expect(await searchPage.evaluate(() => document.activeElement.id) === "ai-agents", "search: focus did not follow the result");
  } finally {
    await searchPage.close();
  }

  const noScriptContext = await browser.newContext({ javaScriptEnabled: false });
  try {
    const page = await noScriptContext.newPage();
    const response = await page.goto(languageUrl("ar"), { waitUntil: "networkidle" });
    expect(response?.ok(), "no-js: page load failed");
    expect(await page.locator("html").getAttribute("lang") === "en", "no-js: fallback is not English");
    expect(await page.locator("html").getAttribute("dir") === "ltr", "no-js: fallback direction is not LTR");
    expect(await page.locator("h1").innerText() === "MemoryWhale", "no-js: brand heading missing");
    expect(await page.locator(".hero-headline").innerText() === "Make your terminal remember.", "no-js: English headline missing");
    expect(await page.locator(".nav-links a[href='#terminal-memory']").innerText() === translations.en["nav.terminal"], "no-js: English navigation missing");
    expect((await page.locator("pre").first().innerText()).includes("cargo install memorywhale-cli --version 0.10.0 --locked"), "no-js: command example missing");
    expect(!await page.locator("#site-search-button").isVisible(), "no-js: nonfunctional search control is visible");
  } finally {
    await noScriptContext.close();
  }
} catch (error) {
  failures.push(error.stack ?? String(error));
} finally {
  await browser.close();
}

if (failures.length) {
  console.error(failures.map((failure) => `- ${failure}`).join("\n"));
  process.exit(1);
}
console.log(`landing-page smoke passed: ${languages.length} languages, mobile/tablet/desktop, hero layout, search, RTL, preferences, and no-JS fallback`);
