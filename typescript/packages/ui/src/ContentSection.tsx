import { Box, Text, Title } from "@mantine/core";
import classes from "./ContentSection.module.css";
import type { ReactNode } from "react";

export function ContentSection({
  title,
  description,
  children,
}: {
  title: string;
  description?: string;
  children: ReactNode;
}): ReactNode {
  return (
    <Box component="section" className={classes.section}>
      <div className={classes.heading}>
        <Title order={2} size="h4">
          {title}
        </Title>
        {description ? (
          <Text size="sm" c="dimmed">
            {description}
          </Text>
        ) : null}
      </div>
      <div className={classes.body}>{children}</div>
    </Box>
  );
}
