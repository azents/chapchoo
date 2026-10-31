import { displayPart } from "@chapchoo/web-core/display";
import { z } from "zod";
export const displayInput = z
  .object({
    language: z
      .string()
      .refine((value) => ["en", "ko", "ja", "fr"].includes(value)),
    timeZone: z
      .string()
      .min(1)
      .max(128)
      .refine((value) => displayPart("timeZone", value)),
    region: z
      .string()
      .refine((value) => ["en-US", "ko-KR", "ja-JP", "fr-FR"].includes(value)),
  })
  .strict();
export const displayPatch = displayInput
  .partial()
  .refine((value) => Object.keys(value).length > 0);
