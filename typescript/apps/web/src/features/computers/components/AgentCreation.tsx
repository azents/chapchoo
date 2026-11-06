"use client";
import { ApplicationShell } from "@chapchoo/ui/ApplicationShell";
import { InteractiveSurface } from "@chapchoo/ui/InteractiveSurface";
import { NavigationIcon } from "@chapchoo/ui/NavigationIcon";
import { PageFailure } from "@chapchoo/ui/PageFailure";
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
import { useFormatter } from "next-intl";
import { AuthenticationDialog } from "@/shared/authentications/AuthenticationDialog";
import { CodexMark } from "@/shared/authentications/CodexMark";
import { ConnectionPanel } from "@/shared/authentications/ConnectionPanel";
import { profileStyle } from "../profilePalette";
import { ProfileImageChoices, ProfilePaletteChoices } from "./ProfileChoices";
import classes from "./ProfilePresentation.module.css";
import { ProfilePreview } from "./ProfilePreview";
import type { AgentCreationProps } from "../creationTypes";
import type { ReactElement } from "react";

export function AgentCreation(props: AgentCreationProps): ReactElement {
  const { copy: c, profileCopy: p, authenticationCopy: a } = props;
  const format = useFormatter();
  const locked = props.pending || props.creationLocked;
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
            <Group gap="xs">
              <NavigationIcon name="cloud" />
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
      <Stack
        component="form"
        style={profileStyle(props.paletteKey)}
        onSubmit={(event) => {
          event.preventDefault();
          if (props.step === "profile") {
            props.onCreate();
          } else {
            props.onNext();
          }
        }}
      >
        {step === 0 ? (
          <Stack gap="lg">
            <Group gap="xs">
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
                    <NavigationIcon name="cloud" />
                    <Text component="span" fw={600}>
                      {item.name}
                    </Text>
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
            <Group gap="sm">
              <CodexMark basePath={props.basePath} />
              <Text>{a.codex}</Text>
            </Group>
            <Text c="dimmed" size="sm">
              {a.choose}
            </Text>
            <Group
              justify="space-between"
              wrap="nowrap"
              className={classes.authenticationRow}
            >
              <Stack gap="xs" style={{ minWidth: 0 }}>
                <Text style={{ overflowWrap: "anywhere" }}>
                  {props.authentication
                    ? props.authentication.name ||
                      format.dateTime(props.authentication.createdAt, {
                        dateStyle: "medium",
                        timeStyle: "short",
                      })
                    : a.unselected}
                </Text>
                {props.authentication?.email ? (
                  <Text
                    c="dimmed"
                    size="sm"
                    style={{ overflowWrap: "anywhere" }}
                  >
                    {props.authentication.email}
                  </Text>
                ) : null}
              </Stack>
              <Button
                variant="default"
                disabled={locked}
                onClick={props.onAuthenticationPicker}
              >
                {props.authentication ? a.change : a.choose}
              </Button>
            </Group>
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
                      onClick={() => props.onImagePage(props.images.nextAfter)}
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
              !props.creationLocked &&
              (step === 0
                ? !providerValid
                : step === 1
                  ? !props.authentication
                  : !profileValid)
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
      <AuthenticationDialog
        mobile={props.authenticationPanel.mobile}
        opened={props.authenticationPickerOpen}
        onClose={props.onCloseAuthenticationPicker}
        title={a.choose}
        size="lg"
        locked={props.authenticationPanel.dialog.type !== "closed"}
        closeLabel={a.close}
      >
        <ConnectionPanel
          {...props.authenticationPanel}
          basePath={props.basePath}
        />
      </AuthenticationDialog>
    </ApplicationShell>
  );
}
