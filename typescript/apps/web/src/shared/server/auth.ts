import "server-only";
import { attempt } from "@chapchoo/web-core/boundary";
import { call } from "@chapchoo/web-core/rpc";
import { challenge, user } from "@chapchoo/web-core/validation";
import { z } from "zod";
import { procedure, router } from "@/shared/server/trpc";

const email = z.string().min(1).max(320);
const password = z.string().min(1).max(64);
export const authRouter = router({
  methods: procedure.query(({ ctx }) =>
    attempt(() => call(ctx.backend.auth.methods.bind(ctx.backend.auth), {})),
  ),
  passwordLogin: procedure
    .input(z.object({ email, password }).strict())
    .mutation(({ ctx, input }) =>
      attempt(async () =>
        ctx.session.acceptAuthentication(
          await call(
            ctx.backend.auth.passwordLogin.bind(ctx.backend.auth),
            input,
          ),
        ),
      ),
    ),
  sendCode: procedure
    .input(z.object({ email }).strict())
    .mutation(({ ctx, input }) =>
      attempt(async () =>
        challenge(
          await call(ctx.backend.auth.sendCode.bind(ctx.backend.auth), input),
        ),
      ),
    ),
  verifyCode: procedure
    .input(
      z
        .object({
          email,
          binding: z.string().min(1).max(256),
          code: z.string().min(1).max(64),
        })
        .strict(),
    )
    .mutation(({ ctx, input }) =>
      attempt(async () =>
        ctx.session.acceptAuthentication(
          await call(ctx.backend.auth.verifyCode.bind(ctx.backend.auth), input),
        ),
      ),
    ),
  identity: procedure.query(({ ctx }) =>
    attempt(() =>
      ctx.session.authenticated(async (token) => {
        const result = user(
          await call(
            ctx.backend.auth.identity.bind(ctx.backend.auth),
            {},
            token,
          ),
        );

        return result;
      }),
    ),
  ),
  logout: procedure.mutation(({ ctx }) =>
    attempt(() =>
      ctx.session.authenticated(async (token) => {
        await call(ctx.backend.auth.logout.bind(ctx.backend.auth), {}, token);
        ctx.session.clear();
        return { completed: true };
      }),
    ),
  ),
});
