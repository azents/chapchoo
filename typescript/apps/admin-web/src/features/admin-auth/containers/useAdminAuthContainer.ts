import { useAuth } from "@chapchoo/web-core/auth-context";
import { useTranslations } from "next-intl";
import { useState } from "react";
import { useAppearanceControl } from "@/shared/appearance/useAppearanceControl";
import { errorKey } from "@/shared/lib/errors";
import { trpc } from "@/shared/rpc/client";
import type {
  AdminAuthProps,
  AuthMethod,
  AuthState,
} from "../components/AdminAuth";
import type { InitialRead, LoginMethods } from "@chapchoo/web-core/browser";

export type AdminAuthInput = { basePath: string } & (
  | { mode: "login"; initial: InitialRead<LoginMethods> }
  | { mode: "setup"; initial: InitialRead<{ available: boolean }> }
);

export function useAdminAuthContainer({
  mode,
  basePath,
  initial,
}: AdminAuthInput): AdminAuthProps {
  const text = useTranslations("AdminAuth");
  const appearance = useAppearanceControl();
  const errors = useTranslations("Errors");
  const auth = useAuth();
  const [method, setMethod] = useState<AuthMethod>("password");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [installationProof, setProof] = useState("");
  const [code, setCode] = useState("");
  const [binding, setBinding] = useState<string | null>(null);
  const methods = trpc.auth.methods.useQuery(void 0, {
    enabled: mode === "login" && initial.type === "LOADED",
    ...(mode === "login" && initial.type === "LOADED"
      ? { initialData: initial.data, refetchOnMount: false }
      : {}),
  });
  const setup = trpc.setup.status.useQuery(void 0, {
    enabled: mode === "setup" && initial.type === "LOADED",
    ...(mode === "setup" && initial.type === "LOADED"
      ? { initialData: initial.data, refetchOnMount: false }
      : {}),
  });

  function authenticated(): void {
    auth.authenticated();
  }

  const login = trpc.auth.passwordLogin.useMutation({
    onSuccess: authenticated,
  });
  const initialize = trpc.setup.initialize.useMutation({
    onSuccess: authenticated,
  });
  const verify = trpc.auth.verifyCode.useMutation({ onSuccess: authenticated });
  const send = trpc.auth.sendCode.useMutation({
    onSuccess: (challenge) => {
      setBinding(challenge.binding);
    },
  });
  const pending =
    login.isPending ||
    initialize.isPending ||
    verify.isPending ||
    send.isPending;
  const mutationError =
    login.error ?? initialize.error ?? verify.error ?? send.error;
  let state: AuthState;
  if (initial.type === "ERROR") {
    state = {
      type: "UNAVAILABLE",
      message: errors(errorKey(initial.reason)),
    };
  } else if (mode === "setup") {
    switch (setup.status) {
      case "pending":
        state = { type: "LOADING" };
        break;
      case "error":
        state = {
          type: "UNAVAILABLE",
          message: errors(errorKey(setup.error.data?.reason ?? null)),
        };
        break;
      case "success":
        state = setup.data.available
          ? {
              type: "FORM",
              pending,
              error: mutationError
                ? errors(errorKey(mutationError.data?.reason ?? null))
                : null,
            }
          : { type: "CLOSED" };
        break;
    }
  } else {
    switch (methods.status) {
      case "pending":
        state = { type: "LOADING" };
        break;
      case "error":
        state = {
          type: "UNAVAILABLE",
          message: errors(errorKey(methods.error.data?.reason ?? null)),
        };
        break;
      case "success":
        state = {
          type: "FORM",
          pending,
          error: mutationError
            ? errors(errorKey(mutationError.data?.reason ?? null))
            : null,
        };
        break;
    }
  }

  function submit(): void {
    if (pending) {
      return;
    }
    if (mode === "setup") {
      initialize.mutate({ installationProof, email, password });
    } else if (method === "password") {
      login.mutate({ email, password });
    } else if (binding) {
      verify.mutate({ email, binding, code });
    } else {
      send.mutate({ email });
    }
  }

  function changeMethod(value: AuthMethod): void {
    setMethod(value);
    setBinding(null);
    setCode("");
    login.reset();
    initialize.reset();
    verify.reset();
    send.reset();
  }

  return {
    mode,
    appearance,
    method,
    state,
    basePath,
    email,
    password,
    installationProof,
    code,
    hasBinding: binding !== null,
    emailCodeAvailable: methods.isSuccess && methods.data.emailCode,
    text,
    onEmail: setEmail,
    onPassword: setPassword,
    onProof: setProof,
    onCode: setCode,
    onMethod: changeMethod,
    onChangeEmail: () => {
      setBinding(null);
      setCode("");
    },
    onSubmit: submit,
  };
}
