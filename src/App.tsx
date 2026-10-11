import { useCallback, useEffect, useRef, useState, type KeyboardEvent } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { AppWindow, FileText, Folder } from "lucide-react";
import { ResultRow } from "./components/ResultRow";
import { ResultSection } from "./components/ResultSection";
import { SearchBar } from "./components/SearchBar";
import { usePanelAnimation } from "./hooks/usePanelAnimation";
import type { SearchItem, SearchReply } from "./types";

const SHOW_PREVIEW = false;
const MAX_VISIBLE_RESULTS = 9;

function App() {
  const [query, setQuery] = useState("");
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [openingId, setOpeningId] = useState<string | null>(null);
  const [searchError, setSearchError] = useState("");
  const [launchError, setLaunchError] = useState("");
  const [isSearching, setIsSearching] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);
  const rowRefs = useRef<Array<HTMLButtonElement | null>>([]);
  const searchSequence = useRef(0);
  const { contentRef, isVisible, panelHeight, openPanel, closePanel, markPanelHidden } =
    usePanelAnimation();

  const [results, setResults] = useState<SearchItem[]>([]);

  useEffect(() => {
    const normalizedQuery = query.trim();
    const sequence = ++searchSequence.current;
    if (!normalizedQuery) {
      setResults([]);
      setSearchError("");
      setIsSearching(false);
      return;
    }

    setResults([]);
    setSearchError("");
    setIsSearching(true);
    const timeout = window.setTimeout(() => {
      void invoke<SearchReply>("search", { query: normalizedQuery })
        .then((reply) => {
          if (sequence !== searchSequence.current) return;
          setResults(
            reply.results.slice(0, MAX_VISIBLE_RESULTS).map<SearchItem>((item) => ({
              ...item,
              detail: item.subtitle,
              Icon:
                item.kind === "Application"
                  ? AppWindow
                  : item.kind === "Folder"
                    ? Folder
                    : FileText,
              tint: item.kind === "Application" ? "blue" : "slate",
            })),
          );
          setSearchError(reply.warning ?? "");
          setIsSearching(false);
        })
        .catch((error: unknown) => {
          if (sequence !== searchSequence.current) return;
          console.error("Spotlight search failed:", error);
          setResults([]);
          setSearchError("Search is temporarily unavailable.");
          setIsSearching(false);
        });
    }, 60);

    return () => window.clearTimeout(timeout);
  }, [query]);

  useEffect(() => {
    setSelectedIndex(0);
    setLaunchError("");
  }, [query]);

  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;
    let focusTimeout: number | undefined;

    void listen("spotlight-open", () => {
      searchSequence.current += 1;
      setQuery("");
      setResults([]);
      setSearchError("");
      setLaunchError("");
      setSelectedIndex(0);
      openPanel();
      if (focusTimeout !== undefined) window.clearTimeout(focusTimeout);

      let attempts = 0;
      const focusInput = () => {
        const input = inputRef.current;
        if (!input || disposed) return;

        input.focus({ preventScroll: true });
        attempts += 1;
        if (document.activeElement !== input && attempts < 10) {
          focusTimeout = window.setTimeout(focusInput, 50);
        }
      };

      window.requestAnimationFrame(focusInput);
    })
      .then((stopListening) => {
        if (disposed) {
          stopListening();
          return;
        }
        unlisten = stopListening;
      })
      .catch((error: unknown) => {
        console.error("Unable to listen for the Spotlight open event:", error);
      });

    return () => {
      disposed = true;
      if (focusTimeout !== undefined) window.clearTimeout(focusTimeout);
      unlisten?.();
    };
  }, [openPanel]);

  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;

    void listen("spotlight-close", () => {
      searchSequence.current += 1;
      setQuery("");
      setResults([]);
      setSearchError("");
      setLaunchError("");
      setSelectedIndex(0);
      markPanelHidden();
    })
      .then((stopListening) => {
        if (disposed) {
          stopListening();
          return;
        }
        unlisten = stopListening;
      })
      .catch((error: unknown) => {
        console.error("Unable to listen for the Spotlight close event:", error);
      });

    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [markPanelHidden]);

  useEffect(() => {
    rowRefs.current[selectedIndex]?.scrollIntoView({ block: "nearest" });
  }, [selectedIndex, results.length]);

  useEffect(() => {
    const handleEscape = (event: globalThis.KeyboardEvent) => {
      if (event.key !== "Escape") return;
      event.preventDefault();
      event.stopPropagation();
      setQuery("");
      setLaunchError("");
      closePanel();
    };

    window.addEventListener("keydown", handleEscape, true);
    return () => window.removeEventListener("keydown", handleEscape, true);
  }, [closePanel]);

  const selectResult = useCallback(
    (index: number) => {
      const result = results[index];
      if (!result) return;

      setOpeningId(result.id);
      setLaunchError("");
      void invoke("launch", { id: result.id })
        .then(() => closePanel())
        .catch((error: unknown) => {
          console.error("Unable to launch search result:", error);
          setLaunchError("Unable to open this result.");
        })
        .finally(() => setOpeningId(null));
    },
    [closePanel, results],
  );

  const onKeyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    if (event.key === "Tab") {
      event.preventDefault();
      return;
    }

    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      if (!results.length) return;
      const direction = event.key === "ArrowDown" ? 1 : -1;
      setSelectedIndex((current) => (current + direction + results.length) % results.length);
      return;
    }

    if (event.key === "Enter") {
      event.preventDefault();
      selectResult(selectedIndex);
      return;
    }

    if (event.altKey && /^[1-9]$/.test(event.key)) {
      event.preventDefault();
      selectResult(Number(event.key) - 1);
    }
  };

  const handleQueryChange = (value: string) => {
    setQuery(value);
    setLaunchError("");
  };

  const renderResult = (item: SearchItem, index: number) => (
    <ResultRow
      key={item.id}
      item={item}
      index={index}
      selected={selectedIndex === index}
      opening={openingId === item.id}
      rowRef={(element) => {
        rowRefs.current[index] = element;
      }}
      onHover={() => setSelectedIndex(index)}
      onSelect={() => selectResult(index)}
    />
  );

  const remainingResults = results.slice(1);
  const applications = remainingResults
    .map((item, index) => ({ item, index: index + 1 }))
    .filter(({ item }) => item.kind === "Application");
  const files = remainingResults
    .map((item, index) => ({ item, index: index + 1 }))
    .filter(({ item }) => item.kind === "File");
  const folders = remainingResults
    .map((item, index) => ({ item, index: index + 1 }))
    .filter(({ item }) => item.kind === "Folder");

  return (
    <main className="window-shell">
      <section
        className={`spotlight-panel${isVisible ? " spotlight-panel--visible" : ""}${query ? " spotlight-panel--with-results" : ""}`}
        style={{ height: panelHeight ? `${panelHeight}px` : undefined }}
        aria-label="Spotlight Search"
        onContextMenu={(event) => event.preventDefault()}
      >
        <div ref={contentRef} className="spotlight-content">
          <SearchBar
            inputRef={inputRef}
            query={query}
            onChange={handleQueryChange}
            onKeyDown={onKeyDown}
          />

          {query && (
            <div
              className={`results${results.length ? " results--has-matches" : ""}`}
              aria-live="polite"
            >
              {results.length > 0 ? (
                <>
                  <ResultSection title="Top Hit">{renderResult(results[0], 0)}</ResultSection>
                  {applications.length > 0 && (
                    <ResultSection title="Applications">
                      {applications.map(({ item, index }) => renderResult(item, index))}
                    </ResultSection>
                  )}
                  {files.length > 0 && (
                    <ResultSection title="Documents">
                      {files.map(({ item, index }) => renderResult(item, index))}
                    </ResultSection>
                  )}
                  {folders.length > 0 && (
                    <ResultSection title="Folders">
                      {folders.map(({ item, index }) => renderResult(item, index))}
                    </ResultSection>
                  )}
                </>
              ) : (
                <div className="empty-state">
                  {searchError || (isSearching ? "Searching…" : `No results for “${query}”`)}
                </div>
              )}
            </div>
          )}

          {searchError && results.length > 0 && (
            <div className="demo-message" role="status">
              {searchError}
            </div>
          )}

          {launchError && (
            <div className="demo-message" role="status">
              {launchError}
            </div>
          )}
        </div>
      </section>
      {SHOW_PREVIEW && results[selectedIndex] && (
        <aside className="preview-pane" aria-label="Result preview">
          <div className={`app-icon app-icon--${results[selectedIndex].tint}`}>
            {(() => {
              const PreviewIcon = results[selectedIndex].Icon;
              return <PreviewIcon size={48} strokeWidth={1.5} />;
            })()}
          </div>
          <strong>{results[selectedIndex].name}</strong>
          <span>{results[selectedIndex].kind}</span>
          <span>{results[selectedIndex].detail}</span>
        </aside>
      )}
    </main>
  );
}

export default App;
