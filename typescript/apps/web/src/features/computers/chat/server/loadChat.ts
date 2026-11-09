import "server-only";
import {
  conversationApi,
  readConversationRegion,
} from "@chapchoo/web-core/conversation";
import { getConfig } from "@/config";
import { getBackend } from "@/shared/server/backend";
import { readPageScope } from "@/shared/server/identity";
import { loadAgents } from "../../server/loadAgents";
import type { ChatInput } from "../types";

export async function loadChat(
  agentHandle: string,
  teamHandle: string,
): Promise<Omit<ChatInput, "basePath">> {
  const initial = await loadAgents(agentHandle, "", teamHandle);
  if (initial.type !== "detail") {
    throw new Error("Chat requires an admitted Agent");
  }
  const scope = await readPageScope();
  const backend = getBackend(getConfig());
  const agentId = initial.agent.id;
  const region = await scope.read((token) =>
    readConversationRegion(conversationApi(backend.conversations, token), {
      agentId,
    }),
  );
  return { initial, initialConversation: region };
}
