import type { ReactElement } from "react";
const paths: Record<string, string> = {
  user: "M20 21v-2a6 6 0 0 0-12 0v2M14 11a4 4 0 1 0 0-8 4 4 0 0 0 0 8",
  assistant: "M4 5h16M12 5v14M8 19h8",
  publication: "m22 2-7 20-4-9-9-4 20-7ZM22 2 11 13",
  read: "M5 3h10l4 4v14H5zM14 3v5h5M8 12h8M8 16h6",
  edit: "m15 4 5 5M4 20l4-1L21 6l-5-5L3 14z",
  search: "M10 17a7 7 0 1 0 0-14 7 7 0 0 0 0 14M15 15l6 6",
  execute: "m4 5 6 6-6 6M13 18h7",
  think: "M9 18h6M10 21h4M8 14a7 7 0 1 1 8 0l-1 2H9z",
  fetch:
    "M3 12h18M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18M12 3c-5 5-5 13 0 18 5-5 5-13 0-18",
  delete: "M4 6h16M9 6V3h6v3M6 6l1 15h10l1-15M10 10v7M14 10v7",
  move: "M3 7h12M11 3l4 4-4 4M21 17H9M13 13l-4 4 4 4",
  switch_mode: "M3 7h18M3 17h18M8 3v8M16 13v8",
  plan: "M4 6h2M10 6h10M4 12h2M10 12h10M4 18h2M10 18h10",
  request:
    "M9 8a3 3 0 1 1 5 2c-2 1-2 2-2 3M12 17h.01M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18",
  other: "m12 3 9 9-9 9-9-9zM12 8v5M12 16h.01",
};
export function RecordIcon({ kind }: { kind: string }): ReactElement {
  return (
    <svg
      data-record-icon={kind}
      width="18"
      height="18"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.55"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <path d={paths[kind] ?? paths.other} />
    </svg>
  );
}
