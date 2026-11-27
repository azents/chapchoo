import {
  Alert,
  Box,
  Button,
  Divider,
  Group,
  Modal,
  NativeSelect,
  Paper,
  Stack,
  Text,
  TextInput,
  Title,
} from "@mantine/core";
import type { McpServersProps } from "../mcpServerTypes";
import type { ReactElement } from "react";

export function McpServers(props: McpServersProps): ReactElement | null {
  const { copy: c, list, editor, write } = props;
  if (list.type === "hidden") {
    return null;
  }
  const locked = write !== "idle";
  const dismissible = write !== "saving" && write !== "checking";
  const reconciliation = (
    <Stack gap="sm">
      <Alert color="yellow" role="status">
        {c.uncertain}
      </Alert>
      <Group justify="flex-end">
        <Button
          type="button"
          onClick={props.onCheck}
          loading={write === "checking"}
        >
          {c.check}
        </Button>
      </Group>
    </Stack>
  );
  const test = editor?.test;
  const outcome =
    test?.type === "running"
      ? c.testing
      : test?.type === "listed"
        ? c.count(test.count)
        : test?.type === "failed"
          ? c.failures[test.reason]
          : test?.type === "unavailable"
            ? c.testUnavailable
            : null;
  return (
    <Stack component="section" aria-label={c.title} gap="xs" mt="lg">
      <Group justify="space-between">
        <Title order={2} size="sm">
          {c.title}
        </Title>
        {list.type === "loaded" && (
          <Button
            size="xs"
            variant="subtle"
            onClick={() => props.onOpen(null)}
            disabled={locked}
          >
            + {c.add}
          </Button>
        )}
      </Group>
      {list.type === "loading" && (
        <Text size="sm" c="dimmed" role="status">
          {c.loading}
        </Text>
      )}
      {list.type === "unavailable" && (
        <Group>
          <Text size="sm" role="status">
            {c.unavailable}
          </Text>
          <Button variant="subtle" size="xs" onClick={props.onRetry}>
            {c.retry}
          </Button>
        </Group>
      )}
      {list.type === "loaded" && (
        <>
          {list.items.length === 0 && (
            <Text size="sm" c="dimmed">
              {c.empty}
            </Text>
          )}
          {list.items.map((item) => (
            <Paper key={item.id} variant="row" px="sm" py="xs">
              <Group justify="space-between" wrap="nowrap">
                <Box miw={0}>
                  <Text size="sm" fw={500} style={{ overflowWrap: "anywhere" }}>
                    {item.displayName}
                  </Text>
                  <Text
                    size="xs"
                    c="dimmed"
                    style={{ overflowWrap: "anywhere" }}
                  >
                    {item.endpoint}
                  </Text>
                </Box>
                <Button
                  size="xs"
                  variant="default"
                  aria-label={`${c.edit}: ${item.displayName}`}
                  disabled={locked}
                  onClick={() => props.onOpen(item)}
                >
                  {c.edit}
                </Button>
              </Group>
            </Paper>
          ))}
          {(list.more || list.previous) && (
            <Group justify="flex-end">
              {list.previous && (
                <Button
                  size="xs"
                  variant="subtle"
                  disabled={locked}
                  onClick={props.onPrevious}
                >
                  {c.previous}
                </Button>
              )}
              {list.more && (
                <Button
                  size="xs"
                  variant="subtle"
                  disabled={locked}
                  onClick={props.onNext}
                >
                  {c.next}
                </Button>
              )}
            </Group>
          )}
        </>
      )}
      {!editor &&
        (write === "uncertain" || write === "checking") &&
        reconciliation}
      <Modal
        opened={!!editor}
        onClose={props.onClose}
        title={editor?.saved ? c.editTitle : c.addTitle}
        closeButtonProps={{ "aria-label": c.close, disabled: !dismissible }}
        closeOnClickOutside={dismissible}
        closeOnEscape={dismissible}
        fullScreen={props.mobile}
        centered
        size="30rem"
      >
        {editor && (
          <Box
            component="form"
            onSubmit={(event) => {
              event.preventDefault();
              if (!editor.confirming) {
                props.onSave();
              }
            }}
          >
            <Stack gap="sm">
              {write === "uncertain" || write === "checking" ? (
                reconciliation
              ) : editor.confirming ? (
                <>
                  <Text>{c.confirm}</Text>
                  {editor.rejected && (
                    <Alert color="red" role="alert">
                      {c.rejected}
                    </Alert>
                  )}
                  <Group justify="flex-end">
                    <Button
                      type="button"
                      variant="default"
                      disabled={locked}
                      onClick={() => props.onConfirmDelete(false)}
                    >
                      {c.cancel}
                    </Button>
                    <Button
                      type="button"
                      color="red"
                      loading={locked}
                      onClick={props.onDelete}
                    >
                      {c.delete}
                    </Button>
                  </Group>
                </>
              ) : (
                <>
                  <TextInput
                    label={c.name}
                    data-autofocus
                    value={editor.draft.displayName}
                    maxLength={120}
                    disabled={locked}
                    onChange={(event) =>
                      props.onDraft({ displayName: event.currentTarget.value })
                    }
                  />
                  <TextInput
                    label={c.endpoint}
                    type="url"
                    value={editor.draft.endpoint}
                    maxLength={4096}
                    disabled={locked}
                    onChange={(event) =>
                      props.onDraft({ endpoint: event.currentTarget.value })
                    }
                  />
                  <NativeSelect
                    label={c.auth}
                    value={editor.draft.auth}
                    data={[
                      { value: "none", label: c.none },
                      { value: "bearer", label: c.bearer },
                    ]}
                    disabled={locked}
                    onChange={(event) =>
                      props.onDraft({
                        auth:
                          event.currentTarget.value === "bearer"
                            ? "bearer"
                            : "none",
                        bearer: "",
                      })
                    }
                  />
                  {editor.draft.auth === "bearer" && (
                    <TextInput
                      type="password"
                      label={c.bearer}
                      autoComplete="new-password"
                      value={editor.draft.bearer}
                      maxLength={8192}
                      placeholder={props.keepsToken ? c.keep : void 0}
                      disabled={locked}
                      onChange={(event) =>
                        props.onDraft({ bearer: event.currentTarget.value })
                      }
                    />
                  )}
                  <Group gap="sm" wrap="wrap">
                    <Button
                      type="button"
                      variant="default"
                      size="compact-sm"
                      disabled={!props.canTest || test?.type === "running"}
                      onClick={props.onTest}
                    >
                      {c.test}
                    </Button>
                    {outcome && (
                      <Text
                        size="sm"
                        c={
                          test?.type === "failed" ||
                          test?.type === "unavailable"
                            ? "red"
                            : "dimmed"
                        }
                        role="status"
                      >
                        {outcome}
                      </Text>
                    )}
                  </Group>
                  {editor.rejected && (
                    <Alert color="red" role="alert">
                      {c.rejected}
                    </Alert>
                  )}
                  <Divider />
                  <Group justify="space-between">
                    <Box>
                      {editor.saved && (
                        <Button
                          type="button"
                          color="red"
                          variant="subtle"
                          disabled={locked}
                          onClick={() => props.onConfirmDelete(true)}
                        >
                          {c.delete}
                        </Button>
                      )}
                    </Box>
                    <Group gap="xs">
                      <Button
                        type="button"
                        variant="default"
                        disabled={locked}
                        onClick={props.onClose}
                      >
                        {c.cancel}
                      </Button>
                      <Button
                        type="submit"
                        loading={write === "saving"}
                        disabled={!props.canSave}
                      >
                        {c.save}
                      </Button>
                    </Group>
                  </Group>
                </>
              )}
            </Stack>
          </Box>
        )}
      </Modal>
    </Stack>
  );
}
