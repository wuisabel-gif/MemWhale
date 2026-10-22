import { useEffect, useRef, type ReactNode } from "react";
import { openUrl } from "../api";

/** The website's ocean hero: the 3D whale swims behind whatever is inside. */
export function OceanHero({ children, label }: { children: ReactNode; label: string }) {
  const hero = useRef<HTMLElement>(null);
  const canvas = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    let stop: (() => void) | null = null;
    let gone = false;
    // three.js is most of the bundle; fetch it after the first paint.
    void import("../../ocean.js").then(({ startOcean }) => {
      if (!gone && hero.current && canvas.current) stop = startOcean(canvas.current, hero.current, { openUrl });
    });
    return () => {
      gone = true;
      stop?.();
    };
  }, []);

  return (
    <section className="hero" ref={hero} aria-label={label}>
      <canvas className="ocean-canvas" ref={canvas} aria-hidden="true" />
      {children}
    </section>
  );
}
