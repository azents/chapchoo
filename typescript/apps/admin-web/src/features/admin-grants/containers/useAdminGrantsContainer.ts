import { useClipboard } from "@mantine/hooks";
import { useQueryClient } from "@tanstack/react-query";
import { useLocale, useTranslations } from "next-intl";
import { useState } from "react";
import { useAppearanceControl } from "@/shared/appearance/useAppearanceControl";
import { errorKey } from "@/shared/lib/errors";
import { trpc } from "@/shared/rpc/client";
import { refreshAdminState } from "../refreshAdminState";
import type {
  AdminGrantsProps,
  GrantPageKind,
  GrantRow,
  GrantViewState,
} from "../components/AdminGrants";
import type { InitialAdminIdentity } from "../types";
import type { ConfirmationState } from "@chapchoo/ui/ConfirmationDialog";
import type { Grant, IssuedGrant } from "@chapchoo/web-core/browser";

export interface AdminGrantsInput {
  kind: GrantPageKind;
  basePath: string;
  initial: InitialAdminIdentity;
}

function project(grant: Grant, locale: string): GrantRow {
  if (!grant.issuedAt || !grant.expiresAt) {
    throw new Error("Invalid grant projection");
  }
  const issued = Number(grant.issuedAt.seconds) * 1000;
  const expires = Number(grant.expiresAt.seconds) * 1000;
  const formatter = new Intl.DateTimeFormat(locale, {
    dateStyle: "medium",
    timeStyle: "short",
  });
  return {
    id: grant.id,
    email: grant.email,
    issuedAt: formatter.format(issued),
    expiresAt: formatter.format(expires),
    status: grant.consumedAt
      ? "used"
      : grant.revokedAt
        ? "revoked"
        : expires <= Date.now()
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
  const locale = useLocale();
  const cache = useQueryClient();
  const utils = trpc.useUtils();
  const clipboard = useClipboard();
  const [email, setEmail] = useState("");
  const [offset, setOffset] = useState(0n);
  const [issued, setIssued] = useState<{ url: string; email: string } | null>(
    null,
  );
  const [revokeId, setRevokeId] = useState<string | null>(null);
  const signup = trpc.signupGrants.list.useQuery(
    { offset: offset.toString(), limit: 25 },
    { enabled: kind === "signup" && initial.type === "LOADED" },
  );
  const reset = trpc.resetGrants.list.useQuery(
    { offset: offset.toString(), limit: 25 },
    { enabled: kind === "reset" && initial.type === "LOADED" },
  );
  const identity = trpc.auth.identity.useQuery(void 0, {
    enabled: initial.type === "LOADED",
    ...(initial.type === "LOADED"
      ? { initialData: initial.user, refetchOnMount: false }
      : {}),
  });

  async function refresh(): Promise<void> {
    if (initial.type === "ERROR") {
      window.location.reload();
      return;
    }
    await refreshAdminState(utils);
  }

  function accepted(value: IssuedGrant): void {
    if (!value.grant) {
      throw new Error("Invalid issued grant response");
    }
    setIssued({ url: value.url, email: value.grant.email });
    setEmail("");
    void refresh();
  }
  function revoked(): void {
    setRevokeId(null);
    void refresh();
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
  const logout = trpc.auth.logout.useMutation({
    onSuccess: () => {
      cache.clear();
      window.location.replace(`${basePath}/login`);
    },
  });
  const pending =
    issueSignup.isPending ||
    issueReset.isPending ||
    revokeSignup.isPending ||
    revokeReset.isPending ||
    logout.isPending;
  const issueError = issueSignup.error ?? issueReset.error ?? logout.error;
  const revokeError = revokeSignup.error ?? revokeReset.error;
  const query = kind === "signup" ? signup : reset;
  let state: GrantViewState;
  if (initial.type === "ERROR") {
    state = { type: "ERROR", message: errors(errorKey(initial.reason)) };
  } else if (identity.isError) {
    state = {
      type: "ERROR",
      message: errors(errorKey(identity.error.data?.reason ?? null)),
    };
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
          items: query.data.items.map((grant) => project(grant, locale)),
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
    if (pending || initial.type !== "LOADED") {
      return;
    }
    if (kind === "signup") {
      issueSignup.mutate({ email });
    } else {
      issueReset.mutate({ email });
    }
  }
  function revoke(): void {
    if (pending || initial.type !== "LOADED" || revokeId === null) {
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
    identityLabel: identity.isSuccess
      ? identity.data.email
      : initial.type === "LOADED"
        ? initial.user.email
        : null,
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
      setRevokeId(id);
    },
    onCancelRevoke: () => {
      if (!pending) {
        setRevokeId(null);
      }
    },
    onConfirmRevoke: revoke,
    onPrevious: () => {
      setOffset((value) => (value > 25n ? value - 25n : 0n));
    },
    onNext: () => {
      setOffset((value) => value + 25n);
    },
    onRefresh: () => {
      void refresh();
    },
    onLogout: () => {
      if (!pending) {
        logout.mutate();
      }
    },
  };
}
