"use client";
import { CharacterArtwork } from "@chapchoo/ui/CharacterArtwork";
import {
  draftField,
  initialQuestionDraft,
} from "@chapchoo/web-core/conversation-form";
import { Button, Group, Image, Stack, Text } from "@mantine/core";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { nativeStatus, permissionScope } from "../copy";
import classes from "./ConversationBody.module.css";
import {
  ContentDisclosure,
  ConversationRecordDisclosure,
} from "./ConversationRecord";
import { RecordDiff } from "./RecordDiff";
import { ToolRecordDetails } from "./ToolRecordDetails";
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
import type { ReactElement, ReactNode } from "react";

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
function UserMessageBody({
  children,
  copy,
}: {
  children: ReactNode;
  copy: ChatCopy;
}): ReactElement {
  return (
    <>
      <Text size="xs" c="dimmed" ta="right" className={classes.userAuthor}>
        {copy.you}
      </Text>
      <div className={classes.userBubble}>{children}</div>
    </>
  );
}
export function ConversationContent({
  parts,
  rawText = false,
}: {
  parts: ContentPart[];
  rawText?: boolean;
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
          rawText ? (
            <pre key={index}>{part.text}</pre>
          ) : (
            <Markdown key={index} text={part.text} />
          )
        ) : part.type === "diff" ? (
          <RecordDiff
            key={index}
            path={part.path}
            oldText={part.oldText}
            newText={part.newText}
          />
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
            <ContentDisclosure title="Base64">
              <pre>{part.data}</pre>
            </ContentDisclosure>
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
  idPrefix = "question",
}: {
  record: ConversationRecord;
  controls: ConversationControls;
  copy: ChatCopy;
  writable: boolean;
  idPrefix?: string;
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
  const prefix = `${idPrefix}-${encodeURIComponent(record.key)}`;
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
    </section>
  );
}
export function ConversationBody({
  controls,
  copy,
  writable,
  agentName,
  agentProfile,
  initiallyExpandedKey = null,
  surface = "transcript",
}: {
  controls: ConversationControls;
  copy: ChatCopy;
  writable: boolean;
  agentName: string;
  agentProfile?: { basePath: string; assetKey: string };
  initiallyExpandedKey?: string | null;
  surface?: "transcript" | "history";
}): ReactElement {
  return (
    <Stack gap={surface === "history" ? 0 : "lg"}>
      {controls.read === "error" || controls.olderRead === "error" ? (
        <Group role="alert">
          <Text>{copy.error}</Text>
        </Group>
      ) : controls.read === "loading" || controls.olderRead === "loading" ? (
        <Text role="status">{copy.loading}</Text>
      ) : null}
      {controls.region.records
        .filter(
          (record) =>
            surface === "history" ||
            record.kind === "user" ||
            record.kind === "assistant" ||
            record.kind === "request",
        )
        .map((record) => (
          <article
            id={`${surface === "history" ? "history" : "conversation"}-${encodeURIComponent(record.key)}`}
            key={record.key}
            data-kind={record.kind}
            data-surface={surface}
            aria-label={record.kind === "user" ? copy.you : void 0}
            className={classes.record}
            tabIndex={-1}
            {...(surface === "history" && initiallyExpandedKey === record.key
              ? { "data-autofocus": true }
              : {})}
          >
            {record.kind === "user" || record.kind === "assistant" ? (
              surface === "history" ? (
                <ConversationRecordDisclosure
                  record={record}
                  copy={copy}
                  agentName={agentName}
                  initiallyOpened={initiallyExpandedKey === record.key}
                >
                  <ConversationContent parts={record.content} />
                </ConversationRecordDisclosure>
              ) : (
                <>
                  {record.kind === "assistant" ? (
                    <>
                      <div className={classes.assistantAuthor}>
                        {agentProfile ? (
                          <CharacterArtwork
                            basePath={agentProfile.basePath}
                            assetKey={agentProfile.assetKey}
                            className={classes.authorPortrait}
                          />
                        ) : null}
                        <strong>{agentName}</strong>
                      </div>
                      <ConversationContent parts={record.content} />
                    </>
                  ) : (
                    <UserMessageBody copy={copy}>
                      <ConversationContent parts={record.content} />
                    </UserMessageBody>
                  )}
                </>
              )
            ) : record.kind === "request" ? (
              surface === "history" ? (
                <ConversationRecordDisclosure
                  record={record}
                  copy={copy}
                  agentName={agentName}
                  initiallyOpened={initiallyExpandedKey === record.key}
                >
                  <ConversationQuestion
                    record={record}
                    controls={controls}
                    copy={copy}
                    writable={false}
                    idPrefix="history-question"
                  />
                </ConversationRecordDisclosure>
              ) : (
                <ConversationQuestion
                  record={record}
                  controls={controls}
                  copy={copy}
                  writable={writable && controls.read === "ready"}
                  idPrefix="question"
                />
              )
            ) : (
              <ConversationRecordDisclosure
                record={record}
                copy={copy}
                agentName={agentName}
                initiallyOpened={initiallyExpandedKey === record.key}
              >
                {record.kind === "tool" ? (
                  <ToolRecordDetails record={record} copy={copy}>
                    <ConversationContent
                      parts={record.content}
                      rawText={[
                        "read",
                        "execute",
                        "search",
                        "delete",
                        "move",
                      ].includes(record.toolKind ?? "")}
                    />
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
                        <ContentDisclosure title="Base64">
                          <pre>{record.terminal.base64}</pre>
                        </ContentDisclosure>
                      </>
                    ) : null}
                  </ToolRecordDetails>
                ) : (
                  <>
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
                        <ContentDisclosure title="Base64">
                          <pre>{record.terminal.base64}</pre>
                        </ContentDisclosure>
                      </>
                    ) : null}
                    <ConversationContent parts={record.content} />
                  </>
                )}
              </ConversationRecordDisclosure>
            )}
          </article>
        ))}
      {(controls.region.inputs ?? [])
        .filter((input) => !input.deleted)
        .map((input) => (
          <section
            className={`${classes.record} ${classes.pendingInput}`}
            data-kind="user"
            aria-label={copy.you}
            key={input.id}
          >
            <UserMessageBody copy={copy}>
              {input.text !== void 0 ? <Markdown text={input.text} /> : null}
            </UserMessageBody>
            {input.observation === "failed" &&
            controls.onRetryInput &&
            controls.onDeleteInput ? (
              <Group justify="flex-end" gap="xs">
                <Button
                  size="compact-xs"
                  variant="subtle"
                  disabled={
                    !writable ||
                    controls.inputAction?.requestId === input.requestId ||
                    !input.text
                  }
                  onClick={() => controls.onRetryInput?.(input)}
                >
                  {copy.retryMessage}
                </Button>
                <Button
                  size="compact-xs"
                  variant="subtle"
                  color="red"
                  disabled={
                    !controls.inputDeletable ||
                    controls.inputAction?.requestId === input.requestId
                  }
                  onClick={() => controls.onDeleteInput?.(input)}
                >
                  {copy.deleteMessage}
                </Button>
              </Group>
            ) : null}
            {controls.inputAction?.requestId === input.requestId ? (
              <Text size="xs" role="status">
                {controls.inputAction.state === "deleting"
                  ? copy.deletingMessage
                  : copy.deleteUnconfirmed}
              </Text>
            ) : null}
          </section>
        ))}
      {controls.inputs.map((input) => (
        <section
          key={input.requestId}
          className={`${classes.record} ${classes.pendingInput}`}
          data-kind="user"
          aria-label={copy.you}
        >
          <UserMessageBody copy={copy}>
            <Markdown text={input.text} />
          </UserMessageBody>
        </section>
      ))}
    </Stack>
  );
}
