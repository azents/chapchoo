"use client";

import { createContext, useContext } from "react";
import type { InitialRead, User, WireReason } from "./browser.js";

export type InitialIdentity = InitialRead<User | null>;
export type IdentityState =
  | { type: "AUTHENTICATED"; user: User }
  | { type: "SIGNED_OUT" }
  | { type: "ERROR"; reason: WireReason | null };

export interface Authentication {
  state: IdentityState;
  checking: boolean;
  logoutPending: boolean;
  logoutError: string | null;
  logout: () => void;
  recheck: () => Promise<void>;
  authenticated: () => void;
  clearIdentity: () => void;
}

/** Shares a query projection and actions, never a second mutable User store. */
export const AuthenticationContext = createContext<Authentication | null>(null);

export function useAuth(): Authentication {
  const auth = useContext(AuthenticationContext);
  if (!auth) throw new Error("Authentication provider is required");
  return auth;
}
