import remarkGfm from "remark-gfm";
import remarkParse from "remark-parse";
import { unified } from "unified";
import { z } from "zod";
import type {
  ContentPart,
  ConversationRecord,
} from "@chapchoo/web-core/conversation-types";
import type { Nodes } from "mdast";

export const toolKinds = [
  "read",
  "edit",
  "delete",
  "move",
  "search",
  "execute",
  "think",
  "fetch",
  "switch_mode",
  "other",
] as const;
export type ToolKind = (typeof toolKinds)[number];
export function toolKind(value?: string): ToolKind {
  return toolKinds.find((kind) => kind === value) ?? "other";
}
const markdown = unified().use(remarkParse).use(remarkGfm);
function nodeText(node: Nodes): string {
  if (node.type === "html" || node.type === "definition") {
    return "";
  }
  if (node.type === "image" || node.type === "imageReference") {
    return node.alt ?? "";
  }
  if (node.type === "break") {
    return " ";
  }
  if ("value" in node) {
    return node.value;
  }
  if ("children" in node) {
    const inline = [
      "paragraph",
      "heading",
      "emphasis",
      "strong",
      "delete",
      "link",
      "linkReference",
      "tableCell",
    ].includes(node.type);
    return node.children
      .map((child) => nodeText(child))
      .join(inline ? "" : " ");
  }
  return "";
}
export function plainPreview(value: string): string {
  return Array.from(
    nodeText(markdown.parse(value)).replace(/\s+/gu, " ").trim(),
  )
    .slice(0, 180)
    .join("");
}
/** Native text chunks concatenate verbatim; non-text parts establish a boundary. */
export function contentText(parts: ContentPart[]): string {
  return parts
    .map((part) =>
      part.type === "text"
        ? part.text
        : `\n${part.type === "resource" ? part.title : ""}\n`,
    )
    .join("");
}
const inputSchema = z.object({
  command: z.string().optional(),
  cmd: z.string().optional(),
  path: z.string().optional(),
  file_path: z.string().optional(),
  query: z.string().optional(),
  pattern: z.string().optional(),
  url: z.string().optional(),
  source: z.string().optional(),
  destination: z.string().optional(),
  cwd: z.string().optional(),
  mode: z.string().optional(),
});
export function recordInput(
  record: ConversationRecord,
): z.infer<typeof inputSchema> {
  try {
    return inputSchema.parse(JSON.parse(record.toolInput?.json ?? "{}"));
  } catch {
    return {};
  }
}
const publicationInput = z.looseObject({
  server: z.literal("chapchoo"),
  tool: z.literal("send_message"),
  arguments: z.unknown().optional(),
});
const publicationText = z.looseObject({ text: z.string() });
const textResult = z.looseObject({ type: z.literal("text"), text: z.string() });
const publicationOutput = z.looseObject({
  result: z.looseObject({ content: z.array(z.unknown()).optional() }).nullish(),
  error: z.looseObject({ message: z.string() }).nullish(),
});
function publicationResult(json?: string): string | null {
  if (json === void 0) {
    return null;
  }
  try {
    const output = publicationOutput.safeParse(JSON.parse(json));
    if (!output.success) {
      return null;
    }
    const text = [
      ...(output.data.result?.content ?? []).flatMap((part) => {
        const parsed = textResult.safeParse(part);
        return parsed.success ? [parsed.data.text] : [];
      }),
      ...(output.data.error ? [output.data.error.message] : []),
    ];
    return text.length ? text.join("\n") : null;
  } catch {
    return null;
  }
}
/** Captured invocation intent is not acknowledgement of physical publication. */
export function recordPublication(
  record: ConversationRecord,
): { text?: string; resultText: string | null } | null {
  if (record.kind !== "tool" || !record.toolInput) {
    return null;
  }
  try {
    const input = publicationInput.safeParse(JSON.parse(record.toolInput.json));
    if (!input.success) {
      return null;
    }
    // Extract known display fields without adjudicating domain arguments.
    // Original payloads, including rejected extra fields, stay inspectable.
    const body = publicationText.safeParse(input.data.arguments);
    return {
      text: body.success ? body.data.text : void 0,
      resultText: publicationResult(record.toolOutput?.json),
    };
  } catch {
    return null;
  }
}
export function recordPreview(record: ConversationRecord): string {
  const publication = recordPublication(record);
  if (publication) {
    return plainPreview(publication.text ?? "");
  }
  const input = recordInput(record);
  const path = record.locations[0]?.path ?? input.path ?? input.file_path;
  const kind = toolKind(record.toolKind);
  const target =
    kind === "execute"
      ? (input.command ?? input.cmd)
      : kind === "search"
        ? (input.query ?? input.pattern)
        : kind === "fetch"
          ? input.url
          : kind === "move"
            ? input.source && input.destination
              ? `${input.source} → ${input.destination}`
              : path
            : kind === "switch_mode"
              ? input.mode
              : ["read", "edit", "delete"].includes(kind)
                ? (path ??
                  record.content.find((part) => part.type === "diff")?.path)
                : void 0;
  const text =
    record.kind === "tool"
      ? (target ?? record.title ?? contentText(record.content))
      : contentText(record.content);
  const question = record.question;
  const questionPreview =
    question && question.type !== "unsupported"
      ? question.description ||
        (question.type === "permission"
          ? contentText(question.content)
          : question.schemaTitle)
      : "";
  return plainPreview(
    text ||
      record.terminal?.text ||
      record.description ||
      (record.plan?.type === "text"
        ? record.plan.text
        : record.plan?.items.map((item) => item.content).join(" · ")) ||
      questionPreview ||
      question?.title ||
      "",
  );
}
