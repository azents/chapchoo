import assert from "node:assert/strict";
import test from "node:test";
import { MantineProvider } from "@mantine/core";
import { renderToStaticMarkup } from "react-dom/server";
import { Feedback } from "./Feedback";
import type { ReactNode } from "react";

function render(children: ReactNode): string {
  return renderToStaticMarkup(<MantineProvider>{children}</MantineProvider>);
}

await test("loading feedback announces its app-owned text without a loaded surface", () => {
  const html = render(
    <Feedback state={{ type: "LOADING", message: "Loading fixture" }} />,
  );
  assert.ok(html.includes('role="status"'));
  assert.ok(html.includes("Loading fixture"));
});

await test("error feedback exposes the supplied retry action and alert", () => {
  const html = render(
    <Feedback
      state={{
        type: "ERROR",
        title: "Fixture unavailable",
        message: "Try loading the fixture again.",
        retryLabel: "Retry fixture",
        onRetry: () => {},
      }}
    />,
  );
  assert.ok(html.includes('role="alert"'));
  assert.ok(html.includes("Fixture unavailable"));
  assert.ok(html.includes("Retry fixture"));
});

await test("empty and loaded feedback use their distinct presentation states", () => {
  const empty = render(
    <Feedback
      state={{
        type: "EMPTY",
        title: "No fixtures",
        action: <button type="button">Create fixture</button>,
      }}
    />,
  );
  const loaded = render(
    <Feedback state={{ type: "LOADED", children: <p>Fixture contents</p> }} />,
  );
  assert.ok(empty.includes("No fixtures"));
  assert.ok(empty.includes("Create fixture"));
  assert.ok(loaded.includes("Fixture contents"));
  assert.ok(!loaded.includes("No fixtures"));
});
