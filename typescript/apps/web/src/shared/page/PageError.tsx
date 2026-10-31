"use client";

import { createReactContainer } from "@chapchoo/ui/createReactContainer";
import { PageFailure } from "@chapchoo/ui/PageFailure";
import { useAuth } from "@chapchoo/web-core/auth-context";
import { useMessages } from "next-intl";
import { useAppearanceControl } from "@/shared/appearance/useAppearanceControl";
import type { PageFailureProps } from "@chapchoo/ui/PageFailure";

function usePageError({ reset }: { reset: () => void }): PageFailureProps {
  const copy = useMessages().accounts;
  const auth = useAuth();
  return {
    title: copy.errorTitle,
    message: copy.genericError,
    retryLabel: copy.retry,
    onRetry: reset,
    appearance: useAppearanceControl(),
    ...(auth.state.type === "ERROR"
      ? { logout: { label: copy.logout, onLogout: auth.logout } }
      : {}),
    ...(auth.state.type === "AUTHENTICATED"
      ? {
          identityLabel: auth.state.user.email,
          logout: { label: copy.logout, onLogout: auth.logout },
        }
      : {}),
  };
}
export const PageError = createReactContainer(
  "PageError",
  usePageError,
  PageFailure,
);
