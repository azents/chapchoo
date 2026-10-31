import { Box, Text, Title } from "@mantine/core";
import { AgentMascotMotion } from "./AgentMascotMotion";
import { AppFrame } from "./AppFrame";
import classes from "./AuthPanel.module.css";
import type { ColorModeMenuProps } from "./ColorModeMenu";
import type { MascotId } from "./designTokens";
import type { ReactNode } from "react";

export interface AuthPanelProps {
  mascotReplayLabel: string;
  title: string;
  description?: ReactNode;
  children: ReactNode;
  footer?: ReactNode;
  appearance?: ColorModeMenuProps;
  basePath?: string;
  character?: MascotId;
}

/** Focused identity and form; the app owns submission, copy and navigation. */
export function AuthPanel({
  mascotReplayLabel,
  title,
  description,
  children,
  footer,
  appearance,
  basePath = "",
  character,
}: AuthPanelProps): ReactNode {
  return (
    <AppFrame appearance={appearance}>
      <Box component="main" className={classes.page}>
        <div className={classes.panel}>
          <div className={classes.heading}>
            <AgentMascotMotion
              basePath={basePath}
              size={72}
              character={character}
              playing
              replayLabel={mascotReplayLabel}
            />
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
    </AppFrame>
  );
}
