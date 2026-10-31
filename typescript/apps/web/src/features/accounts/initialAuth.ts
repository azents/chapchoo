import type {
  InitialRead,
  LoginMethods,
  ProofPreview,
} from "@chapchoo/web-core/browser";

export type InitialAuth =
  | { type: "METHODS"; result: InitialRead<LoginMethods> }
  | { type: "PROOF"; result: InitialRead<ProofPreview> }
  | { type: "NO_PROOF" };
