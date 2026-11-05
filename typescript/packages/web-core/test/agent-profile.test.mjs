import assert from "node:assert/strict";
import { it } from "node:test";
import {
  agentPaletteKeys,
  randomAgentPalette,
  validAgentPalette,
} from "../dist/agent-profile.js";

it("explicit random-color actions retain the palette contract and always replace the current color", () => {
  for (const current of agentPaletteKeys) {
    for (let attempt = 0; attempt < 16; attempt += 1) {
      const chosen = randomAgentPalette(current);
      assert.equal(validAgentPalette(chosen), true);
      assert.notEqual(chosen, current);
    }
  }
});
