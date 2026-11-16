import { ChatPresenceItem } from "../ChatPresence";
import classes from "./ChatPresenceView.module.css";
import type {
  ChatPresenceItemView as ItemProps,
  ChatPresenceView as ListProps,
} from "../containers/useChatPresence";
import type { ReactElement } from "react";

export function ChatPresenceView({
  entries,
  spaced,
  historyHead,
  onExited,
}: ListProps): ReactElement {
  const firstPresent = entries.findIndex((entry) => entry.present);
  return (
    <div className={classes.list}>
      {entries.map((entry, index) => (
        <ChatPresenceItem
          key={entry.key}
          entry={entry}
          gap={spaced && (entry.present ? index !== firstPresent : index > 0)}
          historyHead={historyHead}
          onExited={onExited}
        />
      ))}
    </div>
  );
}

export function ChatPresenceItemView({
  outerRef,
  innerRef,
  style,
  present,
  gap,
  children,
}: ItemProps): ReactElement {
  return (
    <div
      ref={outerRef}
      style={style}
      className={classes.item}
      data-chat-presence={present ? "present" : "exiting"}
      inert={!present}
      aria-hidden={!present || void 0}
    >
      <div ref={innerRef} className={classes.inner} data-gap={gap || void 0}>
        {children}
      </div>
    </div>
  );
}
