import { useState } from "react";
import { AppMantineProvider } from "@chapchoo/ui/provider";
import { FontFaces } from "@chapchoo/ui/FontFaces";
import { Button, Group, Paper, Stack, Text, Title } from "@mantine/core";
import { AgentMascotMotion } from "./AgentMascotMotion";
import { MascotArtwork } from "./MascotArtwork";
import classes from "./MascotMotionDocument.module.css";
import type { MascotCharacter } from "./mascotArtwork";
import type { ReactElement } from "react";
import "@mantine/core/styles.css";

const characters: readonly MascotCharacter[] = [
  "bichon",
  "corgi",
  "dachshund",
  "husky",
  "collie",
];

/** Fixture-only controls; no agent lifecycle or product placement is implied. */
export default function MascotMotionDocument({
  mode,
}: {
  mode: "comparison" | "family";
}): ReactElement {
  const [playing, setPlaying] = useState(true);
  const [dark, setDark] = useState(false);
  return (
    <AppMantineProvider forceColorScheme={dark ? "dark" : "light"}>
      <FontFaces basePath="" />
      <main className={classes.page}>
        <Stack gap="xl">
          <Group justify="space-between">
            <div>
              <Text size="xs" c="dimmed">
                CHAPCHOO / MASCOT MOTION REFERENCE
              </Text>
              <Title order={1} size="h2">
                {mode === "family"
                  ? "One gesture, five companions"
                  : "Actor in motion. Accessory at rest."}
              </Title>
            </div>
            <Button variant="default" onClick={() => setDark(!dark)}>
              {dark ? "Light background" : "Dark background"}
            </Button>
          </Group>
          <Text c="dimmed">
            5% extension · 30° clockwise · 1200ms gesture · grounded with a
            small return bounce. Six seconds of rest between gestures.
          </Text>
          <Group>
            <Button onClick={() => setPlaying(!playing)} aria-pressed={playing}>
              {playing ? "Pause motion" : "Play motion"}
            </Button>
            <Text size="sm" c="dimmed">
              Motion is reserved for the Agent as the subject of interaction.
            </Text>
          </Group>
          {mode === "comparison" ? (
            <div className={classes.comparison}>
              <Paper withBorder p="xl" radius="lg">
                <Stack gap="md">
                  <Title order={2} size="h4">
                    Agent as interaction subject
                  </Title>
                  <div className={classes.stage}>
                    <AgentMascotMotion
                      character="corgi"
                      assetBasePath="./mascots"
                      size={180}
                      playing={playing}
                    />
                  </div>
                  <Text size="sm">
                    A character participating in the interaction.
                  </Text>
                </Stack>
              </Paper>
              <Paper withBorder p="xl" radius="lg">
                <Stack gap="md">
                  <Title order={2} size="h4">
                    Accessory / decoration
                  </Title>
                  <div className={classes.stage}>
                    <MascotArtwork
                      character="corgi"
                      assetBasePath="./mascots"
                      size={180}
                    />
                  </div>
                  <Text size="sm">Supporting artwork remains static.</Text>
                </Stack>
              </Paper>
            </div>
          ) : (
            <div className={classes.family}>
              {characters.map((character, index) => (
                <Paper withBorder p="lg" radius="lg" key={character}>
                  <div className={classes.stage}>
                    <AgentMascotMotion
                      character={character}
                      assetBasePath="./mascots"
                      size={140}
                      playing={playing}
                      delayMs={index * 180}
                    />
                  </div>
                  <Text size="sm" mt="md">
                    {character}
                  </Text>
                </Paper>
              ))}
            </div>
          )}
          <Title order={2} size="h4">
            Small identity / 64px
          </Title>
          <Group align="flex-end" gap="xl">
            {characters.map((character, index) => (
              <AgentMascotMotion
                key={character}
                character={character}
                assetBasePath="./mascots"
                size={64}
                playing={playing}
                delayMs={index * 180}
              />
            ))}
          </Group>
          <Text size="sm" c="dimmed">
            Reduced motion, hidden documents and offscreen characters stay
            still. Reference-local React components; existing product pages are
            unchanged.
          </Text>
        </Stack>
      </main>
    </AppMantineProvider>
  );
}
