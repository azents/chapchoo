import type { Grant } from "@chapchoo/web-core/browser";
export interface InitialInventory {
  offset: string;
  page: { items: Grant[]; total: bigint };
  renderedAt: number;
}
