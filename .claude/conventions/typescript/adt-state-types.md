---
title: "Model genuine multi-state UI with a discriminated-union ADT in the container; resolved SSR renderers require data, not artificial loading states."
---

# ADT State Modeling

Spreading `isLoading`, `error`, `data` across separate `useState` calls makes impossible states representable (loading=true AND data=present). A discriminated union encodes the rule that exactly one state is active at a time, and the component's `switch` is exhaustively checked.

- Define `type State = | { type: "LOADING" } | { type: "ERROR"; message: string } | { type: "LOADED"; ... }` in `types.ts`
- The container converts `isLoading / isError / data` into the ADT
- The component switches on `state.type`
- Minimize `useState` — derive from query state where possible
- Include only reachable states. [Resolved-data SSR renderers](ssr-snapshot-renderers.md) take required data directly; they do not need a loading/error ADT merely for the initial render.
- [Interactive/live SSR content](ssr-client-state-handoff.md) starts from its server-rendered initial data, not an empty client loading state.

## Bad

```tsx
function useContainer() {
  const { data, isLoading, isError, error } = trpc.foods.list.useQuery();
  return { data, isLoading, isError, error };
}

function Component({ data, isLoading, isError, error }) {
  if (isLoading) return <Loader />;
  if (isError) return <Text>{error?.message}</Text>;
  return <List foods={data} />;
}
```

## Good

```tsx
type State =
  | { type: "LOADING" }
  | { type: "ERROR"; message: string }
  | { type: "LOADED"; foods: Food[] };

function useContainer() {
  const query = trpc.foods.list.useQuery();
  const state: State = query.isLoading
    ? { type: "LOADING" }
    : query.isError
      ? { type: "ERROR", message: query.error.message }
      : { type: "LOADED", foods: query.data ?? [] };
  return { state };
}

function Component({ state }: { state: State }) {
  switch (state.type) {
    case "LOADING": return <Loader />;
    case "ERROR":   return <Text c="red">{state.message}</Text>;
    case "LOADED":  return <List foods={state.foods} />;
  }
}
```
