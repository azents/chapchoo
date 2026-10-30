"use client";

import { useQueryClient } from "@tanstack/react-query";
import { useMessages } from "next-intl";
import { useRouter } from "next/navigation";
import { useRef, useState } from "react";
import { trpc } from "@/shared/rpc/client";
import { accountError } from "../errors";
import type { AuthInput, AuthProps, Submission } from "../types";

export function useAuthContainer(input: AuthInput): AuthProps {
  const copy = useMessages().accounts;
  const router = useRouter();
  const client = useQueryClient();
  const utils = trpc.useUtils();
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [code, setCode] = useState("");
  const [challenge, setChallenge] = useState<{
    email: string;
    binding: string;
  } | null>(null);
  const [submission, setSubmission] = useState<Submission>({ type: "READY" });
  const busy = useRef(false);
  const methods = trpc.auth.methods.useQuery(void 0, {
    enabled:
      input.mode === "LOGIN" || (input.mode === "SIGNUP" && !input.token),
    retry: false,
  });
  const signup = trpc.signup.preview.useQuery(
    { proof: input.token },
    { enabled: input.mode === "SIGNUP" && Boolean(input.token), retry: false },
  );
  const reset = trpc.reset.preview.useQuery(
    { proof: input.token },
    {
      enabled:
        input.mode === "RESET" &&
        Boolean(input.token) &&
        submission.type !== "DONE",
      retry: false,
    },
  );
  const login = trpc.auth.passwordLogin.useMutation({
    retry: false,
    onError: (error) => {
      setSubmission({
        type: "ERROR",
        message: accountError(error.data?.reason ?? null, copy),
      });
    },
  });
  const send = trpc.auth.sendCode.useMutation({
    retry: false,
    onError: (error) => {
      setSubmission({
        type: "ERROR",
        message: accountError(error.data?.reason ?? null, copy),
      });
    },
  });
  const verify = trpc.auth.verifyCode.useMutation({
    retry: false,
    onError: (error) => {
      setSubmission({
        type: "ERROR",
        message: accountError(error.data?.reason ?? null, copy),
      });
    },
  });
  const redeemSignup = trpc.signup.redeem.useMutation({
    retry: false,
    onError: (error) => {
      setSubmission({
        type: "ERROR",
        message: accountError(error.data?.reason ?? null, copy),
      });
    },
  });
  const requestSignup = trpc.signup.requestEmail.useMutation({
    retry: false,
    onError: (error) => {
      setSubmission({
        type: "ERROR",
        message: accountError(error.data?.reason ?? null, copy),
      });
    },
  });
  const redeemReset = trpc.reset.redeem.useMutation({
    retry: false,
    onError: (error) => {
      setSubmission({
        type: "ERROR",
        message: accountError(error.data?.reason ?? null, copy),
      });
    },
  });
  const proofQuery = input.mode === "SIGNUP" ? signup : reset;
  let state: AuthProps["state"];
  if (input.mode !== "LOGIN" && input.token) {
    state = proofQuery.isPending
      ? { type: "LOADING" }
      : proofQuery.isError
        ? {
            type: "ERROR",
            message: accountError(proofQuery.error.data?.reason ?? null, copy),
          }
        : !proofQuery.data.usable
          ? { type: "ERROR", message: copy.invalidProof }
          : {
              type: "LOADED",
              data: {
                password: true,
                emailCode: false,
                signupEmail: false,
                email: proofQuery.data.emailHint ?? "",
              },
            };
  } else if (input.mode === "RESET") {
    state = {
      type: "LOADED",
      data: {
        password: false,
        emailCode: false,
        signupEmail: false,
        email: "",
      },
    };
  } else {
    state = methods.isPending
      ? { type: "LOADING" }
      : methods.isError
        ? {
            type: "ERROR",
            message: accountError(methods.error.data?.reason ?? null, copy),
          }
        : { type: "LOADED", data: { ...methods.data, email: "" } };
  }
  function authenticated(): void {
    client.clear();
    setPassword("");
    setCode("");
    router.replace("/");
    router.refresh();
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
        await redeemSignup.mutateAsync({ proof: input.token, email, password });
        authenticated();
      } else if (input.mode === "SIGNUP") {
        await requestSignup.mutateAsync({ email });
        setSubmission({ type: "DONE", message: copy.signupSent });
      } else if (input.token) {
        await redeemReset.mutateAsync({ proof: input.token, password });
        setPassword("");
        client.clear();
        setSubmission({ type: "DONE", message: copy.resetDone });
      }
    } catch {
      // Each mutation's typed onError callback owns the localized feedback.
    } finally {
      busy.current = false;
    }
  }
  return {
    ...input,
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
    onRetry: () => {
      void utils.auth.methods.invalidate();
      void utils.signup.preview.invalidate();
      void utils.reset.preview.invalidate();
    },
  };
}
