import assert from "node:assert/strict";
import test from "node:test";
import { renderToStaticMarkup } from "react-dom/server";
import { createReactContainer } from "./createReactContainer";
import type { ReactNode } from "react";

function Output({ label }: { label: string }): ReactNode {
  return <span>{label}</span>;
}

function Override({ label }: { label: string }): ReactNode {
  return <strong>{label}</strong>;
}

function useFixture({ value }: { value: string }): { label: string } {
  return { label: `fixture:${value}` };
}

await test("container connects an explicitly typed hook to its default view", () => {
  const Container = createReactContainer("Fixture", useFixture, Output);
  assert.equal(
    renderToStaticMarkup(<Container value="ready" />),
    "<span>fixture:ready</span>",
  );
});

await test("container accepts a caller-owned component override", () => {
  const Container = createReactContainer("Fixture", useFixture, Output);
  assert.equal(
    renderToStaticMarkup(<Container value="ready" component={Override} />),
    "<strong>fixture:ready</strong>",
  );
});

await test("container without a default requires an explicit presentation component", () => {
  const Container = createReactContainer("Fixture", useFixture);
  assert.equal(
    renderToStaticMarkup(<Container value="ready" component={Output} />),
    "<span>fixture:ready</span>",
  );
});
