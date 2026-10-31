import { AppFrame } from "@chapchoo/ui/AppFrame";
import {
  browserCookie,
  decodeDisplayCookie,
  explicitDisplayCookie,
  projectedDisplayCookie,
  resolveGuestDisplay,
} from "@chapchoo/web-core/display";
import { Button, Container, Stack, Text } from "@mantine/core";
import { useState } from "react";
import { DisplayProvider } from "@/shared/preferences/DisplayProvider";
import { RpcProvider } from "@/shared/rpc/provider";
import { ProfilePage } from "./ProfilePage";
import type { DisplayInitial } from "@chapchoo/web-core/display-context";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import type { ReactElement } from "react";

function Preferences({
  guest = false,
  profile = false,
}: {
  guest?: boolean;
  profile?: boolean;
}): ReactElement {
  const [showProfile, setShowProfile] = useState(profile);
  const value = { language: "ko", timeZone: "Asia/Seoul", region: "ko-KR" };
  const explicit = decodeDisplayCookie(browserCookie(explicitDisplayCookie));
  const projection = decodeDisplayCookie(browserCookie(projectedDisplayCookie));
  const initial: DisplayInitial = {
    identity: {
      type: "LOADED",
      data: guest
        ? null
        : {
            id: "fixture-user",
            email: "owner@example.com",
            isAdmin: false,
            createdAt: { seconds: 0n, nanos: 0 },
            display: value,
          },
    },
    value: guest ? resolveGuestDisplay(value, explicit, projection) : value,
    defaults: value,
    explicit,
    instant: 1793428200000,
    zones: [
      "UTC",
      "Asia/Seoul",
      "America/New_York",
      "Europe/London",
      "Asia/Tokyo",
    ],
  };
  return (
    <RpcProvider initialIdentity={initial.identity} basePath="">
      <DisplayProvider initial={initial} basePath="">
        {showProfile ? (
          <ProfilePage basePath="" />
        ) : (
          <AppFrame>
            <Container component="main" py="xl">
              <Stack>
                <Text>
                  {guest
                    ? "공개 페이지 배경 예시"
                    : "로그인 일반 화면 배경 예시"}
                </Text>
                {!guest ? (
                  <Button onClick={(): void => setShowProfile(true)}>
                    프로필 보기
                  </Button>
                ) : null}
              </Stack>
            </Container>
          </AppFrame>
        )}
      </DisplayProvider>
    </RpcProvider>
  );
}
const meta = {
  title: "Preferences/Integrated",
  component: Preferences,
  parameters: {
    layout: "fullscreen",
    docs: {
      description: {
        component:
          "Production display controller, provider and profile with seeded identity. The non-profile backdrop and local view switch are explanatory fixtures; browser tests intercept RPC, not a live account backend.",
      },
    },
  },
} satisfies Meta<typeof Preferences>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Account: Story = {};
export const Profile: Story = { args: { profile: true } };
export const Guest: Story = { args: { guest: true } };
