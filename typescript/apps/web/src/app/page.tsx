import { redirect } from "next/navigation";
import { getPublicConfig } from "@/config";
import { getLoginRedirect } from "@/config";
import { readNavigation } from "@/shared/navigation/server";

export default async function Home(): Promise<never> {
  const navigation = await readNavigation();
  if (navigation.type === "guest") {
    redirect(getLoginRedirect());
  }
  if (navigation.type === "error") {
    throw new Error("Working contexts are unavailable");
  }
  if (!navigation.preferred) {
    throw new Error("Personal context is unavailable");
  }
  redirect(
    `${getPublicConfig().publicBasePath}/agents?team=${encodeURIComponent(navigation.preferred)}`,
  );
}
