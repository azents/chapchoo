import { z } from "zod";
import { conversationReadInteger } from "./conversation-types.js";

const order = conversationReadInteger.pipe(z.bigint().nonnegative());
export const conversationMessageWindow = z.discriminatedUnion("type", [
  z.object({ type: z.literal("latest") }).strict(),
  z.object({ type: z.literal("before"), order }).strict(),
  z
    .object({
      type: z.literal("after"),
      order,
      through: order.optional(),
    })
    .strict(),
]);
export type ConversationMessageWindow = z.output<
  typeof conversationMessageWindow
>;

export interface ConversationMessage {
  id: string;
  role: "user" | "agent";
  order: bigint;
  receivedAt: bigint;
  text: string;
  sourceInputId?: string;
  sourceInputRequestId?: string;
  originalAcceptedAt?: bigint;
}
export interface ConversationMessageRegion {
  agentId: string;
  conversationId?: string;
  messages: ConversationMessage[];
  window: ConversationMessageWindow;
  through: bigint;
  older?: bigint;
  next?: { order: bigint; through: bigint };
  renderedAt: number;
}
