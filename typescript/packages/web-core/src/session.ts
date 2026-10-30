import type { AuthSession, User } from "@chapchoo/rpc-client/accounts";
import type { CookiePolicy } from "./cookies.js";
import { clearSessionCookie, sessionCookie } from "./cookies.js";
import {
  invalidResponse,
  invalidSession,
  permissionDenied,
  SafeRenderFailure,
  reportRpcFailure,
  rpcFailure,
} from "./errors.js";
import {
  session as validateSession,
  timestampMilliseconds,
  user as validateUser,
} from "./validation.js";

export interface SessionTransport {
  renew(token: string): Promise<AuthSession>;
  identity(token: string): Promise<User>;
}

interface Authenticated {
  token: string;
  user: User;
}

/** Request-owned credentials and outgoing cookie disposition are never DTOs. */
export class RequestSession {
  #presented: string | null;
  #renewal: Promise<Authenticated> | null = null;
  #cookie: string | null = null;
  #invalidated = false;
  #clock: () => number;
  constructor(
    private readonly transport: SessionTransport,
    token: string | null,
    private readonly policy: CookiePolicy,
    private readonly admin: boolean,
    clock: () => number = Date.now,
  ) {
    this.#presented = token;
    this.#clock = clock;
  }

  async #renew(): Promise<Authenticated> {
    if (this.#presented == null) {
      this.clear();
      throw invalidSession();
    }
    try {
      const value = validateSession(
        await this.transport.renew(this.#presented),
      );
      if (this.admin && !value.user.isAdmin) {
        this.clear();
        throw permissionDenied();
      }
      this.#publish(value);
      return { token: value.token, user: value.user };
    } catch (error) {
      const failure = rpcFailure(error);
      if (failure.reason === "DOMAIN_REASON_INVALID_SESSION") {
        this.clear();
      }
      throw failure;
    }
  }

  async authenticated<Output>(
    operation: (token: string) => Promise<Output>,
  ): Promise<Output> {
    this.#renewal ??= this.#renew();
    const authenticated = await this.#renewal;
    if (this.#invalidated) {
      throw invalidSession();
    }
    try {
      return await operation(authenticated.token);
    } catch (error) {
      const failure = rpcFailure(error);
      if (failure.reason === "DOMAIN_REASON_INVALID_SESSION") {
        this.clear();
      }
      throw failure;
    }
  }

  acceptAuthentication(value: AuthSession): User {
    const validated = validateSession(value);
    if (this.admin && !validated.user.isAdmin) {
      throw permissionDenied();
    }
    this.#publish(validated);
    return validated.user;
  }

  #publish(value: ReturnType<typeof validateSession>): void {
    const expiresAt = timestampMilliseconds(value.expiresAt);
    const now = this.#clock();
    if (expiresAt <= now) {
      return invalidResponse();
    }
    if (!this.#invalidated) {
      this.#cookie = sessionCookie(this.policy, value.token, expiresAt, now);
    }
  }

  clear(): void {
    this.#invalidated = true;
    this.#cookie = clearSessionCookie(this.policy);
  }
  outgoingCookie(): string | null {
    return this.#cookie;
  }
}

/** Read-only SSR authentication never rotates a credential or changes cookies. */
export async function readOnlyIdentity(
  transport: Pick<SessionTransport, "identity">,
  token: string | null,
  admin: boolean,
): Promise<User | null> {
  if (token == null) {
    return null;
  }
  try {
    const user = validateUser(await transport.identity(token));
    if (admin && !user.isAdmin) {
      throw permissionDenied();
    }
    return user;
  } catch (error) {
    const failure = rpcFailure(error);
    if (failure.reason === "DOMAIN_REASON_INVALID_SESSION") {
      return null;
    }
    reportRpcFailure(failure, "web_ssr_authentication_failure");
    throw new SafeRenderFailure(failure);
  }
}
