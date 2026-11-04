import { AccountSettings } from "@chapchoo/ui/AccountSettings";
import { HeaderControlsContext } from "@chapchoo/ui/AppHeader";
import { WorkspaceChrome } from "@chapchoo/ui/WorkspaceChrome";
import { computerCopy } from "@chapchoo/web-core/computer-messages";
import { shellCopy } from "@chapchoo/web-core/shell-messages";
import { useState } from "react";
import { creationCopy } from "../creationCopy";
import { Agents } from "./Agents";
import type { AgentsProps, InitialAgents } from "../types";
import type { AgentView } from "@chapchoo/web-core/computer";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import type { ReactElement } from "react";

const agent: AgentView = {
  id: "agent",
  teamId: "team",
  name: "Research companion",
  handle: "research",
  profileImage: { id: "image", assetKey: "03-corgi" },
  paletteKey: "sage",
  computer: {
    intent: "running",
    requestId: "request",
    power: "running",
    lastKnownPower: "running",
    phase: "completed",
    management: "connected",
    connector: "connected",
    saved: "verified",
    observedAt: 1793577600000,
    actions: [
      { action: "start", available: false, reason: "requiresStopped" },
      { action: "stop", available: true, reason: null },
      { action: "restart", available: true, reason: null },
      { action: "delete", available: true, reason: null },
    ],
    forceRemoveAvailable: false,
    startDelayed: false,
    preparationRetryAvailable: false,
  },
};
const initial: InitialAgents = {
  type: "list",
  teamId: "team",
  teamHandle: "my",
  page: { items: [agent], nextAfter: null },
  after: "",
  identity: "owner@example.invalid",
  renderedAt: 1793577600000,
};
const detail: InitialAgents = {
  type: "detail",
  teamId: agent.teamId,
  teamHandle: "my",
  agent,
  identity: initial.identity,
  renderedAt: initial.renderedAt,
};
const meta = {
  title: "Computers/Agents",
  component: Agents,
  args: {
    access: "authorized",
    initial,
    data: initial,
    basePath: "",
    copy: computerCopy("en-US"),
    profileCopy: creationCopy("en-US"),
    mobile: false,
    editor: { type: "closed" },
    images: { items: [], nextAfter: null },
    imageAfter: null,
    imagesUnavailable: false,
    onEditProfile: () => {},
    onProfileName: () => {},
    onProfileImage: () => {},
    onProfilePalette: () => {},
    onImagePage: () => {},
    onSaveProfile: () => {},
    onCancelProfile: () => {},
    stale: false,
    pending: false,
    error: null,
    confirmation: null,
    onAction: () => {},
    onConfirm: () => {},
    onCancel: () => {},
    onRefresh: () => {},
    onLogout: () => {},
    formatTime: () => "Nov 2, 2026, 9:00 AM",
  },
} satisfies Meta<typeof Agents>;
export default meta;
type Story = StoryObj<typeof meta>;
export const AgentList: Story = {};
export const ProfileCards: Story = {
  args: {
    data: {
      ...initial,
      type: "list",
      after: "",
      page: {
        items: [
          agent,
          {
            ...agent,
            id: "another-agent",
            name: "A long-running research companion with a descriptive display name",
            handle: "another-agent",
            paletteKey: "rose",
          },
        ],
        nextAfter: null,
      },
    },
  },
};
export const StaleList: Story = { args: { stale: true } };
export const SignedOut: Story = {
  args: { access: "signedOut", initial: detail, data: detail },
};
export const Empty: Story = {
  args: {
    data: {
      ...initial,
      type: "list",
      after: "",
      page: { items: [], nextAfter: null },
    },
  },
};
function FeedbackChrome({
  mobile,
  ...props
}: AgentsProps & { mobile: boolean }): ReactElement {
  const [drawer, setDrawer] = useState(false);
  const [picker, setPicker] = useState(false);
  const c = shellCopy("ko");
  const current = {
    key: "personal",
    kind: "personal" as const,
    label: c.yourSpace,
    href: "/teams/my/agents",
    selected: true,
  };
  return (
    <WorkspaceChrome
      identity
      basePath=""
      homeHref="/"
      mobile={mobile}
      drawer={drawer}
      picker={picker}
      creating={false}
      name=""
      handle=""
      prospectiveAddress={null}
      handleStatus={{
        message: null,
        description: "핸들은 팀을 만든 뒤 변경할 수 없습니다.",
        available: false,
      }}
      error={null}
      status="ready"
      busy={false}
      copy={c}
      choices={[
        current,
        {
          key: "research",
          kind: "team",
          handle: "research",
          label: "리서치 팀",
          href: "/teams/research/agents",
          selected: false,
        },
        {
          key: "admin",
          kind: "admin",
          label: c.administration,
          href: "/admin",
          selected: false,
        },
      ]}
      current={current}
      items={[
        {
          key: "agents",
          label: c.agents,
          href: "/teams/my/agents",
          active: true,
        },
        { key: "invitations", label: c.invitations, href: "/invitations" },
      ]}
      mascotReplayLabel="캐릭터 동작 다시 보기"
      openDrawer={() => setDrawer(true)}
      closeDrawer={() => {
        setDrawer(false);
        setPicker(false);
      }}
      openPicker={() => setPicker(true)}
      closePicker={() => setPicker(false)}
      openCreation={() => {
        setPicker(false);
      }}
      closeCreation={() => {}}
      onName={() => {}}
      onHandle={() => {}}
      onSubmit={() => {}}
      retry={() => {}}
      select={() => {
        setPicker(false);
      }}
    >
      <HeaderControlsContext.Provider
        value={
          <AccountSettings
            opened={false}
            mobile={mobile}
            onOpen={() => {}}
            onClose={() => {}}
            identityLabel={initial.identity}
            profileHref="/profile"
            profileLabel="프로필"
            loginHref="/login"
            loginLabel="로그인"
            logoutLabel="로그아웃"
            onLogout={() => {}}
            value={{ language: "ko", timeZone: "Asia/Seoul", region: "ko-KR" }}
            selected={{}}
            automatic={false}
            disabled={false}
            zones={["Asia/Seoul"]}
            copy={{
              title: "계정 및 설정",
              language: "언어",
              timeZone: "시간대",
              region: "지역 형식",
              follow: "자동",
              preview: "미리 보기",
              unavailable: "불러오지 못했습니다.",
              scope: "",
              done: "완료",
              close: "닫기",
              immediate: "",
              saved: "저장했습니다.",
              error: "저장하지 못했습니다.",
              storageError: "기기에 저장하지 못했습니다.",
              retry: "다시 시도",
            }}
            appearance={{
              value: "light",
              label: "화면 모드",
              labels: { system: "시스템", light: "라이트", dark: "다크" },
              onChange: () => {},
            }}
            changed={false}
            error={false}
            storageError={false}
            instant={initial.renderedAt}
            onChange={() => {}}
            onRetry={() => {}}
          />
        }
      >
        <Agents {...props} mobile={mobile} />
      </HeaderControlsContext.Provider>
    </WorkspaceChrome>
  );
}
export const FeedbackMobile: Story = {
  args: { ...Empty.args, copy: computerCopy("ko-KR") },
  render: (args): ReactElement => <FeedbackChrome {...args} mobile />,
};
export const FeedbackDesktop: Story = {
  args: { ...Empty.args, copy: computerCopy("ko-KR") },
  render: (args): ReactElement => <FeedbackChrome {...args} mobile={false} />,
};
export const FeedbackPopulated: Story = {
  args: { copy: computerCopy("ko-KR") },
  render: (args): ReactElement => <FeedbackChrome {...args} mobile />,
};
export const Running: Story = { args: { initial: detail, data: detail } };
export const StopConfirmation: Story = {
  args: { initial: detail, data: detail, confirmation: "stop" },
};
export const StaleUnknown: Story = {
  args: {
    initial: detail,
    stale: true,
    data: {
      ...detail,
      agent: {
        ...agent,
        computer: {
          ...agent.computer,
          power: "unknown",
          phase: "uncertain",
          management: "disconnected",
          connector: "disconnected",
        },
      },
    },
  },
};
export const DelayedStart: Story = {
  args: {
    initial: detail,
    data: {
      ...detail,
      agent: {
        ...agent,
        computer: {
          ...agent.computer,
          power: "unknown",
          phase: "starting",
          startDelayed: true,
        },
      },
    },
  },
};
export const ForceRemoval: Story = {
  args: {
    copy: computerCopy("ko-KR"),
    initial: detail,
    confirmation: "force",
    data: {
      ...detail,
      agent: {
        ...agent,
        computer: {
          ...agent.computer,
          intent: "deleting",
          phase: "deleting",
          forceRemoveAvailable: true,
        },
      },
    },
  },
};
export const RetryPreparation: Story = {
  args: {
    initial: detail,
    data: {
      ...detail,
      agent: {
        ...agent,
        computer: {
          ...agent.computer,
          phase: "blocked",
          power: "unknown",
          preparationRetryAvailable: true,
          actions: agent.computer.actions.map((action) =>
            action.action === "delete"
              ? action
              : { ...action, available: false, reason: "conflicting" },
          ),
        },
      },
    },
  },
};
export const RetryPreparationConfirmation: Story = {
  ...RetryPreparation,
  args: { ...RetryPreparation.args, confirmation: "retryPreparation" },
};
