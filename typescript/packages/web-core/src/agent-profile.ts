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
