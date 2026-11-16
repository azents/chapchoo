import { expect, test } from "@playwright/test";

for (const width of [390, 1440]) {
  test(`return action appears while reading and retains unread label through exit/${width}`, async ({
    page,
  }) => {
    await page.setViewportSize({ width, height: 1000 });
    await page.goto(
      "/iframe.html?id=chat-conversation--unread-messages&viewMode=story",
    );
    const log = page.getByRole("log", { name: "Message", exact: true });
    await expect
      .poll(() => log.evaluate((node) => node.scrollTop))
      .toBeGreaterThan(50);
    await expect(page.locator("[data-return-to-latest]")).toHaveCount(0);
    await log.evaluate((node) => {
      node.scrollTop -= 30;
    });
    const down = page.getByRole("button", { name: "Scroll down", exact: true });
    await expect(down).toBeVisible();
    await page
      .getByRole("button", { name: "Receive message", exact: true })
      .click();
    const unread = page.getByRole("button", {
      name: "View new messages",
      exact: true,
    });
    await expect(unread).toBeVisible();
    await expect
      .poll(() =>
        unread.evaluate((node) => node.getAnimations({ subtree: true }).length),
      )
      .toBe(0);
    const material = await unread.evaluate((node) => {
      const status = document.querySelector(
        'header button[aria-haspopup="dialog"][aria-label*=" · "]',
      )?.parentElement;
      if (!status) {
        throw new Error("Missing status summary");
      }
      const surface = (element: Element): string[] => {
        const css = getComputedStyle(element);
        return [css.background, css.backdropFilter, css.border, css.boxShadow];
      };
      return { button: surface(node), summary: surface(status) };
    });
    expect(material.button).toEqual(material.summary);
    await page.evaluate(() => {
      const button = document.querySelector("[data-return-to-latest]");
      if (!button) {
        throw new Error("Missing return action");
      }
      button.setAttribute("data-wrong-label", "false");
      const observer = new MutationObserver(() => {
        if (button.getAttribute("aria-label") !== "View new messages") {
          button.setAttribute("data-wrong-label", "true");
        }
      });
      observer.observe(button, {
        attributes: true,
        attributeFilter: ["aria-label"],
      });
    });
    await unread.click();
    await expect(
      page.locator('[data-return-to-latest="exiting"]'),
    ).toHaveAttribute("aria-label", "View new messages");
    await expect(page.locator("[data-wrong-label=true]")).toHaveCount(0);
    await expect(page.locator("[data-return-to-latest]")).toHaveCount(0);
    await expect
      .poll(() =>
        log.evaluate(
          (node) => node.scrollHeight - node.clientHeight - node.scrollTop,
        ),
      )
      .toBeLessThan(2);
    await log.evaluate((node) => {
      node.scrollTop -= 100;
    });
    await expect(down).toBeVisible();
    const scrollbar = page.locator(
      '.mantine-ScrollArea-scrollbar[data-orientation="vertical"]',
    );
    await log.hover();
    if (width < 768) {
      await expect(scrollbar).toBeHidden();
    } else {
      await expect(scrollbar).toBeVisible();
    }
  });
}

test("desktop sidebar collapses inline, reverses and remains fixed when expanded", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1440, height: 1000 });
  await page.goto(
    "/iframe.html?id=chat-conversation--unread-messages&viewMode=story",
  );
  const sidebar = page.locator("#chat-sidebar");
  const main = page.getByRole("main");
  const original = await main.boundingBox();
  await page
    .getByRole("button", { name: "Collapse sidebar", exact: true })
    .click();
  await expect(sidebar).toHaveAttribute("inert", "");
  await expect(page.getByRole("dialog")).toHaveCount(0);
  await expect
    .poll(() => main.evaluate((node) => node.getBoundingClientRect().left))
    .toBe(0);
  await page
    .getByRole("button", { name: "Expand sidebar", exact: true })
    .click();
  await expect
    .poll(() => main.evaluate((node) => node.getBoundingClientRect().left))
    .toBeCloseTo(original?.x ?? -1, 0);
  await expect(sidebar).not.toHaveAttribute("inert");
  await page
    .getByRole("button", { name: "Collapse sidebar", exact: true })
    .click();
  await page
    .getByRole("button", { name: "Expand sidebar", exact: true })
    .click();
  await expect
    .poll(() => main.evaluate((node) => node.getBoundingClientRect().left))
    .toBeCloseTo(original?.x ?? -1, 0);
  await expect(page.getByRole("dialog")).toHaveCount(0);
});

test("mobile navigation remains a drawer and releases its trap on desktop resize", async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(
    "/iframe.html?id=chat-conversation--connected&viewMode=story",
  );
  await page
    .getByRole("button", { name: "에이전트와 대화", exact: true })
    .click();
  await expect(
    page.getByRole("dialog", { name: "에이전트와 대화", exact: true }),
  ).toBeVisible();
  await page.setViewportSize({ width: 1440, height: 1000 });
  await expect(page.getByRole("dialog")).toHaveCount(0);
  const collapse = page.getByRole("button", {
    name: "사이드바 접기",
    exact: true,
  });
  await expect(collapse).toBeFocused();
  await collapse.press("Enter");
  await expect(
    page.getByRole("button", { name: "사이드바 펼치기", exact: true }),
  ).toBeFocused();
});

test("changing motion preference preserves detached reading", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=chat-conversation--unread-messages&viewMode=story",
  );
  const log = page.getByRole("log");
  await expect
    .poll(() => log.evaluate((node) => node.scrollTop))
    .toBeGreaterThan(100);
  await log.evaluate((node) => {
    node.scrollTop = 100;
  });
  await expect(
    page.getByRole("button", { name: "Scroll down", exact: true }),
  ).toBeVisible();
  await page.emulateMedia({ reducedMotion: "reduce" });
  await expect.poll(() => log.evaluate((node) => node.scrollTop)).toBe(100);
  await expect(
    page.getByRole("button", { name: "Scroll down", exact: true }),
  ).toBeVisible();
});

test("reduced motion returns immediately and preserves the exit label", async ({
  page,
}) => {
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.goto(
    "/iframe.html?id=chat-conversation--unread-messages&viewMode=story",
  );
  const log = page.getByRole("log");
  await expect
    .poll(() => log.evaluate((node) => node.scrollTop))
    .toBeGreaterThan(50);
  await log.evaluate((node) => {
    node.scrollTop = 100;
  });
  await page
    .getByRole("button", { name: "Receive message", exact: true })
    .click();
  await page
    .getByRole("button", { name: "View new messages", exact: true })
    .click();
  await expect(page.locator("[data-return-to-latest]")).toHaveCount(0);
  await expect
    .poll(() =>
      log.evaluate(
        (node) => node.scrollHeight - node.clientHeight - node.scrollTop,
      ),
    )
    .toBeLessThan(2);
});
