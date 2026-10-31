export interface DisplayValue {
  language: string;
  timeZone: string;
  region: string;
}
export type DisplayKey = keyof DisplayValue;
export type DisplayChoices = Partial<DisplayValue>;
export function displayPatchPayload(value: DisplayChoices): {
  language: string | undefined;
  timeZone: string | undefined;
  region: string | undefined;
} {
  return {
    language: value.language,
    timeZone: value.timeZone,
    region: value.region,
  };
}
export const displayKeys: readonly DisplayKey[] = [
  "language",
  "timeZone",
  "region",
];
export const languageLocales: Readonly<Record<string, string>> = {
  en: "en-US",
  ko: "ko-KR",
  ja: "ja-JP",
  fr: "fr-FR",
};
export const languageNames: Readonly<Record<string, string>> = {
  en: "English",
  ko: "한국어",
  ja: "日本語",
  fr: "Français",
};
export const explicitDisplayCookie = "chapchoo-display-choice";
export const projectedDisplayCookie = "chapchoo-display-projection";

export function displayPart(key: DisplayKey, value: unknown): value is string {
  if (typeof value !== "string" || !value || value.length > 128) return false;
  if (key === "language") return Object.hasOwn(languageLocales, value);
  if (key === "region") return Object.values(languageLocales).includes(value);
  try {
    new Intl.DateTimeFormat("en-US", { timeZone: value });
    return !/^[+-]/.test(value);
  } catch {
    return false;
  }
}
export function displayChoices(value: unknown): DisplayChoices {
  if (!value || typeof value !== "object") return {};
  const result: DisplayChoices = {};
  for (const key of displayKeys) {
    const candidate: unknown = Reflect.get(value, key);
    if (displayPart(key, candidate)) result[key] = candidate;
  }
  return result;
}
export function completeDisplay(value: unknown): DisplayValue | null {
  const parsed = displayChoices(value);
  return parsed.language && parsed.timeZone && parsed.region
    ? {
        language: parsed.language,
        timeZone: parsed.timeZone,
        region: parsed.region,
      }
    : null;
}
export function decodeDisplayCookie(value: string | undefined): DisplayChoices {
  if (!value || value.length > 2048) return {};
  try {
    return displayChoices(JSON.parse(decodeURIComponent(value)));
  } catch {
    return {};
  }
}
export function resolveGuestDisplay(
  defaults: DisplayValue,
  explicit: DisplayChoices,
  projection: DisplayChoices,
): DisplayValue {
  return { ...defaults, ...projection, ...explicit };
}
export function detectDisplayValues(
  languages: readonly string[],
  resolved: { locale?: string; timeZone?: string },
): DisplayChoices {
  const result: DisplayChoices = {};
  for (const locale of languages) {
    const language = locale.toLowerCase().split("-")[0] ?? "";
    if (!Object.hasOwn(languageLocales, language)) continue;
    result.language = language;
    break;
  }
  const regionLanguage = resolved.locale?.toLowerCase().split("-")[0] ?? "";
  const region = languageLocales[regionLanguage];
  if (region) result.region = region;
  if (displayPart("timeZone", resolved.timeZone))
    result.timeZone = resolved.timeZone;
  return result;
}
export function browserDisplay(): DisplayChoices {
  try {
    return detectDisplayValues(
      navigator.languages,
      new Intl.DateTimeFormat().resolvedOptions(),
    );
  } catch {
    return detectDisplayValues(navigator.languages, {});
  }
}
export function sameDisplayValue(
  key: DisplayKey,
  current: string,
  target: string,
): boolean {
  if (key !== "timeZone") return current === target;
  return (
    new Intl.DateTimeFormat("en-US", { timeZone: current }).resolvedOptions()
      .timeZone ===
    new Intl.DateTimeFormat("en-US", { timeZone: target }).resolvedOptions()
      .timeZone
  );
}
export function browserCookie(name: string): string | undefined {
  return document.cookie
    .split(";")
    .map((value) => value.trim())
    .find((value) => value.startsWith(`${name}=`))
    ?.slice(name.length + 1);
}
export function persistDisplayCookie(
  name: string,
  value: DisplayChoices,
): boolean {
  const encoded = encodeURIComponent(JSON.stringify(value));
  document.cookie = `${name}=${encoded}; Path=/; Max-Age=31536000; SameSite=Lax${location.protocol === "https:" ? "; Secure" : ""}`;
  return browserCookie(name) === encoded;
}
