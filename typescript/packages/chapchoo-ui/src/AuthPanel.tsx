import { Container, Paper, Stack, Text, Title, rem } from "@mantine/core";
import type { ReactNode } from "react";

export interface AuthPanelProps {
  title: string;
  description?: ReactNode;
  children: ReactNode;
  footer?: ReactNode;
}

/** A focused form surface; the app owns submission, state, copy and navigation. */
export function AuthPanel({
  title,
  description,
  children,
  footer,
}: AuthPanelProps): ReactNode {
  return (
    <Container component="main" size={rem(480)} py={{ base: "xl", sm: "4xl" }}>
      <Paper withBorder p={{ base: "lg", sm: "xl" }} radius="md">
        <Stack gap="lg">
          <Stack gap="xs">
            <Title order={1} size="h2" style={{ overflowWrap: "anywhere" }}>
              {title}
            </Title>
            {description ? (
              <Text c="dimmed" size="sm">
                {description}
              </Text>
            ) : null}
          </Stack>
          {children}
          {footer}
        </Stack>
      </Paper>
    </Container>
  );
}
