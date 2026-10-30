import { test as base } from "@playwright/test";
import { Assembly } from "@chapchoo/testenv/assembly";

async function verifyCanonicalRedirects(assembly: Assembly): Promise<void> {
  for (const base of [assembly.generalUrl, assembly.adminUrl]) {
    for (const suffix of ["/login/", "//login"]) {
      const response = await fetch(`${base}${suffix}?fixture=canonical`, {
        redirect: "manual",
      });
      const location = response.headers.get("location");
      if (
        response.status !== 308 ||
        location !==
          `${new URL(base).pathname.replace(/\/$/, "")}/login?fixture=canonical`
      ) {
        throw new Error(
          "Canonical web redirect must preserve the configured public mount and query",
        );
      }
      await response.body?.cancel();
    }
  }
}

export const test = base.extend<object, { assembly: Assembly }>({
  assembly: [
    async ({ playwright }, use) => {
      void playwright;
      const assembly = await Assembly.create();
      try {
        await verifyCanonicalRedirects(assembly);
        await use(assembly);
      } finally {
        await assembly.close();
      }
    },
    { scope: "worker", timeout: 660_000 },
  ],
});
export { expect } from "@playwright/test";
export type { Assembly } from "@chapchoo/testenv/assembly";
