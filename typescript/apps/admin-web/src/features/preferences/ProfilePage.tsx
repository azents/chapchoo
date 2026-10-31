"use client";
import { createReactContainer } from "@chapchoo/ui/createReactContainer";
import { DisplaySettingsPage } from "@chapchoo/ui/DisplaySettings";
import { useAuth } from "@chapchoo/web-core/auth-context";
import { useDisplay } from "@chapchoo/web-core/display-context";
import { displayText } from "@chapchoo/web-core/display-messages";
import { useAppearanceControl } from "@/shared/appearance/useAppearanceControl";
import type { DisplaySettingsPageProps } from "@chapchoo/ui/DisplaySettings";

function useProfile({
  basePath,
}: {
  basePath: string;
}): DisplaySettingsPageProps {
  const controller = useDisplay();
  const auth = useAuth();
  const appearance = useAppearanceControl();
  const text = displayText(controller.value.language);
  return {
    basePath,
    contextLabel: "ADMINISTRATION",
    title: text.profile,
    heading: text.title,
    identityLabel:
      auth.state.type === "AUTHENTICATED" ? auth.state.user.email : "",
    accountHeading: text.account,
    emailLabel: text.email,
    appearance,
    navigation: [
      { key: "home", label: text.home, href: `${basePath}/` },
      { key: "profile", label: text.profile, active: true },
      { key: "settings", label: text.server, href: `${basePath}/settings` },
    ],
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
export const ProfilePage = createReactContainer(
  "ProfilePage",
  useProfile,
  DisplaySettingsPage,
);
