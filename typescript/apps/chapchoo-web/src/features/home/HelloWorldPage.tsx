import { Container, Stack, Text, Title } from "@mantine/core";
import type { ReactElement } from "react";

export interface HelloWorldPageProps {
  heading: string;
}

export function HelloWorldPage({ heading }: HelloWorldPageProps): ReactElement {
  return (
    <Container component="main" size="sm" py="3xl">
      <Stack gap="sm">
        <Text c="dimmed" ff="monospace" size="sm">
          chapchoo
        </Text>
        <Title order={1}>{heading}</Title>
      </Stack>
    </Container>
  );
}
