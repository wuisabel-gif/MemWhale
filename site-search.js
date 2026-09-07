// Homepage-only search. Queries and page content stay in the browser.
(() => {
  if (typeof document === "undefined") return;
  const dialog = document.getElementById("site-search");
  const opener = document.getElementById("site-search-button");
  const input = document.getElementById("site-search-input");
  const results = document.getElementById("site-search-results");
  const status = document.getElementById("site-search-status");
  const close = document.getElementById("site-search-close");
  const form = document.getElementById("site-search-form");
  if (!globalThis.MEMORYWHALE_I18N || !dialog || typeof dialog.showModal !== "function"
    || !opener || !input || !results || !status || !close || !form) return;

  let returnFocus = true;
  const dictionary = () => globalThis.MEMORYWHALE_I18N.translations[document.documentElement.lang]
    || globalThis.MEMORYWHALE_I18N.translations.en;
  const normalize = (text) => text.normalize("NFKC").toLocaleLowerCase(document.documentElement.lang).trim();

  function renderResults() {
    const query = normalize(input.value);
    const matches = query ? [...document.querySelectorAll("main > section[id]")]
      .map((section) => {
        const title = section.getAttribute("aria-label")
          || section.querySelector(".hero-headline, h2, h3")?.textContent
          || section.id;
        const score = normalize(title).includes(query) ? 2 : normalize(section.textContent).includes(query) ? 1 : 0;
        return { section, title: title.trim(), score };
      })
      .filter(({ score }) => score > 0)
      .sort((a, b) => b.score - a.score)
      .slice(0, 10) : [];

    results.replaceChildren();
    for (const { section, title } of matches) {
      const item = document.createElement("li");
      const link = document.createElement("a");
      link.href = `#${section.id}`;
      link.textContent = title;
      link.addEventListener("click", () => {
        returnFocus = false;
        dialog.close();
        section.setAttribute("tabindex", "-1");
        section.focus({ preventScroll: true });
      });
      item.append(link);
      results.append(item);
    }
    const key = !query ? "search.hint" : matches.length ? "search.results" : "search.empty";
    status.dataset.i18n = key;
    status.textContent = dictionary()[key];
  }

  function openSearch() {
    if (!dialog.open) {
      returnFocus = true;
      renderResults();
      dialog.showModal();
    }
    input.focus();
    input.select();
  }

  opener.addEventListener("click", openSearch);
  close.addEventListener("click", () => dialog.close());
  form.addEventListener("submit", (event) => event.preventDefault());
  input.addEventListener("input", renderResults);
  dialog.addEventListener("keydown", (event) => {
    if (event.key === "Escape" && !event.isComposing) {
      // Search inputs may consume Escape to clear their value first. Closing
      // the dialog should still take one keypress, with focus restored below.
      event.preventDefault();
      dialog.close();
    }
  });
  dialog.addEventListener("close", () => {
    if (returnFocus) opener.focus({ preventScroll: true });
  });
  dialog.addEventListener("click", (event) => {
    if (event.target !== dialog) return;
    const bounds = dialog.getBoundingClientRect();
    if (event.clientX < bounds.left || event.clientX > bounds.right
      || event.clientY < bounds.top || event.clientY > bounds.bottom) dialog.close();
  });
  document.addEventListener("keydown", (event) => {
    if (!event.isComposing && !event.altKey && (event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k") {
      event.preventDefault();
      openSearch();
    }
  });
  document.addEventListener("memorywhale:languagechange", renderResults);
  opener.hidden = false;
})();
