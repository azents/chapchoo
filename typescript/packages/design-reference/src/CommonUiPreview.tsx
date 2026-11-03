import { EmptyFeedback } from "@chapchoo/ui/Feedback";
import { FontFaces } from "@chapchoo/ui/FontFaces";
import { AppMantineProvider } from "@chapchoo/ui/provider";
import { AppHeader, HeaderControlsContext } from "@chapchoo/ui/AppHeader";
import { DisplayEntry, DisplayPanel } from "@chapchoo/ui/DisplaySettings";
import { useState } from "react";
import { Mascot } from "@chapchoo/ui/Mascot";
import { NavigationIcon } from "@chapchoo/ui/NavigationIcon";
import { InteractiveSurface } from "@chapchoo/ui/InteractiveSurface";
import { DropdownButton } from "@chapchoo/ui/DropdownButton";
import {
  Button,
  Chip,
  Group,
  Stack,
  TextInput,
  Select,
  Paper,
  Text,
  Table,
  NavLink,
  rem,
} from "@mantine/core";
import type { ReactElement } from "react";
import "@mantine/core/styles.css";

/** Loaded only by the isolated reuse document, never by neutral story canvases. */
export default function CommonUiPreview(): ReactElement {
  const [opened, setOpened] = useState(false);
  const [workspaceOpened, setWorkspaceOpened] = useState(false);
  const [value, setValue] = useState({
    language: "en",
    timeZone: "UTC",
    region: "en-US",
  });
  const copy = {
    title: "Display settings",
    language: "Language",
    timeZone: "Time zone",
    region: "Regional format",
    follow: "Follow browser settings",
    preview: "Display preview",
    unavailable: "This environment cannot display the selected time zone.",
    scope: "Reference-local interaction; no persistence.",
    done: "Done",
    close: "Close settings",
    immediate: "Changes apply immediately.",
    saved: "Applied",
    error: "Saving could not be confirmed.",
    storageError: "Storage is unavailable.",
    retry: "Try again",
  };
  const mode = new URLSearchParams(window.location.search).has("dark")
    ? "dark"
    : "light";
  return (
    <AppMantineProvider forceColorScheme={mode}>
      <FontFaces basePath="" />
      <HeaderControlsContext.Provider
        value={
          <DisplayEntry
            label={copy.title}
            onOpen={(): void => setOpened(true)}
          />
        }
      >
        <AppHeader
          appearance={{
            value: mode,
            label: "Appearance",
            labels: { system: "System", light: "Light", dark: "Dark" },
            onChange: (): void => {},
          }}
        />
      </HeaderControlsContext.Provider>
      <DisplayPanel
        opened={opened}
        onClose={(): void => setOpened(false)}
        value={value}
        selected={{}}
        automatic={false}
        disabled={false}
        zones={["UTC", "Asia/Seoul", "America/New_York"]}
        copy={copy}
        onChange={(key, next): void => {
          if (next) {
            setValue((previous) => ({ ...previous, [key]: next }));
          }
        }}
        changed={false}
        error={false}
        storageError={false}
        onRetry={(): void => {}}
        instant={1793428200000}
      />
      <Stack p="lg">
        <Stack gap="xs" maw="18rem">
          <NavLink label="Agents" active />
          <NavLink label="Invitations" />
        </Stack>
        <InteractiveSurface href="#profile-example">
          <Group justify="space-between">
            <Text>Profile and account</Text>
            <NavigationIcon name="forward" size="sm" />
          </Group>
        </InteractiveSurface>
        <InteractiveSurface disabled>Unavailable action</InteractiveSurface>
        <Mascot size={72} />
        <EmptyFeedback
          title="Shared presentation example"
          message="The implementation is imported from @chapchoo/ui/Feedback. Changes to it belong in common UI."
        />
        <Group>
          <Button variant="subtle" leftSection={<NavigationIcon name="back" />}>
            Back
          </Button>
          <DropdownButton
            accessibleLabel="Choose workspace"
            opened={workspaceOpened}
            onChange={setWorkspaceOpened}
            label={
              <Group gap="sm">
                <Text>Workspace</Text>
                <NavigationIcon name="down" size="sm" />
              </Group>
            }
          >
            <Button variant="subtle" onClick={() => setWorkspaceOpened(false)}>
              Personal space
            </Button>
          </DropdownButton>
          <Button>Log in</Button>
          <Button variant="subtle" size="xs">
            Log out
          </Button>
          <Chip defaultChecked>Typography gyp 한글</Chip>
        </Group>
        <Group align="end">
          <TextInput label="Email" defaultValue="member@example.com" />
          <Select
            label="Space"
            data={["Personal", "Team"]}
            defaultValue="Personal"
          />
          <Button>Save</Button>
        </Group>
        <Paper variant="row">
          <Text fw={600}>Personal space</Text>
          <Text size="sm" c="dimmed">
            A divider row keeps the surrounding content baseline.
          </Text>
        </Paper>
        <Table.ScrollContainer minWidth={rem(480)}>
          <Table withTableBorder>
            <Table.Thead>
              <Table.Tr>
                <Table.Th>Member</Table.Th>
                <Table.Th>Role</Table.Th>
              </Table.Tr>
            </Table.Thead>
            <Table.Tbody>
              <Table.Tr>
                <Table.Td>member@example.com</Table.Td>
                <Table.Td>Owner</Table.Td>
              </Table.Tr>
            </Table.Tbody>
          </Table>
        </Table.ScrollContainer>
      </Stack>
    </AppMantineProvider>
  );
}
