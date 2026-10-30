"use client";

import { AuthPanel } from "@chapchoo/ui/AuthPanel";
import { ColorModeSelector } from "@chapchoo/ui/ColorModeSelector";
import { EmailField, PasswordField } from "@chapchoo/ui/fields";
import {
  Alert,
  Anchor,
  Button,
  SegmentedControl,
  Stack,
  Text,
  TextInput,
} from "@mantine/core";
import type en from "../../../../messages/en-US.json";
import type { ColorModeSelectorProps } from "@chapchoo/ui/ColorModeSelector";
import type { ReactElement } from "react";

export type AuthMode = "login" | "setup";
export type AuthMethod = "password" | "code";
export type AuthState =
  | { type: "LOADING" }
  | { type: "CLOSED" }
  | { type: "UNAVAILABLE"; message: string }
  | { type: "FORM"; pending: boolean; error: string | null };

export interface AdminAuthProps {
  appearance?: ColorModeSelectorProps;
  mode: AuthMode;
  method: AuthMethod;
  state: AuthState;
  basePath: string;
  email: string;
  password: string;
  installationProof: string;
  code: string;
  hasBinding: boolean;
  emailCodeAvailable: boolean;
  text: (key: keyof typeof en.AdminAuth) => string;
  onEmail: (value: string) => void;
  onPassword: (value: string) => void;
  onProof: (value: string) => void;
  onCode: (value: string) => void;
  onMethod: (value: AuthMethod) => void;
  onChangeEmail: () => void;
  onSubmit: () => void;
}

export function AdminAuth(props: AdminAuthProps): ReactElement {
  const { text, state, mode, method } = props;
  const setup = mode === "setup";
  return (
    <AuthPanel
      actions={
        props.appearance ? <ColorModeSelector {...props.appearance} /> : null
      }
      basePath={props.basePath}
      title={text(setup ? "setupTitle" : "loginTitle")}
      description={text(setup ? "setupDescription" : "loginDescription")}
      footer={
        <Anchor
          href={`${props.basePath}${setup ? "/login" : "/setup"}`}
          size="sm"
        >
          {text(setup ? "loginLink" : "setupLink")}
        </Anchor>
      }
    >
      {state.type === "LOADING" ? (
        <Text role="status">{text("loading")}</Text>
      ) : null}
      {state.type === "CLOSED" ? <Alert>{text("closed")}</Alert> : null}
      {state.type === "UNAVAILABLE" ? (
        <Alert color="red" role="alert">
          {state.message}
        </Alert>
      ) : null}
      {state.type === "FORM" ? (
        <form
          onSubmit={(event) => {
            event.preventDefault();
            props.onSubmit();
          }}
        >
          <Stack gap="md">
            {!setup && props.emailCodeAvailable ? (
              <SegmentedControl
                value={method}
                onChange={props.onMethod}
                data={[
                  { value: "password", label: text("passwordMethod") },
                  { value: "code", label: text("codeMethod") },
                ]}
                disabled={state.pending}
              />
            ) : null}
            <EmailField
              label={text("email")}
              value={props.email}
              onValueChange={props.onEmail}
              required
              disabled={state.pending || props.hasBinding}
            />
            {setup ? (
              <PasswordField
                label={text("setupProof")}
                visibilityToggleLabel={text("showProof")}
                autoComplete="off"
                value={props.installationProof}
                onValueChange={props.onProof}
                required
                disabled={state.pending}
              />
            ) : null}
            {setup || method === "password" ? (
              <PasswordField
                label={text("password")}
                visibilityToggleLabel={text("showPassword")}
                autoComplete={setup ? "new-password" : "current-password"}
                value={props.password}
                onValueChange={props.onPassword}
                description={setup ? text("passwordPolicy") : null}
                minLength={8}
                maxLength={64}
                required
                disabled={state.pending}
              />
            ) : null}
            {!setup && method === "code" && props.hasBinding ? (
              <>
                <Text size="sm" c="dimmed">
                  {text("codeRequested")}
                </Text>
                <TextInput
                  label={text("code")}
                  value={props.code}
                  onChange={(event) => props.onCode(event.currentTarget.value)}
                  autoComplete="one-time-code"
                  autoCapitalize="characters"
                  minLength={6}
                  maxLength={6}
                  required
                  disabled={state.pending}
                />
                <Button
                  variant="subtle"
                  onClick={props.onChangeEmail}
                  disabled={state.pending}
                >
                  {text("changeEmail")}
                </Button>
              </>
            ) : null}
            {state.error ? (
              <Alert color="red" role="alert">
                {state.error}
              </Alert>
            ) : null}
            <Button type="submit" loading={state.pending} fullWidth>
              {text(
                setup
                  ? "initialize"
                  : method === "password"
                    ? "login"
                    : props.hasBinding
                      ? "verifyCode"
                      : "sendCode",
              )}
            </Button>
          </Stack>
        </form>
      ) : null}
    </AuthPanel>
  );
}
