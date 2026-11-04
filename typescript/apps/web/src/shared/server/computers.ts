import "server-only";
import { agentPaletteKeys } from "@chapchoo/web-core/agent-profile";
import { attempt } from "@chapchoo/web-core/boundary";
import {
  actionNumber,
  agent,
  agents,
  choices,
  creation,
  profileImages,
} from "@chapchoo/web-core/computer";
import { validHandle } from "@chapchoo/web-core/handle";
import { call } from "@chapchoo/web-core/rpc";
import { z } from "zod";
import { procedure, router } from "@/shared/server/trpc";

const id = z.string().min(1).max(128);
const page = z.object({ after: id.optional() }).strict();
export const computersRouter = router({
  images: procedure
    .input(page)
    .query(({ ctx, input }) =>
      attempt(() =>
        ctx.session.authenticated(async (token) =>
          profileImages(
            await call(
              ctx.backend.agents.listProfileImages.bind(ctx.backend.agents),
              { after: input.after, limit: 20 },
              token,
            ),
          ),
        ),
      ),
    ),
  handleAvailable: procedure
    .input(
      z
        .object({
          teamId: id,
          candidate: z.string().refine(validHandle),
        })
        .strict(),
    )
    .query(({ ctx, input }) =>
      attempt(() =>
        ctx.session.authenticated(
          async (token) =>
            (
              await call(
                ctx.backend.agents.checkHandle.bind(ctx.backend.agents),
                input,
                token,
              )
            ).available,
        ),
      ),
    ),
  list: procedure
    .input(page.extend({ teamId: id }))
    .query(({ ctx, input }) =>
      attempt(() =>
        ctx.session.authenticated(async (token) =>
          agents(
            await call(
              ctx.backend.agents.list.bind(ctx.backend.agents),
              { teamId: input.teamId, page: { after: input.after, limit: 20 } },
              token,
            ),
          ),
        ),
      ),
    ),
  get: procedure
    .input(z.object({ agentId: id }).strict())
    .query(({ ctx, input }) =>
      attempt(() =>
        ctx.session.authenticated(async (token) =>
          agent(
            await call(
              ctx.backend.agents.get.bind(ctx.backend.agents),
              input,
              token,
            ),
          ),
        ),
      ),
    ),
  providers: procedure
    .input(page)
    .query(({ ctx, input }) =>
      attempt(() =>
        ctx.session.authenticated(async (token) =>
          choices(
            await call(
              ctx.backend.providerCatalog.list.bind(
                ctx.backend.providerCatalog,
              ),
              { after: input.after, limit: 20 },
              token,
            ),
          ),
        ),
      ),
    ),
  create: procedure
    .input(
      z
        .object({
          requestId: id,
          providerId: id,
          teamId: id,
          name: z.string().trim().min(1).max(120),
          handle: z.string().refine(validHandle),
          profileImageId: id,
          paletteKey: z.enum(agentPaletteKeys),
        })
        .strict(),
    )
    .mutation(({ ctx, input }) =>
      attempt(() =>
        ctx.session.authenticated(async (token) =>
          creation(
            await call(
              ctx.backend.agents.create.bind(ctx.backend.agents),
              input,
              token,
            ),
          ),
        ),
      ),
    ),
  lifecycle: procedure
    .input(
      z
        .object({
          agentId: id,
          requestId: id,
          expectedRequestId: id,
          action: z.enum([
            "start",
            "stop",
            "restart",
            "delete",
            "retryPreparation",
          ]),
        })
        .strict(),
    )
    .mutation(({ ctx, input }) =>
      attempt(() =>
        ctx.session.authenticated(async (token) =>
          agent(
            input.action === "retryPreparation"
              ? await call(
                  ctx.backend.computers.retryPreparation.bind(
                    ctx.backend.computers,
                  ),
                  {
                    agentId: input.agentId,
                    requestId: input.requestId,
                    expectedRequestId: input.expectedRequestId,
                  },
                  token,
                )
              : await call(
                  ctx.backend.computers.requestLifecycle.bind(
                    ctx.backend.computers,
                  ),
                  { ...input, action: actionNumber(input.action) },
                  token,
                ),
          ),
        ),
      ),
    ),
  forceRemove: procedure
    .input(z.object({ agentId: id, deletionRequestId: id }).strict())
    .mutation(({ ctx, input }) =>
      attempt(() =>
        ctx.session.authenticated(async (token) => {
          await call(
            ctx.backend.agents.forceRemove.bind(ctx.backend.agents),
            input,
            token,
          );
          return { completed: true };
        }),
      ),
    ),
});
