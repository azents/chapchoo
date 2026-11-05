import { CharacterArtwork } from "@chapchoo/ui/CharacterArtwork";
import { Box, Text } from "@mantine/core";
import { profileStyle } from "../profilePalette";
import classes from "./ProfilePresentation.module.css";
import type { AgentPaletteKey } from "@chapchoo/web-core/agent-profile";
import type { ReactElement } from "react";

export function ProfilePreview(props: {
  basePath: string;
  assetKey: string;
  paletteKey: AgentPaletteKey;
  name: string;
  handle: string;
  label: string;
  caption?: string;
}): ReactElement {
  return (
    <Box
      component="section"
      aria-label={props.label}
      className={classes.preview}
      style={profileStyle(props.paletteKey)}
    >
      <CharacterArtwork
        basePath={props.basePath}
        assetKey={props.assetKey}
        className={classes.previewImage}
      />
      <Box className={classes.previewIdentity}>
        {props.caption ? (
          <Text size="xs" className={classes.previewCaption}>
            {props.caption}
          </Text>
        ) : null}
        <Text className={classes.previewName}>{props.name}</Text>
        {props.handle ? (
          <Text className={classes.previewHandle}>@{props.handle}</Text>
        ) : null}
      </Box>
    </Box>
  );
}
