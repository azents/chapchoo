import { EmptyFeedback } from "@chapchoo/ui/Feedback";
import { FontFaces } from "@chapchoo/ui/FontFaces";
import { AppMantineProvider } from "@chapchoo/ui/provider";
import { AppHeader } from "@chapchoo/ui/AppHeader";
import { Mascot } from "@chapchoo/ui/Mascot";
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
  rem,
} from "@mantine/core";
import type { ReactElement } from "react";
import "@mantine/core/styles.css";

/** Loaded only by the isolated reuse document, never by neutral story canvases. */
export default function CommonUiPreview(): ReactElement {
  const mode = new URLSearchParams(window.location.search).has("dark")
    ? "dark"
    : "light";
  return (
    <AppMantineProvider forceColorScheme={mode}>
      <FontFaces basePath="" />
      <AppHeader
        appearance={{
          value: mode,
          label: "Appearance",
          labels: { system: "System", light: "Light", dark: "Dark" },
          onChange: (): void => {},
        }}
      />
      <Stack p="lg">
        <Mascot size={72} />
        <EmptyFeedback
          title="Shared presentation example"
          message="The implementation is imported from @chapchoo/ui/Feedback. Changes to it belong in common UI."
        />
        <Group>
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
