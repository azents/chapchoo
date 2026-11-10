import { Ajv } from "ajv";
import addFormats from "ajv-formats";

const validator = new Ajv({ strict: true });
addFormats.default(validator, {
  mode: "full",
  formats: ["email", "uri", "date", "date-time"],
});
const formats = new Map([
  ["email", validator.compile({ type: "string", format: "email" })],
  ["uri", validator.compile({ type: "string", format: "uri" })],
  ["date", validator.compile({ type: "string", format: "date" })],
  ["date-time", validator.compile({ type: "string", format: "date-time" })],
]);
/** Only producer-defined known formats constrain input; unknown formats are annotations. */
export function validConversationFormat(
  format: string | undefined,
  value: string,
): boolean {
  const check = format === undefined ? undefined : formats.get(format);
  return check === undefined || check(value);
}
