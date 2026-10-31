import { getRequestConfig } from "next-intl/server";
import { readDisplay } from "@/shared/server/display";
import en from "../../messages/en-US.json";
import fr from "../../messages/fr-FR.json";
import ja from "../../messages/ja-JP.json";
import ko from "../../messages/ko-KR.json";

const messages = { "en-US": en, "ko-KR": ko, "ja-JP": ja, "fr-FR": fr };

export default getRequestConfig(async () => {
  const { value } = await readDisplay();
  const locale =
    value.language === "ko"
      ? "ko-KR"
      : value.language === "ja"
        ? "ja-JP"
        : value.language === "fr"
          ? "fr-FR"
          : "en-US";
  return { locale, messages: messages[locale] };
});
