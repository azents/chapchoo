import classes from "./ConversationRecord.module.css";
import type { ReactElement } from "react";
/** Retained before/after text, not an invented line-based patch or edit count. */
export function RecordDiff({
  path,
  oldText,
  newText,
}: {
  path: string;
  oldText?: string;
  newText: string;
}): ReactElement {
  return (
    <section className={classes.detailBox}>
      <div className={classes.detailBar}>{path}</div>
      <pre className={classes.diff}>
        {oldText !== void 0 ? (
          <del className={classes.removed}>
            {oldText
              .split("\n")
              .map((line) => `− ${line}`)
              .join("\n")}
          </del>
        ) : null}
        <ins className={classes.added}>
          {newText
            .split("\n")
            .map((line) => `+ ${line}`)
            .join("\n")}
        </ins>
      </pre>
    </section>
  );
}
