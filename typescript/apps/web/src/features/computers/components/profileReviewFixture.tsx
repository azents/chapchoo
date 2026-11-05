"use client";
import { AccountSettings } from "@chapchoo/ui/AccountSettings";
import { HeaderControlsContext } from "@chapchoo/ui/AppHeader";
import { WorkspaceChrome } from "@chapchoo/ui/WorkspaceChrome";
import { shellCopy } from "@chapchoo/web-core/shell-messages";
import { useMediaQuery } from "@mantine/hooks";
import { useState } from "react";
import type { WorkspaceChoice } from "@chapchoo/ui/WorkspaceSelector";
import type { AgentProfileImage } from "@chapchoo/web-core/agent-profile";
import type { ReactElement, ReactNode } from "react";

/** Static, credential-free visual fixtures; not product acceptance evidence. */
export const reviewRetriever: AgentProfileImage = {
  id: "retriever",
  assetKey: "01-retriever",
  internalName: "Retriever",
};
export const reviewImages: AgentProfileImage[] = [
  reviewRetriever,
  { id: "bichon", assetKey: "02-bichon", internalName: "Bichon" },
  { id: "image", assetKey: "03-corgi", internalName: "Corgi" },
  { id: "dachshund", assetKey: "04-dachshund", internalName: "Dachshund" },
  { id: "husky", assetKey: "05-husky", internalName: "Husky" },
  { id: "collie", assetKey: "06-border-collie", internalName: "Border collie" },
];
export function ProfileReviewFrame({
  children,
  teamState = "closed",
}: {
  children: ReactNode;
  teamState?: "closed" | "picker" | "creating" | "created" | "added";
}): ReactElement {
  const mobile = useMediaQuery("(max-width: 48em)", false);
  const [drawer, setDrawer] = useState(
    teamState === "picker" || teamState === "creating" || teamState === "added",
  );
  const [picker, setPicker] = useState(
    teamState === "picker" || teamState === "added",
  );
  const [creating, setCreating] = useState(teamState === "creating");
  const [created, setCreated] = useState(
    teamState === "created" || teamState === "added",
  );
  const [name, setName] = useState("디자인 팀");
  const [handle, setHandle] = useState("dijain-tim");
  const c = shellCopy("ko");
  const current: WorkspaceChoice = created
    ? {
        key: "design",
        kind: "team",
        label: name,
        handle,
        href: `/teams/${handle}/agents`,
        selected: true,
      }
    : {
        key: "personal",
        kind: "personal",
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
      creating={creating}
      name={name}
      handle={handle}
      prospectiveAddress={`/teams/${handle}`}
      handleStatus={{
        description: "핸들은 팀을 만든 뒤 변경할 수 없습니다.",
        message: "사용할 수 있는 handle이에요.",
        available: true,
      }}
      error={null}
      status="ready"
      busy={false}
      copy={c}
      current={current}
      choices={[
        {
          key: "personal",
          kind: "personal",
          label: c.yourSpace,
          href: "/teams/my/agents",
          selected: !created,
        },
        {
          key: "product",
          kind: "team",
          label: "프로덕트 팀",
          handle: "product",
          href: "/teams/product/agents",
          selected: false,
        },
        {
          key: "research",
          kind: "team",
          label: "리서치 팀",
          handle: "research",
          href: "/teams/research/agents",
          selected: false,
        },
        ...(created ? [current] : []),
      ]}
      items={[
        {
          key: "agents",
          label: c.agents,
          icon: "agents",
          href: "/teams/my/agents",
          active: true,
        },
        {
          key: "invitations",
          label: c.invitations,
          icon: "mail",
          href: "/invitations",
        },
        ...(created
          ? [
              {
                key: "team",
                label: c.teamSettings,
                icon: "team" as const,
                href: `/teams/${handle}`,
              },
            ]
          : []),
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
        setCreating(true);
      }}
      closeCreation={() => setCreating(false)}
      onName={setName}
      onHandle={setHandle}
      onSubmit={() => {
        setCreating(false);
        setCreated(true);
      }}
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
            identityLabel="g@example.invalid"
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
            instant={1793577600000}
            onChange={() => {}}
            onRetry={() => {}}
          />
        }
      >
        {children}
      </HeaderControlsContext.Provider>
    </WorkspaceChrome>
  );
}
