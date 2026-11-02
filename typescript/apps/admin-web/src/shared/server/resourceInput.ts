import { z } from "zod";
export const resourceInput = z
  .object({
    computerId: z.string().min(1).max(128),
    after: z
      .object({
        kind: z.enum(["POD", "BOOTSTRAP_SECRET", "PVC"]),
        namespace: z.string().min(1).max(253),
        name: z.string().min(1).max(253),
      })
      .strict()
      .optional(),
  })
  .strict();
export type ResourceSelection = z.infer<typeof resourceInput>;
