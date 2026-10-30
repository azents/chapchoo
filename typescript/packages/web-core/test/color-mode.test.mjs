import assert from "node:assert/strict";
import test from "node:test";
import vm from "node:vm";
import {
  colorModeBootstrap,
  bodyColorModeBootstrap,
  resolveServerColorMode,
  serializeAppearanceCookie,
  preferenceCookie,
  resolvedCookie,
  createColorSchemeManager,
} from "../dist/color-mode.js";

function browser(dark = false) {
  const attrs = new Map();
  const bodyAttrs = new Map();
  const document = {
    cookie: "",
    documentElement: {
      setAttribute: (key, value) => attrs.set(key, value),
      getAttribute: (key) => attrs.get(key),
      style: {},
    },
    body: { setAttribute: (key, value) => bodyAttrs.set(key, value) },
  };
  return {
    document,
    window: { matchMedia: () => ({ matches: dark }) },
    location: { protocol: "https:" },
    attrs,
    bodyAttrs,
  };
}

test("validated server resolution gives explicit preference precedence over stale cache", () => {
  assert.deepEqual(resolveServerColorMode("dark", "light"), {
    preference: "dark",
    mode: "dark",
  });
  assert.deepEqual(resolveServerColorMode("light", "dark"), {
    preference: "light",
    mode: "light",
  });
  assert.deepEqual(resolveServerColorMode(undefined, "dark"), {
    preference: "system",
    mode: "dark",
  });
  assert.deepEqual(resolveServerColorMode("</script>", "bad"), {
    preference: "system",
    mode: "light",
  });
});

test("appearance cookies are host-only root-scoped, enum-only presentation state", () => {
  assert.equal(
    serializeAppearanceCookie(preferenceCookie, "dark", true),
    "chapchoo-color-mode=dark; Path=/; Max-Age=31536000; SameSite=Lax; Secure",
  );
  assert.ok(
    !serializeAppearanceCookie(resolvedCookie, "light", false).includes(
      "Secure",
    ),
  );
});

test("pre-paint bootstrap uses current media, ignores stale hints and safely handles untrusted input", () => {
  for (const dark of [false, true]) {
    for (const preference of [
      "system",
      "dark",
      "light",
      "</script><script>bad()",
    ]) {
      const env = browser(dark);
      const script = colorModeBootstrap(
        resolveServerColorMode(preference, dark ? "light" : "dark"),
      );
      assert.ok(!script.includes("</script>"));
      vm.runInNewContext(script, env);
      vm.runInNewContext(bodyColorModeBootstrap, env);
      const expected =
        preference === "light" || preference === "dark"
          ? preference
          : dark
            ? "dark"
            : "light";
      assert.equal(env.attrs.get("data-mantine-color-scheme"), expected);
      assert.equal(env.bodyAttrs.get("data-color-mode"), expected);
      assert.equal(env.document.documentElement.style.colorScheme, expected);
      assert.match(
        env.document.cookie,
        /Path=\/; Max-Age=31536000; SameSite=Lax; Secure/,
      );
    }
  }
});

test("cookie manager owns preference without consulting localStorage", () => {
  const env = browser(true);
  const previous = {
    document: globalThis.document,
    window: globalThis.window,
    location: globalThis.location,
  };
  Object.assign(globalThis, env);
  try {
    const manager = createColorSchemeManager("system");
    assert.equal(manager.get("light"), "auto");
    manager.set("light");
    assert.equal(manager.get("dark"), "light");
    assert.equal(env.attrs.get("data-mantine-color-scheme"), "light");
    assert.match(env.document.cookie, /chapchoo-color-mode=light/);
    manager.clear();
    assert.equal(manager.get("light"), "auto");
    assert.equal(env.bodyAttrs.get("data-color-mode"), "dark");
    Object.defineProperty(env.document, "cookie", {
      set() {
        throw new Error("denied");
      },
    });
    assert.doesNotThrow(() => manager.set("dark"));
  } finally {
    for (const [key, value] of Object.entries(previous)) {
      if (value === undefined) delete globalThis[key];
      else globalThis[key] = value;
    }
  }
});
