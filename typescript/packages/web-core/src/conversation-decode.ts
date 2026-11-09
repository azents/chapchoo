import { z } from "zod";
import { parse } from "lossless-json";
import type {
  Choice,
  ContentPart,
  Question,
  QuestionField,
} from "./conversation-types.js";

const text = z.looseObject({ type: z.literal("text"), text: z.string() });
const media = z.looseObject({
  type: z.enum(["image", "audio"]),
  data: z.string(),
  mimeType: z.string(),
});
const link = z.looseObject({
  type: z.literal("resource_link"),
  uri: z.string(),
  name: z.string(),
  title: z.string().nullish(),
});
const resource = z.looseObject({
  type: z.literal("resource"),
  resource: z.looseObject({ uri: z.string(), text: z.string().optional() }),
});
const diff = z.looseObject({
  type: z.literal("diff"),
  path: z.string(),
  oldText: z.string().nullish(),
  newText: z.string(),
});
const toolContent = z.looseObject({
  type: z.literal("content"),
  content: z.unknown(),
});
const tagged = z.looseObject({ type: z.string() });
/** ACP fields explicitly permit extension properties; unrecognized metadata is ignored. */
export function decodeContent(value: unknown): ContentPart {
  const wrapped = toolContent.safeParse(value);
  if (wrapped.success) return decodeContent(wrapped.data.content);
  const t = text.safeParse(value);
  if (t.success) return { type: "text", text: t.data.text };
  const d = diff.safeParse(value);
  if (d.success)
    return {
      type: "diff",
      path: d.data.path,
      oldText: d.data.oldText ?? undefined,
      newText: d.data.newText,
    };
  const m = media.safeParse(value);
  if (m.success)
    return { type: m.data.type, data: m.data.data, mimeType: m.data.mimeType };
  const l = link.safeParse(value);
  if (l.success)
    return {
      type: "resource",
      uri: l.data.uri,
      title: l.data.title ?? l.data.name,
    };
  const r = resource.safeParse(value);
  if (r.success)
    return {
      type: "resource",
      uri: r.data.resource.uri,
      title: r.data.resource.uri,
      text: r.data.resource.text,
    };
  if (typeof value === "string") return { type: "text", text: value };
  return {
    type: "unsupported",
    kind: tagged.safeParse(value).data?.type ?? "content",
  };
}
export function decodeLines(bytes: Uint8Array): unknown[] {
  const text = new TextDecoder("utf-8", { fatal: true }).decode(bytes);
  if (!text) return [];
  if (!text.endsWith("\n")) throw new Error("Conversation field is incomplete");
  return text
    .slice(0, -1)
    .split("\n")
    .map((line): unknown =>
      parse(line, null, (value) =>
        /^-?\d+$/.test(value) && !Number.isSafeInteger(Number(value))
          ? BigInt(value)
          : Number(value),
      ),
    );
}
export function decodeString(values: unknown[]): string | undefined {
  if (!values.length) return undefined;
  return z.string().parse(values.at(-1));
}
const titled = z.looseObject({
  const: z.string(),
  title: z.string(),
  description: z.string().nullish(),
});
const common = z.looseObject({
  type: z.string(),
  title: z.string().nullish(),
  description: z.string().nullish(),
});
const stringField = common.extend({
  type: z.literal("string"),
  minLength: z.number().int().nonnegative().nullish(),
  maxLength: z.number().int().nonnegative().nullish(),
  pattern: z.string().nullish(),
  format: z.string().nullish(),
  default: z.string().nullish(),
  enum: z.array(z.string()).nonempty().nullish(),
  oneOf: z.array(titled).nonempty().nullish(),
});
const numeric = z.union([z.number().finite(), z.bigint()]);
const numberField = common.extend({
  type: z.enum(["number", "integer"]),
  minimum: numeric.nullish(),
  maximum: numeric.nullish(),
  default: numeric.nullish(),
});
const booleanField = common.extend({
  type: z.literal("boolean"),
  default: z.boolean().nullish(),
});
const arrayField = common.extend({
  type: z.literal("array"),
  minItems: z.number().int().nonnegative().nullish(),
  maxItems: z.number().int().nonnegative().nullish(),
  default: z.array(z.string()).nullish(),
  items: z.looseObject({
    type: z.literal("string"),
    enum: z.array(z.string()).nonempty().optional(),
    anyOf: z.array(titled).nonempty().optional(),
  }),
});
const permission = z.looseObject({
  sessionId: z.string(),
  title: z.string(),
  description: z.string().nullish(),
  options: z
    .array(
      z.looseObject({
        optionId: z.string().min(1),
        name: z.string(),
        kind: z.string(),
      }),
    )
    .nonempty(),
  subject: z.unknown().optional(),
});
const commandSubject = z.looseObject({
  type: z.literal("command"),
  command: z.string(),
  cwd: z.string(),
});
const toolSubject = z.looseObject({
  type: z.literal("tool_call"),
  toolCall: z.looseObject({
    title: z.string().nullish(),
    content: z.array(z.unknown()).nullish(),
    locations: z
      .array(z.looseObject({ path: z.string(), line: z.number().optional() }))
      .nullish(),
  }),
});
const form = z.looseObject({
  mode: z.literal("form"),
  sessionId: z.string(),
  message: z.string(),
  requestedSchema: z.looseObject({
    type: z.literal("object"),
    title: z.string().nullish(),
    description: z.string().nullish(),
    properties: z.record(z.string(), z.unknown()),
    required: z.array(z.string()).nullish(),
  }),
});
export function decodeLocations(
  values: unknown[],
): { path: string; line?: number }[] {
  if (!values.length) return [];
  return z
    .array(z.looseObject({ path: z.string(), line: z.number().optional() }))
    .parse(values.at(-1));
}
function choices(
  values: string[] | null | undefined,
  options: z.infer<typeof titled>[] | null | undefined,
): Choice[] | undefined {
  return (
    options?.map((option) => ({
      value: option.const,
      title: option.title,
      description: option.description ?? undefined,
    })) ?? values?.map((value) => ({ value, title: value }))
  );
}
function decodeField(
  name: string,
  value: unknown,
  required: boolean,
): QuestionField {
  const base = common.parse(value);
  const identity = {
    name,
    title: base.title ?? name,
    description: base.description ?? undefined,
    required,
  };
  const s = stringField.safeParse(value);
  if (s.success && s.data.format !== "password")
    return {
      ...identity,
      type: "text",
      minLength: s.data.minLength ?? undefined,
      maxLength: s.data.maxLength ?? undefined,
      pattern: s.data.pattern ?? undefined,
      format: s.data.format ?? undefined,
      default: s.data.default ?? undefined,
      choices: choices(s.data.enum, s.data.oneOf),
    };
  const n = numberField.safeParse(value);
  if (n.success)
    return {
      ...identity,
      type: n.data.type,
      minimum: n.data.minimum ?? undefined,
      maximum: n.data.maximum ?? undefined,
      default: n.data.default ?? undefined,
    };
  const b = booleanField.safeParse(value);
  if (b.success)
    return {
      ...identity,
      type: "boolean",
      default: b.data.default ?? undefined,
    };
  const a = arrayField.safeParse(value);
  if (a.success) {
    const options = choices(a.data.items.enum, a.data.items.anyOf);
    if (options)
      return {
        ...identity,
        type: "strings",
        choices: options,
        minItems: a.data.minItems ?? undefined,
        maxItems: a.data.maxItems ?? undefined,
        default: a.data.default ?? undefined,
      };
  }
  return { ...identity, type: "unsupported" };
}
export function decodeQuestion(value: unknown, session: string): Question {
  const p = permission.safeParse(value);
  if (
    p.success &&
    p.data.sessionId === session &&
    new Set(p.data.options.map((o) => o.optionId)).size ===
      p.data.options.length
  ) {
    const command = commandSubject.safeParse(p.data.subject);
    const tool = toolSubject.safeParse(p.data.subject);
    return {
      type: "permission",
      title: p.data.title,
      description: p.data.description ?? undefined,
      options: p.data.options.map((o) => ({
        id: o.optionId,
        title: o.name,
        scope: o.kind,
      })),
      content: command.success
        ? [{ type: "text", text: command.data.command }]
        : tool.success
          ? (tool.data.toolCall.content ?? []).map(decodeContent)
          : [],
      locations: command.success
        ? [{ path: command.data.cwd }]
        : tool.success
          ? (tool.data.toolCall.locations ?? [])
          : [],
    };
  }
  const f = form.safeParse(value);
  if (f.success && f.data.sessionId === session) {
    const schema = f.data.requestedSchema;
    const fields = Object.entries(schema.properties).map(([name, property]) =>
      decodeField(name, property, schema.required?.includes(name) ?? false),
    );
    if (
      fields.length <= 64 &&
      !fields.some((field) => field.required && field.type === "unsupported") &&
      (schema.required ?? []).every((name) =>
        fields.some((field) => field.name === name),
      )
    )
      return {
        type: "form",
        title: f.data.message,
        schemaTitle: schema.title ?? undefined,
        description: schema.description ?? undefined,
        fields,
      };
  }
  return { type: "unsupported", title: "This request cannot be answered" };
}
const plan = z.looseObject({
  type: z.literal("items"),
  entries: z.array(
    z.looseObject({
      content: z.string(),
      status: z.string(),
      priority: z.string().optional(),
    }),
  ),
});
export function decodePlan(value: unknown):
  | {
      type: "items";
      items: { content: string; status: string; priority?: string }[];
    }
  | { type: "text"; text: string } {
  const parsed = plan.safeParse(value);
  if (parsed.success) return { type: "items", items: parsed.data.entries };
  const markdown = z
    .looseObject({ type: z.literal("markdown"), content: z.string() })
    .safeParse(value);
  if (markdown.success) return { type: "text", text: markdown.data.content };
  return { type: "text", text: "Unsupported plan content" };
}
