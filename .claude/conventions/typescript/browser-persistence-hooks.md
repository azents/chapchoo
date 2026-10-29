---
title: "For browser persistence and browser-state hooks available in Mantine, use `@mantine/hooks` (`useLocalStorage`, `useSessionStorage`, `readLocalStorageValue`, `readSessionStorageValue`, `useHash`, `useDocumentVisibility`, `useDocumentTitle`, `useWindowEvent`, `useColorScheme`) instead of hand-written browser API wrappers."
---

# Use Mantine Browser Persistence Hooks

chapchoo-web already depends on `@mantine/hooks`. When Mantine provides a hook for
browser persistence or browser state, prefer that hook over hand-written browser
API wrappers so behavior stays consistent across components.

Mantine hooks currently available for this area include:

- `useLocalStorage`
- `readLocalStorageValue`
- `useSessionStorage`
- `readSessionStorageValue`
- `useHash`
- `useDocumentVisibility`
- `useDocumentTitle`
- `useWindowEvent`
- `useColorScheme`

Rules:

- ALWAYS use `useLocalStorage` for localStorage-backed React state.
- ALWAYS use `useSessionStorage` for sessionStorage-backed React state.
- Use `readLocalStorageValue` / `readSessionStorageValue` for one-off reads when
  React state is not needed.
- Prefer the listed Mantine browser-state hooks when they match the use case.
- AVOID direct `window.localStorage.*` and `window.sessionStorage.*` in feature
  code when the Mantine hook can express the same behavior.

## Bad

```tsx
const saved = window.localStorage.getItem("chapchoo.chat.inputDraft.agent-1");
window.localStorage.setItem("chapchoo.chat.inputDraft.agent-1", value);

sessionStorage.setItem("chapchoo_next_toolkit", toolkitId);
```

## Good

```tsx
import { useLocalStorage, useSessionStorage } from "@mantine/hooks";

const [draft, setDraft, clearDraft] = useLocalStorage({
  key: `chapchoo.chat.inputDraft.${agentId}`,
  defaultValue: "",
});

const [nextToolkit, setNextToolkit, clearNextToolkit] = useSessionStorage({
  key: "chapchoo_next_toolkit",
  defaultValue: "",
});
```
