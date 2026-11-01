import {
  AuthenticationClient,
  PersonalPreferencesClient,
  AccountLifecycleClient,
  TeamManagementClient,
  InvitationManagementClient,
  InstallationClient,
  AccountGrantsClient,
  DisplayDefaultsClient,
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
  authentication: AuthenticationClient;
  personalPreferences: PersonalPreferencesClient;
  accountLifecycle: AccountLifecycleClient;
  teams: TeamManagementClient;
  invitations: InvitationManagementClient;
  installation: InstallationClient;
  accountGrants: AccountGrantsClient;
  displayDefaults: DisplayDefaultsClient;
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
  const authentication = new AuthenticationClient(
    endpoint.host,
    security,
    options,
  );
  const personalPreferences = new PersonalPreferencesClient(
    endpoint.host,
    security,
    options,
  );
  const accountLifecycle = new AccountLifecycleClient(
    endpoint.host,
    security,
    options,
  );
  const teams = new TeamManagementClient(endpoint.host, security, options);
  const invitations = new InvitationManagementClient(
    endpoint.host,
    security,
    options,
  );
  const installation = new InstallationClient(endpoint.host, security, options);
  const accountGrants = new AccountGrantsClient(
    endpoint.host,
    security,
    options,
  );
  const displayDefaults = new DisplayDefaultsClient(
    endpoint.host,
    security,
    options,
  );
  return {
    authentication,
    personalPreferences,
    accountLifecycle,
    teams,
    invitations,
    installation,
    accountGrants,
    displayDefaults,
    close(): void {
      authentication.close();
      personalPreferences.close();
      accountLifecycle.close();
      teams.close();
      invitations.close();
      installation.close();
      accountGrants.close();
      displayDefaults.close();
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
