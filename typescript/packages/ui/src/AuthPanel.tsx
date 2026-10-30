import { Box, Text, Title } from "@mantine/core";
import { Mascot } from "./Mascot";
import classes from "./AuthPanel.module.css";
import type { ReactNode } from "react";

export interface AuthPanelProps {
  title: string;
  description?: ReactNode;
  children: ReactNode;
  footer?: ReactNode;
  actions?: ReactNode;
  basePath?: string;
}

/** Focused identity and form; the app owns submission, copy and navigation. */
export function AuthPanel({
  title,
  description,
  children,
  footer,
  actions,
  basePath = "",
}: AuthPanelProps): ReactNode {
  return (
    <Box component="main" className={classes.page}>
      <div className={classes.brand}>chapchoo</div>
      {actions ? <div className={classes.actions}>{actions}</div> : null}
      <div className={classes.panel}>
        <div className={classes.heading}>
          <Mascot basePath={basePath} size={72} />
          <Title order={1} size="h2" style={{ overflowWrap: "anywhere" }}>
            {title}
          </Title>
          {description ? (
            <Text c="dimmed" size="sm">
              {description}
            </Text>
          ) : null}
        </div>
        {children}
        {footer ? <div className={classes.footer}>{footer}</div> : null}
      </div>
    </Box>
  );
}
