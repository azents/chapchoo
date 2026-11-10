import {
  decodeSavedSupply,
  encodeHarnessSupply,
} from "./conversation-supply.js";
import { call } from "./rpc.js";
import type {
  HarnessSupplyChoice,
  SavedHarnessSupply,
} from "./conversation-types.js";
import type { ConversationsClient } from "@chapchoo/rpc-client/conversations";

export async function readHarnessSupply(
  client: ConversationsClient,
  token: string | null,
  agentId: string,
): Promise<SavedHarnessSupply> {
  return decodeSavedSupply(
    await call(client.getSupply.bind(client), { agentId }, token),
  );
}
export async function saveHarnessSupply(
  client: ConversationsClient,
  token: string | null,
  input: {
    agentId: string;
    requestId: string;
    expectedRequestId?: string;
    choice: HarnessSupplyChoice;
  },
): Promise<SavedHarnessSupply> {
  return decodeSavedSupply(
    await call(
      client.saveSupply.bind(client),
      {
        agentId: input.agentId,
        requestId: input.requestId,
        expectedRequestId: input.expectedRequestId,
        supply: encodeHarnessSupply(input.choice),
      },
      token,
    ),
  );
}
