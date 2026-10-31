import { Button, Table, rem } from "@mantine/core";
import { ApplicationShell } from "./ApplicationShell";
import { EmptyFeedback } from "./Feedback";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";

const meta = {
  title: "Shared UI/ApplicationShell",
  component: ApplicationShell,
  parameters: { layout: "fullscreen" },
  args: {
    contextLabel: "Administration",
    title: "Signup grants",
    identityLabel: "administrator@example.com",
    navigation: [
      {
        key: "signup",
        label: "Signup grants",
        active: true,
        onSelect: (): void => {},
      },
      { key: "reset", label: "Password resets", onSelect: (): void => {} },
    ],
    actions: (
      <Button variant="default" size="xs">
        Sign out
      </Button>
    ),
  },
} satisfies Meta<typeof ApplicationShell>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Loaded: Story = {
  args: {
    children: (
      <Table.ScrollContainer minWidth={rem(640)}>
        <Table striped highlightOnHover withTableBorder>
          <Table.Thead>
            <Table.Tr>
              <Table.Th>Email</Table.Th>
              <Table.Th>Issued</Table.Th>
              <Table.Th>Expires</Table.Th>
              <Table.Th>Status</Table.Th>
            </Table.Tr>
          </Table.Thead>
          <Table.Tbody>
            <Table.Tr>
              <Table.Td>member@example.com</Table.Td>
              <Table.Td>Oct 30, 2026</Table.Td>
              <Table.Td>Nov 6, 2026</Table.Td>
              <Table.Td>Available</Table.Td>
            </Table.Tr>
          </Table.Tbody>
        </Table>
      </Table.ScrollContainer>
    ),
  },
};
export const Empty: Story = {
  args: {
    children: (
      <EmptyFeedback
        title="No signup grants"
        message="Issue a grant to invite someone to create an account."
        action={<Button>Issue signup grant</Button>}
      />
    ),
  },
};
export const MobileOverflow: Story = {
  ...Loaded,
  globals: { viewport: { value: "mobile1" } },
  args: {
    ...Loaded.args,
    identityLabel:
      "a-long-address-that-must-wrap-without-hiding-actions@example.com",
  },
};
export const LongContent: Story = {
  args: {
    children: Array.from({ length: 30 }, (_, index) => (
      <p key={index}>
        Workspace paragraph {index + 1}: content scrolls independently of
        navigation.
      </p>
    )),
  },
};
export const LongNavigation: Story = {
  ...LongContent,
  args: {
    ...LongContent.args,
    navigation: Array.from({ length: 20 }, (_, index) => ({
      key: String(index),
      label: `Workspace ${index + 1}`,
      href: `#workspace-${index + 1}`,
    })),
  },
};
