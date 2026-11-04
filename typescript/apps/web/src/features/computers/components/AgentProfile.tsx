import { agentPaletteKeys } from "@chapchoo/web-core/agent-profile";
import {
  Alert,
  Button,
  Drawer,
  Group,
  Modal,
  Paper,
  Select,
  SimpleGrid,
  Stack,
  Text,
  TextInput,
} from "@mantine/core";
import { profilePalette } from "../profilePalette";
import { AgentPortrait } from "./AgentPortrait";
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
  | "onImagePage"
  | "onRefresh"
  | "onSaveProfile"
  | "onCancelProfile"
> & { agent: AgentView };
/** The saved identity and transient modal draft have independent render ownership. */
export function AgentProfile(props: AgentProfileProps): ReactElement {
  const { agent, profileCopy: p, editor } = props;
  const draft = editor.type === "closed" ? null : editor.draft;
  const busy = editor.type === "saving";
  const saved = profilePalette[agent.paletteKey];
  const options = props.images.items.map((image) => ({
    value: image.id,
    label: image.internalName,
  }));
  if (draft && !options.some((item) => item.value === draft.image.id)) {
    options.unshift({ value: draft.image.id, label: p.currentImage });
  }
  const content = draft ? (
    <Stack
      component="form"
      onSubmit={(event) => {
        event.preventDefault();
        props.onSaveProfile();
      }}
    >
      <SimpleGrid cols={{ base: 1, sm: 2 }} spacing="lg">
        <Paper
          p="lg"
          radius="lg"
          bg={profilePalette[draft.paletteKey].surface}
          c={profilePalette[draft.paletteKey].foreground}
        >
          <Stack align="center">
            <AgentPortrait
              basePath={props.basePath}
              assetKey={draft.image.assetKey}
              paletteKey={draft.paletteKey}
            />
            <Text
              fw={600}
              size="xl"
              ta="center"
              style={{ overflowWrap: "anywhere" }}
            >
              {draft.name || props.copy.name}
            </Text>
            <Text size="sm" c={profilePalette[draft.paletteKey].emphasis}>
              @{agent.handle}
            </Text>
          </Stack>
        </Paper>
        <Stack>
          <TextInput
            label={props.copy.name}
            value={draft.name}
            onChange={(event) => props.onProfileName(event.currentTarget.value)}
            required
            maxLength={120}
            disabled={busy}
            data-autofocus
          />
          <TextInput
            label={p.handle}
            description={p.permanent}
            value={agent.handle}
            readOnly
          />
          <Select
            label={p.image}
            value={draft.image.id}
            data={options}
            disabled={busy || props.imagesUnavailable}
            onChange={(value) => {
              if (value) {
                props.onProfileImage(value);
              }
            }}
          />
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
          {props.imagesUnavailable ? (
            <Alert color="yellow" role="status">
              {props.copy.unavailable}
              <Button type="button" variant="subtle" onClick={props.onRefresh}>
                {props.copy.retry}
              </Button>
            </Alert>
          ) : null}
          <Select
            label={p.palette}
            value={draft.paletteKey}
            data={agentPaletteKeys.map((key) => ({
              value: key,
              label: p.palettes[key],
            }))}
            disabled={busy}
            onChange={(value) => {
              if (value) {
                props.onProfilePalette(value);
              }
            }}
          />
        </Stack>
      </SimpleGrid>
      {editor.type === "editing" && editor.error ? (
        <Alert color="red" role="alert">
          {editor.error}
        </Alert>
      ) : null}
      <Group justify="flex-end">
        <Button
          type="button"
          variant="subtle"
          disabled={busy}
          onClick={props.onCancelProfile}
        >
          {props.copy.cancel}
        </Button>
        <Button type="submit" loading={busy} disabled={!draft.name.trim()}>
          {p.save}
        </Button>
      </Group>
    </Stack>
  ) : null;
  const dialog = {
    opened: draft !== null,
    onClose: props.onCancelProfile,
    title: p.edit,
    closeOnClickOutside: !busy,
    closeOnEscape: !busy,
    closeButtonProps: { disabled: busy, "aria-label": p.close },
  };
  return (
    <>
      <Paper p="lg" radius="lg" bg={saved.surface} c={saved.foreground}>
        <Group justify="space-between" align="center">
          <Group
            wrap="nowrap"
            flex={{ base: "1 0 100%", sm: 1 }}
            style={{ minWidth: 0 }}
          >
            <AgentPortrait
              basePath={props.basePath}
              assetKey={agent.profileImage.assetKey}
              paletteKey={agent.paletteKey}
            />
            <Stack gap="xs" style={{ minWidth: 0 }}>
              <Text fw={600} size="xl" style={{ overflowWrap: "anywhere" }}>
                {agent.name}
              </Text>
              <Text c={saved.emphasis}>@{agent.handle}</Text>
            </Stack>
          </Group>
          <Button variant="default" onClick={props.onEditProfile}>
            {p.edit}
          </Button>
        </Group>
      </Paper>
      {props.mobile ? (
        <Drawer
          {...dialog}
          position="bottom"
          size="auto"
          styles={{
            content: {
              maxHeight: "90dvh",
              display: "flex",
              flexDirection: "column",
              borderTopLeftRadius: "var(--mantine-radius-lg)",
              borderTopRightRadius: "var(--mantine-radius-lg)",
            },
            body: { flex: 1, overflowY: "auto" },
          }}
        >
          {content}
        </Drawer>
      ) : (
        <Modal {...dialog} size="lg" centered>
          {content}
        </Modal>
      )}
    </>
  );
}
