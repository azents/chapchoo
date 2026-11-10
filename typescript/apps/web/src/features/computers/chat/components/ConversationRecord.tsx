"use client";
import { Accordion } from "@mantine/core";
import { nativeStatus } from "../copy";
import classes from "./ConversationRecord.module.css";
import type { ChatCopy } from "../copy";
import type { ConversationRecord as Record } from "@chapchoo/web-core/conversation-types";
import type { ReactElement, ReactNode } from "react";

function Chevron(): ReactElement {
  return (
    <svg
      width="1rem"
      height="1rem"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.6"
      aria-hidden="true"
    >
      <path d="m9 6 6 6-6 6" />
    </svg>
  );
}
export function recordTitle(
  record: Record,
  copy: ChatCopy,
  agentName: string,
): string {
  if (record.kind === "user") {
    return copy.you;
  }
  if (record.kind === "assistant") {
    return agentName;
  }
  return (
    record.title ??
    (record.kind === "tool"
      ? copy.tools
      : record.kind === "thought"
        ? copy.thought
        : record.kind === "plan"
          ? copy.plan
          : record.kind === "terminal"
            ? copy.terminal
            : record.kind === "request"
              ? (record.question?.title ?? copy.question)
              : copy.notice)
  );
}
function RecordSummary({
  record,
  copy,
  agentName,
}: {
  record: Record;
  copy: ChatCopy;
  agentName: string;
}): ReactElement {
  const preview =
    record.content
      .map((part) =>
        part.type === "text"
          ? part.text
          : part.type === "resource"
            ? part.title
            : "",
      )
      .join(" ") ||
    record.description ||
    (record.question && record.question.type !== "unsupported"
      ? record.question.description
      : "") ||
    (record.plan?.type === "text"
      ? record.plan.text
      : record.plan?.type === "items"
        ? record.plan.items.map((item) => item.content).join(" · ")
        : "");
  return (
    <span className={classes.summary}>
      <span className={classes.heading}>
        <strong>{recordTitle(record, copy, agentName)}</strong>
        {record.status ? (
          <span className={classes.status}>
            {nativeStatus(copy, record.status)}
          </span>
        ) : null}
      </span>
      {preview ? (
        <span className={classes.preview}>
          {Array.from(preview.replace(/\s+/gu, " ").trim())
            .slice(0, 120)
            .join("")}
        </span>
      ) : null}
    </span>
  );
}
export function RecentConversationRecord({
  record,
  copy,
  agentName,
  onOpen,
}: {
  record: Record;
  copy: ChatCopy;
  agentName: string;
  onOpen: () => void;
}): ReactElement {
  return (
    <button
      type="button"
      className={`${classes.control} ${classes.recent}`}
      onClick={onOpen}
    >
      <span className={classes.chevron}>
        <Chevron />
      </span>
      <RecordSummary record={record} copy={copy} agentName={agentName} />
    </button>
  );
}
export function ConversationRecordDisclosure({
  record,
  copy,
  agentName,
  children,
  initiallyOpened = false,
}: {
  record: Record;
  copy: ChatCopy;
  agentName: string;
  children: ReactNode;
  initiallyOpened?: boolean;
}): ReactElement {
  return (
    <Accordion
      variant="default"
      chevronPosition="left"
      chevron={<Chevron />}
      transitionDuration={160}
      defaultValue={initiallyOpened ? record.key : null}
      classNames={{
        control: classes.control,
        label: classes.label,
        chevron: classes.chevron,
        item: classes.item,
        content: classes.content,
      }}
    >
      <Accordion.Item value={record.key}>
        <Accordion.Control>
          <RecordSummary record={record} copy={copy} agentName={agentName} />
        </Accordion.Control>
        <Accordion.Panel>{children}</Accordion.Panel>
      </Accordion.Item>
    </Accordion>
  );
}
export function ContentDisclosure({
  title,
  children,
}: {
  title: string;
  children: ReactNode;
}): ReactElement {
  return (
    <Accordion
      variant="default"
      chevronPosition="left"
      chevron={<Chevron />}
      transitionDuration={160}
      classNames={{
        control: classes.control,
        label: classes.label,
        chevron: classes.chevron,
        item: classes.item,
        content: classes.content,
      }}
    >
      <Accordion.Item value="content">
        <Accordion.Control>{title}</Accordion.Control>
        <Accordion.Panel>{children}</Accordion.Panel>
      </Accordion.Item>
    </Accordion>
  );
}
