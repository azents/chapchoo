"use client";

import { PasswordInput, TextInput } from "@mantine/core";
import type { PasswordInputProps, TextInputProps } from "@mantine/core";
import type { ReactNode } from "react";

export interface EmailFieldProps extends Omit<
  TextInputProps,
  "type" | "value" | "defaultValue" | "onChange"
> {
  value: string;
  onValueChange: (value: string) => void;
}

export interface PasswordFieldProps extends Omit<
  PasswordInputProps,
  "value" | "defaultValue" | "onChange" | "visibilityToggleButtonProps"
> {
  value: string;
  onValueChange: (value: string) => void;
  visibilityToggleLabel: string;
}

export function EmailField({
  value,
  onValueChange,
  ...props
}: EmailFieldProps): ReactNode {
  return (
    <TextInput
      type="email"
      autoComplete="email"
      autoCapitalize="none"
      spellCheck={false}
      {...props}
      value={value}
      onChange={(event) => onValueChange(event.currentTarget.value)}
    />
  );
}

export function PasswordField({
  value,
  onValueChange,
  visibilityToggleLabel,
  ...props
}: PasswordFieldProps): ReactNode {
  return (
    <PasswordInput
      autoComplete="current-password"
      {...props}
      value={value}
      onChange={(event) => onValueChange(event.currentTarget.value)}
      visibilityToggleButtonProps={{ "aria-label": visibilityToggleLabel }}
    />
  );
}
