import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import { checkDesignBoundaries } from "./design-boundaries.mjs";

function fixture(t) {
  const root = mkdtempSync(join(tmpdir(), "chapchoo-boundaries-"));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const locations = {
    reference: "packages/chapchoo-design-reference",
    ui: "packages/chapchoo-ui",
    app: "apps/web",
  };
  const names = {
    reference: "@chapchoo/design-reference",
    ui: "@chapchoo/ui",
    app: "@chapchoo/web",
  };
  function write(owner, file, content) {
    const path = join(root, locations[owner], file);
    mkdirSync(join(path, ".."), { recursive: true });
    writeFileSync(path, content);
  }
  for (const owner of Object.keys(locations)) {
    write(owner, "package.json", JSON.stringify({ name: names[owner] }));
    write(owner, "src/component.ts", "export const example = 1;");
  }
  return { root, write, names };
}

test("allows reference-to-UI and app-to-UI reuse", (t) => {
  const { root, write } = fixture(t);
  write(
    "reference",
    "src/example.ts",
    'export { example } from "@chapchoo/ui/component";',
  );
  write(
    "app",
    "src/example.ts",
    'import { example } from "@chapchoo/ui/component";',
  );
  assert.deepEqual(checkDesignBoundaries(root), []);
});

for (const statement of [
  'import { example } from "@chapchoo/design-reference/component";',
  'export { example } from "../../chapchoo-design-reference/src/component";',
  'const load = () => import("@chapchoo/design-reference/component");',
  'const value = require("@chapchoo/design-reference/component");',
  'type Value = import("@chapchoo/design-reference/component").Value;',
]) {
  test(`rejects reverse source edge: ${statement}`, (t) => {
    const { root, write } = fixture(t);
    write("ui", "src/example.ts", statement);
    assert.equal(checkDesignBoundaries(root).length, 1);
  });
}

test("rejects a TypeScript path alias into reference", (t) => {
  const { root, write } = fixture(t);
  write(
    "app",
    "tsconfig.json",
    JSON.stringify({
      compilerOptions: {
        paths: {
          "@reference/*": ["../../packages/chapchoo-design-reference/src/*"],
        },
      },
    }),
  );
  write(
    "app",
    "src/example.ts",
    'import { example } from "@reference/component";',
  );
  assert.equal(checkDesignBoundaries(root).length, 1);
});

for (const statement of [
  'export { example } from "@chapchoo/web/component";',
  'export { example } from "../../../apps/web/src/component";',
]) {
  test(`rejects reference-to-app edge: ${statement}`, (t) => {
    const { root, write } = fixture(t);
    write("reference", "src/example.ts", statement);
    assert.equal(checkDesignBoundaries(root).length, 1);
  });
}

for (const section of [
  "dependencies",
  "devDependencies",
  "peerDependencies",
  "optionalDependencies",
]) {
  test(`rejects reverse ${section} manifest edge`, (t) => {
    const { root, write, names } = fixture(t);
    write(
      "app",
      "package.json",
      JSON.stringify({
        name: names.app,
        [section]: { [names.reference]: "workspace:*" },
      }),
    );
    assert.equal(checkDesignBoundaries(root).length, 1);
  });
}

test("rejects local-link manifest aliases", (t) => {
  const { root, write, names } = fixture(t);
  write(
    "ui",
    "package.json",
    JSON.stringify({
      name: names.ui,
      dependencies: { alias: "link:../chapchoo-design-reference" },
    }),
  );
  assert.equal(checkDesignBoundaries(root).length, 1);
});

for (const range of ["*", "^", "~", "^0.1.0"]) {
  test(`rejects workspace manifest alias range ${range}`, (t) => {
    const { root, write, names } = fixture(t);
    write(
      "ui",
      "package.json",
      JSON.stringify({
        name: names.ui,
        devDependencies: { alias: `workspace:${names.reference}@${range}` },
      }),
    );
    assert.equal(checkDesignBoundaries(root).length, 1);
  });
}

for (const protocol of ["file:", "link:"]) {
  test(`rejects absolute ${protocol} manifest aliases`, (t) => {
    const { root, write, names } = fixture(t);
    write(
      "ui",
      "package.json",
      JSON.stringify({
        name: names.ui,
        dependencies: {
          alias: `${protocol}${join(root, "packages/chapchoo-design-reference")}`,
        },
      }),
    );
    assert.equal(checkDesignBoundaries(root).length, 1);
  });
}

test("rejects reference-to-app manifest dependencies", (t) => {
  const { root, write, names } = fixture(t);
  write(
    "reference",
    "package.json",
    JSON.stringify({
      name: names.reference,
      dependencies: { [names.app]: "workspace:*" },
    }),
  );
  assert.equal(checkDesignBoundaries(root).length, 1);
});
