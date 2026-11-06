import { AccountSettings } from "@chapchoo/ui/AccountSettings";
import { HeaderControlsContext } from "@chapchoo/ui/AppHeader";
import { WorkspaceChrome } from "@chapchoo/ui/WorkspaceChrome";
import { displayMessages } from "@chapchoo/web-core/display-messages";
import { shellCopy } from "@chapchoo/web-core/shell-messages";
import { useMediaQuery } from "@mantine/hooks";
import { useState } from "react";
import {
  fixtureCatalogue,
  fixturePanel,
} from "@/shared/authentications/connectionFixture";
import { Connections } from "./Connections";
import type { ConnectionsProps } from "./types";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import type { ReactElement } from "react";
function Frame(props: ConnectionsProps): ReactElement {
  const mobile = useMediaQuery("(max-width: 48em)", false);
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
      handleStatus={{ message: null, description: "", available: false }}
      error={null}
      status="ready"
      busy={false}
      copy={c}
      current={current}
      choices={[current]}
      items={[
        {
          key: "agents",
          label: c.agents,
          icon: "agents",
          href: "/teams/my/agents",
        },
        {
          key: "connections",
          label: props.panel.copy.title,
          icon: "connections",
          href: "/teams/my/connections",
          active: true,
        },
      ]}
      mascotReplayLabel="캐릭터 동작 다시 보기"
      openDrawer={() => setDrawer(true)}
      closeDrawer={() => setDrawer(false)}
      openPicker={() => setPicker(true)}
      closePicker={() => setPicker(false)}
      openCreation={() => {}}
      closeCreation={() => {}}
      onName={() => {}}
      onHandle={() => {}}
      onSubmit={() => {}}
      retry={() => {}}
      select={() => setPicker(false)}
    >
      <HeaderControlsContext.Provider
        value={
          <AccountSettings
            opened={false}
            mobile={mobile}
            onOpen={() => {}}
            onClose={() => {}}
            identityLabel="gunwoo@example.com"
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
            copy={{ ...displayMessages.ko, scope: "" }}
            appearance={{
              value: "light",
              label: "화면 모드",
              labels: { system: "시스템", light: "라이트", dark: "다크" },
              onChange: () => {},
            }}
            changed={false}
            error={false}
            storageError={false}
            instant={1793923200000}
            onChange={() => {}}
            onRetry={() => {}}
          />
        }
      >
        <Connections {...props} />
      </HeaderControlsContext.Provider>
    </WorkspaceChrome>
  );
}
const meta = {
  title: "Authentication/Catalogue",
  component: Connections,
  args: {
    input: {
      basePath: "",
      teamHandle: "my",
      identity: "gunwoo@example.com",
      snapshot: fixtureCatalogue,
      managed: null,
      after: null,
    },
    panel: fixturePanel(),
    access: true,
    onRetry: (): void => {},
  },
  render: Frame,
} satisfies Meta<typeof Connections>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Populated: Story = {};
export const Empty: Story = {
  args: { panel: { ...fixturePanel(), items: [] } },
};
export const Unavailable: Story = {
  args: { panel: { ...fixturePanel(), unavailable: true } },
};
