import { NavigationIcon } from "@chapchoo/ui/NavigationIcon";
import { agentPaletteKeys } from "@chapchoo/web-core/agent-profile";
import {
  Box,
  Button,
  Group,
  Image,
  SimpleGrid,
  Stack,
  Text,
  UnstyledButton,
} from "@mantine/core";
import { profilePalette } from "../profilePalette";
import classes from "./ProfilePresentation.module.css";
import type { CreationCopy } from "../creationCopy";
import type {
  AgentPaletteKey,
  AgentProfileImage,
} from "@chapchoo/web-core/agent-profile";
import type { ReactElement } from "react";

export function ProfileImageChoices(props: {
  basePath: string;
  label: string;
  images: readonly Pick<
    AgentProfileImage,
    "id" | "assetKey" | "internalName"
  >[];
  selected: string;
  disabled: boolean;
  onChange: (id: string) => void;
}): ReactElement {
  return (
    <Stack gap="sm" role="group" aria-label={props.label}>
      <Text size="sm" fw={500}>
        {props.label}
      </Text>
      <SimpleGrid cols={3} spacing="sm" className={classes.imageGrid}>
        {props.images.map((image) => (
          <UnstyledButton
            key={image.id}
            type="button"
            aria-label={image.internalName}
            aria-pressed={image.id === props.selected}
            disabled={props.disabled}
            onClick={() => props.onChange(image.id)}
            className={classes.imageChoice}
            data-selected={image.id === props.selected || void 0}
          >
            <Image
              src={`${props.basePath}/design-assets/${image.assetKey}-216.webp`}
              alt=""
              fit="contain"
            />
            {image.id === props.selected ? (
              <span className={classes.picked} aria-hidden="true">
                <NavigationIcon name="check" size="sm" />
              </span>
            ) : null}
          </UnstyledButton>
        ))}
      </SimpleGrid>
    </Stack>
  );
}
export function ProfilePaletteChoices(props: {
  copy: CreationCopy;
  selected: AgentPaletteKey;
  disabled: boolean;
  onChange: (key: AgentPaletteKey) => void;
  onRandom?: () => void;
}): ReactElement {
  return (
    <Stack gap="sm" role="group" aria-label={props.copy.palette}>
      <Group justify="space-between" gap="sm">
        <Text size="sm" fw={500}>
          {props.copy.palette}
        </Text>
        {props.onRandom ? (
          <Button
            type="button"
            size="compact-xs"
            variant="subtle"
            disabled={props.disabled}
            onClick={props.onRandom}
            leftSection={<NavigationIcon name="shuffle" size="sm" />}
          >
            {props.copy.randomPalette}
          </Button>
        ) : null}
      </Group>
      <Group className={classes.swatches} justify="space-between" wrap="nowrap">
        {agentPaletteKeys.map((key) => (
          <UnstyledButton
            type="button"
            key={key}
            aria-label={props.copy.palettes[key]}
            title={props.copy.palettes[key]}
            aria-pressed={props.selected === key}
            disabled={props.disabled}
            onClick={() => props.onChange(key)}
            className={classes.swatch}
            data-selected={props.selected === key || void 0}
            style={{ color: profilePalette[key].emphasis }}
          >
            <Box
              component="span"
              style={{ background: profilePalette[key].swatch }}
            >
              {props.selected === key ? (
                <NavigationIcon name="check" size="sm" />
              ) : null}
            </Box>
          </UnstyledButton>
        ))}
      </Group>
    </Stack>
  );
}
