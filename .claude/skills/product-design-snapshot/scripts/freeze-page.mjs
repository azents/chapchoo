import { createRequire } from "node:module";
import { readFile, writeFile, mkdir } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { createServer } from "node:http";
import { createHash } from "node:crypto";

// Use the repository's existing browser-test dependency, not a global install.
const require = createRequire(resolve("typescript/apps/web/package.json"));
const { chromium } = require("@playwright/test");

/** Capture one resolved viewport. Authoring setup belongs to the caller. */
export async function freezePage(page, output, surface = page) {
  await surface.evaluate(async () => {
    await document.fonts.ready;
    await Promise.all(
      [...document.images]
        .filter((image) => image.currentSrc)
        .map((image) => image.decode()),
    );
  });
  const result = await surface.evaluate(() => {
    if (document.compatMode !== "CSS1Compat")
      throw new Error(
        "Use a standards-mode authoring document before exporting.",
      );
    if (
      document.querySelector("iframe,frame,canvas,video,audio,object,embed") ||
      [...document.querySelectorAll("*")].some((element) => element.shadowRoot)
    ) {
      throw new Error(
        "Unsupported embedded, media, canvas or shadow surface; export it deliberately first.",
      );
    }
    const clone = document.documentElement.cloneNode(true);
    const originals = [
      document.documentElement,
      ...document.documentElement.querySelectorAll("*"),
    ];
    const copies = [clone, ...clone.querySelectorAll("*")];
    const computed = (style) =>
      [...style]
        .filter(
          (key) =>
            !key.startsWith("animation") && !key.startsWith("transition"),
        )
        .map((key) => `${key}:${style.getPropertyValue(key)};`)
        .join("");
    originals.forEach((element, index) => {
      const copy = copies[index];
      if (element.matches("dialog:modal")) {
        const backdrop = document.createElement("div");
        backdrop.setAttribute(
          "style",
          computed(getComputedStyle(element, "::backdrop")) +
            ";position:fixed;inset:0;z-index:2147483646;",
        );
        clone.querySelector("body").append(backdrop);
        copy.setAttribute(
          "style",
          computed(getComputedStyle(element)) +
            ";position:fixed;z-index:2147483647;",
        );
      }
      for (const attribute of [...copy.attributes]) {
        if (
          /^on/i.test(attribute.name) ||
          ["srcset", "integrity", "nonce", "autofocus", "ping"].includes(
            attribute.name,
          )
        )
          copy.removeAttribute(attribute.name);
      }
      if (element instanceof HTMLImageElement)
        copy.setAttribute("src", element.currentSrc);
      if (element instanceof HTMLInputElement) {
        if (
          element.type === "file" ||
          (["password", "hidden"].includes(element.type) && element.value)
        )
          throw new Error("Sensitive input surface is not exportable.");
        copy.setAttribute("value", element.value);
        copy.toggleAttribute("checked", element.checked);
      }
      if (element instanceof HTMLTextAreaElement)
        copy.textContent = element.value;
      if (element instanceof HTMLOptionElement)
        copy.toggleAttribute("selected", element.selected);
      if (element instanceof HTMLDetailsElement)
        copy.toggleAttribute("open", element.open);
      if (
        copy.hasAttribute("href") &&
        !copy.getAttribute("href").startsWith("#")
      ) {
        if (["image", "use"].includes(copy.tagName))
          throw new Error(
            "External SVG references require deliberate localization.",
          );
        copy.removeAttribute("href");
      }
      if (copy.hasAttribute("action")) copy.removeAttribute("action");
      if (copy.hasAttribute("formaction")) copy.removeAttribute("formaction");
      if (copy.tagName === "BUTTON") copy.setAttribute("type", "button");
    });
    clone
      .querySelectorAll("script,style,link,base,meta[http-equiv]")
      .forEach((element) => element.remove());
    const inlineStyles = [clone, ...clone.querySelectorAll("[style]")]
      .filter((element) => element.hasAttribute("style"))
      .map((element, index) => {
        const value = element.getAttribute("style");
        const marker = `__snapshot_inline_${index}__`;
        element.setAttribute("style", marker);
        return { marker, value };
      });
    return {
      conditions: {
        locale: navigator.language,
        colorScheme: matchMedia("(prefers-color-scheme: dark)").matches
          ? "dark"
          : "light",
        reducedMotion: matchMedia("(prefers-reduced-motion: reduce)").matches
          ? "reduce"
          : "no-preference",
      },
      html: clone.outerHTML,
      inlineStyles,
      base: document.baseURI,
      sheets: [...document.styleSheets, ...document.adoptedStyleSheets]
        .filter((sheet) => !sheet.disabled)
        .map((sheet) => {
          try {
            const rules = [...sheet.cssRules]
              .map((rule) => rule.cssText)
              .join("\n");
            return {
              base: sheet.href || document.baseURI,
              text: sheet.media.mediaText
                ? `@media ${sheet.media.mediaText}{${rules}}`
                : rules,
            };
          } catch {
            return { base: sheet.href, text: null };
          }
        }),
    };
  });
  const assets = new Map();
  const embed = async (value, base) => {
    if (value.startsWith("data:") || value.startsWith("#")) return value;
    const url = new URL(value, base).href;
    if (!assets.has(url)) {
      assets.set(
        url,
        (async () => {
          if (!/^https?:/.test(url))
            throw new Error(
              `Unsupported resource protocol: ${new URL(url).protocol}`,
            );
          const response = await page.request.get(url);
          if (!response.ok())
            throw new Error(`Asset request failed: ${response.status()}`);
          const bytes = await response.body();
          const mime = (response.headers()["content-type"] || "").split(";")[0];
          const extension =
            {
              "image/svg+xml": "svg",
              "image/png": "png",
              "image/webp": "webp",
              "image/jpeg": "jpg",
              "font/woff2": "woff2",
              "font/woff": "woff",
            }[mime] || "bin";
          const name = `${createHash("sha256").update(bytes).digest("hex")}.${extension}`;
          await mkdir(resolve(dirname(output), "assets"), { recursive: true });
          await writeFile(resolve(dirname(output), "assets", name), bytes);
          return `assets/${name}`;
        })(),
      );
    }
    return assets.get(url);
  };
  const localize = async (text, base) => {
    const matches = [
      ...text.matchAll(/url\(\s*(?:"([^"]*)"|'([^']*)'|([^\s)]+))\s*\)/g),
    ];
    for (const match of matches)
      text = text.replaceAll(
        match[0],
        `url('${await embed(match[1] ?? match[2] ?? match[3], base)}')`,
      );
    return text;
  };
  let stylesheet = "";
  for (const sheet of result.sheets) {
    let text = sheet.text;
    if (text === null) {
      const response = await page.request.get(sheet.base);
      if (!response.ok()) throw new Error("Cannot read font stylesheet.");
      text = await response.text();
    }
    if (/@import/i.test(text))
      throw new Error(
        "Nested CSS imports require an explicitly flattened authoring stylesheet.",
      );
    stylesheet += await localize(text, sheet.base);
  }
  let html = result.html;
  for (const match of [...html.matchAll(/\bsrc="([^"]+)"/g)]) {
    html = html.replaceAll(
      match[0],
      `src="${await embed(match[1].replaceAll("&amp;", "&"), result.base)}"`,
    );
  }
  for (const { marker, value } of result.inlineStyles) {
    html = html.replace(
      marker,
      (await localize(value, result.base))
        .replaceAll("&", "&amp;")
        .replaceAll('"', "&quot;"),
    );
  }
  const css = stylesheet;
  const viewport = page.viewportSize();
  html =
    "<!doctype html>\n" +
    html.replace(
      "</head>",
      `<meta name="snapshot-viewport" content="${viewport.width}x${viewport.height}"><meta name="snapshot-conditions" content="${Buffer.from(JSON.stringify(result.conditions)).toString("base64")}"><meta http-equiv="Content-Security-Policy" content="default-src 'none'; img-src 'self' data:; font-src 'self' data:; style-src 'unsafe-inline'; form-action 'none'; base-uri 'none'"><style>${css}</style></head>`,
    );
  await mkdir(dirname(output), { recursive: true });
  await writeFile(output, html);
  return { viewport, assets: assets.size, bytes: Buffer.byteLength(html) };
}

/** Replay with scripts disabled and every non-local request blocked. */
export async function verifySnapshot(browser, output, viewport) {
  const bytes = await readFile(output);
  const encoded = bytes
    .toString()
    .match(/name="snapshot-conditions" content="([^"]+)"/)?.[1];
  if (!encoded) throw new Error("Snapshot capture conditions are required.");
  const conditions = JSON.parse(Buffer.from(encoded, "base64").toString());
  if (
    typeof conditions.locale !== "string" ||
    !["light", "dark"].includes(conditions.colorScheme) ||
    !["reduce", "no-preference"].includes(conditions.reducedMotion)
  )
    throw new Error("Invalid snapshot capture conditions.");
  const server = createServer(async (request, response) => {
    if (request.url === "/") {
      response.setHeader("Content-Type", "text/html; charset=utf-8");
      response.end(bytes);
      return;
    }
    if (!/^\/assets\/[a-f0-9]{64}\.[a-z0-9]+$/.test(request.url || "")) {
      response.writeHead(404).end();
      return;
    }
    try {
      const extension = request.url.split(".").at(-1);
      response.setHeader(
        "Content-Type",
        {
          svg: "image/svg+xml",
          webp: "image/webp",
          png: "image/png",
          jpg: "image/jpeg",
          woff2: "font/woff2",
          woff: "font/woff",
        }[extension] || "application/octet-stream",
      );
      response.end(
        await readFile(resolve(dirname(output), request.url.slice(1))),
      );
    } catch {
      response.writeHead(404).end();
    }
  });
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  const url = `http://127.0.0.1:${server.address().port}/`;
  const context = await browser.newContext({
    locale: conditions.locale,
    colorScheme: conditions.colorScheme,
    reducedMotion: conditions.reducedMotion,
    viewport,
    deviceScaleFactor: 1,
    javaScriptEnabled: false,
  });
  const blocked = [];
  try {
    await context.route("**/*", (route) => {
      if (route.request().url().startsWith(url)) return route.continue();
      blocked.push(route.request().url());
      return route.abort();
    });
    const page = await context.newPage();
    await page.goto(url, { waitUntil: "networkidle" });
    await page.evaluate(async () => {
      await document.fonts.ready;
      await Promise.all(
        [...document.images]
          .filter((image) => image.currentSrc)
          .map((image) => image.decode()),
      );
    });
    await page.screenshot({ path: `${output}.png`, fullPage: true });
    if (blocked.length)
      throw new Error(
        `Replay attempted ${blocked.length} non-local resource requests.`,
      );
  } finally {
    await context.close();
    await new Promise((resolve) => server.close(resolve));
  }
}

if (
  process.argv[1] &&
  import.meta.url === pathToFileURL(resolve(process.argv[1])).href
) {
  const args = Object.fromEntries(
    process.argv
      .slice(2)
      .reduce(
        (pairs, value, index, all) =>
          index % 2
            ? pairs
            : [...pairs, [value.replace(/^--/, ""), all[index + 1]]],
        [],
      ),
  );
  if (!args.url || !args.output || !/^\d+x\d+$/.test(args.viewport || ""))
    throw new Error(
      "Required: --url URL --output FILE --viewport WIDTHxHEIGHT",
    );
  const [width, height] = args.viewport.split("x").map(Number);
  const browser = await chromium.launch({ headless: true });
  try {
    const page = await browser.newPage({
      viewport: { width, height },
      deviceScaleFactor: 1,
    });
    await page.goto(args.url, { waitUntil: "networkidle" });
    const report = await freezePage(page, resolve(args.output));
    await verifySnapshot(browser, resolve(args.output), { width, height });
    console.log(JSON.stringify(report));
  } finally {
    await browser.close();
  }
}
