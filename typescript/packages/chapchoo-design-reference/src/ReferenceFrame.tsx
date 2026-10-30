import type { ReactNode } from "react";

export interface ReferenceFrameProps {
  title: string;
  note: string;
  children: ReactNode;
}

/** Reference-local framing, not a production component or a design token source. */
export function ReferenceFrame({
  title,
  note,
  children,
}: ReferenceFrameProps): ReactNode {
  return (
    <main className="reference-frame">
      <header>
        <p className="reference-label">Scaffolding · design not selected</p>
        <h1>{title}</h1>
        <p>{note}</p>
      </header>
      {children}
    </main>
  );
}
