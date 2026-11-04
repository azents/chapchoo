import { agentPaletteKeys } from "@chapchoo/web-core/agent-profile";
import type {
  AgentPaletteKey,
  AgentProfileImage,
} from "@chapchoo/web-core/computer";

/** Form presentation state only; acceptance always receives the final selection. */
export function initialProfile(
  images: readonly AgentProfileImage[],
  choose: (upperBound: number) => number,
): { initialImage: AgentProfileImage; initialPalette: AgentPaletteKey } {
  if (images.length === 0) {
    throw new Error("Agent profile catalogue is empty");
  }
  const initialImage = images[choose(images.length)];
  const initialPalette = agentPaletteKeys[choose(agentPaletteKeys.length)];
  if (!initialImage || !initialPalette) {
    throw new Error("Profile draft selection failed");
  }
  return { initialImage, initialPalette };
}
