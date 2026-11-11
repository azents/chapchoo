"use client";
import { Button, CopyButton } from "@mantine/core";
import { recordInput, toolKind } from "../recordPresentation";
import classes from "./ConversationRecord.module.css";
import type { ChatCopy } from "../copy";
import type { ConversationRecord } from "@chapchoo/web-core/conversation-types";
import type { ReactElement, ReactNode } from "react";

/** Uses retained structured values only; raw payloads remain available without inference. */
export function ToolRecordDetails({
  record,
  copy,
  children,
}: {
  record: ConversationRecord;
  copy: ChatCopy;
  children: ReactNode;
}): ReactElement {
  const kind = toolKind(record.toolKind);
  const input = recordInput(record);
  const command = kind === "execute" ? (input.command ?? input.cmd) : void 0;
  const path = record.locations[0]?.path ?? input.path ?? input.file_path;
  const subject =
    kind === "search"
      ? (input.query ?? input.pattern)
      : kind === "fetch"
        ? input.url
        : kind === "move" && input.source && input.destination
          ? `${input.source} → ${input.destination}`
          : kind === "switch_mode"
            ? input.mode
            : null;
  return (
    <div
      className={`${classes.detailBox} ${command ? classes.commandDetail : classes.genericDetail}`}
    >
      {command ? (
        <>
          <div className={classes.detailBar}>
            <span title={record.title}>{copy.records.command}</span>
            <CopyButton value={command}>
              {({ copied, copy: copyCommand }) => (
                <Button
                  variant="subtle"
                  size="compact-xs"
                  onClick={copyCommand}
                >
                  {copied ? copy.records.copied : copy.records.copy}
                </Button>
              )}
            </CopyButton>
          </div>
          <pre className={classes.command}>$ {command}</pre>
          {input.cwd ? (
            <div className={classes.detailBar}>{input.cwd}</div>
          ) : null}
        </>
      ) : null}
      {subject ? <pre>{subject}</pre> : null}
      {path ? (
        <div className={classes.detailBar}>
          <span>{path}</span>
          {record.locations[0]?.line !== void 0 ? (
            <span>{record.locations[0].line}</span>
          ) : null}
        </div>
      ) : null}
      {record.locations.slice(1).map((location) => (
        <pre key={`${location.path}:${location.line ?? ""}`}>
          {location.path}
          {location.line !== void 0 ? `:${location.line}` : ""}
        </pre>
      ))}
      {!command && record.title ? <p>{record.title}</p> : null}
      {record.description ? <p>{record.description}</p> : null}
      {record.toolInput ? (
        <section aria-label={copy.toolInput}>
          <details>
            <summary>{copy.toolInput}</summary>
            <pre>{record.toolInput.json}</pre>
          </details>
        </section>
      ) : null}
      {record.toolOutput ? (
        <section aria-label={copy.toolOutput}>
          <div className={classes.outputLabel}>{copy.toolOutput}</div>
          <pre>{record.toolOutput.json}</pre>
        </section>
      ) : null}
      {children}
    </div>
  );
}
