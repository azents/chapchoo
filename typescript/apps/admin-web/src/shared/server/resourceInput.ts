import { z } from "zod";

const resourceCursor = z
  .object({
    kind: z.enum(["POD", "BOOTSTRAP_SECRET", "PVC"]),
    namespace: z.string().min(1).max(253),
    name: z.string().min(1).max(253),
  })
  .strict();
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
  });
  return resourceInput.parse({
    computerId,
    ...(cursor.success ? { after: cursor.data } : {}),
  });
}
