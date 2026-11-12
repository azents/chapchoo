import { MessageRole } from "@chapchoo/rpc-client/conversations";
import { readConversationBytes } from "./conversation.js";
import { call } from "./rpc.js";
import type * as Wire from "@chapchoo/rpc-client/conversations";
import type {
  ConversationMessage,
  ConversationMessageRegion,
  ConversationMessageWindow,
} from "./conversation-message-types.js";

/** Message permission follows the product conversation, not a native source. */
export interface ConversationMessageApi {
  current(
    value: Wire.OwnerConversationRequest,
  ): Promise<Wire.OwnerConversation>;
  messages(value: Wire.OwnerMessagesRequest): Promise<Wire.MessageRegion>;
  readMessage(value: Wire.OwnerMessageRead): Promise<Wire.FieldChunk>;
}
export function conversationMessageApi(
  client: Wire.ConversationsClient,
  token: string | null,
): ConversationMessageApi {
  return {
    current: (value) => call(client.current.bind(client), value, token),
    messages: (value) => call(client.messages.bind(client), value, token),
    readMessage: (value) => call(client.readMessage.bind(client), value, token),
  };
}

export async function readConversationMessages(
  api: ConversationMessageApi,
  input: {
    agentId: string;
    expectedConversationId?: string | null;
    window: ConversationMessageWindow;
  },
): Promise<ConversationMessageRegion> {
  const conversationId = (await api.current({ agentId: input.agentId }))
    .conversation?.conversationId;
  const window =
    input.expectedConversationId !== undefined &&
    input.expectedConversationId !== conversationId
      ? { type: "latest" as const }
      : input.window;
  if (!conversationId)
    return {
      agentId: input.agentId,
      messages: [],
      window,
      through: 0n,
      renderedAt: Date.now(),
    };
  const scope = { agentId: input.agentId, conversationId };
  const page = await api.messages({
    scope,
    limit: 25,
    before: window.type === "before" ? window.order : undefined,
    after:
      window.type === "after"
        ? { order: window.order, through: window.through }
        : undefined,
  });
  if (
    page.messages.length > 25 ||
    page.through < 0n ||
    (window.type === "after" &&
      (page.through < window.order ||
        (window.through !== undefined && page.through !== window.through)))
  )
    throw new Error("Conversation message boundary changed");
  const messages: ConversationMessage[] = [];
  const ids = new Set<string>();
  let previous = 0n;
  for (const metadata of page.messages) {
    if (
      !metadata.id ||
      ids.has(metadata.id) ||
      metadata.receiptOrder <= previous ||
      metadata.receiptOrder > page.through ||
      metadata.totalBytes < 0n ||
      (window.type === "before" && metadata.receiptOrder >= window.order) ||
      (window.type === "after" && metadata.receiptOrder <= window.order)
    )
      throw new Error("Conversation message metadata changed");
    ids.add(metadata.id);
    previous = metadata.receiptOrder;
    const role =
      metadata.role === MessageRole.MESSAGE_ROLE_USER
        ? "user"
        : metadata.role === MessageRole.MESSAGE_ROLE_AGENT
          ? "agent"
          : undefined;
    if (!role) throw new Error("Conversation message role is invalid");
    const bytes = await readConversationBytes(
      (offset) =>
        api.readMessage({
          scope,
          messageId: metadata.id,
          offset,
          length: 32 * 1024,
        }),
      metadata.totalBytes,
    );
    const text = new TextDecoder("utf-8", { fatal: true }).decode(bytes);
    messages.push({
      id: metadata.id,
      role,
      order: metadata.receiptOrder,
      receivedAt: metadata.receivedAt,
      text,
      sourceInputId: metadata.sourceInputId,
      sourceInputRequestId: metadata.sourceInputRequestId,
      originalAcceptedAt: metadata.originalAcceptedAt,
    });
  }
  const finalConversation = (await api.current({ agentId: input.agentId }))
    .conversation?.conversationId;
  if (finalConversation !== conversationId)
    throw new Error("Conversation selection changed during message read");
  const first = messages[0]?.order;
  const last = messages.at(-1)?.order;
  return {
    agentId: input.agentId,
    conversationId,
    messages,
    window,
    through: page.through,
    older:
      window.type !== "after" &&
      page.messages.length === 25 &&
      first !== undefined &&
      first > 1n
        ? first
        : undefined,
    next:
      window.type === "after" &&
      page.messages.length === 25 &&
      last !== undefined &&
      last < page.through
        ? { order: last, through: page.through }
        : undefined,
    renderedAt: Date.now(),
  };
}
