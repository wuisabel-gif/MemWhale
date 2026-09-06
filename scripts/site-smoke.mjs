import { chromium } from "playwright";
import { mkdir } from "node:fs/promises";

const baseUrl = process.env.SITE_URL ?? "http://127.0.0.1:4173/index.html";
const artifactDir = "artifacts/site-smoke";
const languages = ["en", "fr", "zh-CN", "zh-TW", "ko", "ja"];
const expected = {
  en: {
    title: "MemoryWhale — terminal memory for you and your AI agent",
    heading: "MemoryWhale remembers what your terminal forgets.",
    nav: "Terminal Memory"
  },
  fr: {
    title: "MemoryWhale — une mémoire de terminal pour vous et votre agent IA",
    heading: "MemoryWhale se souvient de ce que votre terminal oublie.",
    nav: "Mémoire du terminal"
  },
  "zh-CN": {
    title: "MemoryWhale — 为你和 AI 智能体提供的终端记忆",
    heading: "MemoryWhale 记住终端忘记的事。",
    nav: "终端记忆"
  },
  "zh-TW": {
    title: "MemoryWhale — 為你與 AI 代理提供的終端機記憶",
    heading: "MemoryWhale 記住終端機忘記的事。",
    nav: "終端機記憶"
  },
  ko: {
    title: "MemoryWhale — 나와 AI 에이전트를 위한 터미널 메모리",
    heading: "MemoryWhale은 터미널이 잊는 것을 기억합니다.",
    nav: "터미널 메모리"
  },
  ja: {
    title: "MemoryWhale — あなたと AI エージェントのためのターミナルメモリ",
    heading: "MemoryWhale はターミナルが忘れることを覚えています。",
    nav: "ターミナルメモリ"
  }
};
const viewports = [
  ["mobile", 390, 844],
  ["desktop", 1440, 900]
];
await mkdir(artifactDir, { recursive: true });

const languageUrl = (language) => {
  const url = new URL(baseUrl);
  url.searchParams.set("lang", language);
  url.hash = "demo";
  return url.toString();
};

const browser = await chromium.launch({ headless: true });
const failures = [];
try {
  for (const language of languages) {
    for (const [name, width, height] of viewports) {
      const page = await browser.newPage({ viewport: { width, height } });
      const consoleErrors = [];
      const pageErrors = [];
      page.on("console", (message) => {
        if (message.type() === "error") consoleErrors.push(message.text());
      });
      page.on("pageerror", (error) => pageErrors.push(error.message));

      const url = languageUrl(language);
      const response = await page.goto(url, { waitUntil: "networkidle" });
      if (!response || !response.ok()) {
        failures.push(`${language}/${name}: page did not load successfully (${response?.status() ?? "no response"})`);
      }
      await page.screenshot({ path: `${artifactDir}/${language}-${name}.png`, fullPage: true });

      const heading = await page.locator("h1").innerText();
      const navText = await page.locator(".nav-links a[href='#terminal-memory']").innerText();
      const pageTitle = await page.title();
      const requiredSections = ["#demo", "#data", "#security", "#run"];
      for (const selector of requiredSections) {
        if ((await page.locator(selector).count()) !== 1) {
          failures.push(`${language}/${name}: missing required section ${selector}`);
        }
      }
      if ((await page.locator("a[href^='http']").count()) < 3) {
        failures.push(`${language}/${name}: expected external release/security links`);
      }
      if ((await page.locator("#language-select option").count()) !== languages.length) {
        failures.push(`${language}/${name}: language selector does not expose all six options`);
      }
      const selectedLanguage = await page.locator("#language-select").inputValue();
      if (selectedLanguage !== language) {
        failures.push(`${language}/${name}: selector selected ${selectedLanguage}, expected ${language}`);
      }
      const documentLanguage = await page.locator("html").getAttribute("lang");
      if (documentLanguage !== language) {
        failures.push(`${language}/${name}: html lang is ${documentLanguage}, expected ${language}`);
      }
      if (pageTitle !== expected[language].title) {
        failures.push(`${language}/${name}: unexpected document title: ${pageTitle}`);
      }
      const jsonLd = await page.locator('script[type="application/ld+json"]').textContent();
      try {
        if (JSON.parse(jsonLd ?? "{}").inLanguage !== language) {
          failures.push(`${language}/${name}: JSON-LD language metadata was not updated`);
        }
      } catch {
        failures.push(`${language}/${name}: JSON-LD metadata is not valid JSON`);
      }
      const currentUrl = new URL(page.url());
      if (currentUrl.searchParams.get("lang") !== language || currentUrl.hash !== "#demo") {
        failures.push(`${language}/${name}: language query or anchor was not preserved (${page.url()})`);
      }
      const horizontalOverflow = await page.evaluate(
        () => document.documentElement.scrollWidth > document.documentElement.clientWidth + 1
      );
      if (horizontalOverflow) failures.push(`${language}/${name}: horizontal overflow detected`);
      if (heading !== expected[language].heading) {
        failures.push(`${language}/${name}: unexpected hero heading: ${heading}`);
      }
      if (navText !== expected[language].nav) {
        failures.push(`${language}/${name}: unexpected visible nav label: ${navText}`);
      }
      if (consoleErrors.length) failures.push(`${language}/${name}: console errors: ${consoleErrors.join(" | ")}`);
      if (pageErrors.length) failures.push(`${language}/${name}: page errors: ${pageErrors.join(" | ")}`);

      if (language === "en" && name === "desktop") {
        const beforePath = new URL(page.url()).pathname;
        await page.locator("#language-select").selectOption("fr");
        const afterUrl = new URL(page.url());
        if (afterUrl.pathname !== beforePath || afterUrl.hash !== "#demo" || afterUrl.searchParams.get("lang") !== "fr") {
          failures.push(`selector change navigated away or lost the anchor (${page.url()})`);
        }
        if ((await page.locator("h1").innerText()) !== expected.fr.heading) {
          failures.push("selector change did not render the French hero heading");
        }
      }
      await page.close();
    }
  }

  const noScriptContext = await browser.newContext({
    viewport: { width: 1440, height: 900 },
    javaScriptEnabled: false
  });
  try {
    const page = await noScriptContext.newPage();
    const response = await page.goto(languageUrl("en"), { waitUntil: "networkidle" });
    if (!response || !response.ok()) {
      failures.push(`no-js/en: page did not load successfully (${response?.status() ?? "no response"})`);
    }
    if ((await page.locator("h1").innerText()) !== expected.en.heading) {
      failures.push("no-js/en: English hero heading is not present without JavaScript");
    }
    if ((await page.locator(".nav-links a[href='#terminal-memory']").innerText()) !== expected.en.nav) {
      failures.push("no-js/en: English navigation is not present without JavaScript");
    }
    if (!(await page.locator("pre").first().innerText()).includes("https://raw.githubusercontent.com/wuisabel-gif/MemWhale/main/install.sh")) {
      failures.push("no-js/en: install command example is not present without JavaScript");
    }
    await page.close();
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
console.log("landing-page browser smoke passed for all six languages at mobile and desktop widths");
