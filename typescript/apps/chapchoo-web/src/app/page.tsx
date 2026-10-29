import { getTranslations } from "next-intl/server";
import { HelloWorldPage } from "@/features/home/HelloWorldPage";
import type { ReactElement } from "react";

export default async function Page(): Promise<ReactElement> {
  const t = await getTranslations("home");
  return <HelloWorldPage heading={t("heading")} />;
}
