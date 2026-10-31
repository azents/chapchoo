import { formatDisplayDate } from "@chapchoo/ui/displayFormatting";
import { useAuth } from "@chapchoo/web-core/auth-context";
import { useDisplay } from "@chapchoo/web-core/display-context";
import { displayText } from "@chapchoo/web-core/display-messages";
import {
  initialQuery,
  inventoryOffset,
  navigateSelection,
  readSelection,
} from "@chapchoo/web-core/page-state";
import { useClipboard } from "@mantine/hooks";
import { useTranslations } from "next-intl";
import { useSearchParams } from "next/navigation";
import { useState } from "react";
import { useAppearanceControl } from "@/shared/appearance/useAppearanceControl";
import { errorKey } from "@/shared/lib/errors";
import { trpc } from "@/shared/rpc/client";
import { currentRevokeId } from "../revokeSelection";
import type {
  AdminGrantsProps,
  GrantPageKind,
  GrantRow,
  GrantViewState,
} from "../components/AdminGrants";
import type { InitialInventory } from "../initialInventory";
import type { RevokeSelection } from "../revokeSelection";
import type { ConfirmationState } from "@chapchoo/ui/ConfirmationDialog";
import type { Grant, IssuedGrant } from "@chapchoo/web-core/browser";

export interface AdminGrantsInput {
  kind: GrantPageKind;
  basePath: string;
  initial: InitialInventory;
}

function project(
  grant: Grant,
  locale: string,
  timeZone: string,
  now: number,
  unavailable: string,
): GrantRow {
  if (!grant.issuedAt || !grant.expiresAt) {
    throw new Error("Invalid grant projection");
  }
  const issued = Number(grant.issuedAt.seconds) * 1000;
  const expires = Number(grant.expiresAt.seconds) * 1000;
  return {
    id: grant.id,
    email: grant.email,
    issuedAt:
      formatDisplayDate(issued, locale, timeZone, "medium") ?? unavailable,
    expiresAt:
      formatDisplayDate(expires, locale, timeZone, "medium") ?? unavailable,
    status: grant.consumedAt
      ? "used"
      : grant.revokedAt
        ? "revoked"
        : expires <= now
          ? "expired"
          : "active",
  };
}

export function useAdminGrantsContainer({
  kind,
  basePath,
  initial,
}: AdminGrantsInput): AdminGrantsProps {
  const text = useTranslations("AdminGrants");
  const appearance = useAppearanceControl();
  const authText = useTranslations("AdminAuth");
  const errors = useTranslations("Errors");
  const { value: display } = useDisplay();
  const auth = useAuth();
  const utils = trpc.useUtils();
  const clipboard = useClipboard();
  const [email, setEmail] = useState("");
  const params = useSearchParams();
  const offset = readSelection(params, "offset", inventoryOffset);
  const seed =
    offset.toString() === initial.offset
      ? initialQuery(initial.page, initial.renderedAt)
      : {};
  const [issued, setIssued] = useState<{ url: string; email: string } | null>(
    null,
  );
  const [revokeSelection, setRevokeSelection] =
    useState<RevokeSelection | null>(null);
  const revokeId = currentRevokeId(revokeSelection, offset);
  const signup = trpc.signupGrants.list.useQuery(
    { offset: offset.toString(), limit: 25 },
    {
      ...(kind === "signup" ? seed : {}),
      enabled: kind === "signup" && auth.state.type === "AUTHENTICATED",
    },
  );
  const reset = trpc.resetGrants.list.useQuery(
    { offset: offset.toString(), limit: 25 },
    {
      ...(kind === "reset" ? seed : {}),
      enabled: kind === "reset" && auth.state.type === "AUTHENTICATED",
    },
  );
  async function refreshBusiness(): Promise<void> {
    await Promise.all([
      utils.signupGrants.list.invalidate(),
      utils.resetGrants.list.invalidate(),
    ]);
  }

  function accepted(value: IssuedGrant): void {
    if (!value.grant) {
      throw new Error("Invalid issued grant response");
    }
    setIssued({ url: value.url, email: value.grant.email });
    setEmail("");
    void refreshBusiness();
  }
  function revoked(): void {
    setRevokeSelection(null);
    void refreshBusiness();
  }
  const issueSignup = trpc.signupGrants.issue.useMutation({
    onSuccess: accepted,
  });
  const issueReset = trpc.resetGrants.issue.useMutation({
    onSuccess: accepted,
  });
  const revokeSignup = trpc.signupGrants.revoke.useMutation({
    onSuccess: revoked,
  });
  const revokeReset = trpc.resetGrants.revoke.useMutation({
    onSuccess: revoked,
  });
  const pending =
    issueSignup.isPending ||
    issueReset.isPending ||
    revokeSignup.isPending ||
    revokeReset.isPending;
  const issueError = issueSignup.error ?? issueReset.error;
  const revokeError = revokeSignup.error ?? revokeReset.error;
  const query = kind === "signup" ? signup : reset;
  let state: GrantViewState;
  if (auth.state.type === "ERROR") {
    state = {
      type: "ERROR",
      message: errors(errorKey(auth.state.reason)),
    };
  } else if (auth.state.type === "SIGNED_OUT") {
    state = { type: "LOADING" };
  } else {
    switch (query.status) {
      case "pending":
        state = { type: "LOADING" };
        break;
      case "error":
        state = {
          type: "ERROR",
          message: errors(errorKey(query.error.data?.reason ?? null)),
        };
        break;
      case "success":
        state = {
          type: "LOADED",
          items: query.data.items.map((grant) =>
            project(
              grant,
              display.region,
              display.timeZone,
              query.dataUpdatedAt,
              displayText(display.language).unavailable,
            ),
          ),
          range: text("range", {
            from: query.data.items.length ? (offset + 1n).toString() : "0",
            to: (offset + BigInt(query.data.items.length)).toString(),
            total: query.data.total.toString(),
          }),
          hasPrevious: offset > 0n,
          hasNext: offset + 25n < query.data.total,
        };
        break;
    }
  }
  const revokeState: ConfirmationState =
    revokeSignup.isPending || revokeReset.isPending
      ? { type: "PENDING" }
      : revokeError
        ? {
            type: "ERROR",
            message: errors(errorKey(revokeError.data?.reason ?? null)),
          }
        : { type: "READY" };

  function issue(): void {
    if (pending || auth.state.type !== "AUTHENTICATED") {
      return;
    }
    if (kind === "signup") {
      issueSignup.mutate({ email });
    } else {
      issueReset.mutate({ email });
    }
  }
  function revoke(): void {
    if (pending || auth.state.type !== "AUTHENTICATED" || revokeId === null) {
      return;
    }
    if (kind === "signup") {
      revokeSignup.mutate({ grantId: revokeId });
    } else {
      revokeReset.mutate({ grantId: revokeId });
    }
  }
  return {
    kind,
    appearance,
    basePath,
    identityLabel:
      auth.state.type === "AUTHENTICATED" ? auth.state.user.email : null,
    signInLabel: authText("login"),
    state,
    email,
    pending,
    error: issueError
      ? errors(errorKey(issueError.data?.reason ?? null))
      : null,
    issued,
    copied: clipboard.copied,
    revokeId,
    revokeState,
    text,
    onEmail: setEmail,
    onIssue: issue,
    onCopy: () => {
      if (issued) {
        clipboard.copy(issued.url);
      }
    },
    onDismiss: () => {
      setIssued(null);
      clipboard.reset();
    },
    onRevoke: (id) => {
      revokeSignup.reset();
      revokeReset.reset();
      setRevokeSelection({ id, offset });
    },
    onCancelRevoke: () => {
      if (!pending) {
        setRevokeSelection(null);
      }
    },
    onConfirmRevoke: revoke,
    onPrevious: () => {
      navigateSelection(
        "offset",
        offset > 25n ? offset - 25n : 0n,
        inventoryOffset,
        "push",
      );
    },
    onNext: () => {
      navigateSelection("offset", offset + 25n, inventoryOffset, "push");
    },
    onRefresh: () => {
      void auth
        .recheck()
        .then(refreshBusiness)
        .catch(() => {
          // The common authentication query owns recheck failure state.
        });
    },
    onLogout: () => {
      if (!pending) {
        auth.logout();
      }
    },
  };
}
