import type { KeyboardEvent, RefObject } from "react";
import { Search } from "lucide-react";

interface SearchBarProps {
  inputRef: RefObject<HTMLInputElement>;
  query: string;
  onChange: (value: string) => void;
  onKeyDown: (event: KeyboardEvent<HTMLInputElement>) => void;
}

export function SearchBar({ inputRef, query, onChange, onKeyDown }: SearchBarProps) {
  return (
    <div className="search-bar">
      <Search className="search-icon" size={22} strokeWidth={1.75} aria-hidden="true" />
      <input
        ref={inputRef}
        autoFocus
        aria-label="Search sample apps"
        autoComplete="off"
        spellCheck={false}
        value={query}
        onChange={(event) => onChange(event.currentTarget.value)}
        onKeyDown={onKeyDown}
        placeholder="Spotlight Search"
      />
      {query && (
        <span className="search-shortcut" aria-hidden="true">
          ESC
        </span>
      )}
    </div>
  );
}
