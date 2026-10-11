import type { ReactNode } from "react";

interface ResultSectionProps {
  title: string;
  children: ReactNode;
}

export function ResultSection({ title, children }: ResultSectionProps) {
  return (
    <section className="result-section" aria-label={title}>
      <h2 className="section-heading">{title}</h2>
      {children}
    </section>
  );
}
