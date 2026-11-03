"use client";
import { ApplicationShell } from "@chapchoo/ui/ApplicationShell";
import { PageFailure } from "@chapchoo/ui/PageFailure";
import {
  Alert,
  Anchor,
  Button,
  Group,
  Loader,
  Select,
  Stack,
  Text,
  TextInput,
} from "@mantine/core";
import type { AgentCreationProps } from "../creationTypes";
import type { ReactElement } from "react";

export function AgentCreation(props: AgentCreationProps): ReactElement {
  const { copy: c } = props;
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
        <TextInput
          label={c.name}
          value={props.name}
          onChange={(event) => props.onName(event.currentTarget.value)}
          required
          maxLength={120}
          disabled={props.pending || props.creationLocked}
        />
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
