/** Browser-safe values; RPC decoding and transport errors stay in computer.ts. */
export const agentPaletteKeys = [
  "butter",
  "sage",
  "apricot",
  "teal",
  "lilac",
  "rose",
] as const;
export type AgentPaletteKey = (typeof agentPaletteKeys)[number];
export interface AgentProfileImage {
  id: string;
  internalName: string;
  assetKey: string;
}
export function validAgentPalette(value: string): value is AgentPaletteKey {
  return agentPaletteKeys.some((key) => key === value);
}
/** Explicit presentation action; unrelated draft edits never call this. */
export function randomAgentPalette(current: AgentPaletteKey): AgentPaletteKey {
  const choices = agentPaletteKeys.filter((key) => key !== current);
  const byte = new Uint8Array(1);
  for (;;) {
    crypto.getRandomValues(byte);
    const value = byte[0];
    if (value !== undefined && value < 255) {
      const selected = choices[value % choices.length];
      if (selected) return selected;
    }
  }
}
