import { z } from "zod";

// Query keys use JSON-compatible inputs. Keep the full backend int64 range
// without passing BigInt through TanStack Query's default JSON key hashing.
export const inventoryInput = z
  .object({
    offset: z
      .string()
      .regex(/^(0|[1-9][0-9]{0,18})$/)
      .transform((value) => BigInt(value))
      .pipe(z.bigint().max(9223372036854775807n))
      .optional(),
    limit: z
      .number()
      .int()
      .min(1)
      .max(1000)
      .transform((value) => BigInt(value))
      .optional(),
  })
  .strict();
