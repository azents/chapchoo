import "server-only";
import { attempt } from "@chapchoo/web-core/boundary";
import { invalidResponse } from "@chapchoo/web-core/errors";
import { call } from "@chapchoo/web-core/rpc";
import { grant, grants } from "@chapchoo/web-core/validation";
import { z } from "zod";
import { authRouter } from "@/shared/server/auth";
import { inventoryInput } from "@/shared/server/inventory";
import { procedure, router } from "@/shared/server/trpc";
import type { IssuedGrant } from "@chapchoo/web-core/browser";

const email = z.object({ email: z.string().min(1).max(320) }).strict();
const revoke = z.object({ grantId: z.string().min(1).max(128) }).strict();

function issued(value: IssuedGrant): IssuedGrant {
  if (value.grant == null || !/^[A-Za-z0-9_-]{43}$/.test(value.proof)) {
    return invalidResponse();
  }
  grant(value.grant);
  let url: URL;
  try {
    url = new URL(value.url);
  } catch {
    return invalidResponse();
  }
  if (
    !["https:", "http:"].includes(url.protocol) ||
    url.username ||
    url.password
  ) {
    return invalidResponse();
  }
  // This newly issued bearer link is an intentional management result, unlike
  // login-session credentials, which never leave the server auth boundary.
  return value;
}

export const appRouter = router({
  auth: authRouter,
  setup: router({
    status: procedure.query(({ ctx }) =>
      attempt(() =>
        call(ctx.backend.admin.setupStatus.bind(ctx.backend.admin), {}),
      ),
    ),
    initialize: procedure
      .input(
        z
          .object({
            installationProof: z.string().min(1).max(256),
            email: z.string().min(1).max(320),
            password: z.string().min(1).max(64),
          })
          .strict(),
      )
      .mutation(({ ctx, input }) =>
        attempt(async () =>
          ctx.session.acceptAuthentication(
            await call(
              ctx.backend.admin.initialize.bind(ctx.backend.admin),
              input,
            ),
          ),
        ),
      ),
  }),
  signupGrants: router({
    list: procedure
      .input(inventoryInput)
      .query(({ ctx, input }) =>
        attempt(() =>
          ctx.session.authenticated(async (token) =>
            grants(
              await call(
                ctx.backend.admin.listSignupGrants.bind(ctx.backend.admin),
                { offset: input.offset, limit: input.limit },
                token,
              ),
            ),
          ),
        ),
      ),
    issue: procedure
      .input(email)
      .mutation(({ ctx, input }) =>
        attempt(() =>
          ctx.session.authenticated(async (token) =>
            issued(
              await call(
                ctx.backend.admin.issueSignupGrant.bind(ctx.backend.admin),
                input,
                token,
              ),
            ),
          ),
        ),
      ),
    revoke: procedure.input(revoke).mutation(({ ctx, input }) =>
      attempt(() =>
        ctx.session.authenticated(async (token) => {
          await call(
            ctx.backend.admin.revokeSignupGrant.bind(ctx.backend.admin),
            input,
            token,
          );
          return { completed: true };
        }),
      ),
    ),
  }),
  resetGrants: router({
    list: procedure
      .input(inventoryInput)
      .query(({ ctx, input }) =>
        attempt(() =>
          ctx.session.authenticated(async (token) =>
            grants(
              await call(
                ctx.backend.admin.listPasswordResets.bind(ctx.backend.admin),
                { offset: input.offset, limit: input.limit },
                token,
              ),
            ),
          ),
        ),
      ),
    issue: procedure
      .input(email)
      .mutation(({ ctx, input }) =>
        attempt(() =>
          ctx.session.authenticated(async (token) =>
            issued(
              await call(
                ctx.backend.admin.issuePasswordReset.bind(ctx.backend.admin),
                input,
                token,
              ),
            ),
          ),
        ),
      ),
    revoke: procedure.input(revoke).mutation(({ ctx, input }) =>
      attempt(() =>
        ctx.session.authenticated(async (token) => {
          await call(
            ctx.backend.admin.revokePasswordReset.bind(ctx.backend.admin),
            input,
            token,
          );
          return { completed: true };
        }),
      ),
    ),
  }),
});

export type AppRouter = typeof appRouter;
