import {
  AdminClient,
  AuthClient,
  GeneralClient,
} from "@chapchoo/rpc-client/accounts";
import { credentials, Metadata } from "@grpc/grpc-js";
import type {
  CallOptions,
  ClientUnaryCall,
  requestCallback,
} from "@grpc/grpc-js";
import type { FixtureUser } from "./safety.js";

type Unary<I, O> = (
  input: I,
  metadata: Metadata,
  options: CallOptions,
  callback: requestCallback<O>,
) => ClientUnaryCall;

function call<I, O>(
  method: Unary<I, O>,
  input: I,
  token: string | null = null,
): Promise<O> {
  return new Promise((resolve, reject) => {
    const metadata = new Metadata();
    if (token) {
      metadata.set("authorization", `Bearer ${token}`);
    }
    method(
      input,
      metadata,
      { deadline: Date.now() + 15_000 },
      (error, result) => {
        if (error || result == null) {
          reject(new Error("Secondary fixture RPC failed; response withheld"));
        } else {
          resolve(result);
        }
      },
    );
  });
}

/** Only the registered second user is seeded, after the browser bootstraps Admin. */
export async function seedRegisteredUser(
  generalEndpoint: string,
  adminEndpoint: string,
  administrator: FixtureUser,
  user: FixtureUser,
): Promise<void> {
  const options = { "grpc.enable_retries": 0 };
  const auth = new AuthClient(
    adminEndpoint,
    credentials.createInsecure(),
    options,
  );
  const admin = new AdminClient(
    adminEndpoint,
    credentials.createInsecure(),
    options,
  );
  const general = new GeneralClient(
    generalEndpoint,
    credentials.createInsecure(),
    options,
  );
  try {
    const session = await call(auth.passwordLogin.bind(auth), administrator);
    if (!session.token) {
      throw new Error("Secondary fixture administrator login failed");
    }
    const grant = await call(
      admin.issueSignupGrant.bind(admin),
      { email: user.email },
      session.token,
    );
    if (!grant.proof) {
      throw new Error("Secondary fixture signup grant was not issued");
    }
    await call(general.redeemSignup.bind(general), {
      proof: grant.proof,
      email: user.email,
      password: user.password,
      display: undefined,
    });
    await call(auth.logout.bind(auth), {}, session.token);
  } finally {
    auth.close();
    admin.close();
    general.close();
  }
}
