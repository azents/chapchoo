"use client";
import { createContext, useContext } from "react";
import type { DisplayChoices, DisplayKey, DisplayValue } from "./display.js";
import type { InitialIdentity } from "./auth-context.js";

export interface DisplayInitial {
  identity: InitialIdentity;
  value: DisplayValue;
  explicit: DisplayChoices;
  defaults: DisplayValue;
  instant: number;
  zones: string[];
}

export interface DisplayController {
  value: DisplayValue;
  fields: DisplayValue;
  defaults: DisplayValue;
  choices: DisplayChoices;
  guest: boolean;
  available: boolean;
  busy: boolean;
  error: boolean;
  storageError: boolean;
  changed: boolean;
  instant: number;
  zones: readonly string[];
  opened: boolean;
  open: () => void;
  close: () => void;
  change: (key: DisplayKey, value: string | null) => void;
  retry: () => void;
  suggestions: { key: DisplayKey; current: string; target: string }[];
  dismiss: () => void;
  creationDisplay: () => DisplayChoices;
}
export const DisplayContext = createContext<DisplayController | null>(null);
export function useDisplay(): DisplayController {
  const value = useContext(DisplayContext);
  if (!value) throw new Error("Display preference provider is required");
  return value;
}
