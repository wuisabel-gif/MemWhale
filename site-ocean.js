// Hero: replay the recall transcript in the hero device, and start the ocean
// scene (ocean.js) behind the copy. Without WebGL the CSS gradient stays.
import { startOcean } from "./ocean.js";

const canvas = document.querySelector(".ocean-canvas");
const hero = document.querySelector(".hero");
const reduce = matchMedia("(prefers-reduced-motion: reduce)").matches;

// Replay the recall transcript in the hero device: commands type out, output
// lines appear. The markup is the page's own static transcript, so without
// JavaScript (or with reduced motion) the full text simply stays visible.
const demo = document.querySelector(".recall-demo");
if (demo && !reduce) {
  const PROMPT = '<span class="prompt">$</span> ';
  const lines = demo.innerHTML.split("\n");
  // Pause while the device is off screen or the tab is hidden.
  let visible = true, wake = null;
  const resume = () => { if (visible && !document.hidden && wake) { wake(); wake = null; } };
  new IntersectionObserver(([e]) => { visible = e.isIntersecting; resume(); }).observe(demo);
  document.addEventListener("visibilitychange", resume);
  const sleep = async (ms) => {
    await new Promise((r) => setTimeout(r, ms));
    if (!visible || document.hidden) await new Promise((r) => { wake = r; });
  };
  (async () => {
    for (;;) {
      let shown = "";
      for (const line of lines) {
        if (line.startsWith(PROMPT)) {
          const cmd = line.slice(PROMPT.length);
          for (let i = 1; i <= cmd.length; i++) {
            demo.innerHTML = `${shown}${PROMPT}${cmd.slice(0, i)}<span class="recall-caret"></span>`;
            await sleep(40);
          }
          await sleep(380);
        } else {
          await sleep(240);
        }
        shown += line + "\n";
        demo.innerHTML = shown;
      }
      demo.innerHTML = `${shown}${PROMPT}<span class="recall-caret"></span>`;
      await sleep(5200);
    }
  })();
}

if (canvas && hero && startOcean(canvas, hero, { reduce })) hero.classList.add("has-ocean");
