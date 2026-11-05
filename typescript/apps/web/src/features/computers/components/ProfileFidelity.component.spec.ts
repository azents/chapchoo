import { computerCopy } from "@chapchoo/web-core/computer-messages";
import { expect, test } from "@playwright/test";
import { creationCopy } from "../creationCopy";

const ko = creationCopy("ko");
const computer = computerCopy("ko-KR");
test("approved cards keep character/name hierarchy and collapse before the sidebar crowds them", async ({
  page,
}) => {
  for (const { width, columns } of [
    { width: 1440, columns: 2 },
    { width: 900, columns: 1 },
    { width: 390, columns: 1 },
  ]) {
    await page.setViewportSize({ width, height: 1000 });
    await page.goto(
      "/iframe.html?id=computers-agents--approved-list&viewMode=story",
    );
    const first = page.getByRole("link", { name: "두부", exact: true });
    const second = page.getByRole("link", { name: "보리", exact: true });
    await expect(first).toBeVisible();
    const a = await first.boundingBox();
    const b = await second.boundingBox();
    expect(a).not.toBeNull();
    expect(b).not.toBeNull();
    if (!a || !b) {
      throw new Error("Profile card geometry is unavailable");
    }
    expect(Math.abs(a.y - b.y) < 2).toBe(columns === 2);
    expect(a.width).toBeGreaterThan(280);
    expect(
      await first
        .locator("img")
        .evaluate((element) => element.getBoundingClientRect().width),
    ).toBeGreaterThanOrEqual(width === 1440 ? 160 : 120);
    await expect(first).toContainText(ko.open);
    await expect(first).toContainText(computer.running);
    await expect(
      first.getByLabel(`${computer.phase}: ${computer.completed}`, {
        exact: true,
      }),
    ).toHaveCount(1);
    expect(
      await page.evaluate(
        () => document.documentElement.scrollWidth <= innerWidth,
      ),
    ).toBe(true);
  }
});
test("approved editor uses a desktop image grid and mobile single-row choices with a reachable primary footer", async ({
  page,
}) => {
  for (const width of [1440, 390]) {
    await page.setViewportSize({ width, height: width === 1440 ? 1000 : 844 });
    await page.goto(
      "/iframe.html?id=computers-agents--approved-edit&viewMode=story",
    );
    const dialog = page.getByRole("dialog", { name: ko.edit, exact: true });
    await expect(dialog).toBeVisible();
    const choices = dialog
      .getByRole("group", { name: ko.image, exact: true })
      .getByRole("button");
    await expect(choices).toHaveCount(6);
    const rows = await choices.evaluateAll((elements) =>
      elements.map((element) =>
        Math.round(element.getBoundingClientRect().top),
      ),
    );
    expect(new Set(rows).size).toBe(width === 1440 ? 2 : 1);
    const save = dialog.getByRole("button", { name: ko.save, exact: true });
    await expect(save).toBeEnabled();
    if (width === 390) {
      expect(
        await save.evaluate((element) => element.getBoundingClientRect().width),
      ).toBeGreaterThan(200);
    }
    await expect(
      dialog.getByRole("group", { name: ko.handle, exact: true }),
    ).toContainText("@dubu");
    expect(
      await page.evaluate(
        () => document.documentElement.scrollWidth <= innerWidth,
      ),
    ).toBe(true);
  }
});
test("Team creation keeps count, automatic-proposal guidance, address and cancel together on both sizes", async ({
  page,
}) => {
  for (const width of [1440, 390]) {
    await page.setViewportSize({ width, height: 1000 });
    await page.goto(
      "/iframe.html?id=computers-agents--approved-team-creation&viewMode=story",
    );
    const dialog = page.getByRole("dialog", { name: "팀 만들기", exact: true });
    await expect(dialog).toBeVisible();
    await expect(
      dialog.getByRole("textbox", { name: "핸들", exact: true }),
    ).toHaveValue("dijain-tim");
    await expect(dialog.getByText("10/16", { exact: true })).toBeVisible();
    await expect(
      dialog.getByText("/teams/dijain-tim", { exact: true }),
    ).toBeVisible();
    await expect(dialog.getByText(/이름에서 자동 생성/)).toBeVisible();
    await expect(
      dialog.getByRole("button", { name: "팀 만들기", exact: true }),
    ).toBeEnabled();
    await dialog.getByRole("button", { name: "취소", exact: true }).click();
    await expect(dialog).toHaveCount(0);
    await expect(
      page.getByRole("button", { name: "공간 선택", exact: true }).last(),
    ).toContainText("내 공간");
  }
});

test("long profile names keep a separate edit target on narrow screens", async ({
  page,
}) => {
  for (const width of [320, 390]) {
    await page.setViewportSize({ width, height: 844 });
    await page.goto(
      "/iframe.html?id=computers-agents--approved-long-name&viewMode=story",
    );
    const name = await page.getByRole("heading", { level: 1 }).boundingBox();
    const edit = await page
      .getByRole("button", { name: ko.edit, exact: true })
      .boundingBox();
    expect(name).not.toBeNull();
    expect(edit).not.toBeNull();
    if (!name || !edit) {
      throw new Error("Profile geometry is unavailable");
    }
    expect(
      edit.y + edit.height <= name.y ||
        edit.x + edit.width <= name.x ||
        name.x + name.width <= edit.x,
    ).toBe(true);
    expect(
      await page.evaluate(
        () => document.documentElement.scrollWidth <= innerWidth,
      ),
    ).toBe(true);
  }
});

test("profile editor initially shows the whole preview instead of scrolling to the name input", async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(
    "/iframe.html?id=computers-agents--approved-edit&viewMode=story",
  );
  const dialog = page.getByRole("dialog", { name: ko.edit, exact: true });
  await expect(
    dialog.getByRole("region", { name: ko.preview, exact: true }),
  ).toBeInViewport({ ratio: 1 });
  await expect(
    dialog.getByRole("textbox", { name: ko.name, exact: true }),
  ).not.toBeFocused();
});

test("large profile placements opt into motion while image-choice thumbnails remain static", async ({
  page,
}) => {
  for (const { story, count } of [
    { story: "computers-agents--approved-list", count: 4 },
    { story: "computers-agents--approved-detail", count: 1 },
    { story: "computers-agents--approved-team-created", count: 1 },
    { story: "computers-agent-creation--approved", count: 1 },
  ]) {
    await page.goto(`/iframe.html?id=${story}&viewMode=story`);
    await expect(page.getByRole("main").locator("[data-playing]")).toHaveCount(
      count,
    );
  }
  const handle = page.getByRole("textbox", { name: ko.handle, exact: true });
  await expect(handle).toHaveAttribute("autocapitalize", "none");
  await expect(handle).toHaveAttribute("autocorrect", "off");
  await expect(handle).toHaveAttribute("autocomplete", "off");
  await expect(handle).toHaveAttribute("spellcheck", "false");
  await expect(handle).toHaveAttribute("inputmode", "text");
  await expect(
    page
      .getByRole("group", { name: ko.image, exact: true })
      .locator("[data-playing]"),
  ).toHaveCount(0);
});
