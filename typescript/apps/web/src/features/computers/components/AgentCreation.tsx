"use client";
import { ApplicationShell } from "@chapchoo/ui/ApplicationShell";
import { PageFailure } from "@chapchoo/ui/PageFailure";
import { agentsPath } from "@chapchoo/web-core/workspace";
import {
  Alert,
  Anchor,
  Box,
  Button,
  Group,
  Loader,
  Select,
  Stack,
  Text,
  TextInput,
} from "@mantine/core";
import { ProfileImageChoices, ProfilePaletteChoices } from "./ProfileChoices";
import classes from "./ProfilePresentation.module.css";
import { ProfilePreview } from "./ProfilePreview";
import type { AgentCreationProps } from "../creationTypes";
import type { ReactElement } from "react";

export function AgentCreation(props: AgentCreationProps): ReactElement {
  const { copy: c } = props;
  const p = props.profileCopy;
  const locked = props.pending || props.creationLocked;
  const imageOptions = props.images.items.some(
    (image) => image.id === props.image.id,
  )
    ? props.images.items
    : [props.image, ...props.images.items];
  const listHref = agentsPath(props.basePath, props.initial.teamHandle);
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
        <Box className={classes.creationGrid}>
          <Box className={classes.creationPreview}>
            <ProfilePreview
              basePath={props.basePath}
              assetKey={props.image.assetKey}
              paletteKey={props.paletteKey}
              name={props.name || c.name}
              handle={props.handle.draft.candidate}
              label={p.preview}
            />
          </Box>
          <Stack gap="xl">
            <ProfileImageChoices
              basePath={props.basePath}
              label={p.image}
              images={imageOptions}
              selected={props.image.id}
              disabled={locked || props.imagesUnavailable}
              onChange={props.onImage}
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
            <ProfilePaletteChoices
              copy={p}
              selected={props.paletteKey}
              disabled={locked}
              onChange={props.onPalette}
              onRandom={props.onRandomPalette}
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
        </Box>
      </Stack>
    </ApplicationShell>
  );
}
