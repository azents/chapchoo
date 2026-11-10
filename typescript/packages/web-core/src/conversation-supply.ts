import { harnessSupplyChoice } from "./conversation-types.js";
import type {
  HarnessSupplyChoice,
  SavedHarnessSupply,
} from "./conversation-types.js";
import type {
  HarnessSupply,
  SavedSupply,
} from "@chapchoo/rpc-client/conversations";

export function decodeHarnessSupply(value: HarnessSupply): HarnessSupplyChoice {
  if (value.managed !== undefined && value.existingCodexPath === undefined)
    return { type: "managed" };
  if (value.managed === undefined && value.existingCodexPath !== undefined)
    return harnessSupplyChoice.parse({
      type: "existing",
      path: value.existingCodexPath,
    });
  throw new Error("Harness supply choice is invalid");
}
export function encodeHarnessSupply(value: HarnessSupplyChoice): HarnessSupply {
  const checked = harnessSupplyChoice.parse(value);
  return checked.type === "managed"
    ? { managed: {} }
    : { existingCodexPath: checked.path };
}
export function decodeSavedSupply(value: SavedSupply): SavedHarnessSupply {
  if (!value.supply) throw new Error("Saved harness supply is missing");
  return {
    choice: decodeHarnessSupply(value.supply),
    requestId: value.requestId,
  };
}
export function sameHarnessSupply(
  a: HarnessSupplyChoice,
  b: HarnessSupplyChoice,
): boolean {
  return (
    a.type === b.type &&
    (a.type === "managed" || (b.type === "existing" && a.path === b.path))
  );
}
