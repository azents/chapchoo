import { getRequestConfig } from "next-intl/server";
import { cookies, headers } from "next/headers";
import {
  DEFAULT_LOCALE,
  isSupportedLocale,
  LOCALE_COOKIE,
  resolveLocaleFromHeader,
} from "@/shared/lib/locale";
import en from "../../messages/en-US.json";
import fr from "../../messages/fr-FR.json";
import ja from "../../messages/ja-JP.json";
import ko from "../../messages/ko-KR.json";

const messages = { "en-US": en, "ko-KR": ko, "ja-JP": ja, "fr-FR": fr };

export default getRequestConfig(async () => {
  const cookieLocale = (await cookies()).get(LOCALE_COOKIE)?.value;
  const locale =
    cookieLocale && isSupportedLocale(cookieLocale)
      ? cookieLocale
      : (resolveLocaleFromHeader((await headers()).get("accept-language")) ??
        DEFAULT_LOCALE);
  return { locale, messages: messages[locale] };
});
