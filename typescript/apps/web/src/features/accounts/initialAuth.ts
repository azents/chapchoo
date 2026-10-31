import type { LoginMethods, ProofPreview } from "@chapchoo/web-core/browser";

export type InitialAuth =
  | { type: "METHODS"; data: LoginMethods }
  | { type: "PROOF"; data: ProofPreview }
  | { type: "NO_PROOF" };
