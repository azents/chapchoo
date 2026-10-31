import { useAuth } from "@chapchoo/web-core/auth-context";
import {
  loginMethod,
  navigateSelection,
  readSelection,
} from "@chapchoo/web-core/page-state";
import { useTranslations } from "next-intl";
import { useSearchParams } from "next/navigation";
import { useState } from "react";
import { useAppearanceControl } from "@/shared/appearance/useAppearanceControl";
import { errorKey } from "@/shared/lib/errors";
import { trpc } from "@/shared/rpc/client";
import type {
  AdminAuthProps,
  AuthMethod,
  AuthState,
} from "../components/AdminAuth";
import type { LoginMethods } from "@chapchoo/web-core/browser";

export type AdminAuthInput = { basePath: string } & (
  | { mode: "login"; initial: LoginMethods }
  | { mode: "setup"; initial: { available: boolean } }
);

export function useAdminAuthContainer(input: AdminAuthInput): AdminAuthProps {
  const { mode, basePath } = input;
  const text = useTranslations("AdminAuth");
  const appearance = useAppearanceControl();
  const errors = useTranslations("Errors");
  const auth = useAuth();
  const params = useSearchParams();
  const emailCodeAvailable = input.mode === "login" && input.initial.emailCode;
  const method = emailCodeAvailable
    ? readSelection(params, "method", loginMethod)
    : "password";
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [installationProof, setProof] = useState("");
  const [code, setCode] = useState("");
  const [binding, setBinding] = useState<{
    email: string;
    value: string;
  } | null>(null);
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
    onSuccess: (challenge) => setBinding({ email, value: challenge.binding }),
  });
  const pending =
    login.isPending ||
    initialize.isPending ||
    verify.isPending ||
    send.isPending;
  const mutationError =
    login.error ?? initialize.error ?? verify.error ?? send.error;
  const state: AuthState =
    input.mode === "setup" && !input.initial.available
      ? { type: "CLOSED" }
      : {
          type: "FORM",
          pending,
          error: mutationError
            ? errors(errorKey(mutationError.data?.reason ?? null))
            : null,
        };
  function submit(): void {
    if (pending) {
      return;
    }
    if (mode === "setup") {
      initialize.mutate({ installationProof, email, password });
    } else if (method === "password") {
      login.mutate({ email, password });
    } else if (binding && binding.email === email) {
      verify.mutate({ email: binding.email, binding: binding.value, code });
    } else {
      send.mutate({ email });
    }
  }
  function changeMethod(value: AuthMethod): void {
    navigateSelection("method", value, loginMethod, "push");
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
    hasBinding: binding !== null && binding.email === email,
    emailCodeAvailable,
    text,
    onEmail: (value) => {
      setEmail(value);
      setBinding(null);
      setCode("");
    },
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
