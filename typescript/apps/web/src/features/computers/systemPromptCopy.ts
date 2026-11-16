const english = {
  title: "System prompt",
  edit: "Edit system prompt",
  field: "Instructions",
  save: "Save",
  close: "Close",
  loading: "Loading system prompt…",
  readFailed: "The system prompt could not be loaded.",
  saveFailed: "The system prompt could not be saved. Your text is retained.",
  tooLong: "The system prompt must be no more than 16 KiB of UTF-8 text.",
  retry: "Try again",
};
export type SystemPromptCopy = typeof english;
const korean: SystemPromptCopy = {
  title: "시스템 프롬프트",
  edit: "시스템 프롬프트 편집",
  field: "지시문",
  save: "저장",
  close: "닫기",
  loading: "시스템 프롬프트를 불러오는 중…",
  readFailed: "시스템 프롬프트를 불러오지 못했습니다.",
  saveFailed:
    "시스템 프롬프트를 저장하지 못했습니다. 입력한 내용은 유지됩니다.",
  tooLong: "시스템 프롬프트는 UTF-8 기준 16 KiB 이하여야 합니다.",
  retry: "다시 시도",
};
const japanese: SystemPromptCopy = {
  title: "システムプロンプト",
  edit: "システムプロンプトを編集",
  field: "指示",
  save: "保存",
  close: "閉じる",
  loading: "システムプロンプトを読み込み中…",
  readFailed: "システムプロンプトを読み込めませんでした。",
  saveFailed:
    "システムプロンプトを保存できませんでした。入力内容は保持されています。",
  tooLong: "システムプロンプトはUTF-8で16 KiB以下にしてください。",
  retry: "再試行",
};
const french: SystemPromptCopy = {
  title: "Prompt système",
  edit: "Modifier le prompt système",
  field: "Instructions",
  save: "Enregistrer",
  close: "Fermer",
  loading: "Chargement du prompt système…",
  readFailed: "Impossible de charger le prompt système.",
  saveFailed:
    "Impossible d’enregistrer le prompt système. Votre texte est conservé.",
  tooLong: "Le prompt système doit faire au maximum 16 Kio en UTF-8.",
  retry: "Réessayer",
};
export function systemPromptCopy(locale: string): SystemPromptCopy {
  switch (locale.split("-")[0]) {
    case "ko":
      return korean;
    case "ja":
      return japanese;
    case "fr":
      return french;
    default:
      return english;
  }
}
