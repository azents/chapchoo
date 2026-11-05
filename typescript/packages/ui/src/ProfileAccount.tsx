import { Button, Stack, Text } from "@mantine/core";
import { ApplicationShell } from "./ApplicationShell";
import type { ApplicationShellProps } from "./ApplicationShell";
import type { ReactElement } from "react";

export interface ProfileAccountProps extends Omit<
  ApplicationShellProps,
  "children" | "actions"
> {
  emailLabel: string;
  logoutLabel: string;
  onLogout: () => void;
}
export function ProfileAccount({
  emailLabel,
  logoutLabel,
  onLogout,
  ...shell
}: ProfileAccountProps): ReactElement {
  return (
    <ApplicationShell {...shell}>
      <Stack gap="sm">
        <Text size="sm" c="dimmed">
          {emailLabel}
        </Text>
        <Text>{shell.identityLabel}</Text>
      </Stack>
      <Button variant="subtle" color="red" onClick={onLogout}>
        {logoutLabel}
      </Button>
    </ApplicationShell>
  );
}
