import { authenticationCopy } from "@chapchoo/web-core/authentication-messages";
import { expect, test } from "@playwright/test";
const copy = authenticationCopy("en");
test("creation stages require explicit environment and authentication choices", async ({
  page,
}) => {
  await page.goto(
    "/iframe.html?id=computers-agent-creation--environment&viewMode=story",
  );
  await expect(
    page.getByRole("radio", { name: "Community Kubernetes", exact: true }),
  ).toHaveAttribute("aria-checked", "false");
  await expect(
    page.getByRole("button", { name: copy.next, exact: true }),
  ).toBeDisabled();
  await page.goto(
    "/iframe.html?id=computers-agent-creation--authentication&viewMode=story",
  );
  await expect(page.getByText(copy.unselected, { exact: true })).toBeVisible();
  await expect(
    page.getByRole("button", { name: copy.next, exact: true }),
  ).toBeDisabled();
  await page.goto(
    "/iframe.html?id=computers-agent-creation--selected-authentication&viewMode=story",
  );
  await expect(
    page.getByRole("button", { name: copy.next, exact: true }),
  ).toBeEnabled();
});
test("creation labels and retained profile controls reflow in every product locale", async ({
  page,
}) => {
  await page.setViewportSize({ width: 320, height: 844 });
  for (const story of ["ready", "korean", "japanese", "french"]) {
    await page.goto(
      `/iframe.html?id=computers-agent-creation--${story}&viewMode=story`,
    );
    await expect(page.getByRole("textbox").first()).toBeVisible();
    const geometry = await page.evaluate(() => ({
      reflow: document.documentElement.scrollWidth <= innerWidth,
      overflow: Array.from(document.querySelectorAll("#storybook-root *"))
        .filter(
          (element) => element.getBoundingClientRect().right > innerWidth + 1,
        )
        .slice(-12)
        .map((element) => ({
          tag: element.tagName,
          className: element.className,
          right: element.getBoundingClientRect().right,
          text: element.textContent.slice(0, 40),
        })),
    }));
    expect(geometry.reflow, JSON.stringify({ story, ...geometry })).toBe(true);
  }
});
