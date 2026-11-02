import type { Team } from "@chapchoo/web-core/browser";
export type InitialNavigation =
  | { type: "guest" }
  | { type: "error"; userId: string }
  | {
      type: "ready";
      userId: string;
      teams: Team[];
      preferred: string | null;
      renderedAt: number;
    };
