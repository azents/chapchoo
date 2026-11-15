import { Alert, Button, Collapse, Select, Stack, Text } from "@mantine/core";
import classes from "./InstructionEvidence.module.css";
import type { InstructionEvidenceProps } from "../instructionEvidenceTypes";
import type { InstructionSourceRole } from "@chapchoo/web-core/instruction-evidence";
import type { ReactElement } from "react";

const sourceRoles: InstructionSourceRole[] = [
  "product",
  "function",
  "owner",
  "execution",
];
export function InstructionEvidence(
  props: InstructionEvidenceProps,
): ReactElement {
  const { copy: c, state } = props;
  const ready = state.type === "ready" ? state : null;
  return (
    <Stack gap="sm" mt="md">
      <Button
        variant="default"
        fullWidth
        aria-expanded={props.opened}
        onClick={props.onToggle}
      >
        {c.title}
      </Button>
      <Collapse expanded={props.opened}>
        <Stack gap="lg" role="region" aria-label={c.title}>
          {state.type === "loading" ? (
            <Text role="status">{c.loading}</Text>
          ) : null}
          {state.type === "empty" ? (
            <Text size="sm" c="dimmed">
              {c.empty}
            </Text>
          ) : null}
          {state.type === "error" || ready?.evidence.type === "error" ? (
            <Alert color="red" role="alert">
              <Stack gap="sm">
                <Text>{c.error}</Text>
                <Button variant="default" onClick={props.onRead}>
                  {c.retry}
                </Button>
              </Stack>
            </Alert>
          ) : null}
          {ready ? (
            <Select
              label={c.execution}
              data={ready.options}
              value={ready.selected}
              allowDeselect={false}
              onChange={(value) => {
                if (value) {
                  props.onSelect(value);
                }
              }}
            />
          ) : null}
          {ready?.more.read === "error" ? (
            <Alert color="red" role="alert">
              {c.moreError}
            </Alert>
          ) : null}
          {ready?.more.available ? (
            <Button
              variant="default"
              loading={ready.more.read === "loading"}
              disabled={ready.more.read === "loading"}
              onClick={props.onMore}
            >
              {c.more}
            </Button>
          ) : null}
          {ready?.evidence.type === "loading" ? (
            <Text role="status">{c.loading}</Text>
          ) : null}
          {ready?.evidence.type === "ready" ? (
            <Text size="sm" role="status">
              {ready.evidence.value.appliedAt === void 0
                ? c.prepared
                : c.supplied}
            </Text>
          ) : null}
          {ready?.evidence.type === "ready"
            ? sourceRoles.map((role) => {
                const matching =
                  ready.evidence.type === "ready"
                    ? ready.evidence.value.sources.filter(
                        (source) => source.role === role,
                      )
                    : [];
                const label =
                  role === "execution" ? c.executionSource : c[role];
                return (
                  <section key={role} aria-label={label}>
                    <Text size="sm" fw={600}>
                      {label}
                    </Text>
                    {matching.length ? (
                      matching.map((source, index) => (
                        <pre key={index} className={classes.source}>
                          {source.text}
                        </pre>
                      ))
                    ) : (
                      <Text size="sm" c="dimmed">
                        {c.absent}
                      </Text>
                    )}
                  </section>
                );
              })
            : null}
        </Stack>
      </Collapse>
    </Stack>
  );
}
