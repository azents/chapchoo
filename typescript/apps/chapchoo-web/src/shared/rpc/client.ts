"use client";
import { createTRPCReact } from "@trpc/react-query";
import type { AppRouter } from "@/shared/server/router";

export const trpc = createTRPCReact<AppRouter>();
