import "server-only";
import { attempt } from "@chapchoo/web-core/boundary";
import { inventory, providers, resources } from "@chapchoo/web-core/computer";
import { call } from "@chapchoo/web-core/rpc";
import { z } from "zod";
import { resourceInput } from "@/shared/server/resourceInput";
import { procedure, router } from "@/shared/server/trpc";

const id = z.string().min(1).max(128);
export const computersRouter = router({
  resources: procedure
    .input(resourceInput)
    .query(({ ctx, input }) =>
      attempt(() =>
        ctx.session.authenticated(async (token) =>
          resources(
            await call(
              ctx.backend.providerInventory.listResources.bind(
                ctx.backend.providerInventory,
              ),
              { ...input, after: input.after, limit: 20 },
              token,
            ),
          ),
        ),
      ),
    ),
  providers: procedure
    .input(z.object({ after: id.optional() }).strict())
    .query(({ ctx, input }) =>
      attempt(() =>
        ctx.session.authenticated(async (token) =>
          providers(
            await call(
              ctx.backend.providerInventory.listProviders.bind(
                ctx.backend.providerInventory,
              ),
              { after: input.after, limit: 20 },
              token,
            ),
          ),
        ),
      ),
    ),
  list: procedure
    .input(z.object({ providerId: id, after: id.optional() }).strict())
    .query(({ ctx, input }) =>
      attempt(() =>
        ctx.session.authenticated(async (token) =>
          inventory(
            await call(
              ctx.backend.providerInventory.listComputers.bind(
                ctx.backend.providerInventory,
              ),
              {
                providerId: input.providerId,
                page: { after: input.after, limit: 20 },
              },
              token,
            ),
          ),
        ),
      ),
    ),
});
