"use client";
import { AgentTheme } from "@chapchoo/ui/AgentTheme";
import { ApplicationShell } from "@chapchoo/ui/ApplicationShell";
import { InteractiveSurface } from "@chapchoo/ui/InteractiveSurface";
import { NavigationIcon } from "@chapchoo/ui/NavigationIcon";
import { PageFailure } from "@chapchoo/ui/PageFailure";
import { ProviderIcon } from "@chapchoo/ui/ProviderIcon";
import { agentsPath } from "@chapchoo/web-core/workspace";
import {
  Alert,
  Anchor,
  Box,
  Button,
  Group,
  Loader,
  Stack,
  Stepper,
  Text,
  TextInput,
  Title,
} from "@mantine/core";
import { ConnectionPanel } from "@/shared/authentications/ConnectionPanel";
import { profileStyle } from "../profilePalette";
import { ProfileImageChoices, ProfilePaletteChoices } from "./ProfileChoices";
import classes from "./ProfilePresentation.module.css";
import { ProfilePreview } from "./ProfilePreview";
import type { AgentCreationProps } from "../creationTypes";
import type { ReactElement } from "react";

export function AgentCreation(props: AgentCreationProps): ReactElement {
  const { copy: c, profileCopy: p, authenticationCopy: a } = props;
  const selectedProvider =
    props.providers.type === "loaded"
      ? props.providers.items.find((value) => value.id === props.providerId)
      : void 0;
  const connectionOpen = props.authenticationPanel.dialog.type !== "closed";
  const locked = props.pending || props.creationLocked || connectionOpen;
  const imageOptions = props.images.items.some(
    (image) => image.id === props.image.id,
  )
    ? props.images.items
    : [props.image, ...props.images.items];
  const listHref = agentsPath(props.basePath, props.initial.teamHandle);
  const providerValid =
    props.providers.type === "loaded" &&
    !props.providers.stale &&
    props.providers.items.some((value) => value.id === props.providerId);
  const profileValid =
    props.name.trim() &&
    props.handle.status === "available" &&
    !props.imagesUnavailable &&
    providerValid &&
    props.authentication;
  const step =
    props.step === "environment" ? 0 : props.step === "authentication" ? 1 : 2;
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
      contentClassName={[
        classes.creationContent,
        step < 2 ? classes.compactCreation : "",
      ].join(" ")}
      header={
        <AgentTheme palette={props.paletteKey}>
          <Box
            className={[
              classes.creationHeader,
              step === 1 ? classes.authenticationHeader : "",
            ].join(" ")}
          >
            <Anchor href={listHref} size="sm" c="dimmed">
              <NavigationIcon name="back" size="sm" />
              {c.back}
            </Anchor>
            <Stepper
              active={step}
              size="xs"
              mt="xl"
              mb="xl"
              allowNextStepsSelect={false}
            >
              <Stepper.Step label={a.environment} />
              <Stepper.Step label={a.authentication} />
              <Stepper.Step label={a.profile} />
            </Stepper>
            <Title order={1} className={classes.creationTitle}>
              {step === 0
                ? a.environmentTitle
                : step === 1
                  ? a.authenticationTitle
                  : p.creationTitle}
            </Title>
            <Text c="dimmed" mt="sm">
              {step === 0
                ? a.environmentHint
                : step === 1
                  ? a.authenticationHint
                  : p.chooseProfile}
            </Text>
            {step === 1 ? (
              <Group gap="sm" mt="xl">
                {selectedProvider ? (
                  <ProviderIcon kind={selectedProvider.kind} />
                ) : null}
                <Text size="sm" c="dimmed">
                  {props.providers.type === "loaded"
                    ? (props.providers.items.find(
                        (value) => value.id === props.providerId,
                      )?.name ?? c.provider)
                    : c.provider}
                </Text>
              </Group>
            ) : null}
          </Box>
        </AgentTheme>
      }
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
      <AgentTheme palette={props.paletteKey}>
        <Stack
          component="form"
          gap="xl"
          style={profileStyle(props.paletteKey)}
          onSubmit={(event) => {
            event.preventDefault();
            if (connectionOpen) {
              return;
            }
            if (props.step === "profile") {
              props.onCreate();
            } else {
              props.onNext();
            }
          }}
        >
          {step === 0 ? (
            <Stack gap="lg">
              <Group gap="xs" mt="xl">
                <NavigationIcon name="cloud" />
                <Text>{a.cloud}</Text>
              </Group>
              {props.providers.type === "loaded" ? (
                <Box
                  role="radiogroup"
                  aria-label={c.provider}
                  className={classes.environmentCards}
                >
                  {props.providers.items.map((item) => (
                    <InteractiveSurface
                      key={item.id}
                      role="radio"
                      aria-label={item.name}
                      aria-checked={props.providerId === item.id}
                      disabled={
                        locked ||
                        props.providers.type !== "loaded" ||
                        props.providers.stale
                      }
                      className={classes.environmentCard}
                      data-selected={props.providerId === item.id || void 0}
                      onClick={() => props.onProvider(item.id)}
                    >
                      <Box className={classes.environmentMark}>
                        <ProviderIcon kind={item.kind} size={24} />
                      </Box>
                      <Stack component="span" gap="xs">
                        <Text component="span" fw={600}>
                          {item.kind === "kubernetes" ? "Kubernetes" : "Docker"}
                        </Text>
                        <Text component="span" size="sm" c="dimmed">
                          {item.name}
                        </Text>
                      </Stack>
                    </InteractiveSurface>
                  ))}
                </Box>
              ) : null}
              {props.providers.type === "loading" ? <Loader size="sm" /> : null}
              {props.providers.type === "loaded" &&
              props.providers.items.length === 0 ? (
                <Text c="dimmed">{c.noProviders}</Text>
              ) : null}
              {props.providers.type === "loaded" &&
              (props.providers.after || props.providers.nextAfter) ? (
                <Group>
                  {props.providers.after ? (
                    <Button
                      variant="subtle"
                      disabled={locked}
                      type="button"
                      onClick={() => props.onProviderPage(null)}
                    >
                      {c.first}
                    </Button>
                  ) : null}
                  {props.providers.nextAfter ? (
                    <Button
                      variant="subtle"
                      disabled={locked}
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
            </Stack>
          ) : step === 1 ? (
            <Stack gap="xl">
              <Box
                component="fieldset"
                disabled={locked}
                style={{ border: 0, padding: 0, margin: 0, minWidth: 0 }}
              >
                <ConnectionPanel
                  {...props.authenticationPanel}
                  basePath={props.basePath}
                />
              </Box>
            </Stack>
          ) : (
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
              <Stack gap="lg">
                <ProfileImageChoices
                  basePath={props.basePath}
                  label={p.image}
                  images={imageOptions}
                  selected={props.image.id}
                  disabled={locked || props.imagesUnavailable}
                  onChange={props.onImage}
                />
                {props.imageAfter || props.images.nextAfter ? (
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
                        onClick={() =>
                          props.onImagePage(props.images.nextAfter)
                        }
                      >
                        {c.next}
                      </Button>
                    ) : null}
                  </Group>
                ) : null}
                <TextInput
                  label={c.name}
                  value={props.name}
                  onChange={(event) => props.onName(event.currentTarget.value)}
                  required
                  maxLength={120}
                  disabled={locked}
                />
                <TextInput
                  label={p.handle}
                  description={p.permanent}
                  inputWrapperOrder={["label", "input", "description", "error"]}
                  value={props.handle.draft.candidate}
                  autoCapitalize="none"
                  autoCorrect="off"
                  autoComplete="off"
                  spellCheck={false}
                  inputMode="text"
                  maxLength={16}
                  required
                  disabled={locked}
                  onChange={(event) =>
                    props.onHandle(event.currentTarget.value)
                  }
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
                {props.handle.status === "checking" ||
                props.handle.status === "available" ? (
                  <Text size="xs" c="dimmed" aria-live="polite">
                    {props.handle.status === "checking"
                      ? p.checking
                      : p.available}
                  </Text>
                ) : null}
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
              </Stack>
            </Box>
          )}
          {step > 0 &&
          (props.providers.type === "error" ||
            (props.providers.type === "loaded" && props.providers.stale)) ? (
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
          <Group
            justify="space-between"
            className={[
              classes.creationFooter,
              step < 2 ? classes.setupFooter : "",
            ].join(" ")}
          >
            {step === 0 ? (
              <Anchor href={listHref} size="sm" c="dimmed">
                {c.cancel}
              </Anchor>
            ) : (
              <Button
                variant="subtle"
                type="button"
                disabled={locked}
                onClick={props.onBack}
              >
                {a.previous}
              </Button>
            )}
            <Button
              type="submit"
              className={step === 2 ? classes.primary : void 0}
              rightSection={<NavigationIcon name="forward" size="sm" />}
              loading={props.pending}
              disabled={
                connectionOpen ||
                (!props.creationLocked &&
                  (step === 0
                    ? !providerValid
                    : step === 1
                      ? !props.authentication
                      : !profileValid))
              }
            >
              {props.creationLocked
                ? c.recoverCreation
                : step === 2
                  ? c.create
                  : a.next}
            </Button>
          </Group>
        </Stack>
      </AgentTheme>
    </ApplicationShell>
  );
}
