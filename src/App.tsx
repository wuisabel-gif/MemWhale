import { Bell, BookOpen, Search, Terminal, Waves, type LucideIcon } from "lucide-react";
import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useRef, useState } from "react";
import { call, inTauri, shortPath } from "./api";
import { RunDrawer } from "./components/RunDrawer";
import { Commands } from "./pages/Commands";
import { Home } from "./pages/Home";
import { Lessons } from "./pages/Lessons";
import { RecallPage } from "./pages/Recall";
import { SearchPage } from "./pages/Search";
import type { Overview, Page, PageProps } from "./types";

const PAGES: { id: Page; label: string; icon: LucideIcon; view: (props: PageProps) => JSX.Element }[] = [
  { id: "home", label: "Home", icon: Waves, view: Home },
  { id: "search", label: "Search", icon: Search, view: SearchPage },
  { id: "recall", label: "Recall", icon: Bell, view: RecallPage },
  { id: "commands", label: "Commands", icon: Terminal, view: Commands },
  { id: "lessons", label: "Lessons", icon: BookOpen, view: Lessons }
];

export default function App() {
  const [page, setPage] = useState<Page>("home");
  const [overview, setOverview] = useState<Overview | null>(null);
  const [refreshKey, setRefreshKey] = useState(0);
  const [openRun, setOpenRun] = useState<number | null>(null);
  const [error, setError] = useState("");

  const refresh = useCallback(() => {
    call<Overview>("overview").then(
      (next) => (setOverview(next), setError("")),
      (e) => setError(String(e))
    );
    setRefreshKey((k) => k + 1);
  }, []);

  // Hooks in other processes write to the store; reread it when the window comes back.
  useEffect(() => {
    refresh();
    window.addEventListener("focus", refresh);
    return () => window.removeEventListener("focus", refresh);
  }, [refresh]);

  // The tray watcher announces each new note or warning; show it right away.
  useEffect(() => {
    if (!inTauri) return;
    const off = listen("recall", refresh);
    return () => void off.then((stop) => stop());
  }, [refresh]);

  const main = useRef<HTMLElement>(null);
  useEffect(() => {
    main.current?.scrollTo(0, 0);
  }, [page]);

  const closeRun = useCallback(() => setOpenRun(null), []);
  const View = PAGES.find((p) => p.id === page)!.view;

  return (
    <div className="app">
      <aside className="sidebar">
        <div className="brand">
          <span className="brand-whale" aria-hidden="true" />
          <span className="script wordmark">MemoryWhale</span>
        </div>
        <nav className="nav" aria-label="Sections">
          {PAGES.map(({ id, label, icon: Icon }) => (
            <button
              className="nav-item"
              aria-current={page === id ? "page" : undefined}
              key={id}
              onClick={() => setPage(id)}
              type="button"
            >
              <Icon size={18} aria-hidden="true" />
              <span>{label}</span>
              {id === "lessons" && !!overview?.pending_lessons && (
                <span className="nav-count" title="Lessons waiting for review">
                  {overview.pending_lessons}
                </span>
              )}
            </button>
          ))}
        </nav>
        <div className="sidebar-foot">
          {!inTauri && <p className="preview-note">Browser preview with sample data</p>}
          <p className="store-label">Stored only on this machine</p>
          <p className="store-path mono" title={overview?.store}>
            {shortPath(overview?.store)}
          </p>
        </div>
      </aside>
      <main className="main" ref={main}>
        <div className="page" key={page}>
          {error && <p className="error-banner">{error}</p>}
          <View overview={overview} refreshKey={refreshKey} refresh={refresh} go={setPage} openRun={setOpenRun} />
        </div>
      </main>
      {openRun !== null && <RunDrawer id={openRun} onOpen={setOpenRun} onClose={closeRun} />}
    </div>
  );
}
