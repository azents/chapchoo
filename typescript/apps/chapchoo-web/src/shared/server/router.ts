import "server-only";
import { attempt } from "@chapchoo/web-core/boundary";
import { call } from "@chapchoo/web-core/rpc";
import {
  invitation,
  invitations,
  members,
  preview,
  team,
  teams,
} from "@chapchoo/web-core/validation";
import { z } from "zod";
import { authRouter } from "@/shared/server/auth";
import { procedure, router } from "@/shared/server/trpc";

const email = z.string().min(1).max(320);
const proof = z.object({ proof: z.string().min(1).max(256) }).strict();
const password = z.string().min(1).max(64);
const id = z.string().min(1).max(128);
const teamInput = z.object({ teamId: id }).strict();
const memberInput = z.object({ teamId: id, userId: id }).strict();
const name = z.string().min(1).max(120);
const handle = z.string().min(1).max(80);

export const appRouter = router({
  auth: authRouter,
  signup: router({
    preview: procedure
      .input(proof)
      .query(({ ctx, input }) =>
        attempt(async () =>
          preview(
            await call(
              ctx.backend.general.previewSignup.bind(ctx.backend.general),
              input,
            ),
          ),
        ),
      ),
    redeem: procedure
      .input(
        z
          .object({ proof: z.string().min(1).max(256), email, password })
          .strict(),
      )
      .mutation(({ ctx, input }) =>
        attempt(async () =>
          ctx.session.acceptAuthentication(
            await call(
              ctx.backend.general.redeemSignup.bind(ctx.backend.general),
              input,
            ),
          ),
        ),
      ),
    requestEmail: procedure
      .input(z.object({ email }).strict())
      .mutation(({ ctx, input }) =>
        attempt(() =>
          call(
            ctx.backend.general.requestSignupEmail.bind(ctx.backend.general),
            input,
          ),
        ),
      ),
  }),
  reset: router({
    preview: procedure
      .input(proof)
      .query(({ ctx, input }) =>
        attempt(async () =>
          preview(
            await call(
              ctx.backend.general.previewPasswordReset.bind(
                ctx.backend.general,
              ),
              input,
            ),
          ),
        ),
      ),
    redeem: procedure
      .input(z.object({ proof: z.string().min(1).max(256), password }).strict())
      .mutation(({ ctx, input }) =>
        attempt(async () => {
          await call(
            ctx.backend.general.redeemPasswordReset.bind(ctx.backend.general),
            input,
          );
          ctx.session.clear();
          return { completed: true };
        }),
      ),
  }),
  teams: router({
    list: procedure.query(({ ctx }) =>
      attempt(() =>
        ctx.session.authenticated(async (token) =>
          teams(
            await call(
              ctx.backend.general.listTeams.bind(ctx.backend.general),
              {},
              token,
            ),
          ),
        ),
      ),
    ),
    get: procedure
      .input(teamInput)
      .query(({ ctx, input }) =>
        attempt(() =>
          ctx.session.authenticated(async (token) =>
            team(
              await call(
                ctx.backend.general.getTeam.bind(ctx.backend.general),
                input,
                token,
              ),
            ),
          ),
        ),
      ),
    create: procedure
      .input(z.object({ name, handle }).strict())
      .mutation(({ ctx, input }) =>
        attempt(() =>
          ctx.session.authenticated(async (token) =>
            team(
              await call(
                ctx.backend.general.createTeam.bind(ctx.backend.general),
                input,
                token,
              ),
            ),
          ),
        ),
      ),
    update: procedure
      .input(
        z
          .object({
            teamId: id,
            name: name.optional(),
            handle: handle.optional(),
          })
          .strict(),
      )
      .mutation(({ ctx, input }) =>
        attempt(() =>
          ctx.session.authenticated(async (token) =>
            team(
              await call(
                ctx.backend.general.updateTeam.bind(ctx.backend.general),
                {
                  teamId: input.teamId,
                  name: input.name,
                  handle: input.handle,
                },
                token,
              ),
            ),
          ),
        ),
      ),
    members: procedure
      .input(teamInput)
      .query(({ ctx, input }) =>
        attempt(() =>
          ctx.session.authenticated(async (token) =>
            members(
              await call(
                ctx.backend.general.listMembers.bind(ctx.backend.general),
                input,
                token,
              ),
            ),
          ),
        ),
      ),
    setRole: procedure
      .input(
        z
          .object({
            teamId: id,
            userId: id,
            role: z.union([z.literal(1), z.literal(2)]),
          })
          .strict(),
      )
      .mutation(({ ctx, input }) =>
        attempt(() =>
          ctx.session.authenticated(async (token) => {
            await call(
              ctx.backend.general.setMemberRole.bind(ctx.backend.general),
              input,
              token,
            );
            return { completed: true };
          }),
        ),
      ),
    removeMember: procedure.input(memberInput).mutation(({ ctx, input }) =>
      attempt(() =>
        ctx.session.authenticated(async (token) => {
          await call(
            ctx.backend.general.removeMember.bind(ctx.backend.general),
            input,
            token,
          );
          return { completed: true };
        }),
      ),
    ),
    transferOwnership: procedure.input(memberInput).mutation(({ ctx, input }) =>
      attempt(() =>
        ctx.session.authenticated(async (token) => {
          await call(
            ctx.backend.general.transferOwnership.bind(ctx.backend.general),
            input,
            token,
          );
          return { completed: true };
        }),
      ),
    ),
    leave: procedure.input(teamInput).mutation(({ ctx, input }) =>
      attempt(() =>
        ctx.session.authenticated(async (token) => {
          await call(
            ctx.backend.general.leaveTeam.bind(ctx.backend.general),
            input,
            token,
          );
          return { completed: true };
        }),
      ),
    ),
    delete: procedure.input(teamInput).mutation(({ ctx, input }) =>
      attempt(() =>
        ctx.session.authenticated(async (token) => {
          await call(
            ctx.backend.general.deleteTeam.bind(ctx.backend.general),
            input,
            token,
          );
          return { completed: true };
        }),
      ),
    ),
  }),
  invitations: router({
    invite: procedure
      .input(z.object({ teamId: id, email }).strict())
      .mutation(({ ctx, input }) =>
        attempt(() =>
          ctx.session.authenticated(async (token) =>
            invitation(
              await call(
                ctx.backend.general.inviteMember.bind(ctx.backend.general),
                input,
                token,
              ),
            ),
          ),
        ),
      ),
    received: procedure.query(({ ctx }) =>
      attempt(() =>
        ctx.session.authenticated(async (token) =>
          invitations(
            await call(
              ctx.backend.general.receivedInvitations.bind(ctx.backend.general),
              {},
              token,
            ),
          ),
        ),
      ),
    ),
    team: procedure
      .input(teamInput)
      .query(({ ctx, input }) =>
        attempt(() =>
          ctx.session.authenticated(async (token) =>
            invitations(
              await call(
                ctx.backend.general.teamInvitations.bind(ctx.backend.general),
                input,
                token,
              ),
            ),
          ),
        ),
      ),
    process: procedure
      .input(
        z
          .object({
            invitationId: id,
            action: z.union([z.literal(1), z.literal(2)]),
          })
          .strict(),
      )
      .mutation(({ ctx, input }) =>
        attempt(() =>
          ctx.session.authenticated(async (token) => {
            await call(
              ctx.backend.general.processInvitation.bind(ctx.backend.general),
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
