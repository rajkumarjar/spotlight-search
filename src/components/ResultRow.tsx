import type { RefCallback } from "react";
import { KeyHint } from "./KeyHint";
import type { SearchItem } from "../types";

interface ResultRowProps {
  item: SearchItem;
  index: number;
  selected: boolean;
  opening: boolean;
  rowRef: RefCallback<HTMLButtonElement>;
  onHover: () => void;
  onSelect: () => void;
}

export function ResultRow({
  item,
  index,
  selected,
  opening,
  rowRef,
  onHover,
  onSelect,
}: ResultRowProps) {
  const Icon = item.Icon;
  return (
    <button
      ref={rowRef}
      type="button"
      className={`result-row${selected ? " result-row--selected" : ""}${opening ? " result-row--opening" : ""}`}
      aria-selected={selected}
      onMouseEnter={onHover}
      onClick={onSelect}
    >
      <span className={`app-icon app-icon--${item.tint}`} aria-hidden="true">
        <Icon size={21} strokeWidth={1.8} />
      </span>
      <span className="result-name">{item.name}</span>
      <span className="result-kind">{item.kind}</span>
      <KeyHint number={index + 1} />
    </button>
  );
}
