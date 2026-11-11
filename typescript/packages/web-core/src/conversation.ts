import {
  CaptureStatus,
  Foreground,
  RecordKind,
  RecordField,
  TypedAnswer,
  AnswerDelivery,
  InputOwnership,
  StopState,
  PreparationFreshness,
  SetupFailure,
} from "@chapchoo/rpc-client/conversations";
import { z } from "zod";
import {
  decodeContent,
  decodeLines,
  decodeLocations,
  decodePlan,
  decodeQuestion,
  decodeString,
  decodeToolPayload,
} from "./conversation-decode.js";
import { call } from "./rpc.js";
import { decodeHarnessSupply } from "./conversation-supply.js";
import type * as Wire from "@chapchoo/rpc-client/conversations";
import type {
  ConversationAnswer,
  ConversationHeader,
  ConversationRegion,
  ConversationRecord,
  ConversationSource,
  ConversationAnchor,
  InputReceipt,
  InputObservation,
  InputPreparation,
  FormValue,
} from "./conversation-types.js";

export interface ConversationApi {
  current(
    value: Wire.OwnerConversationRequest,
  ): Promise<Wire.OwnerConversation>;
  records(value: Wire.OwnerRecordsRequest): Promise<Wire.RecordRegion>;
  latestPlan(value: Wire.RecordSource): Promise<Wire.RecordRegion>;
  inputs(value: Wire.OwnerInputsRequest): Promise<Wire.RetainedInputs>;
  readInput(value: Wire.OwnerInputRead): Promise<Wire.FieldChunk>;
  deleteInput(value: Wire.OwnerDeleteInput): Promise<unknown>;
  fields(value: Wire.OwnerFieldsRequest): Promise<Wire.RecordFields>;
  readField(value: Wire.OwnerFieldRead): Promise<Wire.FieldChunk>;
  liveRequests(value: Wire.OwnerLiveRequests): Promise<Wire.LiveRequestList>;
  requestAnswer(
    value: Wire.OwnerRequestAnswer,
  ): Promise<Wire.OwnerAnswerStatus>;
  readAnswer(value: Wire.OwnerAnswerRead): Promise<Wire.FieldChunk>;
  submitInput(value: Wire.OwnerInputPiece): Promise<Wire.OwnerInputProgress>;
  inputResult(
    value: Wire.OwnerInputResultRequest,
  ): Promise<Wire.OwnerInputStatus>;
  submitAnswer(value: Wire.OwnerAnswerPiece): Promise<Wire.OwnerAnswerProgress>;
  answerResult(
    value: Wire.OwnerAnswerResultRequest,
  ): Promise<Wire.OwnerAnswerStatus>;
  stop(value: Wire.OwnerStopRequest): Promise<Wire.StopReceipt>;
  stopResult(
    value: Wire.OwnerStopResultRequest,
  ): Promise<Wire.OptionalStopReceipt>;
}
export function conversationApi(
  client: Wire.ConversationsClient,
  token: string | null,
): ConversationApi {
  return {
    current: (value) => call(client.current.bind(client), value, token),
    records: (value) => call(client.records.bind(client), value, token),
    latestPlan: (value) => call(client.latestPlan.bind(client), value, token),
    inputs: (value) => call(client.inputs.bind(client), value, token),
    readInput: (value) => call(client.readInput.bind(client), value, token),
    fields: (value) => call(client.fields.bind(client), value, token),
    readField: (value) => call(client.readField.bind(client), value, token),
    liveRequests: (value) =>
      call(client.liveRequests.bind(client), value, token),
    requestAnswer: (value) =>
      call(client.requestAnswer.bind(client), value, token),
    readAnswer: (value) => call(client.readAnswer.bind(client), value, token),
    submitInput: (value) => call(client.submitInput.bind(client), value, token),
    inputResult: (value) => call(client.inputResult.bind(client), value, token),
    deleteInput: (value) => call(client.deleteInput.bind(client), value, token),
    submitAnswer: (value) =>
      call(client.submitAnswer.bind(client), value, token),
    answerResult: (value) =>
      call(client.answerResult.bind(client), value, token),
    stop: (value) => call(client.stop.bind(client), value, token),
    stopResult: (value) => call(client.stopResult.bind(client), value, token),
  };
}
export function conversationHeader(
  agentId: string,
  value: Wire.OwnerConversation,
): ConversationHeader {
  const c = value.conversation;
  const state =
    !c || c.status === CaptureStatus.CAPTURE_STATUS_NOT_STARTED
      ? "notStarted"
      : c.status === CaptureStatus.CAPTURE_STATUS_LOADING
        ? "loading"
        : c.status === CaptureStatus.CAPTURE_STATUS_READY
          ? "ready"
          : undefined;
  if (!state) throw new Error("Conversation state is invalid");
  const observed = value.observedNative;
  const source = c?.snapshotId
    ? { agentId, conversationId: c.conversationId, snapshotId: c.snapshotId }
    : undefined;
  const native =
    source && c?.nativeSessionId
      ? { snapshotId: source.snapshotId, nativeSessionId: c.nativeSessionId }
      : undefined;
  if (
    observed &&
    (!native ||
      native.snapshotId !== observed.target?.snapshotId ||
      native.nativeSessionId !== observed.target.nativeSessionId)
  )
    throw new Error("Conversation observation changed");
  return {
    agentId,
    conversationId: c?.conversationId,
    state,
    source,
    native,
    appliedSupply: observed?.appliedSupply
      ? decodeHarnessSupply(observed.appliedSupply)
      : undefined,
    preparation: observed?.preparation
      ? {
          freshness:
            observed.preparation.freshness ===
            PreparationFreshness.PREPARATION_FRESHNESS_CURRENT
              ? "current"
              : observed.preparation.freshness ===
                  PreparationFreshness.PREPARATION_FRESHNESS_TRANSIENT_LOOKUP_FAILURE
                ? "cached"
                : z.never().parse(observed.preparation.freshness),
          node: z.string().min(1).max(256).parse(observed.preparation.node),
          adapter: z
            .string()
            .min(1)
            .max(256)
            .parse(observed.preparation.adapter),
          codex: z.string().min(1).max(256).parse(observed.preparation.codex),
        }
      : undefined,
    savedAuthenticationId: value.savedAuthenticationId,
    startup: preparationView(value.startup),
    appliedAuthenticationId: observed?.authenticationId,
    foreground: !observed
      ? "unavailable"
      : observed.foreground === Foreground.FOREGROUND_IDLE
        ? "idle"
        : observed.foreground === Foreground.FOREGROUND_REQUIRES_ACTION
          ? "requiresAction"
          : "working",
    workId: observed?.workId,
  };
}
const kinds: Partial<Record<RecordKind, ConversationRecord["kind"]>> = {
  [RecordKind.RECORD_KIND_USER]: "user",
  [RecordKind.RECORD_KIND_ASSISTANT]: "assistant",
  [RecordKind.RECORD_KIND_THOUGHT]: "thought",
  [RecordKind.RECORD_KIND_TOOL]: "tool",
  [RecordKind.RECORD_KIND_PLAN]: "plan",
  [RecordKind.RECORD_KIND_TERMINAL]: "terminal",
  [RecordKind.RECORD_KIND_REQUEST]: "request",
  [RecordKind.RECORD_KIND_STATE]: "state",
  [RecordKind.RECORD_KIND_NOTICE]: "notice",
};
const pieceBytes = 32 * 1024;
/** Assemble UTF-8 only after all byte-bound continuations agree, including the final authority check. */
export async function readConversationBytes(
  read: (offset: number) => Promise<Wire.FieldChunk>,
  expected?: bigint,
): Promise<Buffer> {
  const pieces: Buffer[] = [];
  let offset = 0;
  let total = expected;
  for (;;) {
    const chunk = await read(offset);
    total ??= chunk.totalBytes;
    if (
      chunk.offset !== offset ||
      chunk.totalBytes !== total ||
      chunk.content.length > pieceBytes ||
      BigInt(offset + chunk.content.length) > total ||
      total > BigInt(Number.MAX_SAFE_INTEGER)
    )
      throw new Error("Conversation continuation changed");
    if (BigInt(offset) === total) {
      if (chunk.content.length)
        throw new Error("Conversation continuation exceeds its body");
      return Buffer.concat(pieces, offset);
    }
    if (!chunk.content.length)
      throw new Error("Conversation continuation is incomplete");
    pieces.push(chunk.content);
    offset += chunk.content.length;
  }
}
export async function readConversationRecord(
  api: ConversationApi,
  header: ConversationHeader,
  metadata: Wire.RecordMetadata,
): Promise<ConversationRecord> {
  const source = header.source;
  const kind = kinds[metadata.kind];
  if (!source || !kind || !metadata.anchor)
    throw new Error("Conversation record is invalid");
  const record = { source, kind: metadata.kind, nativeId: metadata.nativeId };
  const values = new Map<RecordField, unknown[]>();
  const initialFields = (await api.fields(record)).fields;
  for (const field of initialFields) {
    const bytes = await readConversationBytes(
      (offset) =>
        api.readField({
          record,
          field: field.field,
          updatedOrdinal: field.updatedOrdinal,
          offset,
          length: pieceBytes,
        }),
      field.totalBytes,
    );
    values.set(field.field, decodeLines(bytes));
  }
  const value: ConversationRecord = {
    key: `${kind}:${metadata.nativeId}`,
    kind,
    nativeId: metadata.nativeId,
    anchor: metadata.anchor,
    updatedOrdinal: initialFields.reduce(
      (ordinal, field) =>
        field.updatedOrdinal > ordinal ? field.updatedOrdinal : ordinal,
      metadata.updatedOrdinal,
    ),
    content: (values.get(RecordField.RECORD_FIELD_CONTENT) ?? []).map(
      decodeContent,
    ),
    toolInput: decodeToolPayload(
      values.get(RecordField.RECORD_FIELD_TOOL_INPUT) ?? [],
    ),
    toolOutput: decodeToolPayload(
      values.get(RecordField.RECORD_FIELD_TOOL_OUTPUT) ?? [],
    ),
    title: decodeString(values.get(RecordField.RECORD_FIELD_TITLE) ?? []),
    description: decodeString(
      values.get(RecordField.RECORD_FIELD_DESCRIPTION) ?? [],
    ),
    status:
      kind === "terminal"
        ? undefined
        : decodeString(values.get(RecordField.RECORD_FIELD_STATUS) ?? []),
    toolKind: decodeString(values.get(RecordField.RECORD_FIELD_KIND) ?? []),
    locations: decodeLocations(
      values.get(RecordField.RECORD_FIELD_LOCATIONS) ?? [],
    ),
  };
  if (kind === "terminal") {
    const pieces = (values.get(RecordField.RECORD_FIELD_CONTENT) ?? []).map(
      (item) => {
        const data = z.looseObject({ data: z.string() }).parse(item).data;
        const bytes = Buffer.from(data, "base64");
        if (bytes.toString("base64") !== data)
          throw new Error("Terminal output encoding is invalid");
        return bytes;
      },
    );
    const bytes = Buffer.concat(pieces);
    const exit = values.get(RecordField.RECORD_FIELD_STATUS)?.at(-1);
    const status =
      exit === undefined
        ? undefined
        : z
            .looseObject({
              exitCode: z.union([z.bigint(), z.number().int()]).nullish(),
              signal: z.string().nullish(),
            })
            .parse(exit);
    value.terminal = {
      text: new TextDecoder().decode(bytes),
      base64: bytes.toString("base64"),
      exitCode: status?.exitCode ?? undefined,
      signal: status?.signal ?? undefined,
    };
    value.content = [];
  }
  if (kind === "plan") {
    const latest = values.get(RecordField.RECORD_FIELD_CONTENT)?.at(-1);
    if (latest !== undefined) value.plan = decodePlan(latest);
    value.content = [];
  }
  if (kind === "request") {
    value.question = decodeQuestion(
      values.get(RecordField.RECORD_FIELD_REQUEST)?.at(-1),
      header.native?.nativeSessionId ?? "",
    );
    const retained = (
      await api.requestAnswer({ source, occurrenceId: metadata.nativeId })
    ).acceptance;
    if (retained) {
      const bytes = await readConversationBytes((offset) =>
        api.readAnswer({
          source,
          operationId: retained.id,
          offset,
          length: pieceBytes,
        }),
      );
      value.answer = {
        value: decodeAnswer(TypedAnswer.decode(bytes)),
        delivery: answerDelivery(retained.delivery),
      };
    }
  }
  // Recheck owner/source authority after completing the coherent initial field
  // extents. Later appends and fields belong to the next poll.
  await api.fields(record);
  return value;
}
export async function readConversationRegion(
  api: ConversationApi,
  input: {
    agentId: string;
    before?: ConversationAnchor;
    expectedSnapshotId?: string;
    expectedConversationId?: string;
    known?: { key: string; updatedOrdinal: bigint }[];
    inputBefore?: Wire.InputAnchor;
    knownInputs?: string[];
    stop?: { requestId: string; conversationId: string; snapshotId: string };
  },
): Promise<ConversationRegion> {
  const header = conversationHeader(
    input.agentId,
    await api.current({ agentId: input.agentId }),
  );
  const sameSource =
    input.expectedSnapshotId === header.source?.snapshotId &&
    input.expectedConversationId === header.conversationId;
  const before = sameSource ? input.before : undefined;
  if (header.state === "loading")
    return { header, records: [], renderedAt: Date.now() };
  if (input.expectedSnapshotId !== undefined && !sameSource)
    return { header, records: [], renderedAt: Date.now(), refill: true };
  const inputScope: Wire.OwnerInputScope | undefined = header.conversationId
    ? {
        agentId: input.agentId,
        conversationId: header.conversationId,
        target: header.source
          ? { snapshotId: header.source.snapshotId }
          : { notStarted: {} },
      }
    : undefined;
  const retainedPage = inputScope
    ? await api.inputs({
        scope: inputScope,
        before: sameSource ? input.inputBefore : undefined,
        limit: 24,
      })
    : { inputs: [], latest: undefined };
  const retained = retainedPage.inputs;
  const inputs: NonNullable<ConversationRegion["inputs"]> = [];
  for (const item of retained) {
    if (!item.acceptance || !inputScope)
      throw new Error("Retained input is invalid");
    const receipt = item.acceptance;
    const text =
      item.deleted || (sameSource && input.knownInputs?.includes(receipt.id))
        ? undefined
        : new TextDecoder("utf-8", { fatal: true }).decode(
            await readConversationBytes(
              (offset) =>
                api.readInput({
                  scope: inputScope,
                  requestId: receipt.requestId,
                  offset,
                  length: pieceBytes,
                }),
              item.totalBytes,
            ),
          );
    const status = await api.inputResult({
      agentId: input.agentId,
      conversationId: receipt.conversationId,
      requestId: receipt.requestId,
    });
    inputs.push({
      id: receipt.id,
      conversationId: receipt.conversationId,
      requestId: receipt.requestId,
      acceptedAt: receipt.acceptedAt,
      text,
      deleted: item.deleted,
      observation: inputObservation(status),
      preparation: inputPreparation(status),
      insertedNativeMessageId: status.delivery?.insertedNativeMessageId,
    });
  }
  const newest = retainedPage.latest;
  const firstInput = newest
    ? inputs.find((item) => !item.deleted && item.id === newest.id)
    : undefined;
  let latestInput: ConversationRegion["latestInput"] = firstInput
    ? {
        requestId: firstInput.requestId,
        observation: firstInput.observation,
        preparation: firstInput.preparation,
      }
    : undefined;
  // The source-qualified latest live operation is independent of history pages.
  if (newest && !firstInput) {
    const status = await api.inputResult({
      agentId: input.agentId,
      conversationId: newest.conversationId,
      requestId: newest.requestId,
    });
    latestInput = status.deleted
      ? undefined
      : {
          requestId: newest.requestId,
          observation: inputObservation(status),
          preparation: inputPreparation(status),
        };
  }
  const inputLast = retained.at(-1)?.acceptance;
  const inputOlder =
    retained.length === 24 && inputLast
      ? { acceptedAt: inputLast.acceptedAt, requestId: inputLast.requestId }
      : undefined;
  const stop =
    input.stop &&
    input.stop.snapshotId === header.source?.snapshotId &&
    input.stop.conversationId === header.conversationId
      ? {
          requestId: input.stop.requestId,
          observation: stopObservation(
            (
              await api.stopResult({
                agentId: input.agentId,
                conversationId: input.stop.conversationId,
                requestId: input.stop.requestId,
              })
            ).receipt,
          ),
        }
      : undefined;
  if (!header.source) {
    const final = conversationHeader(
      input.agentId,
      await api.current({ agentId: input.agentId }),
    );
    if (
      final.state !== header.state ||
      final.conversationId !== header.conversationId
    )
      throw new Error("Conversation source changed during read");
    return {
      header: final,
      records: [],
      inputs: inputs.reverse(),
      latestInput,
      inputOlder,
      renderedAt: Date.now(),
    };
  }
  const region = await api.records({
    source: header.source,
    before,
    limit: 24,
  });
  const live =
    header.native && header.conversationId
      ? (
          await api.liveRequests({
            agentId: input.agentId,
            conversationId: header.conversationId,
            target: header.native,
          })
        ).requests
      : [];
  const records: ConversationRecord[] = [];
  for (const metadata of region.records) {
    const key = `${kinds[metadata.kind]}:${metadata.nativeId}`;
    // Request authority/answer state can change without a new captured body ordinal.
    if (
      metadata.kind !== RecordKind.RECORD_KIND_REQUEST &&
      sameSource &&
      input.known?.some(
        (known) =>
          known.key === key && known.updatedOrdinal === metadata.updatedOrdinal,
      )
    )
      continue;
    const record = await readConversationRecord(api, header, metadata);
    const responder = live.find(
      (request) => request.occurrence?.occurrenceId === metadata.nativeId,
    );
    if (responder && !record.answer)
      record.responder = {
        token: responder.responderToken,
        answerRequestId: responder.answerRequestId,
      };
    records.push(record);
  }
  const planMetadata = (await api.latestPlan(header.source)).records[0];
  const latestPlan = planMetadata
    ? await readConversationRecord(api, header, planMetadata)
    : null;
  const final = conversationHeader(
    input.agentId,
    await api.current({ agentId: input.agentId }),
  );
  if (
    final.source?.snapshotId !== header.source.snapshotId ||
    final.conversationId !== header.conversationId ||
    final.state !== "ready"
  )
    throw new Error("Conversation source changed during read");
  return {
    header: final,
    records: records.reverse(),
    inputs: inputs.reverse(),
    latestInput,
    inputOlder,
    latestPlan,
    stop,
    before,
    older:
      region.records.length === 24 ? region.records.at(-1)?.anchor : undefined,
    renderedAt: Date.now(),
  };
}
export function encodeAnswer(value: ConversationAnswer): Wire.TypedAnswer {
  switch (value.type) {
    case "permission":
      return { permissionOptionId: value.optionId };
    case "permissionCancel":
      return { permissionCancelled: {} };
    case "formDecline":
      return { formDecline: {} };
    case "formCancel":
      return { formCancel: {} };
    case "formAccept":
      return {
        formAccept: {
          content:
            value.content === undefined
              ? undefined
              : {
                  fields: Object.entries(value.content).map(
                    ([name, value]) => ({ name, value: encodeValue(value) }),
                  ),
                },
        },
      };
  }
}
function encodeValue(value: FormValue): Wire.AnswerValue {
  switch (value.type) {
    case "text":
      return { text: value.value };
    case "integer":
      return { integer: value.value };
    case "number":
      return { number: value.value };
    case "boolean":
      return { boolean: value.value };
    case "strings":
      return { strings: { values: value.value } };
  }
}
export function decodeAnswer(value: Wire.TypedAnswer): ConversationAnswer {
  if (value.permissionOptionId !== undefined)
    return { type: "permission", optionId: value.permissionOptionId };
  if (value.permissionCancelled !== undefined)
    return { type: "permissionCancel" };
  if (value.formDecline !== undefined) return { type: "formDecline" };
  if (value.formCancel !== undefined) return { type: "formCancel" };
  if (value.formAccept) {
    const content = new Map<string, FormValue>();
    for (const field of value.formAccept.content?.fields ?? []) {
      const v = field.value;
      if (!v || content.has(field.name))
        throw new Error("Retained answer is invalid");
      if (v.text !== undefined)
        content.set(field.name, { type: "text", value: v.text });
      else if (v.integer !== undefined)
        content.set(field.name, { type: "integer", value: v.integer });
      else if (v.number !== undefined)
        content.set(field.name, { type: "number", value: v.number });
      else if (v.boolean !== undefined)
        content.set(field.name, { type: "boolean", value: v.boolean });
      else if (v.strings !== undefined)
        content.set(field.name, { type: "strings", value: v.strings.values });
      else throw new Error("Retained answer value is invalid");
    }
    return value.formAccept.content === undefined
      ? { type: "formAccept" }
      : { type: "formAccept", content: Object.fromEntries(content) };
  }
  throw new Error("Retained answer is missing");
}
export function answerDelivery(
  value: AnswerDelivery,
): "accepted" | "possible" | "delivered" | "rejected" {
  switch (value) {
    case AnswerDelivery.ANSWER_DELIVERY_ACCEPTED:
      return "accepted";
    case AnswerDelivery.ANSWER_DELIVERY_POSSIBLE_FORWARDING:
      return "possible";
    case AnswerDelivery.ANSWER_DELIVERY_RESPONDER_ACCEPTED:
      return "delivered";
    case AnswerDelivery.ANSWER_DELIVERY_REJECTED:
      return "rejected";
    default:
      throw new Error("Answer delivery is invalid");
  }
}
export async function submitConversationInput(
  api: ConversationApi,
  value: {
    agentId: string;
    conversationId?: string;
    snapshotId?: string;
    requestId: string;
    text: string;
  },
): Promise<InputReceipt> {
  const content = Buffer.from(value.text, "utf8");
  if (!value.text.trim() || value.text.includes("\0"))
    throw new Error("Conversation input is invalid");
  for (let offset = 0; offset < content.length; offset += pieceBytes) {
    const piece = content.subarray(offset, offset + pieceBytes);
    const progress = await api.submitInput({
      agentId: value.agentId,
      conversationId: value.conversationId,
      requestId: value.requestId,
      target: value.snapshotId
        ? { snapshotId: value.snapshotId }
        : { notStarted: {} },
      totalBytes: content.length,
      offset,
      content: piece,
    });
    if (progress.accepted && offset + piece.length === content.length)
      return {
        requestId: progress.accepted.requestId,
        conversationId: progress.accepted.conversationId,
        id: progress.accepted.id,
      };
    if (progress.uploading?.nextOffset !== offset + piece.length)
      throw new Error("Conversation input acceptance is uncertain");
  }
  throw new Error("Conversation input acceptance is uncertain");
}
export async function submitConversationAnswer(
  api: ConversationApi,
  value: {
    source: ConversationSource;
    target: Wire.NativeTarget;
    responderToken: string;
    requestId: string;
    answer: ConversationAnswer;
  },
): Promise<{ id: string; delivery: ReturnType<typeof answerDelivery> }> {
  const content = Buffer.from(
    TypedAnswer.encode(encodeAnswer(value.answer)).finish(),
  );
  for (let offset = 0; offset < content.length; offset += pieceBytes) {
    const piece = content.subarray(offset, offset + pieceBytes);
    const progress = await api.submitAnswer({
      agentId: value.source.agentId,
      conversationId: value.source.conversationId,
      requestId: value.requestId,
      target: value.target,
      responderToken: value.responderToken,
      totalBytes: content.length,
      offset,
      content: piece,
    });
    if (progress.accepted && offset + piece.length === content.length)
      return {
        id: progress.accepted.id,
        delivery: answerDelivery(progress.accepted.delivery),
      };
    if (progress.uploading?.nextOffset !== offset + piece.length)
      throw new Error("Conversation answer acceptance is uncertain");
  }
  throw new Error("Conversation answer acceptance is uncertain");
}
export function inputObservation(
  value: Wire.OwnerInputStatus,
): InputObservation {
  if (!value.acceptance) return "uncertain";
  if (
    value.delivery?.failed !== undefined ||
    value.preparation?.failed !== undefined
  )
    return "failed";
  if (value.delivery?.insertedNativeMessageId !== undefined) return "inserted";
  if (value.delivery?.steered !== undefined) return "steered";
  if (
    value.ownership === InputOwnership.INPUT_OWNERSHIP_NOT_OBSERVED_HERE ||
    value.delivery?.possibleForwarding !== undefined
  )
    return "uncertain";
  if (value.preparation?.possibleForwarding !== undefined) return "preparing";
  return "accepted";
}
export function inputPreparation(
  value: Wire.OwnerInputStatus,
): InputPreparation | undefined {
  if (!value.acceptance) return undefined;
  return preparationView(value.preparation);
}
function preparationView(
  preparation: Wire.InputPreparation | undefined,
): InputPreparation | undefined {
  if (!preparation) return undefined;
  if (preparation.failed !== undefined) {
    const reasons: Partial<
      Record<number, Extract<InputPreparation, { type: "failed" }>["reason"]>
    > = {
      [SetupFailure.SETUP_FAILURE_NOT_ADMITTED]: "notAdmitted",
      [SetupFailure.SETUP_FAILURE_BEFORE_NATIVE_LAUNCH]: "launchFailed",
      [SetupFailure.SETUP_FAILURE_NATIVE_REJECTED]: "nativeRejected",
      [SetupFailure.SETUP_FAILURE_SOURCE_UNAVAILABLE]: "sourceUnavailable",
      [SetupFailure.SETUP_FAILURE_RETIREMENT_UNKNOWN]: "retirementUnknown",
    };
    const reason = reasons[preparation.failed];
    if (!reason) throw new Error("Input preparation failure is invalid");
    return { type: "failed", reason };
  }
  if (preparation.sessionReady) return { type: "ready" };
  if (preparation.possibleForwarding !== undefined)
    return { type: "preparing" };
  return undefined;
}
export function stopObservation(
  value: Wire.StopReceipt | undefined,
): "pending" | "published" | "rejected" | "uncertain" {
  if (!value) return "uncertain";
  if (value.state === StopState.STOP_STATE_CANCEL_PUBLISHED) return "published";
  if (value.state === StopState.STOP_STATE_REJECTED) return "rejected";
  return "pending";
}
