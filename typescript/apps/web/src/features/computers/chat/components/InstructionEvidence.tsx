import {
  ApplicationShell,
  WorkspaceHeading,
} from "@chapchoo/ui/ApplicationShell";
import { agentChatPath, agentsPath } from "@chapchoo/web-core/workspace";
import { Anchor, Stack, Text, Title } from "@mantine/core";
import classes from "./InstructionEvidence.module.css";
import type { InstructionEvidenceProps } from "../instructionEvidenceTypes";
import type { InstructionSourceRole } from "@chapchoo/web-core/instruction-evidence";
import type { ReactElement } from "react";

const roles: InstructionSourceRole[] = [
  "product",
  "function",
  "owner",
  "execution",
];
export function InstructionEvidence(
  props: InstructionEvidenceProps,
): ReactElement {
  const { copy: c, prompt } = props;
  const back = agentChatPath(
    props.basePath,
    props.teamHandle,
    props.agentHandle,
  );
  return (
    <ApplicationShell
      basePath={props.basePath}
      appearance={props.appearance}
      contextLabel={props.agentsLabel}
      title={c.title}
      identityLabel={props.identity}
      mascotReplayLabel={props.replayLabel}
      navigation={[
        {
          key: "agents",
          label: props.agentsLabel,
          active: true,
          href: agentsPath(props.basePath, props.teamHandle),
        },
      ]}
      header={
        <Stack gap="lg">
          <Anchor href={back}>{c.back}</Anchor>
          <WorkspaceHeading title={c.title} />
        </Stack>
      }
    >
      <Stack gap="xl" maw="60rem">
        {prompt ? (
          <>
            <Text size="sm" c="dimmed" role="status">
              {prompt.appliedAt === void 0 ? c.prepared : c.supplied}
            </Text>
            {roles.map((role) => {
              const instructions = prompt.sources.filter(
                (item) => item.role === role,
              );
              if (!instructions.length) {
                return null;
              }
              return (
                <section key={role} aria-label={c[role]}>
                  <Title order={2} size="h4">
                    {c[role]}
                  </Title>
                  {instructions.map((item, index) => (
                    <pre key={index} className={classes.source}>
                      {item.text}
                    </pre>
                  ))}
                </section>
              );
            })}
          </>
        ) : (
          <Text size="sm" c="dimmed">
            {c.empty}
          </Text>
        )}
      </Stack>
    </ApplicationShell>
  );
}
