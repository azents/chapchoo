export interface ChatCopy {
  activity: string;
  close: string;
  message: string;
  send: string;
  latest: string;
  menu: string;
  detail: string;
  readOnly: string;
  idle: string;
  working: string;
  requiresAction: string;
  unavailable: string;
  stop: string;
  empty: string;
  input: string;
  codex: string;
  loading: string;
  error: string;
  retry: string;
  older: string;
  history: string;
  plan: string;
  execution: string;
  tools: string;
  toolInput: string;
  toolOutput: string;
  thought: string;
  terminal: string;
  notice: string;
  permission: string;
  question: string;
  answer: string;
  characters: string;
  items: string;
  pattern: string;
  format: string;
  cancel: string;
  decline: string;
  yes: string;
  no: string;
  optional: string;
  required: string;
  include: string;
  omitted: string;
  emptyValue: string;
  selected: string;
  invalid: string;
  unsupported: string;
  accepted: string;
  preparing: string;
  inserted: string;
  steered: string;
  failed: string;
  uncertain: string;
  sending: string;
  delivered: string;
  rejected: string;
  pending: string;
  published: string;
  nativeStatuses: Partial<Record<string, string>>;
  permissionScopes: Partial<Record<string, string>>;
}
const en: ChatCopy = {
  activity: "Activity",
  close: "Close",
  message: "Message",
  send: "Send",
  latest: "Jump to latest",
  menu: "Agents and conversations",
  detail: "Agent details",
  readOnly: "Read-only",
  idle: "Ready to chat",
  working: "Working",
  requiresAction: "Waiting for your answer",
  unavailable: "Work status unavailable",
  stop: "Stop work",
  empty: "What shall we work on today?",
  input: "Send a message",
  codex: "Codex",
  loading: "Loading conversation…",
  error: "Conversation could not be updated",
  retry: "Read again",
  older: "Load older records",
  history: "Full history",
  plan: "Current plan",
  execution: "Agent work",
  tools: "Tool activity",
  toolInput: "Tool input",
  toolOutput: "Tool output",
  thought: "Reasoning",
  terminal: "Terminal",
  notice: "Notice",
  permission: "Permission",
  question: "Question",
  answer: "Send answers",
  characters: "Characters",
  items: "Items",
  pattern: "Pattern",
  format: "Format",
  cancel: "Cancel",
  decline: "Decline",
  yes: "Yes",
  no: "No",
  optional: "Optional",
  required: "Required",
  include: "Provide an answer",
  omitted: "Not answered",
  emptyValue: "Empty answer",
  selected: "Selected",
  invalid: "Check this answer",
  unsupported: "This request cannot be answered",
  accepted: "Accepted",
  preparing: "Preparing",
  inserted: "Delivered to the agent",
  steered: "Steering delivered",
  failed: "Delivery failed",
  uncertain: "Delivery not confirmed",
  sending: "Sending…",
  delivered: "Response delivered",
  rejected: "Response rejected",
  pending: "Stop requested",
  published: "Cancellation sent",
  nativeStatuses: {
    pending: "Not started",
    in_progress: "Working",
    completed: "Completed",
    failed: "Failed",
    cancelled: "Cancelled",
  },
  permissionScopes: {
    allow_once: "Allow once",
    allow_always: "Always allow",
    reject_once: "Reject once",
    reject_always: "Always reject",
  },
};
const ko: ChatCopy = {
  activity: "작업 상태",
  close: "닫기",
  message: "메시지",
  send: "보내기",
  latest: "최신 대화로 이동",
  menu: "에이전트와 대화",
  detail: "에이전트 상세",
  readOnly: "읽기 전용",
  idle: "대화 가능",
  working: "작업 중",
  requiresAction: "답변을 기다리는 중",
  unavailable: "작업 상태를 확인할 수 없음",
  stop: "작업 중지",
  empty: "오늘은 어떤 일을 함께 해볼까요?",
  input: "메시지 보내기",
  codex: "Codex",
  loading: "대화 불러오는 중…",
  error: "대화를 갱신하지 못했습니다",
  retry: "다시 불러오기",
  older: "이전 기록 불러오기",
  history: "전체 기록",
  plan: "현재 계획",
  execution: "에이전트 작업",
  tools: "도구 작업",
  toolInput: "도구 입력",
  toolOutput: "도구 출력",
  thought: "추론",
  terminal: "터미널",
  notice: "알림",
  permission: "권한 요청",
  question: "질문",
  answer: "답변 보내기",
  characters: "글자 수",
  items: "항목 수",
  pattern: "입력 패턴",
  format: "형식",
  cancel: "취소",
  decline: "거절",
  yes: "예",
  no: "아니요",
  optional: "선택 항목",
  required: "필수 항목",
  include: "답변 입력",
  omitted: "답변하지 않음",
  emptyValue: "빈 답변",
  selected: "선택됨",
  invalid: "답변을 확인해 주세요",
  unsupported: "이 요청에는 답변할 수 없습니다",
  accepted: "접수됨",
  preparing: "준비 중",
  inserted: "에이전트에 전달됨",
  steered: "실행 중 지시로 전달됨",
  failed: "전달 실패",
  uncertain: "전달 여부를 확인할 수 없음",
  sending: "보내는 중…",
  delivered: "응답 전달됨",
  rejected: "응답이 거부됨",
  pending: "중지 요청됨",
  published: "취소 요청 전달됨",
  nativeStatuses: {
    pending: "시작 전",
    in_progress: "진행 중",
    completed: "완료",
    failed: "실패",
    cancelled: "취소됨",
  },
  permissionScopes: {
    allow_once: "한 번 허용",
    allow_always: "항상 허용",
    reject_once: "한 번 거절",
    reject_always: "항상 거절",
  },
};
const ja: ChatCopy = {
  activity: "作業状況",
  close: "閉じる",
  message: "メッセージ",
  send: "送信",
  latest: "最新の会話へ",
  menu: "エージェントと会話",
  detail: "エージェントの詳細",
  readOnly: "閲覧のみ",
  idle: "会話できます",
  working: "作業中",
  requiresAction: "回答を待っています",
  unavailable: "作業状況を確認できません",
  stop: "作業を停止",
  empty: "今日は何を一緒にしましょうか？",
  input: "メッセージを送る",
  codex: "Codex",
  loading: "会話を読み込み中…",
  error: "会話を更新できませんでした",
  retry: "再読み込み",
  older: "以前の記録を読み込む",
  history: "すべての記録",
  plan: "現在の計画",
  execution: "エージェントの作業",
  tools: "ツールの動作",
  toolInput: "ツールの入力",
  toolOutput: "ツールの出力",
  thought: "推論",
  terminal: "ターミナル",
  notice: "通知",
  permission: "権限の確認",
  question: "質問",
  answer: "回答を送信",
  characters: "文字数",
  items: "項目数",
  pattern: "入力パターン",
  format: "形式",
  cancel: "キャンセル",
  decline: "辞退",
  yes: "はい",
  no: "いいえ",
  optional: "任意",
  required: "必須",
  include: "回答する",
  omitted: "未回答",
  emptyValue: "空の回答",
  selected: "選択済み",
  invalid: "回答を確認してください",
  unsupported: "このリクエストには回答できません",
  accepted: "受付済み",
  preparing: "準備中",
  inserted: "エージェントへ配信済み",
  steered: "実行中の指示を配信済み",
  failed: "配信失敗",
  uncertain: "配信を確認できません",
  sending: "送信中…",
  delivered: "応答を配信済み",
  rejected: "応答が拒否されました",
  pending: "停止を要求しました",
  published: "キャンセルを送信しました",
  nativeStatuses: {
    pending: "未開始",
    in_progress: "進行中",
    completed: "完了",
    failed: "失敗",
    cancelled: "キャンセル済み",
  },
  permissionScopes: {
    allow_once: "今回のみ許可",
    allow_always: "常に許可",
    reject_once: "今回のみ拒否",
    reject_always: "常に拒否",
  },
};
const fr: ChatCopy = {
  activity: "Activité",
  close: "Fermer",
  message: "Message",
  send: "Envoyer",
  latest: "Aller aux derniers messages",
  menu: "Agents et conversations",
  detail: "Détails de l’agent",
  readOnly: "Lecture seule",
  idle: "Disponible",
  working: "Travail en cours",
  requiresAction: "En attente de votre réponse",
  unavailable: "État du travail indisponible",
  stop: "Arrêter le travail",
  empty: "Sur quoi allons-nous travailler aujourd’hui ?",
  input: "Envoyer un message",
  codex: "Codex",
  loading: "Chargement de la conversation…",
  error: "Impossible d’actualiser la conversation",
  retry: "Relire",
  older: "Charger les anciens éléments",
  history: "Historique complet",
  plan: "Plan actuel",
  execution: "Travail de l’agent",
  tools: "Activité des outils",
  toolInput: "Entrée de l’outil",
  toolOutput: "Sortie de l’outil",
  thought: "Raisonnement",
  terminal: "Terminal",
  notice: "Notification",
  permission: "Autorisation",
  question: "Question",
  answer: "Envoyer les réponses",
  characters: "Caractères",
  items: "Éléments",
  pattern: "Motif",
  format: "Format",
  cancel: "Annuler",
  decline: "Décliner",
  yes: "Oui",
  no: "Non",
  optional: "Facultatif",
  required: "Obligatoire",
  include: "Répondre",
  omitted: "Sans réponse",
  emptyValue: "Réponse vide",
  selected: "Sélectionné",
  invalid: "Vérifiez cette réponse",
  unsupported: "Cette demande ne peut pas recevoir de réponse",
  accepted: "Accepté",
  preparing: "Préparation",
  inserted: "Transmis à l’agent",
  steered: "Instruction transmise au travail en cours",
  failed: "Échec de transmission",
  uncertain: "Transmission non confirmée",
  sending: "Envoi…",
  delivered: "Réponse transmise",
  rejected: "Réponse rejetée",
  pending: "Arrêt demandé",
  published: "Annulation transmise",
  nativeStatuses: {
    pending: "Pas encore commencé",
    in_progress: "En cours",
    completed: "Terminé",
    failed: "Échec",
    cancelled: "Annulé",
  },
  permissionScopes: {
    allow_once: "Autoriser une fois",
    allow_always: "Toujours autoriser",
    reject_once: "Refuser une fois",
    reject_always: "Toujours refuser",
  },
};
export function chatCopy(locale: string): ChatCopy {
  return locale.startsWith("ko")
    ? ko
    : locale.startsWith("ja")
      ? ja
      : locale.startsWith("fr")
        ? fr
        : en;
}
export function nativeStatus(copy: ChatCopy, status: string): string {
  return Object.hasOwn(copy.nativeStatuses, status)
    ? (copy.nativeStatuses[status] ?? status)
    : status;
}
export function permissionScope(copy: ChatCopy, scope: string): string {
  return Object.hasOwn(copy.permissionScopes, scope)
    ? (copy.permissionScopes[scope] ?? scope)
    : scope;
}
