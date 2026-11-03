import { z } from "zod";

const kubernetesName = z.string().regex(/^[a-z0-9](?:[a-z0-9.-]*[a-z0-9])?$/);
const resourceCursor = z.union([
  z
    .object({
      kind: z.enum(["POD", "BOOTSTRAP_SECRET", "PVC"]),
      namespace: kubernetesName.max(63),
      name: kubernetesName.max(253),
      nativeScopeKind: z.literal("").default(""),
      nativeScope: z.literal("").default(""),
    })
    .strict(),
  z
    .object({
      kind: z.enum(["DOCKER_CONTAINER", "DOCKER_VOLUME"]),
      namespace: z.literal("").default(""),
      name: z
        .string()
        .regex(/^[a-zA-Z0-9][a-zA-Z0-9_.-]*$/)
        .max(255),
      nativeScopeKind: z.literal("DOCKER_DAEMON"),
      nativeScope: z
        .string()
        .min(1)
        .max(1024)
        .refine(
          (value) =>
            new TextEncoder().encode(value).length <= 1024 &&
            !/[\s\p{Cc}]/u.test(value),
        ),
    })
    .strict(),
]);
export const resourceInput = z
  .object({
    computerId: z.string().min(1).max(128),
    after: resourceCursor.optional(),
  })
  .strict();
export type ResourceSelection = z.infer<typeof resourceInput>;

/** Browser URL decoration never becomes arbitrary strict RPC input. */
export function resourceSelection(
  computerId: string,
  search: Readonly<Partial<Record<string, string | string[]>>>,
): ResourceSelection {
  const cursor = resourceCursor.safeParse({
    kind: search.kind,
    namespace: search.namespace,
    name: search.name,
    nativeScopeKind: search.nativeScopeKind,
    nativeScope: search.nativeScope,
  });
  return resourceInput.parse({
    computerId,
    ...(cursor.success ? { after: cursor.data } : {}),
  });
}
