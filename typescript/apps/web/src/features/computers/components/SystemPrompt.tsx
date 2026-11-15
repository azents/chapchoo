import { InteractiveSurface } from "@chapchoo/ui/InteractiveSurface";
import { NavigationIcon } from "@chapchoo/ui/NavigationIcon";
import {
  Alert,
  Box,
  Button,
  Group,
  Modal,
  Paper,
  Stack,
  Text,
  Textarea,
} from "@mantine/core";
import classes from "./SystemPrompt.module.css";
import type { SystemPromptProps } from "../systemPromptTypes";
import type { ReactElement } from "react";

export function SystemPrompt(props: SystemPromptProps): ReactElement {
  const { copy: c, editor } = props;
  const saving = editor.type === "saving";
  const content = editor.type === "editing" || saving ? editor.content : null;
  return (
    <>
      <Paper variant="row">
        <InteractiveSurface
          className={classes.row}
          onClick={props.onOpen}
          disabled={props.unavailable}
          aria-label={c.edit}
        >
          <Text component="span" size="sm">
            {c.title}
          </Text>
          <NavigationIcon name="forward" size="sm" />
        </InteractiveSurface>
      </Paper>
      <Modal
        opened={editor.type !== "closed"}
        onClose={props.onClose}
        title={c.title}
        closeButtonProps={{ "aria-label": c.close, disabled: saving }}
        closeOnClickOutside={!saving}
        closeOnEscape={!saving}
        centered
        fullScreen={props.mobile}
        size="48rem"
        padding="xl"
        classNames={{ body: classes.body, content: classes.content }}
      >
        {editor.type === "loading" ? (
          <Text role="status">{c.loading}</Text>
        ) : null}
        {editor.type === "readError" ? (
          <Stack gap="md">
            <Alert color="red" role="alert">
              {c.readFailed}
            </Alert>
            <Button variant="default" onClick={props.onRead}>
              {c.retry}
            </Button>
          </Stack>
        ) : null}
        {content !== null ? (
          <Box
            component="form"
            className={classes.form}
            onSubmit={(event) => {
              event.preventDefault();
              props.onSave();
            }}
          >
            <Textarea
              label={c.field}
              value={content}
              onChange={(event) => props.onChange(event.currentTarget.value)}
              disabled={saving || props.unavailable}
              autosize={false}
              rows={16}
              classNames={{ root: classes.field, input: classes.input }}
              error={editor.type === "editing" ? editor.error : null}
              data-autofocus
            />
            <Group justify="flex-end" className={classes.footer}>
              <Button
                type="submit"
                loading={saving}
                disabled={saving || props.unavailable}
              >
                {c.save}
              </Button>
            </Group>
          </Box>
        ) : null}
      </Modal>
    </>
  );
}
