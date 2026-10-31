import "server-only";
import {
  decodeDisplayCookie,
  explicitDisplayCookie,
  projectedDisplayCookie,
  resolveGuestDisplay,
} from "@chapchoo/web-core/display";
import { requiredPageRead } from "@chapchoo/web-core/page-read";
import { call } from "@chapchoo/web-core/rpc";
import { display } from "@chapchoo/web-core/validation";
import { cookies } from "next/headers";
import { cache } from "react";
import { getConfig } from "@/config";
import { loadIdentity } from "@/shared/auth/server";
import { getBackend } from "@/shared/server/backend";
import type { DisplayInitial } from "@chapchoo/web-core/display-context";

export const readDisplay = cache(async (): Promise<DisplayInitial> => {
  const store = await cookies();
  const explicit = decodeDisplayCookie(store.get(explicitDisplayCookie)?.value);
  const projection = decodeDisplayCookie(
    store.get(projectedDisplayCookie)?.value,
  );
  const backend = getBackend(getConfig());
  const identity = await loadIdentity();
  const defaults = await requiredPageRead(async () =>
    display(await call(backend.auth.displayDefaults.bind(backend.auth), {})),
  );
  const value =
    identity.type === "LOADED" && identity.data
      ? display(identity.data.display)
      : resolveGuestDisplay(defaults, explicit, projection);
  return {
    identity,
    value,
    explicit,
    defaults,
    instant: Date.now(),
    zones: Intl.supportedValuesOf("timeZone"),
  };
});
