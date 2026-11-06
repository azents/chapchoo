import "server-only";
import { Buffer } from "node:buffer";
import { attempt } from "@chapchoo/web-core/boundary";
import {
  authentication,
  authenticationDeletion,
  authenticationHistory,
  authenticationList,
  authenticationRecordResult,
  authenticationSelectionResult,
  deviceProgress,
  wireAuthenticationScope,
} from "@chapchoo/web-core/external-authentication";
import { call } from "@chapchoo/web-core/rpc";
import { z } from "zod";
import { procedure, router } from "@/shared/server/trpc";

const id = z.string().min(1).max(128);
const scope = z.discriminatedUnion("type", [
  z.object({ type: z.literal("personal") }).strict(),
  z.object({ type: z.literal("team"), teamId: id }).strict(),
]);
const scoped = z.object({ scope }).strict();
const record = scoped.extend({ authenticationId: id });
const change = record.extend({ requestId: id, expectedRequestId: id });
const name = z
  .string()
  .trim()
  .refine(
    (value) =>
      [...value].length <= 256 && Buffer.byteLength(value, "utf8") <= 1024,
  );
const key = z
  .string()
  .min(1)
  .refine((value) => Buffer.byteLength(value, "utf8") <= 65536);
const device = z.object({ attemptId: id }).strict();

export const authenticationsRouter = router({
  list: procedure
    .input(scoped.extend({ after: id.optional() }))
    .query(({ ctx, input }) =>
      attempt(() =>
        ctx.session.authenticated(async (token) =>
          authenticationList(
            await call(
              ctx.backend.authentications.list.bind(
                ctx.backend.authentications,
              ),
              {
                scope: wireAuthenticationScope(input.scope),
                after: input.after,
                limit: 20,
              },
              token,
            ),
          ),
        ),
      ),
    ),
  get: procedure
    .input(record)
    .query(({ ctx, input }) =>
      attempt(() =>
        ctx.session.authenticated(async (token) =>
          authentication(
            await call(
              ctx.backend.authentications.get.bind(ctx.backend.authentications),
              { ...input, scope: wireAuthenticationScope(input.scope) },
              token,
            ),
          ),
        ),
      ),
    ),
  registerKey: procedure
    .input(scoped.extend({ requestId: id, name, key }))
    .mutation(({ ctx, input }) =>
      attempt(() =>
        ctx.session.authenticated(async (token) => {
          const bytes = Buffer.from(input.key, "utf8");
          try {
            return authenticationRecordResult(
              await call(
                ctx.backend.authentications.registerKey.bind(
                  ctx.backend.authentications,
                ),
                {
                  scope: wireAuthenticationScope(input.scope),
                  requestId: input.requestId,
                  name: input.name,
                  key: bytes,
                },
                token,
              ),
            );
          } finally {
            bytes.fill(0);
          }
        }),
      ),
    ),
  replaceKey: procedure
    .input(change.extend({ key }))
    .mutation(({ ctx, input }) =>
      attempt(() =>
        ctx.session.authenticated(async (token) => {
          const bytes = Buffer.from(input.key, "utf8");
          try {
            return authenticationRecordResult(
              await call(
                ctx.backend.authentications.replaceKey.bind(
                  ctx.backend.authentications,
                ),
                {
                  scope: wireAuthenticationScope(input.scope),
                  requestId: input.requestId,
                  authenticationId: input.authenticationId,
                  expectedRequestId: input.expectedRequestId,
                  key: bytes,
                },
                token,
              ),
            );
          } finally {
            bytes.fill(0);
          }
        }),
      ),
    ),
  rename: procedure
    .input(change.extend({ name }))
    .mutation(({ ctx, input }) =>
      attempt(() =>
        ctx.session.authenticated(async (token) =>
          authenticationRecordResult(
            await call(
              ctx.backend.authentications.rename.bind(
                ctx.backend.authentications,
              ),
              { ...input, scope: wireAuthenticationScope(input.scope) },
              token,
            ),
          ),
        ),
      ),
    ),
  delete: procedure
    .input(change)
    .mutation(({ ctx, input }) =>
      attempt(() =>
        ctx.session.authenticated(async (token) =>
          authenticationDeletion(
            await call(
              ctx.backend.authentications.delete.bind(
                ctx.backend.authentications,
              ),
              { ...input, scope: wireAuthenticationScope(input.scope) },
              token,
            ),
          ),
        ),
      ),
    ),
  requestResult: procedure
    .input(z.object({ requestId: id }).strict())
    .query(({ ctx, input }) =>
      attempt(() =>
        ctx.session.authenticated(async (token) =>
          authenticationHistory(
            await call(
              ctx.backend.authentications.requestResult.bind(
                ctx.backend.authentications,
              ),
              input,
              token,
            ),
          ),
        ),
      ),
    ),
  startDevice: procedure
    .input(
      scoped.extend({
        requestId: id,
        name,
        target: z
          .object({ authenticationId: id, expectedRequestId: id })
          .strict()
          .optional(),
      }),
    )
    .mutation(({ ctx, input }) =>
      attempt(() =>
        ctx.session.authenticated(async (token) =>
          deviceProgress(
            await call(
              ctx.backend.authentications.startDevice.bind(
                ctx.backend.authentications,
              ),
              {
                ...input,
                scope: wireAuthenticationScope(input.scope),
                target: input.target,
              },
              token,
            ),
          ),
        ),
      ),
    ),
  deviceStatus: procedure
    .input(device)
    .query(({ ctx, input }) =>
      attempt(() =>
        ctx.session.authenticated(async (token) =>
          deviceProgress(
            await call(
              ctx.backend.authentications.deviceStatus.bind(
                ctx.backend.authentications,
              ),
              input,
              token,
            ),
          ),
        ),
      ),
    ),
  pollDevice: procedure
    .input(device)
    .mutation(({ ctx, input }) =>
      attempt(() =>
        ctx.session.authenticated(async (token) =>
          deviceProgress(
            await call(
              ctx.backend.authentications.pollDevice.bind(
                ctx.backend.authentications,
              ),
              input,
              token,
            ),
          ),
        ),
      ),
    ),
  cancelDevice: procedure
    .input(device.extend({ requestId: id }))
    .mutation(({ ctx, input }) =>
      attempt(() =>
        ctx.session.authenticated(async (token) =>
          deviceProgress(
            await call(
              ctx.backend.authentications.cancelDevice.bind(
                ctx.backend.authentications,
              ),
              input,
              token,
            ),
          ),
        ),
      ),
    ),
  selectAgent: procedure
    .input(
      z
        .object({
          requestId: id,
          agentId: id,
          authenticationId: id,
          expected: z
            .object({ authenticationId: id, requestId: id })
            .strict()
            .optional(),
        })
        .strict(),
    )
    .mutation(({ ctx, input }) =>
      attempt(() =>
        ctx.session.authenticated(async (token) =>
          authenticationSelectionResult(
            await call(
              ctx.backend.authentications.selectAgent.bind(
                ctx.backend.authentications,
              ),
              { ...input, expected: input.expected },
              token,
            ),
          ),
        ),
      ),
    ),
});
