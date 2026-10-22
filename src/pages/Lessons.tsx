import { Check, Trash2 } from "lucide-react";
import { useEffect, useState } from "react";
import { call } from "../api";
import { Empty, PageHead } from "../components/bits";
import type { Lesson, PageProps } from "../types";

export function Lessons({ refreshKey, refresh }: PageProps) {
  const [lessons, setLessons] = useState<Lesson[] | null>(null);
  const [draft, setDraft] = useState("");
  const [confirming, setConfirming] = useState<number | null>(null);
  const [error, setError] = useState("");

  const load = () => call<Lesson[]>("list_lessons").then(setLessons, (e) => setError(String(e)));
  useEffect(() => void load(), [refreshKey]);

  async function act(command: string, args: Record<string, unknown>) {
    try {
      await call(command, args);
      setError("");
      setConfirming(null);
      refresh();
      await load();
      return true;
    } catch (e) {
      setError(String(e));
      return false;
    }
  }

  async function save() {
    if (draft.trim() && (await act("add_lesson", { text: draft }))) setDraft("");
  }

  return (
    <>
      <PageHead title="Lessons">
        What you learned, in your words. When the failure a lesson mentions comes back, agents see the lesson first.
      </PageHead>
      <form
        className="composer card"
        onSubmit={(e) => {
          e.preventDefault();
          void save();
        }}
      >
        <label htmlFor="lesson">New lesson</label>
        <textarea
          id="lesson"
          value={draft}
          onChange={(e) => setDraft(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) void save();
          }}
          placeholder="On this Mac, cargo build needs xcode-select --install before the first build."
          rows={3}
        />
        <div className="composer-foot">
          <span className="muted">Name the program or the error so it matches when it comes back. ⌘↩ saves.</span>
          <button className="button primary" disabled={!draft.trim()} type="submit">
            Save lesson
          </button>
        </div>
      </form>
      {error && <p className="error-banner">{error}</p>}
      {lessons !== null && lessons.length === 0 && <Empty>No lessons yet. The first one is the most useful.</Empty>}
      <ul className="lessons">
        {lessons?.map((lesson) => (
          <li className={`lesson card ${lesson.approved ? "" : "pending"}`} key={lesson.id}>
            <p className="lesson-text">{lesson.label}</p>
            <div className="lesson-foot">
              <span className="run-meta">{lesson.provenance}</span>
              <span className="lesson-actions">
                {!lesson.approved && (
                  <button className="button small good" onClick={() => void act("approve_lesson", { id: lesson.id })} type="button">
                    <Check size={15} aria-hidden="true" /> Approve
                  </button>
                )}
                {confirming === lesson.id ? (
                  <>
                    <button className="button small danger" onClick={() => void act("delete_lesson", { id: lesson.id })} type="button">
                      Delete for good
                    </button>
                    <button className="button small" onClick={() => setConfirming(null)} type="button">
                      Keep
                    </button>
                  </>
                ) : (
                  <button
                    className="icon-button"
                    onClick={() => setConfirming(lesson.id)}
                    type="button"
                    aria-label={`Delete lesson ${lesson.id}`}
                  >
                    <Trash2 size={16} />
                  </button>
                )}
              </span>
            </div>
          </li>
        ))}
      </ul>
    </>
  );
}
