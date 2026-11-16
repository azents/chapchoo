"use client";
import { useLayoutEffect, useRef } from "react";
import type { FormEvent, KeyboardEvent, RefObject } from "react";

export interface MessageEditorInput {
  id: string;
  value: string;
  writable: boolean;
  pending: boolean;
  label: string;
  placeholder: string;
  error: boolean;
  editorRef: RefObject<HTMLDivElement | null>;
  onChange: (text: string) => void;
  onSend: () => void;
}
export interface MessageEditorViewProps extends MessageEditorInput {
  initialText: string;
  onInput: (event: FormEvent<HTMLDivElement>) => void;
  onKeyDown: (event: KeyboardEvent<HTMLDivElement>) => void;
  onCompositionStart: () => void;
  onCompositionEnd: (event: FormEvent<HTMLDivElement>) => void;
}
/** Keep native selection, composition and undo; synchronize only external draft changes. */
export function useMessageEditor(
  props: MessageEditorInput,
): MessageEditorViewProps {
  const { value, editorRef, onChange, onSend } = props;
  const editable = props.writable && !props.pending;
  const initialText = useRef(value);
  const composing = useRef(false);
  const terminalLine = useRef<Node | null>(null);
  const published = useRef(value);
  const deferredExternal = useRef<string | null>(null);
  function read(editor: HTMLDivElement): string {
    function text(node: Node): string {
      if (node === terminalLine.current) {
        return "";
      }
      if (node.nodeType === Node.TEXT_NODE) {
        return node.textContent ?? "";
      }
      const children = Array.from(node.childNodes);
      let result = "";
      let previousBlock = false;
      children.forEach((child, index) => {
        const block = child.nodeName === "DIV" || child.nodeName === "P";
        if (
          index > 0 &&
          (block || previousBlock) &&
          children[index - 1]?.nodeName !== "BR"
        ) {
          result += "\n";
        }
        if (child.nodeName === "BR") {
          // A final BR supplies a caret line; preceding BRs represent actual breaks.
          if (index < children.length - 1) {
            result += "\n";
          }
        } else {
          result += text(child);
        }
        previousBlock = block;
      });
      return result;
    }
    return text(editor);
  }
  useLayoutEffect(() => {
    const editor = editorRef.current;
    if (composing.current) {
      if (value !== published.current) {
        deferredExternal.current = value;
      }
      return;
    }
    if (editor && read(editor) !== value) {
      editor.textContent = value;
      terminalLine.current = null;
    }
    published.current = value;
  }, [value, editorRef]);
  function publish(editor: HTMLDivElement): void {
    const last = editor.lastChild;
    const selection = editor.ownerDocument.getSelection();
    if (
      last?.nodeName === "BR" ||
      (last?.nodeType === Node.TEXT_NODE &&
        last.textContent === "\n" &&
        selection?.anchorNode === last &&
        selection.anchorOffset === 0)
    ) {
      terminalLine.current = last;
    } else if (last !== terminalLine.current || last?.textContent !== "\n") {
      terminalLine.current = null;
    }
    const next = read(editor);
    published.current = next;
    onChange(next);
  }
  return {
    ...props,
    initialText: initialText.current,
    onInput: (event) => {
      if (editable && deferredExternal.current === null) {
        publish(event.currentTarget);
      }
    },
    onKeyDown: (event) => {
      if (
        event.key !== "Enter" ||
        event.nativeEvent.isComposing ||
        composing.current ||
        event.keyCode === 229
      ) {
        return;
      }
      if (!editable || event.shiftKey) {
        return;
      }
      event.preventDefault();
      onSend();
    },
    onCompositionStart: () => {
      composing.current = true;
    },
    onCompositionEnd: (event) => {
      composing.current = false;
      if (deferredExternal.current !== null) {
        event.currentTarget.textContent = deferredExternal.current;
        published.current = deferredExternal.current;
        terminalLine.current = null;
        deferredExternal.current = null;
      } else if (editable) {
        publish(event.currentTarget);
      }
    },
  };
}
