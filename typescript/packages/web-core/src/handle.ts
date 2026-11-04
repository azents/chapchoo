import anyAscii from "any-ascii";

export const HANDLE_MIN_LENGTH = 3;
export const HANDLE_MAX_LENGTH = 16;
const SUFFIX_LENGTH = 5;
const AUTOMATIC_CHECK_LIMIT = 8;
const ALPHABET = "abcdefghijklmnopqrstuvwxyz0123456789";

export type HandleKind = "agent" | "team";

/** Syntax only. Availability never grants access or reserves a handle. */
export function validHandle(value: string): boolean {
  return (
    value.length >= HANDLE_MIN_LENGTH &&
    value.length <= HANDLE_MAX_LENGTH &&
    /^[a-z0-9][a-z0-9-]*[a-z0-9](?![\s\S])/.test(value)
  );
}

export type HandleDraft =
  | {
      source: "automatic";
      kind: HandleKind;
      base: string;
      candidate: string;
    }
  | { source: "manual"; candidate: string };

/** Injected in tests; normal clients use Web Crypto rather than Math.random. */
export type HandleSuffix = () => string;

export function randomHandleSuffix(): string {
  let suffix = "";
  while (suffix.length < SUFFIX_LENGTH) {
    const bytes = crypto.getRandomValues(new Uint8Array(16));
    for (const byte of bytes) {
      // 252 is the largest multiple of 36 below 256: reject modulo bias.
      if (byte < 252) {
        suffix += ALPHABET.charAt(byte % ALPHABET.length);
        if (suffix.length === SUFFIX_LENGTH) return suffix;
      }
    }
  }
  return suffix;
}

function withSuffix(base: string, suffix: HandleSuffix): string {
  const value = suffix();
  if (!/^[a-z0-9]{5}(?![\s\S])/.test(value)) {
    throw new Error(
      "Handle suffix must contain five lowercase letters or digits",
    );
  }
  const prefix = base
    .slice(0, HANDLE_MAX_LENGTH - SUFFIX_LENGTH - 1)
    .replace(/-+$/, "");
  return `${prefix}-${value}`;
}

/** Client proposal only; the server accepts and validates the final value. */
export function proposeHandle(
  kind: HandleKind,
  displayName: string,
  suffix: HandleSuffix = randomHandleSuffix,
): HandleDraft {
  const converted = anyAscii(displayName)
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");
  const base = converted || kind;
  const truncated = base.slice(0, HANDLE_MAX_LENGTH).replace(/-+$/, "");
  return {
    source: "automatic",
    kind,
    base,
    candidate:
      !converted || truncated.length < HANDLE_MIN_LENGTH
        ? withSuffix(base, suffix)
        : truncated,
  };
}

export function editHandle(candidate: string): HandleDraft {
  // Manual input is neither normalized nor replaced, including an empty draft.
  return { source: "manual", candidate };
}

export function renameHandleDraft(
  draft: HandleDraft,
  displayName: string,
  suffix: HandleSuffix = randomHandleSuffix,
): HandleDraft {
  return draft.source === "manual"
    ? draft
    : proposeHandle(draft.kind, displayName, suffix);
}

export type HandleResolution =
  | { status: "available"; draft: HandleDraft }
  | { status: "invalid"; draft: HandleDraft }
  | { status: "occupied"; draft: HandleDraft }
  | { status: "exhausted"; draft: HandleDraft }
  | { status: "unavailable"; draft: HandleDraft }
  | { status: "cancelled"; draft: HandleDraft };

export interface HandleAvailability {
  check: (candidate: string, signal: AbortSignal) => Promise<boolean>;
  signal: AbortSignal;
  suffix?: HandleSuffix;
}

/**
 * Resolve one draft, never a live form. The caller must cancel an old resolution
 * when its draft or Team context changes, and apply only the current result.
 * The result is advisory; it cannot unlock or rewrite an uncertain create payload.
 */
export async function resolveHandle(
  initial: HandleDraft,
  availability: HandleAvailability,
): Promise<HandleResolution> {
  let draft = initial;
  for (let attempt = 0; attempt < AUTOMATIC_CHECK_LIMIT; attempt += 1) {
    if (availability.signal.aborted) return { status: "cancelled", draft };
    if (!validHandle(draft.candidate)) return { status: "invalid", draft };
    try {
      const available = await availability.check(
        draft.candidate,
        availability.signal,
      );
      if (availability.signal.aborted) return { status: "cancelled", draft };
      if (available) return { status: "available", draft };
    } catch {
      return {
        status: availability.signal.aborted ? "cancelled" : "unavailable",
        draft,
      };
    }
    if (draft.source === "manual") return { status: "occupied", draft };
    if (attempt + 1 === AUTOMATIC_CHECK_LIMIT) {
      return { status: "exhausted", draft };
    }
    draft = {
      ...draft,
      candidate: withSuffix(
        draft.base,
        availability.suffix ?? randomHandleSuffix,
      ),
    };
  }
  return { status: "exhausted", draft };
}
