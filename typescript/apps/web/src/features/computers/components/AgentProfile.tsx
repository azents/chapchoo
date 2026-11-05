import {
  ActionIcon,
  Alert,
  Box,
  Button,
  Drawer,
  Group,
  Image,
  Modal,
  Stack,
  Text,
  TextInput,
  Title,
} from "@mantine/core";
import { profileStyle } from "../profilePalette";
import { ProfileImageChoices, ProfilePaletteChoices } from "./ProfileChoices";
import classes from "./ProfilePresentation.module.css";
import { ProfilePreview } from "./ProfilePreview";
import type { AgentsProps } from "../types";
import type { AgentView } from "@chapchoo/web-core/computer";
import type { ReactElement } from "react";

export type AgentProfileProps = Pick<
  AgentsProps,
  | "profileCopy"
  | "mobile"
  | "editor"
  | "images"
  | "imageAfter"
  | "imagesUnavailable"
  | "basePath"
  | "copy"
  | "onEditProfile"
  | "onProfileName"
  | "onProfileImage"
  | "onProfilePalette"
  | "onRandomProfilePalette"
  | "onImagePage"
  | "onRefresh"
  | "onSaveProfile"
  | "onCancelProfile"
> & { agent: AgentView };
/** Saved identity and transient draft have independent presentation ownership. */
export function AgentProfile(props: AgentProfileProps): ReactElement {
  const { agent, profileCopy: p, editor } = props;
  const draft = editor.type === "closed" ? null : editor.draft;
  const busy = editor.type === "saving";
  const options = [...props.images.items];
  if (draft && !options.some((item) => item.id === draft.image.id)) {
    options.unshift({ ...draft.image, internalName: p.currentImage });
  }
  const content = draft ? (
    <Box
      component="form"
      style={profileStyle(draft.paletteKey)}
      onSubmit={(event) => {
        event.preventDefault();
        props.onSaveProfile();
      }}
    >
      <Box className={classes.editorGrid}>
        <Box className={classes.editorPreview}>
          <ProfilePreview
            basePath={props.basePath}
            assetKey={draft.image.assetKey}
            paletteKey={draft.paletteKey}
            name={draft.name || props.copy.name}
            handle={agent.handle}
            label={p.preview}
          />
        </Box>
        <Stack gap="xl">
          <ProfileImageChoices
            basePath={props.basePath}
            label={p.image}
            images={options}
            selected={draft.image.id}
            disabled={busy || props.imagesUnavailable}
            onChange={props.onProfileImage}
          />
          {props.imageAfter || props.images.nextAfter ? (
            <Group>
              {props.imageAfter ? (
                <Button
                  type="button"
                  variant="subtle"
                  disabled={busy}
                  onClick={() => props.onImagePage(null)}
                >
                  {props.copy.first}
                </Button>
              ) : null}
              {props.images.nextAfter ? (
                <Button
                  type="button"
                  variant="subtle"
                  disabled={busy}
                  onClick={() => props.onImagePage(props.images.nextAfter)}
                >
                  {props.copy.next}
                </Button>
              ) : null}
            </Group>
          ) : null}
          {props.imagesUnavailable ? (
            <Alert color="yellow" role="status">
              {props.copy.unavailable}
              <Button type="button" variant="subtle" onClick={props.onRefresh}>
                {props.copy.retry}
              </Button>
            </Alert>
          ) : null}
          <TextInput
            label={p.name}
            value={draft.name}
            onChange={(event) => props.onProfileName(event.currentTarget.value)}
            required
            withAsterisk={false}
            maxLength={120}
            disabled={busy}
            data-autofocus
          />
          <ProfilePaletteChoices
            copy={p}
            selected={draft.paletteKey}
            disabled={busy}
            onChange={props.onProfilePalette}
            onRandom={props.onRandomProfilePalette}
          />
          <Box
            role="group"
            aria-label={p.handle}
            className={classes.handleNote}
          >
            <Text size="xs">@{agent.handle}</Text>
            <Text size="xs" c="dimmed">
              {p.handlePreserved}
            </Text>
          </Box>
        </Stack>
      </Box>
      {editor.type === "editing" && editor.error ? (
        <Alert color="red" role="alert" mt="lg">
          {editor.error}
        </Alert>
      ) : null}
      <Group justify="flex-end" className={classes.editorFooter}>
        <Button
          type="button"
          variant="default"
          disabled={busy}
          onClick={props.onCancelProfile}
        >
          {props.copy.cancel}
        </Button>
        <Button
          type="submit"
          className={classes.primary}
          loading={busy}
          disabled={!draft.name.trim()}
        >
          {p.save}
        </Button>
      </Group>
    </Box>
  ) : null;
  const dialog = {
    opened: draft !== null,
    onClose: props.onCancelProfile,
    title: p.edit,
    closeOnClickOutside: !busy,
    closeOnEscape: !busy,
    closeButtonProps: { disabled: busy, "aria-label": p.close },
    overlayProps: { backgroundOpacity: 0.35, blur: 4 },
    classNames: { header: classes.editorHeader, title: classes.editorTitle },
  };
  return (
    <>
      <Box
        component="section"
        className={classes.hero}
        style={profileStyle(agent.paletteKey)}
      >
        <Image
          src={`${props.basePath}/design-assets/${agent.profileImage.assetKey}-216.webp`}
          alt=""
          className={classes.heroPortrait}
        />
        <Box className={classes.heroIdentity}>
          <Title order={1} className={classes.heroName}>
            {agent.name}
          </Title>
          <Text className={classes.previewHandle}>@{agent.handle}</Text>
          <Text size="sm" mt="lg">
            {props.copy[agent.computer.connector]}
          </Text>
        </Box>
        <ActionIcon
          className={classes.heroEdit}
          variant="subtle"
          color="dark"
          aria-label={p.edit}
          onClick={props.onEditProfile}
          size="lg"
        >
          <svg
            viewBox="0 0 24 24"
            width="1.25rem"
            height="1.25rem"
            fill="none"
            stroke="currentColor"
            strokeWidth="1.5"
            aria-hidden="true"
          >
            <path d="m16 3 5 5-12 12-6 1 1-6zM14 5l5 5" />
          </svg>
        </ActionIcon>
      </Box>
      {props.mobile ? (
        <Drawer
          {...dialog}
          padding="xl"
          position="bottom"
          size="auto"
          styles={{
            content: {
              maxHeight: "92dvh",
              display: "flex",
              flexDirection: "column",
              borderTopLeftRadius: "1.5rem",
              borderTopRightRadius: "1.5rem",
            },
            body: { flex: 1, overflowY: "auto" },
          }}
        >
          {content}
        </Drawer>
      ) : (
        <Modal {...dialog} size="50rem" centered padding="xl">
          {content}
        </Modal>
      )}
    </>
  );
}
