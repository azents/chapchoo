import "server-only";
import { attempt } from "@chapchoo/web-core/boundary";
import {
  answerDelivery,
  conversationApi,
  inputObservation,
  readConversationActivity,
  readConversationRegion,
  stopObservation,
  submitConversationAnswer,
  submitConversationInput,
} from "@chapchoo/web-core/conversation";
import { conversationMessageWindow } from "@chapchoo/web-core/conversation-message-types";
import {
  conversationMessageApi,
  readConversationMessages,
} from "@chapchoo/web-core/conversation-messages";
import {
  readHarnessSupply,
  saveHarnessSupply,
} from "@chapchoo/web-core/conversation-supply-rpc";
import {
  conversationAnswer,
  conversationReadAnchor,
  conversationReadInputAnchor,
  conversationReadInteger,
  conversationSource,
  harnessSupplyChoice,
  nativeTarget,
} from "@chapchoo/web-core/conversation-types";
import { z } from "zod";
import { procedure, router } from "@/shared/server/trpc";

const id = z.string().min(1).max(128);
const operation = z
  .object({ agentId: id, conversationId: id, requestId: id })
  .strict();
export const conversationsRouter = router({
  messages: procedure
    .input(
      z
        .object({
          agentId: id,
          expectedConversationId: id.nullable().optional(),
          window: conversationMessageWindow,
        })
        .strict(),
    )
    .query(({ ctx, input }) =>
      attempt(() =>
        ctx.session.authenticated((token) =>
          readConversationMessages(
            conversationMessageApi(ctx.backend.conversations, token),
            input,
          ),
        ),
      ),
    ),
  activity: procedure
    .input(
      z
        .object({
          agentId: id,
          expectedSnapshotId: z.string().max(512).optional(),
          expectedConversationId: id.optional(),
          inputBefore: conversationReadInputAnchor.optional(),
          knownInputs: z.array(id).max(24).optional(),
          checkInputs: z.array(id).max(25).optional(),
          stop: z
            .object({
              requestId: id,
              conversationId: id,
              snapshotId: z.string().min(1).max(512),
            })
            .strict()
            .optional(),
        })
        .strict(),
    )
    .query(({ ctx, input }) =>
      attempt(() =>
        ctx.session.authenticated((token) =>
          readConversationActivity(
            conversationApi(ctx.backend.conversations, token),
            input,
          ),
        ),
      ),
    ),
  supply: procedure
    .input(z.object({ agentId: id }).strict())
    .query(({ ctx, input }) =>
      attempt(() =>
        ctx.session.authenticated((token) =>
          readHarnessSupply(ctx.backend.conversations, token, input.agentId),
        ),
      ),
    ),
  saveSupply: procedure
    .input(
      z
        .object({
          agentId: id,
          requestId: id,
          expectedRequestId: id.optional(),
          choice: harnessSupplyChoice,
        })
        .strict(),
    )
    .mutation(({ ctx, input }) =>
      attempt(() =>
        ctx.session.authenticated((token) =>
          saveHarnessSupply(ctx.backend.conversations, token, input),
        ),
      ),
    ),
  region: procedure
    .input(
      z
        .object({
          agentId: id,
          before: conversationReadAnchor.optional(),
          expectedSnapshotId: z.string().max(512).optional(),
          expectedConversationId: id.optional(),
          known: z
            .array(
              z
                .object({
                  key: z.string().max(1024),
                  updatedOrdinal: conversationReadInteger.pipe(
                    z.bigint().nonnegative(),
                  ),
                })
                .strict(),
            )
            .max(24)
            .optional(),
          inputBefore: conversationReadInputAnchor.optional(),
          knownInputs: z.array(id).max(24).optional(),
          checkInputs: z.array(id).max(25).optional(),
          stop: z
            .object({
              requestId: id,
              conversationId: id,
              snapshotId: z.string().min(1).max(512),
            })
            .strict()
            .optional(),
        })
        .strict(),
    )
    .query(({ ctx, input }) =>
      attempt(() =>
        ctx.session.authenticated((token) =>
          readConversationRegion(
            conversationApi(ctx.backend.conversations, token),
            input,
          ),
        ),
      ),
    ),
  send: procedure
    .input(
      z
        .object({
          agentId: id,
          conversationId: id.optional(),
          snapshotId: z.string().max(512).optional(),
          requestId: id,
          text: z
            .string()
            .refine((text) => Boolean(text.trim()) && !text.includes("\0")),
        })
        .strict(),
    )
    .mutation(({ ctx, input }) =>
      attempt(() =>
        ctx.session.authenticated((token) =>
          submitConversationInput(
            conversationApi(ctx.backend.conversations, token),
            input,
          ),
        ),
      ),
    ),
  deleteInput: procedure
    .input(
      z
        .object({
          source: z
            .object({
              agentId: id,
              conversationId: id,
              snapshotId: z.string().min(1).max(512).optional(),
            })
            .strict(),
          requestId: id,
        })
        .strict(),
    )
    .mutation(({ ctx, input }) =>
      attempt(() =>
        ctx.session.authenticated((token) =>
          conversationApi(ctx.backend.conversations, token).deleteInput({
            scope: {
              agentId: input.source.agentId,
              conversationId: input.source.conversationId,
              target: input.source.snapshotId
                ? { snapshotId: input.source.snapshotId }
                : { notStarted: {} },
            },
            requestId: input.requestId,
          }),
        ),
      ),
    ),
  retryInput: procedure
    .input(
      z
        .object({
          source: z
            .object({
              agentId: id,
              conversationId: id,
              snapshotId: z.string().min(1).max(512).optional(),
            })
            .strict(),
          originalInputId: id,
          originalRequestId: id,
          requestId: id,
        })
        .strict(),
    )
    .mutation(({ ctx, input }) =>
      attempt(() =>
        ctx.session.authenticated((token) =>
          conversationApi(ctx.backend.conversations, token).retryInput({
            scope: {
              agentId: input.source.agentId,
              conversationId: input.source.conversationId,
              target: input.source.snapshotId
                ? { snapshotId: input.source.snapshotId }
                : { notStarted: {} },
            },
            originalInputId: input.originalInputId,
            originalRequestId: input.originalRequestId,
            requestId: input.requestId,
          }),
        ),
      ),
    ),
  inputResult: procedure.input(operation).query(({ ctx, input }) =>
    attempt(() =>
      ctx.session.authenticated(async (token) => {
        const result = await conversationApi(
          ctx.backend.conversations,
          token,
        ).inputResult(input);
        return {
          observation: inputObservation(result),
          deleted: result.deleted,
          receipt: result.acceptance
            ? {
                id: result.acceptance.id,
                requestId: result.acceptance.requestId,
                conversationId: result.acceptance.conversationId,
              }
            : void 0,
        };
      }),
    ),
  ),
  answer: procedure
    .input(
      z
        .object({
          source: conversationSource,
          target: nativeTarget,
          responderToken: id,
          requestId: id,
          answer: conversationAnswer,
        })
        .strict(),
    )
    .mutation(({ ctx, input }) =>
      attempt(() =>
        ctx.session.authenticated((token) =>
          submitConversationAnswer(
            conversationApi(ctx.backend.conversations, token),
            input,
          ),
        ),
      ),
    ),
  answerResult: procedure.input(operation).query(({ ctx, input }) =>
    attempt(() =>
      ctx.session.authenticated(async (token) => {
        const result = await conversationApi(
          ctx.backend.conversations,
          token,
        ).answerResult(input);
        return {
          delivery: result.acceptance
            ? answerDelivery(result.acceptance.delivery)
            : ("uncertain" as const),
        };
      }),
    ),
  ),
  stop: procedure
    .input(
      operation.extend({
        target: z.object({ source: nativeTarget, workId: id }).strict(),
      }),
    )
    .mutation(({ ctx, input }) =>
      attempt(() =>
        ctx.session.authenticated(async (token) => ({
          observation: stopObservation(
            await conversationApi(ctx.backend.conversations, token).stop(input),
          ),
        })),
      ),
    ),
  stopResult: procedure.input(operation).query(({ ctx, input }) =>
    attempt(() =>
      ctx.session.authenticated(async (token) => ({
        observation: stopObservation(
          (
            await conversationApi(ctx.backend.conversations, token).stopResult(
              input,
            )
          ).receipt,
        ),
      })),
    ),
  ),
});
