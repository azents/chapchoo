import { sameHarnessSupply } from "@chapchoo/web-core/conversation-supply";
import { harnessSupplyChoice } from "@chapchoo/web-core/conversation-types";
import {
  Alert,
  Button,
  Collapse,
  Group,
  Radio,
  Stack,
  Text,
  TextInput,
} from "@mantine/core";
import type { SupplyProps } from "../supplyTypes";
import type { HarnessSupplyChoice } from "@chapchoo/web-core/conversation-types";
import type { ReactElement } from "react";

function choiceLabel(choice: HarnessSupplyChoice, managed: string): string {
  return choice.type === "managed" ? managed : choice.path;
}
export function Supply({
  copy,
  opened,
  state,
  onToggle,
  onMode,
  onPath,
  onSave,
  onRead,
}: SupplyProps): ReactElement {
  const ready = state.type === "ready" ? state : null;
  const locked =
    !ready?.writable ||
    ready.submission === "saving" ||
    ready.submission === "uncertain";
  const changed =
    ready?.applied && !sameHarnessSupply(ready.saved.choice, ready.applied);
  return (
    <Stack gap="sm" mt="md">
      <Button
        variant="default"
        fullWidth
        aria-expanded={opened}
        onClick={onToggle}
      >
        {copy.title}
      </Button>
      <Collapse expanded={opened}>
        <Stack gap="md" role="region" aria-label={copy.title}>
          {state.type === "loading" ? (
            <Text role="status">{copy.loading}</Text>
          ) : state.type === "error" ? (
            <Group role="alert">
              <Text>{copy.error}</Text>
              <Button variant="subtle" onClick={onRead}>
                {copy.read}
              </Button>
            </Group>
          ) : (
            <>
              <div>
                <Text size="sm" fw={600}>
                  {copy.saved}
                </Text>
                <Text
                  style={{ overflowWrap: "anywhere", whiteSpace: "pre-wrap" }}
                >
                  {choiceLabel(state.saved.choice, copy.managed)}
                </Text>
              </div>
              <div>
                <Text size="sm" fw={600}>
                  {copy.applied}
                </Text>
                <Text
                  style={{ overflowWrap: "anywhere", whiteSpace: "pre-wrap" }}
                >
                  {state.applied
                    ? choiceLabel(state.applied, copy.managed)
                    : copy.unknown}
                </Text>
              </div>
              {changed ? (
                <Alert color="blue">{copy.nextStart}</Alert>
              ) : (
                <Text size="sm" c="dimmed">
                  {copy.nextStart}
                </Text>
              )}
              <Radio.Group
                label={copy.title}
                value={state.draft.type}
                onChange={onMode}
              >
                <Stack gap="xs" mt="xs">
                  <Radio
                    value="managed"
                    label={copy.managed}
                    disabled={locked}
                  />
                  <Radio
                    value="existing"
                    label={copy.existing}
                    disabled={locked}
                  />
                </Stack>
              </Radio.Group>
              {state.draft.type === "existing" ? (
                <TextInput
                  label={copy.path}
                  value={state.draft.path}
                  error={
                    state.draft.path &&
                    !harnessSupplyChoice.safeParse(state.draft).success
                      ? copy.invalidPath
                      : void 0
                  }
                  onChange={(event) => onPath(event.currentTarget.value)}
                  disabled={locked}
                  autoComplete="off"
                  spellCheck={false}
                />
              ) : null}
              <Group>
                <Button
                  onClick={onSave}
                  disabled={!state.saveable || locked}
                  loading={state.submission === "saving"}
                >
                  {copy.save}
                </Button>
                <Button
                  variant="subtle"
                  onClick={onRead}
                  disabled={state.submission === "saving"}
                >
                  {copy.read}
                </Button>
              </Group>
              {state.stale ? <Text role="alert">{copy.error}</Text> : null}
              {state.submission === "saved" ? (
                <Text role="status">{copy.savedNotice}</Text>
              ) : state.submission === "uncertain" ? (
                <Text role="alert">{copy.uncertain}</Text>
              ) : state.submission === "rejected" ? (
                <Text role="alert">{copy.rejected}</Text>
              ) : null}
            </>
          )}
        </Stack>
      </Collapse>
    </Stack>
  );
}
