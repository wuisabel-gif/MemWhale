import { chromium } from "playwright";
import { mkdir } from "node:fs/promises";

const baseUrl = process.env.SITE_URL ?? "http://127.0.0.1:4173/index.html";
const artifactDir = "artifacts/site-smoke";
await mkdir(artifactDir, { recursive: true });

const browser = await chromium.launch({ headless: true });
const failures = [];
try {
  for (const [name, width, height] of [
    ["mobile", 390, 844],
    ["tablet", 768, 1024],
    ["desktop", 1440, 900],
  ]) {
    const page = await browser.newPage({ viewport: { width, height } });
    const consoleErrors = [];
    const pageErrors = [];
    page.on("console", (message) => {
      if (message.type() === "error") consoleErrors.push(message.text());
    });
    page.on("pageerror", (error) => pageErrors.push(error.message));
    const response = await page.goto(baseUrl, { waitUntil: "networkidle" });
    if (!response || !response.ok()) {
      failures.push(`${name}: page did not load successfully (${response?.status() ?? "no response"})`);
    }
    await page.screenshot({ path: `${artifactDir}/${name}.png`, fullPage: true });
    const heading = await page.locator("h1").innerText();
    const requiredSections = ["#demo", "#data", "#security", "#run"];
    for (const selector of requiredSections) {
      if ((await page.locator(selector).count()) !== 1) {
        failures.push(`${name}: missing required section ${selector}`);
      }
    }
    if ((await page.locator("a[href^='http']").count()) < 3) {
      failures.push(`${name}: expected external release/security links`);
    }
    const horizontalOverflow = await page.evaluate(
      () => document.documentElement.scrollWidth > document.documentElement.clientWidth + 1
    );
    if (horizontalOverflow) failures.push(`${name}: horizontal overflow detected`);
    if (!heading.includes("MemoryWhale")) failures.push(`${name}: missing hero heading`);
    if (consoleErrors.length) failures.push(`${name}: console errors: ${consoleErrors.join(" | ")}`);
    if (pageErrors.length) failures.push(`${name}: page errors: ${pageErrors.join(" | ")}`);
    await page.close();
  }
} finally {
  await browser.close();
}

if (failures.length) {
  console.error(failures.map((failure) => `- ${failure}`).join("\n"));
  process.exit(1);
}
console.log("landing-page browser smoke passed at mobile, tablet, and desktop widths");
