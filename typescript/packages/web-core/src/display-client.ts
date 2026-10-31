"use client";
import { useCallback, useEffect, useRef, useState } from "react";
import { useLocalStorage, useWindowEvent } from "@mantine/hooks";
import {
  browserDisplay,
  displayPart,
  displayKeys,
  explicitDisplayCookie,
  persistDisplayCookie,
  projectedDisplayCookie,
  resolveGuestDisplay,
  sameDisplayValue,
} from "./display.js";
import type { DisplayChoices, DisplayKey } from "./display.js";
import type { DisplayController, DisplayInitial } from "./display-context.js";
import type { IdentityState } from "./auth-context.js";

interface DismissedValue {
  value: string;
  dismissed: boolean;
}
type RejectionMemory = Record<
  string,
  Partial<Record<DisplayKey, DismissedValue>>
>;
function rejectionMemory(raw: string | undefined): RejectionMemory {
  if (!raw || raw.length > 1_000_000) return {};
  try {
    const input: unknown = JSON.parse(raw);
    if (!input || typeof input !== "object") return {};
    const result: RejectionMemory = {};
    for (const [id, fields] of Object.entries(input)) {
      if (
        !/^[A-Za-z0-9_-]{1,128}$/.test(id) ||
        id === "__proto__" ||
        !fields ||
        typeof fields !== "object"
      )
        continue;
      const values: Partial<Record<DisplayKey, DismissedValue>> = {};
      for (const key of displayKeys) {
        const entry: unknown = Reflect.get(fields, key);
        if (!entry || typeof entry !== "object") continue;
        const value: unknown = Reflect.get(entry, "value");
        const dismissed: unknown = Reflect.get(entry, "dismissed");
        if (displayPart(key, value) && typeof dismissed === "boolean")
          values[key] = { value, dismissed };
      }
      result[id] = values;
    }
    return result;
  } catch {
    return {};
  }
}
export function useDisplayController(input: {
  identity: IdentityState;
  initial: DisplayInitial;
  save: (patch: DisplayChoices, userId: string) => Promise<void>;
}): DisplayController {
  const [choices, setChoices] = useState(input.initial.explicit);
  const [guestValue, setGuestValue] = useState(input.initial.value);
  const [detected, setDetected] = useState<DisplayChoices>({});
  const [opened, setOpened] = useState(false);
  const [attempt, setAttempt] = useState<{
    ownerId: string;
    key: DisplayKey;
    value: string;
  } | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState(false);
  const [storageError, setStorageError] = useState(false);
  const [changed, setChanged] = useState(false);
  const [memory, setMemory] = useLocalStorage<RejectionMemory>({
    key: "chapchoo-display-dismissed",
    defaultValue: {},
    deserialize: rejectionMemory,
  });
  const authenticated =
    input.identity.type === "AUTHENTICATED" ? input.identity.user : null;
  const userId = authenticated?.id ?? null;
  const activeUser = useRef(userId);
  const inFlight = useRef(false);
  activeUser.current = userId;
  const mounted = useRef(true);
  useEffect(() => {
    mounted.current = true;
    return (): void => {
      mounted.current = false;
    };
  }, []);
  const account = authenticated?.display;
  const value = account ?? guestValue;
  const available = input.identity.type !== "ERROR";
  const guest = input.identity.type === "SIGNED_OUT";
  const detect = useCallback((): void => {
    const current = browserDisplay();
    setDetected(current);
    persistDisplayCookie(projectedDisplayCookie, current);
    if (guest)
      setGuestValue(
        resolveGuestDisplay(input.initial.defaults, choices, current),
      );
    if (userId)
      setMemory((previous) => {
        const accountMemory = { ...previous[userId] };
        let update = false;
        for (const key of displayKeys) {
          const target = current[key];
          if (target && accountMemory[key]?.value !== target) {
            accountMemory[key] = { value: target, dismissed: false };
            update = true;
          }
        }
        return update ? { ...previous, [userId]: accountMemory } : previous;
      });
  }, [guest, userId, choices, input.initial.defaults, setMemory]);
  useEffect(detect, [detect]);
  useWindowEvent("focus", detect);
  const suggestions =
    authenticated && available && !opened
      ? displayKeys.flatMap((key) => {
          const target = detected[key];
          const rejected = memory[userId ?? ""]?.[key];
          return target &&
            !sameDisplayValue(key, value[key], target) &&
            !(rejected?.value === target && rejected.dismissed)
            ? [{ key, current: value[key], target }]
            : [];
        })
      : [];
  async function save(key: DisplayKey, target: string): Promise<void> {
    if (!userId || inFlight.current) return;
    inFlight.current = true;
    setAttempt({ ownerId: userId, key, value: target });
    setBusy(true);
    setError(false);
    setChanged(false);
    try {
      await input.save({ [key]: target }, userId);
      if (mounted.current && activeUser.current === userId) {
        setAttempt(null);
        setChanged(true);
      }
    } catch {
      if (mounted.current && activeUser.current === userId) setError(true);
    } finally {
      if (mounted.current && activeUser.current === userId) {
        setBusy(false);
        inFlight.current = false;
      }
    }
  }
  useEffect(() => {
    setAttempt(null);
    setError(false);
    setBusy(false);
    setChanged(false);
    inFlight.current = false;
  }, [userId]);
  return {
    value,
    fields:
      attempt?.ownerId === userId
        ? { ...value, [attempt.key]: attempt.value }
        : value,
    defaults: input.initial.defaults,
    instant: input.initial.instant,
    zones: input.initial.zones,
    choices,
    guest,
    available,
    busy,
    error,
    storageError,
    changed,
    opened,
    open: () => setOpened(true),
    close: () => setOpened(false),
    change: (key, target) => {
      if (!available || busy) return;
      if (!guest) {
        if (target) void save(key, target);
        return;
      }
      const next = { ...choices };
      if (target === null) delete next[key];
      else next[key] = target;
      setChoices(next);
      setGuestValue(
        resolveGuestDisplay(input.initial.defaults, next, browserDisplay()),
      );
      setStorageError(!persistDisplayCookie(explicitDisplayCookie, next));
      setChanged(true);
    },
    retry: () => {
      if (attempt?.ownerId === userId) void save(attempt.key, attempt.value);
    },
    suggestions,
    dismiss: () => {
      if (!userId) return;
      setMemory((previous) => {
        const accountMemory = { ...previous[userId] };
        for (const suggestion of suggestions)
          accountMemory[suggestion.key] = {
            value: suggestion.target,
            dismissed: true,
          };
        return { ...previous, [userId]: accountMemory };
      });
    },
    creationDisplay: () => ({ ...browserDisplay(), ...choices }),
  };
}
