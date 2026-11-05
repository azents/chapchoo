"use client";
import { createReactContainer } from "@chapchoo/ui/createReactContainer";
import { DisplaySettingsPage } from "@chapchoo/ui/DisplaySettings";
import { useAuth } from "@chapchoo/web-core/auth-context";
import { computerCopy } from "@chapchoo/web-core/computer-messages";
import { useDisplay } from "@chapchoo/web-core/display-context";
import { displayText } from "@chapchoo/web-core/display-messages";
import { shellCopy } from "@chapchoo/web-core/shell-messages";
import { useAppearanceControl } from "@/shared/appearance/useAppearanceControl";
import type { DisplaySettingsPageProps } from "@chapchoo/ui/DisplaySettings";

function usePreferences({
  basePath,
}: {
  basePath: string;
}): DisplaySettingsPageProps {
  const controller = useDisplay();
  const auth = useAuth();
  const appearance = useAppearanceControl();
  const text = displayText(controller.value.language);
  const shell = shellCopy(controller.value.language);
  return {
    basePath,
    appearance,
    displayAppearance: appearance,
    mascotReplayLabel: computerCopy(controller.value.language).replay,
    contextLabel: shell.profile,
    title: shell.displaySettings,
    heading: text.title,
    identityLabel:
      auth.state.type === "AUTHENTICATED" ? auth.state.user.email : "",
    navigation: [],
    logoutLabel: text.logout,
    onLogout: auth.logout,
    value: controller.fields,
    selected: {},
    automatic: false,
    disabled: controller.busy || !controller.available,
    zones: controller.zones,
    copy: { ...text, scope: text.accountScope },
    onChange: controller.change,
    instant: controller.instant,
    changed: controller.changed,
    error: controller.error,
    storageError: false,
    onRetry: controller.retry,
  };
}
export const PreferencesPage = createReactContainer(
  "PreferencesPage",
  usePreferences,
  DisplaySettingsPage,
);
