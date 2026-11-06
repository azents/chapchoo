import assert from "node:assert/strict";
import test from "node:test";
import { catalogueOwnerMatches } from "./types.ts";
import type { CatalogueSnapshot } from "./types.ts";

void test("catalogue snapshots withdraw synchronously on identity loss or replacement", () => {
  for (const scope of [
    { type: "personal" },
    { type: "team", teamId: "shared-team" },
  ] as const) {
    const snapshot: CatalogueSnapshot = {
      ownerUserId: "original-user",
      scope,
      canManage: true,
      page: null,
      renderedAt: 1,
    };
    assert.equal(catalogueOwnerMatches(snapshot, "original-user"), true);
    assert.equal(catalogueOwnerMatches(snapshot, null), false);
    assert.equal(catalogueOwnerMatches(snapshot, "replacement-user"), false);
  }
});
