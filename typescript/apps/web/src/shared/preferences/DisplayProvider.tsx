"use client";
import { HeaderControlsContext } from "@chapchoo/ui/AppHeader";
import {
  DisplayEntry,
  DisplayPanel,
  DisplaySaveFeedback,
  DisplayToast,
} from "@chapchoo/ui/DisplaySettings";
import { useAuth } from "@chapchoo/web-core/auth-context";
import { languageLocales, languageNames } from "@chapchoo/web-core/display";
import { useDisplayController } from "@chapchoo/web-core/display-client";
import { DisplayContext } from "@chapchoo/web-core/display-context";
import { displayText } from "@chapchoo/web-core/display-messages";
import { Anchor, Group } from "@mantine/core";
import { NextIntlClientProvider } from "next-intl";
import { useEffect } from "react";
import { trpc } from "@/shared/rpc/client";
import en from "../../../messages/en-US.json";
import fr from "../../../messages/fr-FR.json";
import ja from "../../../messages/ja-JP.json";
import ko from "../../../messages/ko-KR.json";
import type { DisplayInitial } from "@chapchoo/web-core/display-context";
import type { ReactElement, ReactNode } from "react";

const catalogs = { en, ko, ja, fr };
export function DisplayProvider({
  initial,
  basePath,
  admin = false,
  children,
}: {
  initial: DisplayInitial;
  basePath: string;
  admin?: boolean;
  children: ReactNode;
}): ReactElement {
  const auth = useAuth();
  const utils = trpc.useUtils();
  const mutation = trpc.auth.updateDisplay.useMutation();
  const controller = useDisplayController({
    identity: auth.state,
    initial,
    save: async (patch, expectedUser) => {
      if (utils.auth.identity.getData()?.id !== expectedUser) {
        throw new Error("Account changed before preference update");
      }
      const user = await mutation.mutateAsync(patch);
      if (utils.auth.identity.getData()?.id === expectedUser) {
        utils.auth.identity.setData(void 0, user);
      }
    },
  });
  const language = controller.value.language;
  const text = displayText(language);
  const copy = {
    ...text,
    scope: controller.guest ? text.guestScope : text.accountScope,
  };
  const languageSuggestion = controller.suggestions.find(
    (row) => row.key === "language",
  );
  const targetText = languageSuggestion
    ? displayText(languageSuggestion.target)
    : null;
  const messages =
    language === "ko" || language === "ja" || language === "fr"
      ? catalogs[language]
      : catalogs.en;
  const locale = languageLocales[language] ?? "en-US";
  useEffect(() => {
    document.documentElement.lang = locale;
  }, [locale]);
  return (
    <DisplayContext.Provider value={controller}>
      <NextIntlClientProvider locale={locale} messages={messages}>
        <HeaderControlsContext.Provider
          value={<DisplayEntry label={text.title} onOpen={controller.open} />}
        >
          {children}
        </HeaderControlsContext.Provider>
        <DisplayPanel
          opened={controller.opened}
          onClose={controller.close}
          value={controller.fields}
          selected={controller.choices}
          automatic={controller.guest}
          disabled={!controller.available || controller.busy}
          zones={initial.zones}
          copy={copy}
          onChange={controller.change}
          changed={controller.changed}
          error={controller.error}
          storageError={controller.storageError}
          onRetry={controller.retry}
          instant={initial.instant}
          footer={
            !controller.guest && controller.available ? (
              <Group gap="lg">
                <Anchor href={`${basePath}/profile`}>{text.profile}</Anchor>
                {admin ? (
                  <Anchor href={`${basePath}/settings`}>{text.server}</Anchor>
                ) : null}
              </Group>
            ) : null
          }
        />
        <DisplayToast
          title={text.proposal}
          alternateTitle={targetText?.proposal}
          closeLabel={text.dismiss}
          busy={controller.busy}
          onClose={controller.dismiss}
          error={
            <DisplaySaveFeedback
              copy={copy}
              error={controller.error}
              storageError={false}
              onRetry={controller.retry}
            />
          }
          suggestions={controller.suggestions.map((row) => ({
            key: row.key,
            label:
              row.key === "language" && targetText
                ? `${text.language} / ${targetText.language}`
                : row.key === "region"
                  ? text.regionShort
                  : text[row.key],
            detail:
              row.key === "language"
                ? `${languageNames[row.current]} → ${languageNames[row.target]}`
                : `${row.current} → ${row.target}`,
            action:
              row.key === "language" && targetText
                ? `${text.change} / ${targetText.change}`
                : text.change,
            onApply: (): void => controller.change(row.key, row.target),
          }))}
        />
      </NextIntlClientProvider>
    </DisplayContext.Provider>
  );
}
