"use client";

import { useAuth } from "@chapchoo/web-core/auth-context";
import { useDisplay } from "@chapchoo/web-core/display-context";
import { useMessages } from "next-intl";
import { useRef, useState } from "react";
import { useAppearanceControl } from "@/shared/appearance/useAppearanceControl";
import { trpc } from "@/shared/rpc/client";
import { accountError } from "../errors";
import type { InitialAuth } from "../initialAuth";
import type { AuthInput, AuthProps, Submission } from "../types";

export function useAuthContainer({
  initial,
  ...input
}: AuthInput & { initial: InitialAuth }): AuthProps {
  const copy = useMessages().accounts;
  const appearance = useAppearanceControl();
  const auth = useAuth();
  const display = useDisplay();
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [code, setCode] = useState("");
  const [challenge, setChallenge] = useState<{
    email: string;
    binding: string;
  } | null>(null);
  const [submission, setSubmission] = useState<Submission>({ type: "READY" });
  const busy = useRef(false);
  function fail(reason: string | null): void {
    setSubmission({ type: "ERROR", message: accountError(reason, copy) });
  }
  const login = trpc.auth.passwordLogin.useMutation({
    retry: false,
    onError: (error) => fail(error.data?.reason ?? null),
  });
  const send = trpc.auth.sendCode.useMutation({
    retry: false,
    onError: (error) => fail(error.data?.reason ?? null),
  });
  const verify = trpc.auth.verifyCode.useMutation({
    retry: false,
    onError: (error) => fail(error.data?.reason ?? null),
  });
  const redeemSignup = trpc.signup.redeem.useMutation({
    retry: false,
    onError: (error) => fail(error.data?.reason ?? null),
  });
  const requestSignup = trpc.signup.requestEmail.useMutation({
    retry: false,
    onError: (error) => fail(error.data?.reason ?? null),
  });
  const redeemReset = trpc.reset.redeem.useMutation({
    retry: false,
    onError: (error) => fail(error.data?.reason ?? null),
  });
  const state: AuthProps["state"] =
    initial.type === "PROOF"
      ? !initial.data.usable
        ? { type: "ERROR", message: copy.invalidProof }
        : {
            type: "LOADED",
            data: {
              password: true,
              emailCode: false,
              signupEmail: false,
              email: initial.data.emailHint ?? "",
            },
          }
      : {
          type: "LOADED",
          data:
            initial.type === "METHODS"
              ? { ...initial.data, email: "" }
              : {
                  password: false,
                  emailCode: false,
                  signupEmail: false,
                  email: "",
                },
        };
  function authenticated(): void {
    setPassword("");
    setCode("");
    auth.authenticated();
  }
  async function submit(sendCode: boolean): Promise<void> {
    if (busy.current) {
      return;
    }
    busy.current = true;
    setSubmission({ type: "PENDING" });
    try {
      if (sendCode) {
        const result = await send.mutateAsync({ email });
        setChallenge({ email, binding: result.binding });
        setCode("");
        setSubmission({ type: "READY" });
      } else if (input.mode === "LOGIN") {
        if (challenge) {
          await verify.mutateAsync({
            email: challenge.email,
            binding: challenge.binding,
            code,
          });
        } else {
          await login.mutateAsync({ email, password });
        }
        authenticated();
      } else if (input.mode === "SIGNUP" && input.token) {
        await redeemSignup.mutateAsync({
          proof: input.token,
          email,
          password,
          display: display.creationDisplay(),
        });
        authenticated();
      } else if (input.mode === "SIGNUP") {
        await requestSignup.mutateAsync({ email });
        setSubmission({ type: "DONE", message: copy.signupSent });
      } else if (input.token) {
        await redeemReset.mutateAsync({ proof: input.token, password });
        setPassword("");
        auth.clearIdentity();
        setSubmission({ type: "DONE", message: copy.resetDone });
      }
    } catch {
      // Typed mutation callbacks own feedback; never replay an uncertain write.
    } finally {
      busy.current = false;
    }
  }
  return {
    ...input,
    appearance,
    copy,
    state,
    submission,
    email,
    password,
    code,
    challenge: challenge !== null,
    onEmail: (value) => {
      setEmail(value);
      setChallenge(null);
      setCode("");
    },
    onPassword: setPassword,
    onCode: setCode,
    onSubmit: () => {
      if (state.type !== "LOADED") {
        return;
      }
      if (input.mode === "LOGIN" && !challenge && !state.data.password) {
        if (state.data.emailCode) {
          void submit(true);
        }
        return;
      }
      void submit(false);
    },
    onSendCode: () => {
      void submit(true);
    },
    onUsePassword: () => {
      setChallenge(null);
      setCode("");
      setSubmission({ type: "READY" });
    },
    onRetry: () => window.location.reload(),
  };
}
