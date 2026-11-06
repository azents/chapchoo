import { authenticationCopy } from "@chapchoo/web-core/authentication-messages";
import type { CatalogueSnapshot, ConnectionPanelProps } from "./types";
import type { AgentAuthenticationProps } from "./useAgentAuthentication";
import type { AgentView } from "@chapchoo/web-core/computer";
import type { AuthenticationView } from "@chapchoo/web-core/external-authentication";
export const fixtureConnection: AuthenticationView = {
  id: "chatgpt-fixture",
  scope: { type: "personal" },
  kind: "chatgpt",
  name: "건우의 ChatGPT",
  accountId: "synthetic-account",
  email: "gunwoo@example.com",
  planType: "plus",
  expiresAt: 1794000000000,
  currentRequestId: "registered",
  createdAt: 1793923200000,
  updatedAt: 1793923200000,
};
export const fixtureCatalogue = {
  ownerUserId: "fixture-user",
  scope: { type: "personal" },
  canManage: true,
  page: {
    items: [
      fixtureConnection,
      {
        ...fixtureConnection,
        id: "key-fixture",
        kind: "openaiApiKey",
        name: "개발용 API",
        accountId: null,
        email: null,
        planType: null,
        expiresAt: null,
      },
    ],
    nextAfter: null,
  },
  renderedAt: 1793923200000,
} satisfies CatalogueSnapshot;
export function fixturePanel(locale = "ko"): ConnectionPanelProps {
  return {
    display: { region: "ko-KR", timeZone: "Asia/Seoul" },
    copy: authenticationCopy(locale),
    mobile: false,
    scope: fixtureCatalogue.scope,
    scopes: [fixtureCatalogue.scope],
    selectedId: null,
    selectable: false,
    canManage: true,
    items: fixtureCatalogue.page.items,
    nextAfter: null,
    after: null,
    unavailable: false,
    loading: false,
    dialog: { type: "closed" },
    name: "",
    apiKey: "",
    busy: false,
    error: null,
    remainingSeconds: 240,
    onScope: (): void => {},
    onName: (): void => {},
    onKey: (): void => {},
    onPage: (): void => {},
    onReload: (): void => {},
    onSelect: (): void => {},
    onManage: (): void => {},
    onDialog: (): void => {},
    onClose: (): void => {},
    onSubmit: (): void => {},
    onDeviceCheck: (): void => {},
    onDeviceStatus: (): void => {},
    onDeviceCancel: (): void => {},
    onRecover: (): void => {},
    onDeviceRestart: (): void => {},
  };
}
export function fixtureAgentAuthentication(
  locale = "ko",
): AgentAuthenticationProps {
  const agent: AgentView = {
    id: "agent-fixture",
    teamId: "team-fixture",
    name: "두부",
    handle: "dubu",
    profileImage: { id: "retriever", assetKey: "01-retriever" },
    paletteKey: "butter",
    authentication: {
      authenticationId: fixtureConnection.id,
      name: fixtureConnection.name,
      kind: fixtureConnection.kind,
      requestId: "selection",
      selectedAt: 1793923200000,
    },
    computer: {
      operation: "create",
      intent: "running",
      requestId: "created",
      power: "running",
      lastKnownPower: "running",
      phase: "completed",
      management: "connected",
      connector: "connected",
      saved: "verified",
      observedAt: 1793923200000,
      actions: [],
      forceRemoveAvailable: false,
      startDelayed: false,
      preparationRetryAvailable: false,
    },
  };
  return {
    basePath: "",
    agent,
    snapshots: [fixtureCatalogue],
    unavailable: false,
    authorized: true,
    panel: {
      ...fixturePanel(locale),
      selectable: true,
      selectedId: fixtureConnection.id,
    },
    opened: false,
    task: { type: "ready" },
    onOpen: (): void => {},
    onClose: (): void => {},
    onRecover: (): void => {},
  };
}
