export interface NativeEvidence {
  daemon: string;
  engine: string;
  api: string;
  architecture: "amd64" | "arm64";
  image: string;
  original: string;
  successor: string;
  restarted: string;
  volume: string;
  cleanup: "confirmed";
  canaries: "unchanged";
  source: "explicit-unix";
  network: "bridge";
}
function object(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}
function token(value: unknown, limit: number): value is string {
  return (
    typeof value === "string" &&
    value.length > 0 &&
    value.length <= limit &&
    /^[a-zA-Z0-9_.:+-]+$/.test(value)
  );
}
/** Decode a strict, credential-free evidence shape; never forward raw annotations. */
export function nativeEvidence(description: string): NativeEvidence | null {
  let value: unknown;
  try {
    value = JSON.parse(description);
  } catch {
    return null;
  }
  if (
    !object(value) ||
    !token(value.daemon, 1024) ||
    !token(value.engine, 128) ||
    !token(value.api, 32) ||
    !token(value.volume, 255) ||
    (value.architecture !== "amd64" && value.architecture !== "arm64") ||
    typeof value.image !== "string" ||
    !/^sha256:[a-f0-9]{64}$/.test(value.image) ||
    typeof value.original !== "string" ||
    !/^[a-f0-9]{64}$/.test(value.original) ||
    typeof value.successor !== "string" ||
    !/^[a-f0-9]{64}$/.test(value.successor) ||
    typeof value.restarted !== "string" ||
    !/^[a-f0-9]{64}$/.test(value.restarted) ||
    value.cleanup !== "confirmed" ||
    value.canaries !== "unchanged" ||
    value.source !== "explicit-unix" ||
    value.network !== "bridge"
  ) {
    return null;
  }
  return {
    daemon: value.daemon,
    engine: value.engine,
    api: value.api,
    architecture: value.architecture,
    image: value.image,
    original: value.original,
    successor: value.successor,
    restarted: value.restarted,
    volume: value.volume,
    cleanup: value.cleanup,
    canaries: value.canaries,
    source: value.source,
    network: value.network,
  };
}
