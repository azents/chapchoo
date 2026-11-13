import { z } from "zod";

export const harnessSupplyChoice = z.discriminatedUnion("type", [
  z.object({ type: z.literal("managed") }).strict(),
  z
    .object({
      type: z.literal("existing"),
      path: z
        .string()
        .min(1)
        .max(4096)
        .refine((value) => value.startsWith("/") && !/\p{Cc}/u.test(value)),
    })
    .strict(),
]);
export type HarnessSupplyChoice = z.infer<typeof harnessSupplyChoice>;
export interface SavedHarnessSupply {
  choice: HarnessSupplyChoice;
  requestId?: string;
}
export const conversationAnchor = z
  .object({ ordinal: z.bigint().positive(), index: z.bigint().nonnegative() })
  .strict();
export type ConversationAnchor = z.infer<typeof conversationAnchor>;
export const conversationSource = z
  .object({
    agentId: z.string().min(1),
    conversationId: z.string().min(1),
    snapshotId: z.string().min(1),
  })
  .strict();
export type ConversationSource = z.infer<typeof conversationSource>;
export const nativeTarget = z
  .object({ snapshotId: z.string().min(1), nativeSessionId: z.string().min(1) })
  .strict();
export const answerValue = z.discriminatedUnion("type", [
  z.object({ type: z.literal("text"), value: z.string() }).strict(),
  z
    .object({
      type: z.literal("integer"),
      value: z.bigint().min(-9223372036854775808n).max(9223372036854775807n),
    })
    .strict(),
  z.object({ type: z.literal("number"), value: z.number().finite() }).strict(),
  z.object({ type: z.literal("boolean"), value: z.boolean() }).strict(),
  z.object({ type: z.literal("strings"), value: z.array(z.string()) }).strict(),
]);
export type FormValue = z.infer<typeof answerValue>;
export const conversationAnswer = z.discriminatedUnion("type", [
  z
    .object({ type: z.literal("permission"), optionId: z.string().min(1) })
    .strict(),
  z.object({ type: z.literal("permissionCancel") }).strict(),
  z
    .object({
      type: z.literal("formAccept"),
      content: z.record(z.string(), answerValue).optional(),
    })
    .strict(),
  z.object({ type: z.literal("formDecline") }).strict(),
  z.object({ type: z.literal("formCancel") }).strict(),
]);
export type ConversationAnswer = z.infer<typeof conversationAnswer>;
export interface Choice {
  value: string;
  title: string;
  description?: string;
}
interface FieldCommon {
  name: string;
  title: string;
  description?: string;
  required: boolean;
}
export type QuestionField = FieldCommon &
  (
    | {
        type: "text";
        choices?: Choice[];
        minLength?: number;
        maxLength?: number;
        pattern?: string;
        format?: string;
        default?: string;
      }
    | {
        type: "number" | "integer";
        minimum?: number | bigint;
        maximum?: number | bigint;
        default?: number | bigint;
      }
    | { type: "boolean"; default?: boolean }
    | {
        type: "strings";
        choices: Choice[];
        minItems?: number;
        maxItems?: number;
        default?: string[];
      }
    | { type: "unsupported" }
  );
export type Question =
  | {
      type: "permission";
      title: string;
      description?: string;
      options: { id: string; title: string; scope: string }[];
      content: ContentPart[];
      locations: { path: string; line?: number }[];
    }
  | {
      type: "form";
      title: string;
      schemaTitle?: string;
      description?: string;
      fields: QuestionField[];
    }
  | { type: "unsupported"; title: string };
export type ContentPart =
  | { type: "text"; text: string }
  | { type: "diff"; path: string; oldText?: string; newText: string }
  | { type: "image"; data: string; mimeType: string }
  | { type: "audio"; data: string; mimeType: string }
  | { type: "resource"; uri: string; title: string; text?: string }
  | { type: "unsupported"; kind: string };
export type RecordKind =
  | "user"
  | "assistant"
  | "thought"
  | "tool"
  | "plan"
  | "terminal"
  | "request"
  | "state"
  | "notice";
export interface ConversationRecord {
  key: string;
  kind: RecordKind;
  nativeId: string;
  anchor: ConversationAnchor;
  updatedOrdinal: bigint;
  content: ContentPart[];
  title?: string;
  description?: string;
  status?: string;
  toolKind?: string;
  toolInput?: { json: string };
  toolOutput?: { json: string };
  terminal?: {
    text: string;
    base64: string;
    exitCode?: bigint | number;
    signal?: string;
  };
  locations: { path: string; line?: number }[];
  plan?:
    | {
        type: "items";
        items: { content: string; status: string; priority?: string }[];
      }
    | { type: "text"; text: string };
  question?: Question;
  responder?: { token: string; answerRequestId?: string };
  answer?: {
    value: ConversationAnswer;
    delivery: "accepted" | "possible" | "delivered" | "rejected";
  };
}
export interface ConversationHeader {
  agentId: string;
  conversationId?: string;
  state: "notStarted" | "loading" | "ready";
  source?: ConversationSource;
  native?: z.infer<typeof nativeTarget>;
  foreground: "unavailable" | "idle" | "working" | "requiresAction";
  workId?: string;
  appliedSupply?: HarnessSupplyChoice;
  preparation?: NativePreparation;
  startup?: InputPreparation;
  savedAuthenticationId?: string;
  appliedAuthenticationId?: string;
}
export interface NativePreparation {
  freshness: "current" | "cached";
  node: string;
  adapter: string;
  codex: string;
}
export interface ConversationRegion {
  header: ConversationHeader;
  records: ConversationRecord[];
  before?: ConversationAnchor;
  older?: ConversationAnchor;
  renderedAt: number;
  refill?: boolean;
  inputs?: RetainedInputView[];
  closedInputs?: string[];
  latestInput?: Pick<
    RetainedInputView,
    "requestId" | "observation" | "preparation"
  >;
  inputOlder?: InputHistoryAnchor;
  latestPlan?: ConversationRecord | null;
  stop?: {
    requestId: string;
    observation: "pending" | "published" | "rejected" | "uncertain";
  };
}
export const inputHistoryAnchor = z
  .object({ acceptedAt: z.bigint(), requestId: z.string().min(1).max(128) })
  .strict();
export type InputHistoryAnchor = z.infer<typeof inputHistoryAnchor>;
// React Query keys are JSON values. Keep request hints exact without bigint keys.
export const conversationReadInteger = z
  .string()
  .max(20)
  .regex(/^(?:0|[1-9]\d*|-[1-9]\d*)$/)
  .transform((value) => BigInt(value))
  .pipe(z.bigint().min(-9223372036854775808n).max(9223372036854775807n));
export const conversationReadAnchor = z
  .object({
    ordinal: conversationReadInteger,
    index: conversationReadInteger,
  })
  .strict()
  .pipe(conversationAnchor);
export const conversationReadInputAnchor = z
  .object({
    acceptedAt: conversationReadInteger,
    requestId: z.string().min(1).max(128),
  })
  .strict()
  .pipe(inputHistoryAnchor);
export interface RetainedInputView {
  id: string;
  requestId: string;
  conversationId: string;
  acceptedAt: bigint;
  text?: string;
  observation: InputObservation;
  preparation?: InputPreparation;
  insertedNativeMessageId?: string;
}
export type InputPreparation =
  | { type: "preparing" | "ready" }
  | {
      type: "failed";
      reason:
        | "notAdmitted"
        | "launchFailed"
        | "nativeRejected"
        | "sourceUnavailable"
        | "retirementUnknown";
    };
export interface InputReceipt {
  requestId: string;
  conversationId: string;
  id: string;
}
export type InputObservation =
  "accepted" | "preparing" | "inserted" | "steered" | "failed" | "uncertain";
