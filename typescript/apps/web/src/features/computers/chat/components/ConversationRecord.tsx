"use client";
import { createReactContainer } from "@chapchoo/ui/createReactContainer";
import { Accordion, useMantineTheme } from "@mantine/core";
import { useRecordDisclosure } from "../containers/useRecordDisclosure";
import { nativeStatus } from "../copy";
import {
  recordPreview,
  recordPublication,
  toolKind,
} from "../recordPresentation";
import classes from "./ConversationRecord.module.css";
import { RecordIcon } from "./RecordIcon";
import type {
  RecordDisclosure,
  RecordDisclosureInput,
} from "../containers/useRecordDisclosure";
import type { ChatCopy } from "../copy";
import type { ConversationRecord as Record } from "@chapchoo/web-core/conversation-types";
import type { ReactElement, ReactNode } from "react";
function Chevron(): ReactElement {
  return (
    <svg
      width="14"
      height="14"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.7"
      aria-hidden="true"
    >
      <path d="m9 6 6 6-6 6" />
    </svg>
  );
}
export function recordTitle(record: Record, copy: ChatCopy): string {
  if (record.kind === "user") {
    return copy.you;
  }
  if (record.kind === "assistant") {
    return copy.records.output;
  }
  if (record.kind === "tool") {
    return copy.records.kinds[toolKind(record.toolKind)];
  }
  return (
    record.title ??
    (record.kind === "thought"
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
}: {
  record: Record;
  copy: ChatCopy;
}): ReactElement {
  const publication = recordPublication(record) !== null;
  const message = record.kind === "user" || publication;
  const kind = publication
    ? "publication"
    : record.kind === "tool"
      ? toolKind(record.toolKind)
      : record.kind === "thought"
        ? "think"
        : record.kind === "terminal"
          ? "execute"
          : record.kind;
  const role =
    record.kind === "user"
      ? copy.records.human
      : publication
        ? copy.records.agent
        : record.kind === "assistant"
          ? copy.records.output
          : void 0;
  return (
    <>
      <span
        className={classes.kindIcon}
        role={role ? "img" : void 0}
        aria-label={role}
      >
        <RecordIcon kind={kind} />
      </span>
      <span className={classes.summary}>
        {!message ? (
          <span className={classes.heading}>
            <strong>{recordTitle(record, copy)}</strong>
          </span>
        ) : null}
        <span
          className={`${classes.preview} ${kind === "execute" ? classes.mono : ""}`}
        >
          {recordPreview(record)}
        </span>
      </span>
      {record.status ? (
        <span
          className={`${classes.status} ${record.status === "failed" ? classes.failed : ""}`}
        >
          <span
            className={
              record.status === "in_progress" ? classes.live : classes.statusDot
            }
          />
          {nativeStatus(copy, record.status)}
        </span>
      ) : null}
    </>
  );
}
export function RecentConversationRecord({
  record,
  copy,
  onOpen,
}: {
  record: Record;
  copy: ChatCopy;
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
      <RecordSummary record={record} copy={copy} />
    </button>
  );
}
function RecordDisclosureView({
  record,
  copy,
  children,
  disclosure,
}: RecordDisclosureInput & { disclosure: RecordDisclosure }): ReactElement {
  const theme = useMantineTheme();
  const publication = recordPublication(record) !== null;
  const message = record.kind === "user" || publication;
  return (
    <Accordion
      value={disclosure.opened ? record.key : null}
      onChange={(value) => disclosure.setOpened(value !== null)}
      variant="default"
      chevronPosition="left"
      chevron={<Chevron />}
      transitionDuration={160}
      className={`${classes.recordRow} ${message ? classes.messageRow : ""}`}
      data-record-origin={publication ? "publication" : record.kind}
      style={{
        "--record-message-primary": `var(--mantine-color-${theme.primaryColor}-filled)`,
      }}
      classNames={{
        control: classes.control,
        label: classes.label,
        chevron: classes.chevron,
        item: classes.item,
        panel: classes.disclosurePanel,
        content: classes.content,
      }}
    >
      <Accordion.Item value={record.key}>
        <Accordion.Control ref={disclosure.headerRef}>
          <RecordSummary record={record} copy={copy} />
        </Accordion.Control>
        <Accordion.Panel>
          <div className={classes.detailLayout}>
            <button
              type="button"
              className={classes.collapseRail}
              aria-label={copy.records.collapse}
              onClick={disclosure.close}
            />
            <div
              className={classes.detailBody}
              onClick={disclosure.closeFromBody}
            >
              {children}
            </div>
          </div>
        </Accordion.Panel>
      </Accordion.Item>
    </Accordion>
  );
}
export const ConversationRecordDisclosure = createReactContainer(
  "ConversationRecordDisclosure",
  useRecordDisclosure,
  RecordDisclosureView,
);
export function ContentDisclosure({
  title,
  children,
}: {
  title: string;
  children: ReactNode;
}): ReactElement {
  return (
    <Accordion variant="default" chevronPosition="left" chevron={<Chevron />}>
      <Accordion.Item value="content">
        <Accordion.Control>{title}</Accordion.Control>
        <Accordion.Panel>{children}</Accordion.Panel>
      </Accordion.Item>
    </Accordion>
  );
}
