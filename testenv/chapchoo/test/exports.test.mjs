import assert from "node:assert/strict";
import { test } from "node:test";

for (const subpath of ["assembly", "processes", "safety", "config", "seed"]) {
  test(`built ${subpath} export loads without a TypeScript or browser runner`, async () => {
    const exported = await import(`@chapchoo/testenv/${subpath}`);
    assert.ok(Object.keys(exported).length > 0);
  });
}
