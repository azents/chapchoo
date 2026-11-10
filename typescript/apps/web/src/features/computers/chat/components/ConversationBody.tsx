"use client";
import {
  draftField,
  initialQuestionDraft,
} from "@chapchoo/web-core/conversation-form";
import { Button, Group, Image, Stack, Text } from "@mantine/core";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { inputStatus, nativeStatus, permissionScope } from "../copy";
import classes from "./ConversationBody.module.css";
import type { ChatCopy } from "../copy";
import type { ConversationControls } from "../types";
import type {
  DraftValue,
  QuestionDraft,
} from "@chapchoo/web-core/conversation-form";
import type {
  ContentPart,
  ConversationAnswer,
  ConversationRecord,
  FormValue,
  QuestionField,
} from "@chapchoo/web-core/conversation-types";
import type { ReactElement } from "react";

function Markdown({ text }: { text: string }): ReactElement {
  return (
    <div className={classes.markdown}>
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        skipHtml
        components={{
          a: (props) => (
            <a {...props} target="_blank" rel="noopener noreferrer" />
          ),
          img: ({ alt }) => <span>{alt}</span>,
        }}
      >
        {text}
      </ReactMarkdown>
    </div>
  );
}
export function ConversationContent({
  parts,
  plain = false,
}: {
  parts: ContentPart[];
  plain?: boolean;
}): ReactElement {
  const grouped: ContentPart[] = [];
  for (const part of parts) {
    const last = grouped.at(-1);
    if (part.type === "text" && last?.type === "text") {
      last.text += part.text;
    } else {
      grouped.push({ ...part });
    }
  }
  return (
    <>
      {grouped.map((part, index) =>
        part.type === "text" ? (
          plain ? (
            <p key={index} className={classes.answer}>
              {part.text}
            </p>
          ) : (
            <Markdown key={index} text={part.text} />
          )
        ) : part.type === "diff" ? (
          <section key={index}>
            <strong>{part.path}</strong>
            {part.oldText !== void 0 ? <pre>{part.oldText}</pre> : null}
            <pre>{part.newText}</pre>
          </section>
        ) : part.type === "resource" ? (
          <section key={index}>
            <strong>{part.title}</strong>
            <pre>{part.uri}</pre>
            {part.text !== void 0 ? <pre>{part.text}</pre> : null}
          </section>
        ) : part.type === "image" || part.type === "audio" ? (
          <section key={index}>
            <Text c="dimmed">{part.mimeType}</Text>
            {part.type === "image" &&
            ["image/png", "image/jpeg", "image/gif", "image/webp"].includes(
              part.mimeType,
            ) ? (
              <Image
                src={`data:${part.mimeType};base64,${part.data}`}
                alt={part.mimeType}
                fit="contain"
              />
            ) : part.type === "audio" &&
              [
                "audio/wav",
                "audio/mpeg",
                "audio/ogg",
                "audio/flac",
                "audio/mp4",
              ].includes(part.mimeType) ? (
              <audio
                controls
                src={`data:${part.mimeType};base64,${part.data}`}
                aria-label={part.mimeType}
              />
            ) : null}
            <details>
              <summary>Base64</summary>
              <pre>{part.data}</pre>
            </details>
          </section>
        ) : (
          <Text key={index} c="dimmed">
            {part.kind}
          </Text>
        ),
      )}
    </>
  );
}
function valueText(value: FormValue, copy: ChatCopy): string {
  switch (value.type) {
    case "text":
      return value.value === "" ? copy.emptyValue : value.value;
    case "boolean":
      return value.value ? copy.yes : copy.no;
    case "strings":
      return value.value.length ? value.value.join("\n") : copy.emptyValue;
    case "integer":
    case "number":
      return String(value.value);
  }
}
function retainedField(
  answer: ConversationAnswer | null,
  name: string,
): FormValue | null {
  return answer?.type === "formAccept" &&
    answer.content &&
    Object.hasOwn(answer.content, name)
    ? (answer.content[name] ?? null)
    : null;
}
function fieldConstraints(field: QuestionField, copy: ChatCopy): string[] {
  const constraints: string[] = [];
  if (field.type === "text") {
    if (field.minLength !== void 0) {
      constraints.push(`${copy.characters} ≥ ${field.minLength}`);
    }
    if (field.maxLength !== void 0) {
      constraints.push(`${copy.characters} ≤ ${field.maxLength}`);
    }
    if (field.pattern !== void 0) {
      constraints.push(`${copy.pattern}: ${field.pattern}`);
    }
    if (field.format !== void 0) {
      constraints.push(`${copy.format}: ${field.format}`);
    }
  } else if (field.type === "strings") {
    if (field.minItems !== void 0) {
      constraints.push(`${copy.items} ≥ ${field.minItems}`);
    }
    if (field.maxItems !== void 0) {
      constraints.push(`${copy.items} ≤ ${field.maxItems}`);
    }
  } else if (field.type === "integer" || field.type === "number") {
    if (field.minimum !== void 0) {
      constraints.push(`≥ ${String(field.minimum)}`);
    }
    if (field.maximum !== void 0) {
      constraints.push(`≤ ${String(field.maximum)}`);
    }
  }
  return constraints;
}
function FormField({
  field,
  draft,
  retained,
  completed,
  disabled,
  copy,
  onChange,
  prefix,
}: {
  field: QuestionField;
  draft: QuestionDraft;
  retained: FormValue | null;
  completed: boolean;
  disabled: boolean;
  copy: ChatCopy;
  onChange: (value: DraftValue | null) => void;
  prefix: string;
}): ReactElement {
  const value = draftField(draft, field.name);
  const error = Object.hasOwn(draft.errors, field.name)
    ? draft.errors[field.name]
    : null;
  const id = `${prefix}-${encodeURIComponent(field.name)}`;
  const help = [field.description, ...fieldConstraints(field, copy)]
    .filter(Boolean)
    .join(" · ");
  const active = completed ? retained !== null : value !== void 0;
  const stringValue = completed
    ? retained?.type === "text"
      ? retained.value
      : ""
    : value?.type === "text"
      ? value.value
      : "";
  const boolValue = completed
    ? retained?.type === "boolean"
      ? retained.value
      : void 0
    : value?.type === "boolean"
      ? value.value
      : void 0;
  const selected = completed
    ? retained?.type === "strings"
      ? retained.value
      : []
    : value?.type === "strings"
      ? value.value
      : [];
  const defaultInput: DraftValue =
    field.type === "boolean"
      ? { type: "boolean", value: false }
      : field.type === "strings"
        ? { type: "strings", value: [] }
        : { type: "text", value: "" };
  return (
    <fieldset
      className={classes.field}
      disabled={disabled || completed}
      aria-describedby={`${id}-help ${id}-error`}
    >
      <legend>
        {field.title}{" "}
        <small>{field.required ? copy.required : copy.optional}</small>
      </legend>
      {help ? <p id={`${id}-help`}>{help}</p> : null}
      {!field.required && !completed && field.type !== "unsupported" ? (
        <label>
          <input
            type="checkbox"
            checked={active}
            onChange={(event) =>
              onChange(event.target.checked ? defaultInput : null)
            }
          />
          {copy.include}
        </label>
      ) : null}
      {field.type === "text" && field.choices ? (
        <div className={classes.choices}>
          {field.choices.map((choice) => (
            <label key={choice.value}>
              <input
                type="radio"
                name={id}
                checked={active && stringValue === choice.value}
                disabled={!active && !field.required}
                onChange={() => onChange({ type: "text", value: choice.value })}
              />
              <span>
                {choice.title}
                {choice.description ? (
                  <small>{choice.description}</small>
                ) : null}
                {completed && active && stringValue === choice.value
                  ? ` · ${copy.selected}`
                  : ""}
              </span>
            </label>
          ))}
        </div>
      ) : field.type === "boolean" ? (
        <div className={classes.choices}>
          {[true, false].map((choice) => (
            <label key={String(choice)}>
              <input
                type="radio"
                name={id}
                checked={boolValue === choice}
                disabled={!active && !field.required}
                onChange={() => onChange({ type: "boolean", value: choice })}
              />
              {choice ? copy.yes : copy.no}
              {completed && boolValue === choice ? ` · ${copy.selected}` : ""}
            </label>
          ))}
        </div>
      ) : field.type === "strings" ? (
        <div className={classes.choices}>
          {field.choices.map((choice) => (
            <label key={choice.value}>
              <input
                type="checkbox"
                checked={selected.includes(choice.value)}
                disabled={!active && !field.required}
                onChange={(event) =>
                  onChange({
                    type: "strings",
                    value: event.target.checked
                      ? [...selected, choice.value]
                      : selected.filter((value) => value !== choice.value),
                  })
                }
              />
              {choice.title}
              {choice.description ? <small>{choice.description}</small> : null}
              {completed && selected.includes(choice.value)
                ? ` · ${copy.selected}`
                : ""}
            </label>
          ))}
        </div>
      ) : field.type === "unsupported" ? (
        <Text c="dimmed">{copy.unsupported}</Text>
      ) : !completed ? (
        <label>
          <span className={classes.srOnly}>{field.title}</span>
          <input
            type="text"
            inputMode={
              field.type === "integer"
                ? "numeric"
                : field.type === "number"
                  ? "decimal"
                  : "text"
            }
            value={value?.type === "text" ? value.value : ""}
            disabled={!active && !field.required}
            aria-invalid={Boolean(error)}
            onChange={(event) =>
              onChange({ type: "text", value: event.target.value })
            }
          />
        </label>
      ) : null}
      {completed ? (
        <p className={classes.answer}>
          {retained ? valueText(retained, copy) : copy.omitted}
        </p>
      ) : null}
      {error ? (
        <p id={`${id}-error`} role="alert">
          {error === "required" ? copy.required : copy.invalid}
        </p>
      ) : null}
    </fieldset>
  );
}
export function ConversationQuestion({
  record,
  controls,
  copy,
  writable,
}: {
  record: ConversationRecord;
  controls: ConversationControls;
  copy: ChatCopy;
  writable: boolean;
}): ReactElement {
  const question = record.question;
  if (!question || question.type === "unsupported") {
    return <Text role="alert">{copy.unsupported}</Text>;
  }
  const draft = controls.drafts[record.key] ?? initialQuestionDraft(question);
  const retained = record.answer?.value ?? draft.answer;
  const completed = retained !== void 0;
  const disabled =
    !writable ||
    !record.responder ||
    (draft.state !== "editing" && draft.state !== "invalid") ||
    completed;
  const prefix = `question-${encodeURIComponent(record.key)}`;
  const selected =
    retained?.type === "permission" ? retained.optionId : draft.optionId;
  return (
    <section className={classes.question} aria-label={question.title}>
      <h3>{question.title}</h3>
      {question.type === "form" && question.schemaTitle ? (
        <Text fw={600}>{question.schemaTitle}</Text>
      ) : null}
      {question.description ? <Markdown text={question.description} /> : null}
      {question.type === "permission" ? (
        <>
          <ConversationContent parts={question.content} />
          {question.locations.map((location) => (
            <pre key={`${location.path}:${location.line}`}>
              {location.path}
              {location.line !== void 0 ? `:${location.line}` : ""}
            </pre>
          ))}
          <fieldset className={classes.field} disabled={disabled}>
            <legend>{copy.permission}</legend>
            <div className={classes.choices}>
              {question.options.map((option) => (
                <label key={option.id}>
                  <input
                    type="radio"
                    name={prefix}
                    checked={selected === option.id}
                    onChange={() =>
                      controls.onQuestionOption(record, option.id)
                    }
                  />
                  <span>
                    {option.title}
                    <small>{permissionScope(copy, option.scope)}</small>
                    {completed && selected === option.id ? (
                      <strong>{copy.selected}</strong>
                    ) : null}
                  </span>
                </label>
              ))}
            </div>
          </fieldset>
        </>
      ) : (
        question.fields.map((field) => (
          <FormField
            key={field.name}
            field={field}
            draft={draft}
            retained={retainedField(retained ?? null, field.name)}
            completed={completed}
            disabled={disabled}
            copy={copy}
            prefix={prefix}
            onChange={(value) =>
              controls.onQuestionField(record, field.name, value)
            }
          />
        ))
      )}
      {draft.errors.request ? <Text role="alert">{copy.invalid}</Text> : null}
      {completed ? (
        <Text role="status">
          {record.answer
            ? copy[
                record.answer.delivery === "possible"
                  ? "uncertain"
                  : record.answer.delivery
              ]
            : copy.accepted}
          {retained.type === "formDecline"
            ? ` · ${copy.decline}`
            : retained.type === "formCancel" ||
                retained.type === "permissionCancel"
              ? ` · ${copy.cancel}`
              : ""}
        </Text>
      ) : (
        <Group mt="md">
          <Button
            disabled={disabled}
            loading={draft.state === "sending"}
            onClick={() => controls.onAnswer(record)}
          >
            {copy.answer}
          </Button>
          {question.type === "form" ? (
            <Button
              variant="default"
              disabled={disabled}
              onClick={() => controls.onAnswer(record, { type: "formDecline" })}
            >
              {copy.decline}
            </Button>
          ) : null}
          <Button
            variant="subtle"
            disabled={disabled}
            onClick={() =>
              controls.onAnswer(record, {
                type:
                  question.type === "form" ? "formCancel" : "permissionCancel",
              })
            }
          >
            {copy.cancel}
          </Button>
        </Group>
      )}
      {!record.answer &&
      (draft.state === "uncertain" ||
        draft.state === "rejected" ||
        draft.state === "invalid") ? (
        <Text role="alert">{copy[draft.state]}</Text>
      ) : null}
      {!record.answer && (draft.state === "uncertain" || !record.responder) ? (
        <Button variant="subtle" onClick={controls.onRead}>
          {copy.retry}
        </Button>
      ) : null}
    </section>
  );
}
export function ConversationBody({
  controls,
  copy,
  writable,
  agentName,
}: {
  controls: ConversationControls;
  copy: ChatCopy;
  writable: boolean;
  agentName: string;
}): ReactElement {
  return (
    <Stack gap="lg">
      {controls.region.older || controls.region.inputOlder ? (
        <Button
          variant="subtle"
          loading={controls.read === "loading"}
          onClick={controls.onOlder}
        >
          {copy.older}
        </Button>
      ) : null}
      {controls.read === "error" ? (
        <Group role="alert">
          <Text>{copy.error}</Text>
          <Button variant="subtle" onClick={controls.onRead}>
            {copy.retry}
          </Button>
        </Group>
      ) : controls.read === "loading" ? (
        <Text role="status">{copy.loading}</Text>
      ) : null}
      {controls.region.records.map((record) => (
        <article
          id={`conversation-${encodeURIComponent(record.key)}`}
          key={record.key}
          data-kind={record.kind}
          className={classes.record}
          tabIndex={-1}
        >
          {record.kind === "user" || record.kind === "assistant" ? (
            <>
              {record.kind === "assistant" ? (
                <strong>{agentName}</strong>
              ) : null}
              <ConversationContent
                parts={record.content}
                plain={record.kind === "user"}
              />
            </>
          ) : record.kind === "request" ? (
            <ConversationQuestion
              record={record}
              controls={controls}
              copy={copy}
              writable={writable && controls.read === "ready"}
            />
          ) : record.kind === "state" ? null : (
            <details>
              <summary>
                {record.title ??
                  (record.kind === "thought"
                    ? copy.thought
                    : record.kind === "tool"
                      ? copy.tools
                      : record.kind === "plan"
                        ? copy.plan
                        : record.kind === "terminal"
                          ? copy.terminal
                          : copy.notice)}
                {record.status ? ` · ${nativeStatus(copy, record.status)}` : ""}
              </summary>
              {record.description ? (
                <Markdown text={record.description} />
              ) : null}
              {record.toolInput ? (
                <section aria-label={copy.toolInput}>
                  <Text size="sm" fw={600}>
                    {copy.toolInput}
                  </Text>
                  <pre>{record.toolInput.json}</pre>
                </section>
              ) : null}
              {record.toolOutput ? (
                <section aria-label={copy.toolOutput}>
                  <Text size="sm" fw={600}>
                    {copy.toolOutput}
                  </Text>
                  <pre>{record.toolOutput.json}</pre>
                </section>
              ) : null}
              {record.plan?.type === "items" ? (
                <ol>
                  {record.plan.items.map((item, index) => (
                    <li key={index}>
                      {item.content} · {nativeStatus(copy, item.status)}
                    </li>
                  ))}
                </ol>
              ) : record.plan?.type === "text" ? (
                <Markdown text={record.plan.text} />
              ) : null}
              {record.locations.map((location) => (
                <pre key={location.path}>
                  {location.path}
                  {location.line !== void 0 ? `:${location.line}` : ""}
                </pre>
              ))}
              {record.terminal ? (
                <>
                  <pre>{record.terminal.text}</pre>
                  {record.terminal.exitCode !== void 0 ? (
                    <p>
                      {copy.terminal} · {String(record.terminal.exitCode)}
                    </p>
                  ) : null}
                  {record.terminal.signal ? (
                    <p>{record.terminal.signal}</p>
                  ) : null}
                  <details>
                    <summary>Base64</summary>
                    <pre>{record.terminal.base64}</pre>
                  </details>
                </>
              ) : null}
              <ConversationContent parts={record.content} />
            </details>
          )}
        </article>
      ))}
      {(controls.region.inputs ?? []).map((input) =>
        input.insertedNativeMessageId &&
        controls.region.records.some(
          (record) =>
            record.kind === "user" &&
            record.nativeId === input.insertedNativeMessageId,
        ) ? (
          <details className={classes.pending} key={input.id}>
            <summary>
              {copy.message} · {inputStatus(copy, input)}
            </summary>
            <pre>{input.text}</pre>
          </details>
        ) : (
          <section className={classes.record} data-kind="user" key={input.id}>
            <p className={classes.answer}>{input.text}</p>
            <Text
              size="xs"
              c={input.observation === "failed" ? "red" : "dimmed"}
              ta="right"
              role="status"
            >
              {inputStatus(copy, input)}
            </Text>
            {input.observation === "uncertain" ? (
              <Button variant="subtle" onClick={controls.onRead}>
                {copy.retry}
              </Button>
            ) : null}
          </section>
        ),
      )}
      {controls.inputs.map((input) => (
        <section
          key={input.requestId}
          className={classes.record}
          data-kind="user"
        >
          <p className={classes.answer}>{input.text}</p>
          <Text
            size="xs"
            c={input.observation === "failed" ? "red" : "dimmed"}
            ta="right"
            role="status"
          >
            {copy[input.observation]}
          </Text>
          {input.observation === "uncertain" ? (
            <Button variant="subtle" onClick={controls.onRead}>
              {copy.retry}
            </Button>
          ) : null}
        </section>
      ))}
    </Stack>
  );
}
