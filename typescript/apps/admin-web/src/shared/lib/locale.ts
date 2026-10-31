/** Message catalog identifiers. Display authority lives in the display snapshot. */
export const SUPPORTED_LOCALES = ["en-US", "ko-KR", "ja-JP", "fr-FR"] as const;
export type SupportedLocale = (typeof SUPPORTED_LOCALES)[number];
export function isSupportedLocale(value: string): value is SupportedLocale {
  return SUPPORTED_LOCALES.some((locale) => locale === value);
}
