import {
  AdminClient,
  AuthClient,
  GeneralClient,
} from "@chapchoo/rpc-client/accounts";
import { Metadata, credentials } from "@grpc/grpc-js";
import type {
  CallOptions,
  ClientUnaryCall,
  requestCallback,
} from "@grpc/grpc-js";
import type { WebConfig } from "./config.js";
import { rpcFailure } from "./errors.js";

export interface BackendClients {
  auth: AuthClient;
  general: GeneralClient;
  admin: AdminClient;
  close(): void;
}

export function createBackend(config: WebConfig): BackendClients {
  const endpoint = new URL(config.backendUrl);
  const security =
    endpoint.protocol === "https:"
      ? credentials.createSsl()
      : credentials.createInsecure();
  const options = {
    "grpc.enable_retries": 0,
    "grpc.max_receive_message_length": 4 * 1024 * 1024,
  };
  const auth = new AuthClient(endpoint.host, security, options);
  const general = new GeneralClient(endpoint.host, security, options);
  const admin = new AdminClient(endpoint.host, security, options);
  return {
    auth,
    general,
    admin,
    close(): void {
      auth.close();
      general.close();
      admin.close();
    },
  };
}

export type UnaryCall<Input, Output> = (
  request: Input,
  metadata: Metadata,
  options: CallOptions,
  callback: requestCallback<Output>,
) => ClientUnaryCall;

export function call<Input, Output>(
  method: UnaryCall<Input, Output>,
  request: Input,
  token: string | null = null,
): Promise<Output> {
  return new Promise((resolve, reject) => {
    const metadata = new Metadata();
    if (token != null) {
      metadata.set("authorization", `Bearer ${token}`);
    }
    // Service-config retries are disabled on the channel. Each request has its
    // own finite deadline, fresh metadata and no automatic mutation replay.
    method(
      request,
      metadata,
      { deadline: Date.now() + 15_000 },
      (error, response) => {
        if (error != null) {
          reject(rpcFailure(error));
        } else if (response == null) {
          reject(rpcFailure(new Error("Missing RPC response")));
        } else {
          resolve(response);
        }
      },
    );
  });
}
