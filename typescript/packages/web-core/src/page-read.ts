import {
  invalidSession,
  reportRpcFailure,
  rpcFailure,
  SafeRenderFailure,
} from "./errors.js";
import { readOnlyIdentity } from "./session.js";
import type { User } from "./browser.js";

/** Request-local read scope: validates once, never renews or publishes cookies. */
export class PageReadScope {
  #identity: Promise<User | null> | undefined;
  #token: string | null;
  constructor(
    token: string | null,
    private readonly resolveIdentity: (token: string) => Promise<User>,
    private readonly admin: boolean,
  ) {
    this.#token = token;
  }
  identity(): Promise<User | null> {
    this.#identity ??= readOnlyIdentity(
      { identity: this.resolveIdentity },
      this.#token,
      this.admin,
    );
    return this.#identity;
  }
  async read<T>(operation: (token: string) => Promise<T>): Promise<T> {
    const token = this.#token;
    if (!(await this.identity()) || token === null) throw invalidSession();
    return requiredPageRead(() => operation(token));
  }
}

/** Throw sanitized failures to the route error boundary, not serialized causes. */
export async function requiredPageRead<T>(
  operation: () => Promise<T>,
): Promise<T> {
  try {
    return await operation();
  } catch (error) {
    if (error instanceof SafeRenderFailure) throw error;
    const failure = rpcFailure(error);
    reportRpcFailure(failure);
    throw new SafeRenderFailure(failure);
  }
}
