import "server-only";
import {
  conversationApi,
  readConversationActivity,
} from "@chapchoo/web-core/conversation";
import {
  conversationMessageApi,
  readConversationMessages,
} from "@chapchoo/web-core/conversation-messages";
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
  const messages = await scope.read((token) =>
    readConversationMessages(
      conversationMessageApi(backend.conversations, token),
      { agentId, window: { type: "latest" } },
    ),
  );
  const region = await scope.read((token) =>
    readConversationActivity(conversationApi(backend.conversations, token), {
      agentId,
    }),
  );
  return { initial, initialConversation: region, initialMessages: messages };
}
