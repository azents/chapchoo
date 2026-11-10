import "server-only";
import { attempt } from "@chapchoo/web-core/boundary";
import { displayPatchPayload } from "@chapchoo/web-core/display";
import { validHandle } from "@chapchoo/web-core/handle";
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
import { authenticationsRouter } from "@/shared/server/authentications";
import { computersRouter } from "@/shared/server/computers";
import { conversationsRouter } from "@/shared/server/conversations";
import { displayInput } from "@/shared/server/displayInput";
import { procedure, router } from "@/shared/server/trpc";

const email = z.string().min(1).max(320);
const proof = z.object({ proof: z.string().min(1).max(256) }).strict();
const password = z.string().min(1).max(64);
const id = z.string().min(1).max(128);
const teamInput = z.object({ teamId: id }).strict();
const memberInput = z.object({ teamId: id, userId: id }).strict();
const name = z.string().min(1).max(120);
const handle = z.string().min(3).max(16).refine(validHandle);

export const appRouter = router({
  conversations: conversationsRouter,
  computers: computersRouter,
  authentications: authenticationsRouter,
  auth: authRouter,
  signup: router({
    preview: procedure
      .input(proof)
      .query(({ ctx, input }) =>
        attempt(async () =>
          preview(
            await call(
              ctx.backend.accountLifecycle.previewSignup.bind(
                ctx.backend.accountLifecycle,
              ),
              input,
            ),
          ),
        ),
      ),
    redeem: procedure
      .input(
        z
          .object({
            proof: z.string().min(1).max(256),
            email,
            password,
            display: displayInput.partial().optional(),
          })
          .strict(),
      )
      .mutation(({ ctx, input }) =>
        attempt(async () =>
          ctx.session.acceptAuthentication(
            await call(
              ctx.backend.accountLifecycle.redeemSignup.bind(
                ctx.backend.accountLifecycle,
              ),
              {
                ...input,
                display: input.display
                  ? displayPatchPayload(input.display)
                  : void 0,
              },
            ),
          ),
        ),
      ),
    requestEmail: procedure
      .input(z.object({ email }).strict())
      .mutation(({ ctx, input }) =>
        attempt(() =>
          call(
            ctx.backend.accountLifecycle.requestSignupEmail.bind(
              ctx.backend.accountLifecycle,
            ),
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
              ctx.backend.accountLifecycle.previewPasswordReset.bind(
                ctx.backend.accountLifecycle,
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
            ctx.backend.accountLifecycle.redeemPasswordReset.bind(
              ctx.backend.accountLifecycle,
            ),
            input,
          );
          ctx.session.clear();
          return { completed: true };
        }),
      ),
  }),
  teams: router({
    handleAvailable: procedure
      .input(z.object({ handle }).strict())
      .query(({ ctx, input }) =>
        attempt(() =>
          ctx.session.authenticated(
            async (token) =>
              (
                await call(
                  ctx.backend.teams.checkHandle.bind(ctx.backend.teams),
                  input,
                  token,
                )
              ).available,
          ),
        ),
      ),
    list: procedure.query(({ ctx }) =>
      attempt(() =>
        ctx.session.authenticated(async (token) =>
          teams(
            await call(
              ctx.backend.teams.listTeams.bind(ctx.backend.teams),
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
                ctx.backend.teams.getTeam.bind(ctx.backend.teams),
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
                ctx.backend.teams.createTeam.bind(ctx.backend.teams),
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
          })
          .strict(),
      )
      .mutation(({ ctx, input }) =>
        attempt(() =>
          ctx.session.authenticated(async (token) =>
            team(
              await call(
                ctx.backend.teams.updateTeam.bind(ctx.backend.teams),
                {
                  teamId: input.teamId,
                  name: input.name,
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
                ctx.backend.teams.listMembers.bind(ctx.backend.teams),
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
              ctx.backend.teams.setMemberRole.bind(ctx.backend.teams),
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
            ctx.backend.teams.removeMember.bind(ctx.backend.teams),
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
            ctx.backend.teams.transferOwnership.bind(ctx.backend.teams),
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
            ctx.backend.teams.leaveTeam.bind(ctx.backend.teams),
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
            ctx.backend.teams.deleteTeam.bind(ctx.backend.teams),
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
                ctx.backend.invitations.inviteMember.bind(
                  ctx.backend.invitations,
                ),
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
              ctx.backend.invitations.receivedInvitations.bind(
                ctx.backend.invitations,
              ),
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
                ctx.backend.invitations.teamInvitations.bind(
                  ctx.backend.invitations,
                ),
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
              ctx.backend.invitations.processInvitation.bind(
                ctx.backend.invitations,
              ),
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
