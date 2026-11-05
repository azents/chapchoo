"use client";
import { createReactContainer } from "@chapchoo/ui/createReactContainer";
import { ProfileAccount } from "@chapchoo/ui/ProfileAccount";
import { useAuth } from "@chapchoo/web-core/auth-context";
import { computerCopy } from "@chapchoo/web-core/computer-messages";
import { useDisplay } from "@chapchoo/web-core/display-context";
import { displayText } from "@chapchoo/web-core/display-messages";
import { shellCopy } from "@chapchoo/web-core/shell-messages";
import { useAppearanceControl } from "@/shared/appearance/useAppearanceControl";
import type { ProfileAccountProps } from "@chapchoo/ui/ProfileAccount";

function useProfile({ basePath }: { basePath: string }): ProfileAccountProps {
  const { value } = useDisplay();
  const auth = useAuth();
  const text = displayText(value.language);
  const shell = shellCopy(value.language);
  const appearance = useAppearanceControl();
  return {
    basePath,
    appearance,
    navigation: [],
    contextLabel: shell.profile,
    title: shell.accountSettings,
    emailLabel: text.email,
    identityLabel:
      auth.state.type === "AUTHENTICATED" ? auth.state.user.email : "",
    mascotReplayLabel: computerCopy(value.language).replay,
    logoutLabel: text.logout,
    onLogout: auth.logout,
  };
}
export const ProfilePage = createReactContainer(
  "ProfilePage",
  useProfile,
  ProfileAccount,
);
