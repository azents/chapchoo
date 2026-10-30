/** Browser-safe presentation policy. No authentication or Node imports. */
export type ColorMode = "light" | "dark";
export type ColorModePreference = ColorMode | "system";
type MantineMode = ColorMode | "auto";

export interface ColorModeSnapshot {
  preference: ColorModePreference;
  mode: ColorMode;
}

export const preferenceCookie = "chapchoo-color-mode";
export const resolvedCookie = "chapchoo-color-mode-resolved";
const maxAge = 365 * 24 * 60 * 60;

export function parsePreference(value: unknown): ColorModePreference {
  return value === "light" || value === "dark" ? value : "system";
}

export function resolveServerColorMode(
  preference: unknown,
  resolved: unknown,
): ColorModeSnapshot {
  const parsed = parsePreference(preference);
  return {
    preference: parsed,
    mode:
      parsed === "system" ? (resolved === "dark" ? "dark" : "light") : parsed,
  };
}

export function toMantineMode(preference: ColorModePreference): MantineMode {
  return preference === "system" ? "auto" : preference;
}

export function serializeAppearanceCookie(
  name: typeof preferenceCookie | typeof resolvedCookie,
  value: ColorModePreference,
  secure: boolean,
): string {
  return `${name}=${value}; Path=/; Max-Age=${maxAge}; SameSite=Lax${secure ? "; Secure" : ""}`;
}

function browserMode(preference: ColorModePreference): ColorMode {
  return preference === "system"
    ? window.matchMedia("(prefers-color-scheme: dark)").matches
      ? "dark"
      : "light"
    : preference;
}

function persist(
  name: typeof preferenceCookie | typeof resolvedCookie,
  value: ColorModePreference,
): void {
  try {
    document.cookie = serializeAppearanceCookie(
      name,
      value,
      location.protocol === "https:",
    );
  } catch {
    // Storage denial must not prevent an in-page appearance change.
  }
}

export function syncDocumentColorMode(snapshot: ColorModeSnapshot): void {
  const { preference, mode } = snapshot;
  document.documentElement.setAttribute("data-mantine-color-scheme", mode);
  document.documentElement.setAttribute(
    "data-color-mode-preference",
    preference,
  );
  document.documentElement.style.colorScheme = mode;
  document.body?.setAttribute("data-color-mode", mode);
  persist(resolvedCookie, mode);
}

/** Injected into Mantine instead of its default localStorage manager. */
export function createColorSchemeManager(initial: ColorModePreference): {
  get: (fallback: MantineMode) => MantineMode;
  set: (value: MantineMode) => void;
  clear: () => void;
  subscribe: (callback: (value: MantineMode) => void) => void;
  unsubscribe: () => void;
} {
  let preference = initial;
  function set(value: MantineMode): void {
    preference = value === "auto" ? "system" : value;
    syncDocumentColorMode({ preference, mode: browserMode(preference) });
    persist(preferenceCookie, preference);
  }
  return {
    get: () => toMantineMode(preference),
    set,
    clear: () => set("auto"),
    subscribe: () => {},
    unsubscribe: () => {},
  };
}

/**
 * Only validated enum values enter script text. Runs before visible content;
 * a cached System resolution never overrides current browser media.
 */
export function colorModeBootstrap(snapshot: ColorModeSnapshot): string {
  const safe = resolveServerColorMode(snapshot.preference, snapshot.mode);
  return `(()=>{const p=${JSON.stringify(safe.preference)};const m=p==="system"?(window.matchMedia("(prefers-color-scheme: dark)").matches?"dark":"light"):p;const h=document.documentElement;h.setAttribute("data-mantine-color-scheme",m);h.setAttribute("data-color-mode-preference",p);h.style.colorScheme=m;try{document.cookie=${JSON.stringify(`${resolvedCookie}=`)}+m+${JSON.stringify(`; Path=/; Max-Age=${maxAge}; SameSite=Lax`)}+(location.protocol==="https:"?"; Secure":"")}catch{}})();`;
}

export const bodyColorModeBootstrap =
  'document.body.setAttribute("data-color-mode",document.documentElement.getAttribute("data-mantine-color-scheme"));';
