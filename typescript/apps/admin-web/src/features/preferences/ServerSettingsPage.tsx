"use client";
import { createReactContainer } from "@chapchoo/ui/createReactContainer";
import { DisplaySettingsPage } from "@chapchoo/ui/DisplaySettings";
import { useAuth } from "@chapchoo/web-core/auth-context";
import { useDisplay } from "@chapchoo/web-core/display-context";
import { displayText } from "@chapchoo/web-core/display-messages";
import { useTranslations } from "next-intl";
import { useState } from "react";
import { useAppearanceControl } from "@/shared/appearance/useAppearanceControl";
import { trpc } from "@/shared/rpc/client";
import type { DisplaySettingsPageProps } from "@chapchoo/ui/DisplaySettings";
import type { DisplayChoices, DisplayValue } from "@chapchoo/web-core/display";

function useServerSettings({
  basePath,
  initial,
}: {
  basePath: string;
  initial: DisplayValue;
}): DisplaySettingsPageProps {
  const controller = useDisplay();
  const authText = useTranslations("AdminAuth");
  const auth = useAuth();
  const text = displayText(controller.value.language);
  const appearance = useAppearanceControl();
  const utils = trpc.useUtils();
  const query = trpc.displayDefaults.read.useQuery(void 0, {
    initialData: initial,
    refetchOnMount: false,
    refetchOnReconnect: false,
  });
  const mutation = trpc.displayDefaults.update.useMutation();
  const [attempt, setAttempt] = useState<DisplayChoices | null>(null);
  const [changed, setChanged] = useState(false);
  async function save(patch: DisplayChoices): Promise<void> {
    if (mutation.isPending) {
      return;
    }
    setAttempt(patch);
    setChanged(false);
    try {
      const value = await mutation.mutateAsync(patch);
      utils.displayDefaults.read.setData(void 0, value);
      setAttempt(null);
      setChanged(true);
    } catch {
      /* The mutation exposes the retained selection and localized error. */
    }
  }
  return {
    basePath,
    mascotReplayLabel: authText("replayMascot"),
    contextLabel: "ADMINISTRATION",
    title: text.server,
    heading: text.defaultTitle,
    description: text.defaultScope,
    identityLabel:
      auth.state.type === "AUTHENTICATED" ? auth.state.user.email : "",
    appearance,
    navigation: [
      { key: "home", label: text.home, href: `${basePath}/` },
      { key: "profile", label: text.profile, href: `${basePath}/profile` },
      { key: "settings", label: text.server, active: true },
    ],
    logoutLabel: text.logout,
    onLogout: auth.logout,
    value: { ...query.data, ...attempt },
    selected: {},
    automatic: false,
    disabled: mutation.isPending || auth.state.type !== "AUTHENTICATED",
    zones: controller.zones,
    copy: { ...text, scope: text.defaultScope },
    onChange: (key, value) => {
      if (value) {
        void save({ [key]: value });
      }
    },
    instant: controller.instant,
    changed,
    error: mutation.isError || query.isError,
    storageError: false,
    onRetry: () => {
      if (attempt) {
        void save(attempt);
      } else {
        void utils.displayDefaults.read.invalidate();
      }
    },
  };
}
export const ServerSettingsPage = createReactContainer(
  "ServerSettingsPage",
  useServerSettings,
  DisplaySettingsPage,
);
