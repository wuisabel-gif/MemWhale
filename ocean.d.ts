/** Types for ocean.js, the scene shared by the website hero and the desktop app. */
export function startOcean(
  canvas: HTMLCanvasElement,
  hero: HTMLElement,
  options?: { reduce?: boolean; openUrl?: (url: string) => void }
): (() => void) | null;
