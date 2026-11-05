import { Box, Image, Text } from "@mantine/core";
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
}): ReactElement {
  return (
    <Box
      component="section"
      aria-label={props.label}
      className={classes.preview}
      style={profileStyle(props.paletteKey)}
    >
      <Image
        src={`${props.basePath}/design-assets/${props.assetKey}-216.webp`}
        alt=""
        className={classes.previewImage}
      />
      <Box className={classes.previewIdentity}>
        <Text className={classes.previewName}>{props.name}</Text>
        {props.handle ? (
          <Text className={classes.previewHandle}>@{props.handle}</Text>
        ) : null}
      </Box>
    </Box>
  );
}
