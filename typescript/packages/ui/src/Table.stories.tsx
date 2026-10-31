import { Stack, Table, Text, rem } from "@mantine/core";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import type { ReactElement } from "react";

function TableFixture({
  compact = false,
}: {
  compact?: boolean;
}): ReactElement {
  return (
    <Stack p="lg">
      <Text>
        Table outer edges follow the content grid; cells have their own inset.
      </Text>
      <Table.ScrollContainer minWidth={rem(560)}>
        <Table withTableBorder horizontalSpacing={compact ? "sm" : undefined}>
          <Table.Thead>
            <Table.Tr>
              <Table.Th>Email</Table.Th>
              <Table.Th>Role</Table.Th>
              <Table.Th>Status</Table.Th>
            </Table.Tr>
          </Table.Thead>
          <Table.Tbody>
            <Table.Tr>
              <Table.Td>member@example.com</Table.Td>
              <Table.Td>Owner</Table.Td>
              <Table.Td>Active</Table.Td>
            </Table.Tr>
          </Table.Tbody>
        </Table>
      </Table.ScrollContainer>
    </Stack>
  );
}

const meta = {
  title: "Shared UI/Table",
  component: TableFixture,
} satisfies Meta<typeof TableFixture>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Default: Story = {};
export const Compact: Story = { args: { compact: true } };
