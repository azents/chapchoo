"use client";

import { createReactContainer } from "@chapchoo/ui/createReactContainer";
import { PageFailure } from "@chapchoo/ui/PageFailure";
import { useAuth } from "@chapchoo/web-core/auth-context";
import { useTranslations } from "next-intl";
import { useAppearanceControl } from "@/shared/appearance/useAppearanceControl";
import type { PageFailureProps } from "@chapchoo/ui/PageFailure";

function usePageError({ reset }: { reset: () => void }): PageFailureProps {
  const errors = useTranslations("Errors");
  const text = useTranslations("AdminGrants");
  const auth = useAuth();
  return {
    title: text("context"),
    message: errors("generic"),
    retryLabel: text("retry"),
    onRetry: reset,
    appearance: useAppearanceControl(),
    ...(auth.state.type === "ERROR"
      ? { logout: { label: text("logout"), onLogout: auth.logout } }
      : {}),
    ...(auth.state.type === "AUTHENTICATED"
      ? {
          identityLabel: auth.state.user.email,
          logout: { label: text("logout"), onLogout: auth.logout },
        }
      : {}),
  };
}
export const PageError = createReactContainer(
  "PageError",
  usePageError,
  PageFailure,
);
