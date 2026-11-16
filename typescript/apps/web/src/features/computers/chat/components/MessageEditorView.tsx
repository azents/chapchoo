import classes from "./Chat.module.css";
import type { MessageEditorViewProps } from "../containers/useMessageEditor";
import type { ReactElement } from "react";

export function MessageEditorView(props: MessageEditorViewProps): ReactElement {
  return (
    <>
      <span id={`${props.id}-label`} className={classes.srOnly}>
        {props.label}
      </span>
      <div
        id={props.id}
        className={classes.messageEditor}
        role="textbox"
        aria-multiline="true"
        inputMode="text"
        autoCorrect="on"
        autoCapitalize="sentences"
        spellCheck
        ref={props.editorRef}
        contentEditable={
          props.writable && !props.pending ? "plaintext-only" : false
        }
        suppressContentEditableWarning
        tabIndex={props.writable ? 0 : -1}
        aria-disabled={!props.writable}
        aria-readonly={props.pending}
        aria-labelledby={`${props.id}-label`}
        aria-invalid={props.error}
        aria-describedby={props.error ? "chat-input-error" : void 0}
        data-placeholder={props.placeholder}
        aria-placeholder={props.placeholder}
        data-empty={!props.value || void 0}
        onInput={props.onInput}
        onKeyDown={props.onKeyDown}
        onCompositionStart={props.onCompositionStart}
        onCompositionEnd={props.onCompositionEnd}
      >
        {props.initialText}
      </div>
    </>
  );
}
