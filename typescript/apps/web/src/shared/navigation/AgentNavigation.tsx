import { CharacterArtwork } from "@chapchoo/ui/CharacterArtwork";
import { NavigationIcon } from "@chapchoo/ui/NavigationIcon";
import { ActionIcon, Anchor, Group, Loader, Stack, Text } from "@mantine/core";
import classes from "./AgentNavigation.module.css";
import type { AgentView } from "@chapchoo/web-core/computer";
import type { ComputerCopy } from "@chapchoo/web-core/computer-messages";
import type { ReactElement } from "react";

export interface AgentNavigationProps {
  basePath: string;
  copy: ComputerCopy;
  titleHref: string;
  createHref: string;
  createLabel: string;
  allAgentsLabel: string;
  items: readonly AgentView[];
  selectedHandle: string | null;
  agentHref: (handle: string) => string;
  status: "loading" | "ready" | "error";
  onSelect?: () => void;
}

export function AgentNavigation(props: AgentNavigationProps): ReactElement {
  return (
    <Stack gap="sm" mb="lg">
      <Group justify="space-between" gap="sm">
        <Anchor href={props.titleHref} onClick={props.onSelect} size="sm">
          {props.copy.agents}
        </Anchor>
        <ActionIcon
          component="a"
          href={props.createHref}
          variant="subtle"
          aria-label={props.createLabel}
          onClick={props.onSelect}
        >
          <NavigationIcon name="add" />
        </ActionIcon>
      </Group>
      <nav aria-label={props.copy.agents}>
        {props.items.slice(0, 5).map((agent) => (
          <a
            key={agent.id}
            href={props.agentHref(agent.handle)}
            className={classes.agent}
            onClick={props.onSelect}
            aria-current={
              agent.handle === props.selectedHandle ? "page" : void 0
            }
          >
            <CharacterArtwork
              basePath={props.basePath}
              assetKey={agent.profileImage.assetKey}
              className={classes.portrait}
            />
            <span>
              {agent.name}
              <small>@{agent.handle}</small>
            </span>
          </a>
        ))}
        <a
          href={props.titleHref}
          onClick={props.onSelect}
          className={classes.agent}
        >
          <span className={classes.allIcon}>
            <NavigationIcon name="agents" />
          </span>
          <span>{props.allAgentsLabel}</span>
        </a>
      </nav>
      {props.status === "loading" ? (
        <Loader size="xs" aria-label={props.copy.agents} />
      ) : null}
      {props.status === "error" ? (
        <Text size="xs" c="dimmed" role="status">
          {props.copy.stale}
        </Text>
      ) : null}
      {props.status === "ready" && props.items.length === 0 ? (
        <Text size="sm" c="dimmed">
          {props.copy.noAgents}
        </Text>
      ) : null}
    </Stack>
  );
}
