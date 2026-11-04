import assert from "node:assert/strict";
import { describe, it } from "node:test";
import {
  editHandle,
  proposeHandle,
  randomHandleSuffix,
  renameHandleDraft,
  resolveHandle,
  validHandle,
} from "../dist/handle.js";

const suffix = () => "k7m2q";
const signal = () => new AbortController().signal;

// The same cases are exercised by the Rust parser. These assert our accepted
// contract, rather than the transliteration library's complete character map.
const accepted = [
  "abc",
  "012",
  "a--b",
  "dubu",
  "design-team",
  "a12345678901234z",
];
const rejected = [
  "",
  "a",
  "my",
  " a-b",
  "a-b ",
  "Dubu",
  "-abc",
  "abc-",
  "a_b",
  "a.b",
  "a/b",
  "a\nb",
  "abc\n",
  "abc\r",
  "a\0b",
  "두부",
  "ｄｕｂｕ",
  "a123456789012345z",
];

describe("common canonical handle contract", () => {
  it("accepts the agreed alphabet/bounds without adding a consecutive-hyphen rule", () => {
    for (const value of accepted) assert.equal(validHandle(value), true, value);
    for (const value of rejected)
      assert.equal(validHandle(value), false, value);
  });

  it("generates from a multilingual display name without mutating it", () => {
    const name = "개발 도우미";
    const draft = proposeHandle("agent", name, suffix);
    assert.equal(draft.candidate, "gaebal-doumi");
    assert.equal(draft.source, "automatic");
    assert.equal(name, "개발 도우미");
    assert.equal(validHandle(draft.candidate), true);
  });

  it("supplements short and empty conversions with a kind-specific base", () => {
    assert.equal(proposeHandle("agent", "AI", suffix).candidate, "ai-k7m2q");
    assert.equal(proposeHandle("agent", "X", suffix).candidate, "x-k7m2q");
    assert.equal(
      proposeHandle("agent", "!!!", suffix).candidate,
      "agent-k7m2q",
    );
    assert.equal(proposeHandle("team", "!!!", suffix).candidate, "team-k7m2q");
    assert.equal(proposeHandle("team", "", suffix).candidate, "team-k7m2q");
  });

  it("truncates a long automatic base and removes a trailing hyphen", () => {
    assert.equal(
      proposeHandle("team", "abcdefghijklmnop-more", suffix).candidate,
      "abcdefghijklmnop",
    );
    assert.equal(
      proposeHandle("agent", "abcdefghijklmno-more", suffix).candidate,
      "abcdefghijklmno",
    );
  });

  it("retains exact manual edits through later name changes", () => {
    for (const candidate of ["chosen-team", "UPPERCASE", "", " user "]) {
      const draft = editHandle(candidate);
      assert.equal(renameHandleDraft(draft, "a different name", suffix), draft);
      assert.equal(draft.candidate, candidate);
    }
  });

  it("updates an automatic candidate when the name changes", () => {
    const draft = proposeHandle("agent", "두부", suffix);
    assert.equal(renameHandleDraft(draft, "AI", suffix).candidate, "ai-k7m2q");
  });

  it("does not accept a malformed injected random suffix", () => {
    assert.throws(() => proposeHandle("agent", "!", () => "bad!"));
    assert.throws(() => proposeHandle("agent", "!", () => "abcde\n"));
  });

  it("uses a five-character canonical random suffix", () => {
    assert.match(randomHandleSuffix(), /^[a-z0-9]{5}$/);
  });
});

describe("availability resolution", () => {
  it("checks an automatic candidate and a suffixed collision replacement", async () => {
    const draft = proposeHandle("team", "abcdefghijklmnop-more", suffix);
    const checked = [];
    const result = await resolveHandle(draft, {
      signal: signal(),
      suffix,
      check: async (value) => {
        checked.push(value);
        return checked.length === 2;
      },
    });
    assert.equal(result.status, "available");
    assert.deepEqual(checked, ["abcdefghijklmnop", "abcdefghij-k7m2q"]);
    assert.equal(result.draft.candidate.length, 16);
    assert.equal(validHandle(result.draft.candidate), true);
    assert.equal(draft.candidate, "abcdefghijklmnop");
  });

  it("strips a boundary hyphen before appending the collision suffix", async () => {
    let calls = 0;
    const result = await resolveHandle(
      proposeHandle("team", "abcdefghi-jklmnop", suffix),
      {
        signal: signal(),
        suffix,
        check: async () => ++calls === 2,
      },
    );
    assert.equal(result.status, "available");
    assert.equal(result.draft.candidate, "abcdefghi-k7m2q");
  });

  it("regenerates from the original base rather than accumulating suffixes", async () => {
    let calls = 0;
    let randomCalls = 0;
    const result = await resolveHandle(proposeHandle("agent", "dubu", suffix), {
      signal: signal(),
      suffix: () => (++randomCalls === 1 ? "aaaaa" : "bbbbb"),
      check: async () => ++calls === 3,
    });
    assert.equal(result.status, "available");
    assert.equal(result.draft.candidate, "dubu-bbbbb");
  });

  it("reports manual occupancy without rewriting it", async () => {
    const draft = editHandle("product");
    const result = await resolveHandle(draft, {
      signal: signal(),
      check: async () => false,
      suffix: () => {
        throw new Error("Must not generate for manual input");
      },
    });
    assert.equal(result.status, "occupied");
    assert.equal(result.draft, draft);
  });

  it("does not query malformed manual candidates", async () => {
    const result = await resolveHandle(editHandle("AA"), {
      signal: signal(),
      check: async () => {
        throw new Error("Must not query");
      },
    });
    assert.equal(result.status, "invalid");
  });

  it("bounds repeated collisions and retains the last checked candidate", async () => {
    const checked = [];
    const result = await resolveHandle(
      proposeHandle("team", "product", suffix),
      {
        signal: signal(),
        suffix,
        check: async (value) => {
          checked.push(value);
          return false;
        },
      },
    );
    assert.equal(result.status, "exhausted");
    assert.equal(checked.length, 8);
    assert.equal(result.draft.candidate, checked.at(-1));
  });

  it("keeps transport failure distinct from occupancy", async () => {
    const draft = proposeHandle("agent", "dubu", suffix);
    const result = await resolveHandle(draft, {
      signal: signal(),
      check: async () => {
        throw new Error("Transport down");
      },
    });
    assert.equal(result.status, "unavailable");
    assert.equal(result.draft, draft);
  });

  it("does not query an already cancelled draft", async () => {
    const controller = new AbortController();
    controller.abort();
    const result = await resolveHandle(editHandle("product"), {
      signal: controller.signal,
      check: async () => {
        throw new Error("Must not query");
      },
    });
    assert.equal(result.status, "cancelled");
  });

  it("rejects a stale success after the draft/context was cancelled", async () => {
    const controller = new AbortController();
    const { promise, resolve } = Promise.withResolvers();
    const draft = proposeHandle("agent", "dubu", suffix);
    const pending = resolveHandle(draft, {
      signal: controller.signal,
      check: () => promise,
    });
    controller.abort();
    resolve(true);
    const result = await pending;
    assert.equal(result.status, "cancelled");
    assert.equal(result.draft, draft);
  });

  it("rejects a stale collision without issuing another check", async () => {
    const controller = new AbortController();
    let calls = 0;
    const result = await resolveHandle(proposeHandle("agent", "dubu", suffix), {
      signal: controller.signal,
      suffix,
      check: async () => {
        calls += 1;
        controller.abort();
        return false;
      },
    });
    assert.equal(result.status, "cancelled");
    assert.equal(calls, 1);
  });

  it("does not mutate an input draft while an availability read is in flight", async () => {
    const { promise, resolve } = Promise.withResolvers();
    const draft = proposeHandle("agent", "dubu", suffix);
    const pending = resolveHandle(draft, {
      signal: signal(),
      check: () => promise,
    });
    assert.equal(draft.candidate, "dubu");
    resolve(true);
    assert.deepEqual(await pending, { status: "available", draft });
  });
});
