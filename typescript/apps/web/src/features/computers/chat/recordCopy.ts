import type { ToolKind } from "./recordPresentation";
export interface RecordCopy {
  newEvents: string;
  kinds: Record<ToolKind, string>;
  collapse: string;
  copy: string;
  copied: string;
  command: string;
  human: string;
  agent: string;
  details: string;
}
const en: RecordCopy = {
  newEvents: "New events below",
  kinds: {
    read: "Read file",
    edit: "Edit file",
    delete: "Delete",
    move: "Move",
    search: "Search",
    execute: "Run command",
    think: "Think",
    fetch: "Fetch",
    switch_mode: "Change mode",
    other: "Tool",
  },
  collapse: "Collapse details",
  copy: "Copy",
  copied: "Copied",
  command: "Command",
  human: "Human",
  agent: "Agent",
  details: "Details",
};
const ko: RecordCopy = {
  newEvents: "아래에 새 이벤트",
  kinds: {
    read: "파일 읽기",
    edit: "파일 수정",
    delete: "삭제",
    move: "이동",
    search: "검색",
    execute: "명령 실행",
    think: "생각",
    fetch: "외부 조회",
    switch_mode: "모드 변경",
    other: "도구",
  },
  collapse: "상세 접기",
  copy: "복사",
  copied: "복사됨",
  command: "실행한 명령",
  human: "사람",
  agent: "Agent",
  details: "상세",
};
const ja: RecordCopy = {
  newEvents: "下に新しいイベント",
  kinds: {
    read: "ファイルを読む",
    edit: "ファイルを編集",
    delete: "削除",
    move: "移動",
    search: "検索",
    execute: "コマンド実行",
    think: "思考",
    fetch: "外部取得",
    switch_mode: "モード変更",
    other: "ツール",
  },
  collapse: "詳細を閉じる",
  copy: "コピー",
  copied: "コピー済み",
  command: "コマンド",
  human: "ユーザー",
  agent: "Agent",
  details: "詳細",
};
const fr: RecordCopy = {
  newEvents: "Nouveaux événements plus bas",
  kinds: {
    read: "Lire le fichier",
    edit: "Modifier le fichier",
    delete: "Supprimer",
    move: "Déplacer",
    search: "Rechercher",
    execute: "Exécuter",
    think: "Réfléchir",
    fetch: "Récupérer",
    switch_mode: "Changer de mode",
    other: "Outil",
  },
  collapse: "Replier les détails",
  copy: "Copier",
  copied: "Copié",
  command: "Commande",
  human: "Personne",
  agent: "Agent",
  details: "Détails",
};
export function recordCopy(locale: string): RecordCopy {
  return locale.startsWith("ko")
    ? ko
    : locale.startsWith("ja")
      ? ja
      : locale.startsWith("fr")
        ? fr
        : en;
}
