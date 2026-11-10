import {
  AuthenticationClient,
  PersonalPreferencesClient,
  AccountLifecycleClient,
  TeamManagementClient,
  InvitationManagementClient,
  InstallationClient,
  AccountGrantsClient,
  DisplayDefaultsClient,
  AdminWorkspacesClient,
} from "@chapchoo/rpc-client/accounts";
import { Metadata, credentials } from "@grpc/grpc-js";
import { AuthenticationsClient } from "@chapchoo/rpc-client/authentication";
import { ConversationsClient } from "@chapchoo/rpc-client/conversations";
import {
  AgentsClient,
  ComputersClient,
  ProviderCatalogClient,
  ProviderInventoryClient,
} from "@chapchoo/rpc-client/computers";
import type {
  CallOptions,
  ClientUnaryCall,
  requestCallback,
} from "@grpc/grpc-js";
import type { WebConfig } from "./config.js";
import { rpcFailure } from "./errors.js";

export interface BackendClients {
  conversations: ConversationsClient;
  agents: AgentsClient;
  computers: ComputersClient;
  providerCatalog: ProviderCatalogClient;
  providerInventory: ProviderInventoryClient;
  authentication: AuthenticationClient;
  authentications: AuthenticationsClient;
  personalPreferences: PersonalPreferencesClient;
  accountLifecycle: AccountLifecycleClient;
  teams: TeamManagementClient;
  invitations: InvitationManagementClient;
  installation: InstallationClient;
  accountGrants: AccountGrantsClient;
  displayDefaults: DisplayDefaultsClient;
  adminWorkspaces: AdminWorkspacesClient;
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
  const agents = new AgentsClient(endpoint.host, security, options);
  const conversations = new ConversationsClient(
    endpoint.host,
    security,
    options,
  );
  const authentications = new AuthenticationsClient(
    endpoint.host,
    security,
    options,
  );
  const computers = new ComputersClient(endpoint.host, security, options);
  const providerCatalog = new ProviderCatalogClient(
    endpoint.host,
    security,
    options,
  );
  const providerInventory = new ProviderInventoryClient(
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
  const adminWorkspaces = new AdminWorkspacesClient(
    endpoint.host,
    security,
    options,
  );
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
    conversations,
    agents,
    computers,
    providerCatalog,
    providerInventory,
    authentication,
    authentications,
    personalPreferences,
    accountLifecycle,
    teams,
    adminWorkspaces,
    invitations,
    installation,
    accountGrants,
    displayDefaults,
    close(): void {
      conversations.close();
      agents.close();
      computers.close();
      providerCatalog.close();
      providerInventory.close();
      authentication.close();
      authentications.close();
      personalPreferences.close();
      accountLifecycle.close();
      teams.close();
      adminWorkspaces.close();
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
