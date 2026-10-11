interface KeyHintProps {
  number: number;
}

export function KeyHint({ number }: KeyHintProps) {
  return (
    <kbd className="key-hint" aria-label={`Alt plus ${number}`}>
      Alt+{number}
    </kbd>
  );
}
