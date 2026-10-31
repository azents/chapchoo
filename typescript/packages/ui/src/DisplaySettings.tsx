"use client";
import {
  ActionIcon,
  Alert,
  Box,
  Button,
  Divider,
  Drawer,
  Group,
  Paper,
  Select,
  Stack,
  Text,
  rem,
} from "@mantine/core";
import type { ReactElement, ReactNode } from "react";
import classes from "./DisplaySettings.module.css";
import { formatDisplayDate } from "./displayFormatting";
import { ApplicationShell } from "./ApplicationShell";
import { ContentSection } from "./ContentSection";
import type { NavigationItem } from "./ApplicationShell";
import type { ColorModeMenuProps } from "./ColorModeMenu";

export interface DisplayValues {
  language: string;
  timeZone: string;
  region: string;
}
export type DisplayDimension = keyof DisplayValues;
export interface DisplayCopy {
  title: string;
  language: string;
  timeZone: string;
  region: string;
  follow: string;
  preview: string;
  unavailable: string;
  scope: string;
  done: string;
  close: string;
  immediate: string;
  saved: string;
  error: string;
  storageError: string;
  retry: string;
}
export interface DisplayFieldsProps {
  value: DisplayValues;
  selected: Partial<DisplayValues>;
  automatic: boolean;
  disabled: boolean;
  zones: readonly string[];
  copy: DisplayCopy;
  onChange: (key: DisplayDimension, value: string | null) => void;
}
export function DisplayIcon({
  kind,
}: {
  kind: DisplayDimension | "settings";
}): ReactElement {
  return (
    <svg
      width={rem(20)}
      height={rem(20)}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.65"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      style={{ flexShrink: 0 }}
    >
      {kind === "language" ? (
        <path d="M3 5h12M9 3v2M5 5c0 5 3 8 7 10M13 5c0 5-3 8-7 10M14 21l4-10 4 10M15.5 17h5" />
      ) : kind === "timeZone" ? (
        <>
          <circle cx="12" cy="12" r="9" />
          <path d="M12 7v5l3 2" />
        </>
      ) : kind === "region" ? (
        <>
          <circle cx="12" cy="12" r="9" />
          <ellipse cx="12" cy="12" rx="4" ry="9" />
          <path d="M3 12h18" />
        </>
      ) : (
        <>
          <path d="M4 7h16M4 17h16" />
          <circle cx="9" cy="7" r="3" fill="var(--mantine-color-body)" />
          <circle cx="15" cy="17" r="3" fill="var(--mantine-color-body)" />
        </>
      )}
    </svg>
  );
}
export function DisplayEntry({
  label,
  onOpen,
}: {
  label: string;
  onOpen: () => void;
}): ReactElement {
  return (
    <ActionIcon
      variant="subtle"
      size={rem(44)}
      aria-label={label}
      title={label}
      onClick={onOpen}
    >
      <DisplayIcon kind="settings" />
    </ActionIcon>
  );
}
const languages = [
  { value: "en", label: "English" },
  { value: "ko", label: "한국어" },
  { value: "ja", label: "日本語" },
  { value: "fr", label: "Français" },
];
const regions = [
  { value: "en-US", label: "United States · English" },
  { value: "ko-KR", label: "대한민국 · 한국어" },
  { value: "ja-JP", label: "日本 · 日本語" },
  { value: "fr-FR", label: "France · Français" },
];
export function DisplayFields(props: DisplayFieldsProps): ReactElement {
  const options = {
    language: languages,
    timeZone: [...new Set(["UTC", props.value.timeZone, ...props.zones])].map(
      (value) => ({ value, label: value }),
    ),
    region: regions,
  };
  const dimensions: readonly DisplayDimension[] = [
    "language",
    "timeZone",
    "region",
  ];
  return (
    <Stack gap="xl">
      {dimensions.map((key) => (
        <Select
          key={key}
          label={props.copy[key]}
          searchable
          allowDeselect={false}
          disabled={props.disabled}
          value={
            props.automatic ? (props.selected[key] ?? "auto") : props.value[key]
          }
          data={
            props.automatic
              ? [{ value: "auto", label: props.copy.follow }, ...options[key]]
              : options[key]
          }
          onChange={(value) =>
            props.onChange(key, value === "auto" ? null : value)
          }
        />
      ))}
    </Stack>
  );
}
export function DisplayPreview({
  value,
  label,
  instant,
  unavailable,
}: {
  value: DisplayValues;
  label: string;
  instant: number;
  unavailable: string;
}): ReactElement {
  const date = formatDisplayDate(instant, value.region, value.timeZone, "long");
  return (
    <Paper p="lg" withBorder radius="md">
      <Text size="xs" c="dimmed" mb="sm">
        {label}
      </Text>
      <Text fw={600}>{date ?? unavailable}</Text>
      <Group gap="sm" mt="xs">
        <Text size="sm" c="dimmed">
          {value.timeZone}
        </Text>
        <Text size="sm" c="dimmed">
          ·
        </Text>
        <Text size="sm" c="dimmed">
          {new Intl.NumberFormat(value.region).format(1234567.89)}
        </Text>
      </Group>
    </Paper>
  );
}
export function DisplaySaveFeedback({
  copy,
  error,
  storageError,
  onRetry,
}: {
  copy: DisplayCopy;
  error: boolean;
  storageError: boolean;
  onRetry: () => void;
}): ReactElement | null {
  if (!error && !storageError) {
    return null;
  }
  return (
    <Alert color="red" role="alert">
      {error ? copy.error : copy.storageError}
      {error ? (
        <Box mt="sm">
          <Button variant="default" size="sm" onClick={onRetry}>
            {copy.retry}
          </Button>
        </Box>
      ) : null}
    </Alert>
  );
}
export function DisplayPanel(
  props: DisplayFieldsProps & {
    opened: boolean;
    changed: boolean;
    error: boolean;
    storageError: boolean;
    instant: number;
    footer?: ReactNode;
    onRetry: () => void;
    onClose: () => void;
  },
): ReactElement {
  return (
    <Drawer
      opened={props.opened}
      onClose={props.onClose}
      position="right"
      size="md"
      title={
        <Text fw={700} size="xl">
          {props.copy.title}
        </Text>
      }
      closeButtonProps={{ "aria-label": props.copy.close }}
      padding="xl"
      overlayProps={{ backgroundOpacity: 0.35, blur: 0 }}
    >
      <Stack gap="xl">
        <Text size="sm" c="dimmed">
          {props.copy.scope}
        </Text>
        <DisplayFields {...props} />
        <DisplayPreview
          value={props.value}
          label={props.copy.preview}
          unavailable={props.copy.unavailable}
          instant={props.instant}
        />
        <DisplaySaveFeedback
          copy={props.copy}
          error={props.error}
          storageError={props.storageError}
          onRetry={props.onRetry}
        />
        {props.footer}
        <Divider />
        <Group justify="space-between">
          <Text size="sm" c="dimmed" role="status">
            {props.changed ? props.copy.saved : props.copy.immediate}
          </Text>
          <Button onClick={props.onClose}>{props.copy.done}</Button>
        </Group>
      </Stack>
    </Drawer>
  );
}
export interface DisplaySuggestion {
  key: DisplayDimension;
  label: string;
  detail: string;
  action: string;
  onApply: () => void;
}
export function DisplayToast(props: {
  title: string;
  alternateTitle?: string;
  closeLabel: string;
  suggestions: readonly DisplaySuggestion[];
  busy: boolean;
  error?: ReactNode;
  onClose: () => void;
}): ReactElement | null {
  if (!props.suggestions.length) {
    return null;
  }
  return (
    <Paper
      role="status"
      withBorder
      shadow="md"
      radius="md"
      p="lg"
      className={classes.toast}
    >
      <Stack gap="sm">
        <Group justify="space-between" align="flex-start" wrap="nowrap">
          <Stack gap="xs">
            <Text fw={600} size="sm">
              {props.title}
            </Text>
            {props.alternateTitle ? (
              <Text size="sm" c="dimmed">
                {props.alternateTitle}
              </Text>
            ) : null}
          </Stack>
          <ActionIcon
            variant="subtle"
            size={rem(44)}
            aria-label={props.closeLabel}
            onClick={props.onClose}
          >
            ×
          </ActionIcon>
        </Group>
        {props.suggestions.map((row) => (
          <Group
            key={row.key}
            justify="space-between"
            wrap="nowrap"
            data-proposal={row.key}
          >
            <Group gap="md" wrap="nowrap">
              <DisplayIcon kind={row.key} />
              <Stack gap="xs">
                <Text fw={600} size="sm">
                  {row.label}
                </Text>
                <Text size="sm" c="dimmed">
                  {row.detail}
                </Text>
              </Stack>
            </Group>
            <Button
              size="sm"
              variant="default"
              disabled={props.busy}
              onClick={row.onApply}
            >
              {row.action}
            </Button>
          </Group>
        ))}
        {props.error}
      </Stack>
    </Paper>
  );
}

export interface DisplaySettingsPageProps extends DisplayFieldsProps {
  basePath: string;
  contextLabel: string;
  title: string;
  heading: string;
  description?: string;
  identityLabel: string;
  accountHeading?: string;
  emailLabel?: string;
  navigation: readonly NavigationItem[];
  logoutLabel: string;
  onLogout: () => void;
  appearance: ColorModeMenuProps;
  instant: number;
  changed: boolean;
  error: boolean;
  storageError: boolean;
  onRetry: () => void;
}
export function DisplaySettingsPage(
  props: DisplaySettingsPageProps,
): ReactElement {
  return (
    <ApplicationShell
      basePath={props.basePath}
      contextLabel={props.contextLabel}
      title={props.title}
      identityLabel={props.identityLabel}
      navigation={props.navigation}
      appearance={props.appearance}
      actions={
        <Button variant="subtle" size="xs" onClick={props.onLogout}>
          {props.logoutLabel}
        </Button>
      }
    >
      {props.accountHeading ? (
        <ContentSection title={props.accountHeading}>
          <Text size="sm" c="dimmed">
            {props.emailLabel}
          </Text>
          <Text>{props.identityLabel}</Text>
        </ContentSection>
      ) : null}
      <ContentSection title={props.heading}>
        {props.description ? (
          <Text size="sm" c="dimmed">
            {props.description}
          </Text>
        ) : null}
        <Box maw={rem(540)}>
          <DisplayFields {...props} />
        </Box>
        <Box maw={rem(540)}>
          <DisplayPreview
            value={props.value}
            label={props.copy.preview}
            unavailable={props.copy.unavailable}
            instant={props.instant}
          />
        </Box>
        <DisplaySaveFeedback
          copy={props.copy}
          error={props.error}
          storageError={props.storageError}
          onRetry={props.onRetry}
        />
        <Text size="sm" c="dimmed" role="status">
          {props.changed && !props.error && !props.disabled
            ? props.copy.saved
            : props.copy.immediate}
        </Text>
      </ContentSection>
    </ApplicationShell>
  );
}
