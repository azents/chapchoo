"use client";

import { floatingSurfaceClassName } from "@chapchoo/ui/FloatingSurface";
import { VisuallyHidden } from "@mantine/core";
import { useId, useState } from "react";
import { useWorkSummarySize } from "../containers/useWorkSummarySize";
import classes from "./WorkSummary.module.css";
import type { ChatCopy } from "../copy";
import type { ConversationWorkSummary } from "@chapchoo/web-core/conversation-types";
import type { ReactElement } from "react";

type WorkStatus =
  ConversationWorkSummary["tasks"][number]["status"] | "waiting";

function WorkIcon({ status }: { status: WorkStatus }): ReactElement {
  return (
    <svg
      className={classes.icon}
      data-status={status}
      width="18"
      height="18"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.6"
      aria-hidden="true"
    >
      <circle
        cx="12"
        cy="12"
        r="8"
        opacity={status === "pending" ? ".45" : "1"}
      />
      {status === "completed" ? (
        <path d="m8 12 3 3 5-6" />
      ) : status === "waiting" ? (
        <path d="M12 8v4l3 2" />
      ) : status === "inProgress" ? (
        <circle cx="12" cy="12" r="3" fill="currentColor" stroke="none" />
      ) : null}
    </svg>
  );
}

type SummaryCopy = ChatCopy["workSummary"];

export function WorkSummary({
  work,
  copy,
}: {
  work?: ConversationWorkSummary;
  copy: SummaryCopy;
}): ReactElement | null {
  const [frame, setFrame] = useState(() => ({
    observed: work,
    shown: work,
    animateOnMount: false,
  }));
  if (work !== frame.observed) {
    setFrame({
      observed: work,
      shown: work ?? frame.shown,
      animateOnMount:
        work && work.id !== frame.shown?.id ? true : frame.animateOnMount,
    });
  }
  return frame.shown ? (
    <WorkSurface
      key={frame.shown.id}
      work={frame.shown}
      copy={copy}
      exiting={!work}
      animateOnMount={frame.animateOnMount}
      onExited={() =>
        setFrame((current) =>
          current.observed || current.shown?.id !== frame.shown?.id
            ? current
            : { observed: void 0, shown: void 0, animateOnMount: true },
        )
      }
    />
  ) : null;
}

function WorkSurface({
  work,
  copy,
  exiting,
  animateOnMount,
  onExited,
}: {
  work: ConversationWorkSummary;
  copy: SummaryCopy;
  exiting: boolean;
  animateOnMount: boolean;
  onExited: () => void;
}): ReactElement {
  const detailsId = useId();
  const [expanded, setExpanded] = useState(false);
  const [pressed, setPressed] = useState(false);
  const completed = work.tasks.filter(
    (task) => task.status === "completed",
  ).length;
  const { surface, content } = useWorkSummarySize({
    expanded,
    exiting,
    animateOnMount,
    onExited,
  });
  const release = (): void => setPressed(false);
  return (
    <section
      ref={surface}
      className={`${floatingSurfaceClassName} ${classes.surface}`}
      aria-label={copy.label}
      aria-hidden={exiting || void 0}
      inert={exiting}
      data-expanded={expanded || void 0}
      data-exiting={exiting || void 0}
      data-pressed={(pressed && !exiting) || void 0}
    >
      <div ref={content} className={classes.content}>
        <button
          className={classes.trigger}
          type="button"
          aria-expanded={expanded}
          aria-controls={detailsId}
          onClick={() => setExpanded((open) => !open)}
          onPointerDown={(event) => {
            if (event.isPrimary && event.button === 0) {
              setPressed(true);
            }
          }}
          onPointerUp={release}
          onPointerCancel={release}
          onPointerLeave={release}
          onLostPointerCapture={release}
          onBlur={release}
        >
          <WorkIcon status={work.waitingForInput ? "waiting" : "inProgress"} />
          <span className={classes.title}>{work.title}</span>
          {work.waitingForInput ? (
            <VisuallyHidden>{copy.waiting}</VisuallyHidden>
          ) : null}
          <span className={classes.count}>
            {completed}/{work.tasks.length}
          </span>
          <svg
            className={classes.chevron}
            width="16"
            height="16"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="1.7"
            aria-hidden="true"
          >
            <path d={expanded ? "m6 9 6 6 6-6" : "m6 15 6-6 6 6"} />
          </svg>
        </button>
        <div id={detailsId} className={classes.details} hidden={!expanded}>
          {work.tasks.length ? (
            <ol className={classes.tasks}>
              {work.tasks.map((task, index) => (
                <li
                  key={index}
                  className={classes.task}
                  data-status={task.status}
                >
                  <WorkIcon status={task.status} />
                  <span>
                    <VisuallyHidden>{copy[task.status]}: </VisuallyHidden>
                    {task.text}
                  </span>
                </li>
              ))}
            </ol>
          ) : null}
        </div>
      </div>
    </section>
  );
}
