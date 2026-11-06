import {
  ApplicationShell,
  WorkspaceHeading,
} from "@chapchoo/ui/ApplicationShell";
import { PageFailure } from "@chapchoo/ui/PageFailure";
import { newAgentPath } from "@chapchoo/web-core/workspace";
import { Button, Group } from "@mantine/core";
import { ConnectionPanel } from "@/shared/authentications/ConnectionPanel";
import type { ConnectionsProps } from "./types";
import type { ReactElement } from "react";
export function Connections({
  input,
  panel,
  access,
  onRetry,
}: ConnectionsProps): ReactElement {
  if (!access) {
    return (
      <PageFailure
        title={panel.copy.title}
        message={panel.copy.unavailable}
        retryLabel={panel.copy.retry}
        onRetry={onRetry}
      />
    );
  }
  return (
    <ApplicationShell
      basePath={input.basePath}
      title={panel.copy.title}
      contextLabel={
        panel.scope.type === "personal" ? panel.copy.personal : panel.copy.team
      }
      navigation={[]}
      mascotReplayLabel=""
      identityLabel={input.identity}
      header={
        <WorkspaceHeading
          title={panel.copy.title}
          actions={
            <Group gap="md">
              <Button
                component="a"
                href={newAgentPath(input.basePath, input.teamHandle)}
                variant="subtle"
              >
                {panel.copy.newAgent}
              </Button>
              {panel.canManage ? (
                <Button
                  disabled={panel.unavailable || panel.loading}
                  onClick={() => panel.onDialog({ type: "connect" })}
                >
                  {panel.copy.connect}
                </Button>
              ) : null}
            </Group>
          }
        />
      }
    >
      <ConnectionPanel
        {...panel}
        showConnectAction={false}
        basePath={input.basePath}
      />
    </ApplicationShell>
  );
}
