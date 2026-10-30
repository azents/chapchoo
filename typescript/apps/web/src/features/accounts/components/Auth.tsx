"use client";

import { AuthPanel } from "@chapchoo/ui/AuthPanel";
import { ErrorFeedback, LoadingFeedback } from "@chapchoo/ui/Feedback";
import { EmailField, PasswordField } from "@chapchoo/ui/fields";
import {
  Alert,
  Anchor,
  Button,
  Divider,
  Group,
  Stack,
  Text,
  TextInput,
} from "@mantine/core";
import type { AuthProps } from "../types";
import type { ReactNode } from "react";

export function Auth(props: AuthProps): ReactNode {
  const { copy: c, mode, state, submission } = props;
  const pending = submission.type === "PENDING";
  const title =
    mode === "LOGIN" ? c.login : mode === "SIGNUP" ? c.signup : c.reset;
  const proof = props.token.length > 0;
  return (
    <AuthPanel
      basePath={props.basePath}
      title={title}
      description={c.authDescription}
      footer={
        <Group justify="space-between">
          {mode !== "LOGIN" ? (
            <Anchor href={`${props.basePath}/login`}>{c.login}</Anchor>
          ) : null}
          {mode !== "SIGNUP" ? (
            <Anchor href={`${props.basePath}/signup`}>{c.signup}</Anchor>
          ) : null}
          {mode !== "RESET" ? (
            <Anchor href={`${props.basePath}/reset-password`}>{c.reset}</Anchor>
          ) : null}
        </Group>
      }
    >
      {mode === "RESET" && submission.type === "DONE" ? (
        // Reset consumes the proof and clears identity caches. Its confirmed
        // terminal result must not be replaced by a now-empty preview query.
        <Stack gap="md">
          <Alert color="green" role="status">
            {submission.message}
          </Alert>
          <Anchor href={`${props.basePath}/login`}>{c.backToLogin}</Anchor>
        </Stack>
      ) : state.type === "LOADING" ? (
        <LoadingFeedback message={c.loading} />
      ) : state.type === "ERROR" ? (
        <ErrorFeedback
          title={c.errorTitle}
          message={state.message}
          retryLabel={c.retry}
          onRetry={props.onRetry}
        />
      ) : (
        <Stack gap="md">
          {submission.type === "ERROR" ? (
            <Alert color="red" role="alert">
              {submission.message}
            </Alert>
          ) : null}
          {submission.type === "DONE" ? (
            <Alert color="green" role="status">
              {submission.message}
            </Alert>
          ) : null}
          {(mode === "RESET" && !proof) ||
          (mode === "SIGNUP" && !proof && !state.data.signupEmail) ? (
            <Text size="sm">{c.linkRequired}</Text>
          ) : submission.type === "DONE" && proof ? (
            <Anchor href={`${props.basePath}/login`}>{c.backToLogin}</Anchor>
          ) : (
            <form
              onSubmit={(event) => {
                event.preventDefault();
                props.onSubmit();
              }}
            >
              <Stack gap="md">
                {proof && mode === "SIGNUP" && state.data.email ? (
                  <Text size="sm">{state.data.email}</Text>
                ) : null}
                {mode !== "RESET" ? (
                  <EmailField
                    label={c.email}
                    value={props.email}
                    onValueChange={props.onEmail}
                    required
                    disabled={pending}
                  />
                ) : state.data.email ? (
                  <Text size="sm">{state.data.email}</Text>
                ) : null}
                {mode === "LOGIN" &&
                !state.data.password &&
                !state.data.emailCode ? (
                  <Text c="dimmed">{c.loginUnavailable}</Text>
                ) : null}
                {(mode === "LOGIN" &&
                  state.data.password &&
                  !props.challenge) ||
                (mode !== "LOGIN" && proof) ? (
                  <PasswordField
                    label={c.password}
                    value={props.password}
                    onValueChange={props.onPassword}
                    visibilityToggleLabel={c.showPassword}
                    autoComplete={
                      mode === "LOGIN" ? "current-password" : "new-password"
                    }
                    required
                    disabled={pending}
                    minLength={mode === "LOGIN" ? 1 : 8}
                    maxLength={64}
                    description={mode === "LOGIN" ? null : c.weakPassword}
                  />
                ) : null}
                {mode === "LOGIN" && props.challenge ? (
                  <>
                    <Text size="sm">{c.codeSent}</Text>
                    <TextInput
                      label={c.code}
                      value={props.code}
                      onChange={(event) =>
                        props.onCode(event.currentTarget.value)
                      }
                      autoComplete="one-time-code"
                      inputMode="numeric"
                      required
                      disabled={pending}
                    />
                  </>
                ) : null}
                {mode !== "LOGIN" || state.data.password || props.challenge ? (
                  <Button type="submit" loading={pending}>
                    {mode === "LOGIN"
                      ? c.login
                      : proof
                        ? c.savePassword
                        : c.sendLink}
                  </Button>
                ) : null}
                {mode === "LOGIN" && props.challenge && state.data.password ? (
                  <Button
                    variant="subtle"
                    onClick={props.onUsePassword}
                    disabled={pending}
                  >
                    {c.usePassword}
                  </Button>
                ) : null}
                {mode === "LOGIN" && state.data.emailCode ? (
                  <>
                    <Divider label={c.or} />
                    <Button
                      variant="default"
                      onClick={props.onSendCode}
                      disabled={pending || props.email.length === 0}
                    >
                      {props.challenge ? c.resendCode : c.useCode}
                    </Button>
                  </>
                ) : null}
              </Stack>
            </form>
          )}
        </Stack>
      )}
    </AuthPanel>
  );
}
