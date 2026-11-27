import "server-only";
import { attempt } from "@chapchoo/web-core/boundary";
import {
  integrationReceipt,
  mcpServers,
  mcpTestResult,
} from "@chapchoo/web-core/computer";
import { call } from "@chapchoo/web-core/rpc";
import { z } from "zod";
import { procedure, router } from "@/shared/server/trpc";

const id = z.string().min(1).max(128);
const auth = z.discriminatedUnion("type", [
  z.object({ type: z.literal("none") }).strict(),
  z.object({ type: z.literal("keep") }).strict(),
  z
    .object({
      type: z.literal("replace"),
      bearer: z.string().regex(/^[!-~]{1,8192}$/u),
    })
    .strict(),
]);
const connection = z
  .object({ endpoint: z.string().trim().min(1).max(4096), auth })
  .strict();
const draft = z
  .object({ displayName: z.string().trim().min(1).max(120), connection })
  .strict();
const change = z.discriminatedUnion("type", [
  z.object({ type: z.literal("create"), draft }).strict(),
  z.object({ type: z.literal("update"), integrationId: id, draft }).strict(),
  z.object({ type: z.literal("delete"), integrationId: id }).strict(),
]);

type Connection = { endpoint: string } & (
  { noAuth: object } | { keepBearer: object } | { replaceBearer: string }
);

function wireConnection(input: z.infer<typeof connection>): Connection {
  switch (input.auth.type) {
    case "none":
      return { endpoint: input.endpoint, noAuth: {} };
    case "keep":
      return { endpoint: input.endpoint, keepBearer: {} };
    case "replace":
      return { endpoint: input.endpoint, replaceBearer: input.auth.bearer };
  }
}

export const agentIntegrationsRouter = router({
  list: procedure
    .input(z.object({ agentId: id, after: id.optional() }).strict())
    .query(({ ctx, input }) =>
      attempt(() =>
        ctx.session.authenticated(async (token) =>
          mcpServers(
            await call(
              ctx.backend.agents.listIntegrations.bind(ctx.backend.agents),
              input,
              token,
            ),
          ),
        ),
      ),
    ),
  change: procedure
    .input(z.object({ agentId: id, requestId: id, change }).strict())
    .mutation(({ ctx, input }) =>
      attempt(() =>
        ctx.session.authenticated(async (token) => {
          const mutation = input.change;
          const operation =
            mutation.type === "delete"
              ? { deleteIntegrationId: mutation.integrationId }
              : mutation.type === "create"
                ? {
                    createMcp: {
                      displayName: mutation.draft.displayName,
                      connection: wireConnection(mutation.draft.connection),
                    },
                  }
                : {
                    updateMcp: {
                      integrationId: mutation.integrationId,
                      draft: {
                        displayName: mutation.draft.displayName,
                        connection: wireConnection(mutation.draft.connection),
                      },
                    },
                  };
          return integrationReceipt(
            await call(
              ctx.backend.agents.changeIntegration.bind(ctx.backend.agents),
              {
                agentId: input.agentId,
                requestId: input.requestId,
                ...operation,
              },
              token,
            ),
          );
        }),
      ),
    ),
  result: procedure
    .input(z.object({ agentId: id, requestId: id }).strict())
    .mutation(({ ctx, input }) =>
      attempt(() =>
        ctx.session.authenticated(async (token) => {
          const result = await call(
            ctx.backend.agents.integrationResult.bind(ctx.backend.agents),
            input,
            token,
          );
          return result.receipt ? integrationReceipt(result.receipt) : null;
        }),
      ),
    ),
  test: procedure
    .input(
      z
        .object({ agentId: id, integrationId: id.optional(), connection })
        .strict(),
    )
    .mutation(({ ctx, input }) =>
      attempt(() =>
        ctx.session.authenticated(async (token) =>
          mcpTestResult(
            await call(
              ctx.backend.agents.testMcpConnection.bind(ctx.backend.agents),
              {
                agentId: input.agentId,
                integrationId: input.integrationId,
                connection: wireConnection(input.connection),
              },
              token,
            ),
          ),
        ),
      ),
    ),
});
