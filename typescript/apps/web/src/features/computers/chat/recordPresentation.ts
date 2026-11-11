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
export function recordPreview(record: ConversationRecord): string {
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
  return plainPreview(
    text ||
      record.terminal?.text ||
      record.description ||
      (record.plan?.type === "text"
        ? record.plan.text
        : record.plan?.items.map((item) => item.content).join(" · ")) ||
      record.question?.title ||
      "",
  );
}
