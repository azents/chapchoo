export interface UrlCodec<T> {
  parse(value: string | null): T;
  format(value: T): string | null;
}
export interface SearchValues {
  get(name: string): string | null;
}
/** Match URLSearchParams.get: duplicate keys use the first value on both sides. */
export function serverSearchValues(
  params: Record<string, string | string[] | undefined>,
): SearchValues {
  return {
    get(name) {
      const value = params[name];
      return Array.isArray(value) ? (value[0] ?? null) : (value ?? null);
    },
  };
}
export const inventoryOffset: UrlCodec<bigint> = {
  parse(value) {
    if (value === null || !/^(0|[1-9][0-9]{0,18})$/.test(value)) return 0n;
    const offset = BigInt(value);
    return offset <= 9223372036854775807n ? offset : 0n;
  },
  format(value) {
    return value === 0n ? null : value.toString();
  },
};
export const loginMethod: UrlCodec<"password" | "code"> = {
  parse(value) {
    return value === "code" ? "code" : "password";
  },
  format(value) {
    return value === "password" ? null : value;
  },
};
export function readSelection<T>(
  params: SearchValues,
  key: string,
  codec: UrlCodec<T>,
): T {
  return codec.parse(params.get(key));
}
export function selectionUrl<T>(
  href: string,
  key: string,
  value: T,
  codec: UrlCodec<T>,
): string {
  const url = new URL(href, "https://page.invalid");
  const serialized = codec.format(value);
  if (serialized === null) url.searchParams.delete(key);
  else url.searchParams.set(key, serialized);
  return `${url.pathname}${url.search}${url.hash}`;
}
/** Next's supported native history integration updates useSearchParams subscribers. */
export function navigateSelection<T>(
  key: string,
  value: T,
  codec: UrlCodec<T>,
  history: "push" | "replace",
): void {
  const next = selectionUrl(window.location.href, key, value, codec);
  if (
    next ===
    `${window.location.pathname}${window.location.search}${window.location.hash}`
  )
    return;
  if (history === "push") window.history.pushState(null, "", next);
  else window.history.replaceState(null, "", next);
}
/** Hydrate the matching query without a redundant first/reconnect read. */
export function initialQuery<T>(
  data: T,
  updatedAt?: number,
): {
  initialData: T;
  initialDataUpdatedAt?: number;
  refetchOnMount: false;
  refetchOnReconnect: false;
} {
  return {
    initialData: data,
    ...(updatedAt === undefined ? {} : { initialDataUpdatedAt: updatedAt }),
    refetchOnMount: false,
    refetchOnReconnect: false,
  };
}
