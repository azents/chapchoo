import { validConversationFormat } from "./conversation-formats.js";
import type {
  ConversationAnswer,
  FormValue,
  Question,
  QuestionField,
} from "./conversation-types.js";
export type DraftValue =
  | { type: "text"; value: string }
  | { type: "boolean"; value: boolean }
  | { type: "strings"; value: string[] };
export interface QuestionDraft {
  optionId?: string;
  fields: Partial<Record<string, DraftValue>>;
  errors: Record<string, "required" | "invalid">;
  state: "editing" | "sending" | "accepted" | "uncertain" | "rejected";
  answer?: ConversationAnswer;
}
export function draftField(
  draft: QuestionDraft,
  name: string,
): DraftValue | undefined {
  return Object.hasOwn(draft.fields, name) ? draft.fields[name] : undefined;
}
export function initialQuestionDraft(question: Question): QuestionDraft {
  const fields: [string, DraftValue][] = [];
  if (question.type === "form")
    for (const field of question.fields) {
      if (field.type === "unsupported" || field.default === undefined) continue;
      const value: DraftValue =
        field.type === "boolean"
          ? { type: "boolean", value: field.default }
          : field.type === "strings"
            ? { type: "strings", value: field.default }
            : { type: "text", value: String(field.default) };
      if (readFormValue(field, value)) fields.push([field.name, value]);
    }
  return { fields: Object.fromEntries(fields), errors: {}, state: "editing" };
}
function readFormValue(
  field: QuestionField,
  draft: DraftValue,
): FormValue | undefined {
  switch (field.type) {
    case "unsupported":
      return undefined;
    case "boolean":
      return draft.type === "boolean"
        ? { type: "boolean", value: draft.value }
        : undefined;
    case "strings":
      return draft.type === "strings" &&
        (field.minItems === undefined ||
          draft.value.length >= field.minItems) &&
        (field.maxItems === undefined ||
          draft.value.length <= field.maxItems) &&
        new Set(draft.value).size === draft.value.length &&
        draft.value.every((value) =>
          field.choices.some((choice) => choice.value === value),
        )
        ? { type: "strings", value: draft.value }
        : undefined;
    case "text": {
      if (draft.type !== "text") return undefined;
      const value = draft.value;
      if (
        (field.minLength !== undefined &&
          [...value].length < field.minLength) ||
        (field.maxLength !== undefined &&
          [...value].length > field.maxLength) ||
        (field.choices &&
          !field.choices.some((choice) => choice.value === value))
      )
        return undefined;
      try {
        if (
          field.pattern !== undefined &&
          !new RegExp(field.pattern, "u").test(value)
        )
          return undefined;
      } catch {
        return undefined;
      }
      if (!validConversationFormat(field.format, value)) return undefined;
      return { type: "text", value };
    }
    case "integer": {
      if (draft.type !== "text" || !/^-?\d+$/.test(draft.value))
        return undefined;
      const value = BigInt(draft.value);
      if (
        value < -9223372036854775808n ||
        value > 9223372036854775807n ||
        (field.minimum !== undefined && value < field.minimum) ||
        (field.maximum !== undefined && value > field.maximum)
      )
        return undefined;
      return { type: "integer", value };
    }
    case "number": {
      if (draft.type !== "text" || !draft.value.trim()) return undefined;
      const value = Number(draft.value);
      if (
        !Number.isFinite(value) ||
        (field.minimum !== undefined && value < field.minimum) ||
        (field.maximum !== undefined && value > field.maximum)
      )
        return undefined;
      return { type: "number", value };
    }
  }
}
export function reviewQuestionAnswer(
  question: Question,
  draft: QuestionDraft,
): { answer?: ConversationAnswer; errors: QuestionDraft["errors"] } {
  if (question.type === "unsupported")
    return { errors: { request: "invalid" } };
  if (question.type === "permission")
    return question.options.some((option) => option.id === draft.optionId) &&
      draft.optionId !== undefined
      ? { answer: { type: "permission", optionId: draft.optionId }, errors: {} }
      : { errors: { request: "required" } };
  const errors: [string, "required" | "invalid"][] = [];
  const content: [string, FormValue][] = [];
  for (const field of question.fields) {
    const input = draftField(draft, field.name);
    if (!input) {
      if (field.required) errors.push([field.name, "required"]);
      continue;
    }
    const value = readFormValue(field, input);
    if (value) content.push([field.name, value]);
    else if (field.type !== "unsupported") errors.push([field.name, "invalid"]);
    else if (field.required) errors.push([field.name, "invalid"]);
  }
  return {
    answer: errors.length
      ? undefined
      : { type: "formAccept", content: Object.fromEntries(content) },
    errors: Object.fromEntries(errors),
  };
}
