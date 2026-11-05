import { test } from "node:test";
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { resolve, join } from "node:path";
import { mkdtemp, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { createServer } from "node:http";
import { freezePage, verifySnapshot } from "./freeze-page.mjs";
const require = createRequire(resolve("typescript/apps/web/package.json"));
const { chromium } = require("@playwright/test");

test("resolved form, pseudo-element and SVG survive without authoring scripts", async () => {
  const directory = await mkdtemp(join(tmpdir(), "design-freeze-"));
  const browser = await chromium.launch();
  try {
    const page = await browser.newPage({
      viewport: { width: 700, height: 500 },
    });
    await page.setContent(
      `<!doctype html><html><head><style>body{background:rgb(245,240,220)}label::before{content:'Review';color:red}</style></head><body><label><input value="initial" onclick="alert(1)"></label><textarea>old</textarea><select><option>A</option><option>B</option></select><svg width="20" height="20"><circle cx="10" cy="10" r="8" fill="blue"/></svg><script>document.querySelector('input').value='reviewed';document.querySelector('textarea').value='saved';document.querySelector('select').selectedIndex=1;</script></body></html>`,
    );
    const output = join(directory, "state.html");
    const report = await freezePage(page, output);
    const html = await readFile(output, "utf8");
    assert.ok(html.includes('value="reviewed"'));
    assert.ok(html.includes(">saved</textarea>"));
    assert.match(html, /selected[^>]*>B/);
    assert.ok(html.includes("::before"));
    assert.ok(!html.includes("<script"));
    assert.ok(!html.includes("onclick="));
    const source = await page.screenshot({ fullPage: true });
    await verifySnapshot(browser, output, report.viewport);
    assert.deepEqual(await readFile(output + ".png"), source);
    for (const surface of [
      "<canvas/>",
      "<iframe/>",
      '<input type="password" value="secret">',
    ]) {
      await page.setContent(
        `<!doctype html><html><head></head><body>${surface}</body></html>`,
      );
      await assert.rejects(freezePage(page, output));
    }
  } finally {
    await browser.close();
    await rm(directory, { recursive: true, force: true });
  }
});

test("localizes SVG resources and preserves an open native dialog backdrop", async () => {
  const directory = await mkdtemp(join(tmpdir(), "design-freeze-"));
  const server = createServer((request, response) => {
    if (request.url === "/icon.svg") {
      response.setHeader("Content-Type", "image/svg+xml");
      response.end(
        '<svg xmlns="http://www.w3.org/2000/svg" width="50" height="50"><circle cx="25" cy="25" r="20" fill="red"/></svg>',
      );
    } else {
      response.setHeader("Content-Type", "text/html");
      response.end(
        '<!doctype html><html><head><style>.box{width:90px;height:80px;background:url("/icon.svg")}dialog::backdrop{background:rgba(20,30,40,.3);backdrop-filter:blur(2px)}</style></head><body><h1>Baseline</h1><div class="box"></div><dialog><h2>Review</h2><img src="/icon.svg"><button>Close</button></dialog></body></html>',
      );
    }
  });
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  const browser = await chromium.launch();
  try {
    const page = await browser.newPage({
      viewport: { width: 700, height: 500 },
      locale: "ko-KR",
      reducedMotion: "reduce",
    });
    await page.goto(`http://127.0.0.1:${server.address().port}/`);
    await page.evaluate(() => {
      document.querySelector("dialog").showModal();
      document.activeElement.blur();
    });
    const output = join(directory, "dialog.html");
    const report = await freezePage(page, output);
    assert.equal(report.assets, 1);
    const source = await page.screenshot({ fullPage: true });
    await verifySnapshot(browser, output, report.viewport);
    assert.deepEqual(await readFile(output + ".png"), source);
  } finally {
    await browser.close();
    await new Promise((resolve) => server.close(resolve));
    await rm(directory, { recursive: true, force: true });
  }
});
