import assert from "node:assert/strict";
import test from "node:test";
import { decodeQuestion } from "../dist/conversation-decode.js";

test("retained form presentation keeps schema titles and original choice descriptions", () => {
  const question = decodeQuestion(
    {
      mode: "form",
      sessionId: "native",
      message: "Review the values",
      requestedSchema: {
        type: "object",
        title: "Release configuration",
        description: "Choose explicitly",
        required: ["channel"],
        properties: {
          channel: {
            type: "string",
            title: "Channel",
            description: null,
            minLength: null,
            maxLength: null,
            pattern: null,
            format: null,
            default: null,
            enum: null,
            oneOf: [
              {
                const: "preview",
                title: "Preview",
                description: "Early testing",
              },
              { const: "stable", title: "Stable", description: null },
            ],
          },
        },
      },
    },
    "native",
  );
  assert.equal(question.type, "form");
  assert.equal(question.title, "Review the values");
  assert.equal(question.schemaTitle, "Release configuration");
  assert.equal(question.description, "Choose explicitly");
  assert.deepEqual(question.fields[0].choices, [
    { value: "preview", title: "Preview", description: "Early testing" },
    { value: "stable", title: "Stable", description: undefined },
  ]);
});
