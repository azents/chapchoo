"use client";
import { ApplicationShell } from "@chapchoo/ui/ApplicationShell";
import { PageFailure } from "@chapchoo/ui/PageFailure";
import { agentPaletteKeys } from "@chapchoo/web-core/agent-profile";
import {
  Alert,
  Anchor,
  Button,
  Group,
  Image,
  Loader,
  Paper,
  rem,
  Select,
  Stack,
  Text,
  TextInput,
} from "@mantine/core";
import type { AgentCreationProps } from "../creationTypes";
import type { AgentPaletteKey } from "@chapchoo/web-core/computer";
import type { ReactElement } from "react";

const paletteSurface: Record<AgentPaletteKey, string> = {
  butter: "var(--chapchoo-color-accentSoft)",
  sage: "var(--chapchoo-color-selected)",
  apricot: "var(--mantine-color-orange-light)",
  teal: "var(--mantine-color-teal-light)",
  lilac: "var(--mantine-color-grape-light)",
  rose: "var(--mantine-color-pink-light)",
};
export function AgentCreation(props: AgentCreationProps): ReactElement {
  const { copy: c } = props;
  const p = props.profileCopy;
  const locked = props.pending || props.creationLocked;
  const imageOptions = props.images.items.some(
    (image) => image.id === props.image.id,
  )
    ? props.images.items
    : [props.image, ...props.images.items];
  const listHref = `${props.basePath}/agents?team=${encodeURIComponent(props.initial.teamId)}`;
  if (props.access !== "authorized") {
    return (
      <PageFailure
        title={c.create}
        message={c.accessUnavailable}
        appearance={props.appearance}
        retryLabel={props.access === "signedOut" ? c.signIn : c.retry}
        onRetry={props.onRefresh}
      />
    );
  }
  return (
    <ApplicationShell
      basePath={props.basePath}
      appearance={props.appearance}
      contextLabel={c.agents}
      title={c.create}
      identityLabel={props.initial.identity}
      mascotReplayLabel={c.replay}
      actions={
        <Button variant="subtle" onClick={props.onLogout}>
          {c.logout}
        </Button>
      }
      navigation={[
        { key: "agents", label: c.agents, active: true, href: listHref },
        { key: "accounts", label: c.accounts, href: `${props.basePath}/` },
      ]}
    >
      <Anchor href={listHref}>{c.back}</Anchor>
      <Stack
        component="form"
        onSubmit={(event) => {
          event.preventDefault();
          props.onCreate();
        }}
      >
        <Paper
          p="md"
          radius="lg"
          bg={paletteSurface[props.paletteKey]}
          w="fit-content"
        >
          <Image
            w={rem(96)}
            h={rem(96)}
            fit="contain"
            alt=""
            aria-hidden="true"
            src={`${props.basePath}/design-assets/${props.image.assetKey}-216.webp`}
          />
        </Paper>
        <TextInput
          label={c.name}
          value={props.name}
          onChange={(event) => props.onName(event.currentTarget.value)}
          required
          maxLength={120}
          disabled={props.pending || props.creationLocked}
        />
        <TextInput
          label={p.handle}
          description={p.permanent}
          value={props.handle.draft.candidate}
          maxLength={16}
          required
          disabled={locked}
          onChange={(event) => props.onHandle(event.currentTarget.value)}
          error={
            props.handle.status === "invalid"
              ? p.invalid
              : props.handle.status === "occupied"
                ? p.occupied
                : props.handle.status === "exhausted"
                  ? p.exhausted
                  : props.handle.status === "unavailable"
                    ? p.unavailable
                    : null
          }
        />
        <Text size="sm" c="dimmed" aria-live="polite">
          {props.handle.status === "checking"
            ? p.checking
            : props.handle.status === "available"
              ? p.available
              : null}
        </Text>
        {props.handle.status === "unavailable" ? (
          <Button
            type="button"
            variant="subtle"
            onClick={props.onRefresh}
            disabled={locked}
          >
            {c.retryRead}
          </Button>
        ) : null}
        <Select
          label={p.image}
          value={props.image.id}
          data={imageOptions.map((image) => ({
            value: image.id,
            label: image.internalName,
          }))}
          onChange={(value) => {
            if (value) {
              props.onImage(value);
            }
          }}
          required
          disabled={locked || props.imagesUnavailable}
        />
        <Group role="group" aria-label={p.image}>
          {props.imageAfter ? (
            <Button
              type="button"
              variant="subtle"
              disabled={locked}
              onClick={() => props.onImagePage(null)}
            >
              {c.first}
            </Button>
          ) : null}
          {props.images.nextAfter ? (
            <Button
              type="button"
              variant="subtle"
              disabled={locked}
              onClick={() => props.onImagePage(props.images.nextAfter)}
            >
              {c.next}
            </Button>
          ) : null}
        </Group>
        <Select
          label={p.palette}
          value={props.paletteKey}
          data={agentPaletteKeys.map((key) => ({
            value: key,
            label: p.palettes[key],
          }))}
          onChange={(value) => {
            if (value) {
              props.onPalette(value);
            }
          }}
          required
          disabled={locked}
        />
        {props.imagesUnavailable ? (
          <Alert color="yellow" role="alert">
            <Stack gap="sm">
              <Text>{c.unavailable}</Text>
              <Button
                type="button"
                variant="subtle"
                onClick={props.onRefresh}
                disabled={props.pending}
              >
                {c.retry}
              </Button>
            </Stack>
          </Alert>
        ) : null}
        <Select
          label={c.provider}
          placeholder={c.chooseProvider}
          value={props.providerId || null}
          data={
            props.providers.type === "loaded"
              ? props.providers.items.map((item) => ({
                  value: item.id,
                  label: item.name,
                }))
              : []
          }
          onChange={(value) => props.onProvider(value ?? "")}
          disabled={
            props.pending ||
            props.creationLocked ||
            props.providers.type !== "loaded"
          }
          required
        />
        {props.providers.type === "loading" ? <Loader size="sm" /> : null}
        {props.providers.type === "loaded" &&
        props.providers.items.length === 0 ? (
          <Text c="dimmed">{c.noProviders}</Text>
        ) : null}
        {props.providers.type === "loaded" ? (
          <Group>
            {props.providers.after ? (
              <Button
                variant="subtle"
                disabled={props.pending || props.creationLocked}
                type="button"
                onClick={() => props.onProviderPage(null)}
              >
                {c.first}
              </Button>
            ) : null}
            {props.providers.nextAfter ? (
              <Button
                variant="subtle"
                disabled={props.pending || props.creationLocked}
                type="button"
                onClick={() =>
                  props.providers.type === "loaded" &&
                  props.onProviderPage(props.providers.nextAfter)
                }
              >
                {c.next}
              </Button>
            ) : null}
          </Group>
        ) : null}
        {props.providers.type === "error" ||
        (props.providers.type === "loaded" && props.providers.stale) ? (
          <Alert color="yellow" role="alert">
            <Stack gap="sm">
              <Text>{c.unavailable}</Text>
              <Button
                variant="subtle"
                onClick={props.onRefresh}
                disabled={props.pending}
              >
                {c.retry}
              </Button>
            </Stack>
          </Alert>
        ) : null}
        {props.error ? (
          <Alert color="red" role="alert">
            {props.error}
          </Alert>
        ) : null}
        <Group>
          <Button
            type="submit"
            loading={props.pending}
            disabled={
              !props.creationLocked &&
              (!props.name.trim() ||
                props.handle.status !== "available" ||
                props.imagesUnavailable ||
                !props.providerId ||
                props.providers.type !== "loaded" ||
                props.providers.stale ||
                !props.providers.items.some(
                  (item) => item.id === props.providerId,
                ))
            }
          >
            {props.creationLocked ? c.recoverCreation : c.create}
          </Button>
        </Group>
      </Stack>
    </ApplicationShell>
  );
}
