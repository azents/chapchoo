import type { ConversationActivity } from "@chapchoo/web-core/conversation-activity";
import type {
  InputPreparation,
  RetainedInputView,
} from "@chapchoo/web-core/conversation-types";

export interface ChatCopy {
  activity: string;
  close: string;
  message: string;
  you: string;
  send: string;
  latest: string;
  menu: string;
  detail: string;
  readOnly: string;
  idle: string;
  working: string;
  requiresAction: string;
  unavailable: string;
  notStarted: string;
  authenticationRequired: string;
  starting: string;
  startFailed: string;
  sourceUnavailable: string;
  retirementUnknown: string;
  deliveryFailed: string;
  deliveryUnconfirmed: string;
  authenticationSettings: string;
  stop: string;
  empty: string;
  input: string;
  codex: string;
  loading: string;
  error: string;
  retry: string;
  retryMessage: string;
  deleteMessage: string;
  deletingMessage: string;
  deleteUnconfirmed: string;
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
  you: "You",
  latest: "Jump to latest",
  menu: "Agents and conversations",
  detail: "Agent details",
  readOnly: "Read-only",
  idle: "Ready to chat",
  working: "Working",
  requiresAction: "Waiting for your answer",
  unavailable: "Check {harness} connection",
  notStarted: "Send a message to start",
  authenticationRequired: "AI authentication required",
  starting: "Starting {harness}…",
  startFailed: "{harness} failed to start",
  sourceUnavailable: "{harness} conversation could not be opened",
  retirementUnknown: "Previous {harness} process may still be running",
  deliveryFailed: "Message could not be delivered",
  deliveryUnconfirmed: "Message delivery not confirmed",
  authenticationSettings: "Set up AI authentication",
  stop: "Stop work",
  empty: "What shall we work on today?",
  input: "Send a message",
  codex: "Codex",
  loading: "Loading conversation…",
  error: "Conversation could not be updated",
  retry: "Read again",
  retryMessage: "Retry",
  deleteMessage: "Delete",
  deletingMessage: "Deleting…",
  deleteUnconfirmed: "Deletion not confirmed",
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
  you: "나",
  send: "보내기",
  latest: "최신 대화로 이동",
  menu: "에이전트와 대화",
  detail: "에이전트 상세",
  readOnly: "읽기 전용",
  idle: "대화 가능",
  working: "작업 중",
  requiresAction: "답변을 기다리는 중",
  unavailable: "{harness} 연결 확인 필요",
  notStarted: "메시지를 보내 대화 시작",
  authenticationRequired: "AI 인증 필요",
  starting: "{harness} 실행 준비 중…",
  startFailed: "{harness} 실행 실패",
  sourceUnavailable: "{harness} 대화를 열지 못했습니다",
  retirementUnknown: "이전 {harness}가 아직 실행 중일 수 있습니다",
  deliveryFailed: "메시지 전달 실패",
  deliveryUnconfirmed: "메시지 전달 여부 미확인",
  authenticationSettings: "AI 인증 설정",
  stop: "작업 중지",
  empty: "오늘은 어떤 일을 함께 해볼까요?",
  input: "메시지 보내기",
  codex: "Codex",
  loading: "대화 불러오는 중…",
  error: "대화를 갱신하지 못했습니다",
  retry: "다시 불러오기",
  retryMessage: "재시도",
  deleteMessage: "삭제",
  deletingMessage: "삭제 중…",
  deleteUnconfirmed: "삭제 여부 미확인",
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
  you: "あなた",
  send: "送信",
  latest: "最新の会話へ",
  menu: "エージェントと会話",
  detail: "エージェントの詳細",
  readOnly: "閲覧のみ",
  idle: "会話できます",
  working: "作業中",
  requiresAction: "回答を待っています",
  unavailable: "{harness}の接続を確認してください",
  notStarted: "メッセージを送って会話を開始",
  authenticationRequired: "AI認証が必要です",
  starting: "{harness}を起動しています…",
  startFailed: "{harness}の起動に失敗しました",
  sourceUnavailable: "{harness}の会話を開けませんでした",
  retirementUnknown: "以前の{harness}がまだ実行中の可能性があります",
  deliveryFailed: "メッセージを配信できませんでした",
  deliveryUnconfirmed: "メッセージの配信を確認できません",
  authenticationSettings: "AI認証を設定",
  stop: "作業を停止",
  empty: "今日は何を一緒にしましょうか？",
  input: "メッセージを送る",
  codex: "Codex",
  loading: "会話を読み込み中…",
  error: "会話を更新できませんでした",
  retry: "再読み込み",
  retryMessage: "再送信",
  deleteMessage: "削除",
  deletingMessage: "削除中…",
  deleteUnconfirmed: "削除を確認できません",
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
  you: "Vous",
  latest: "Aller aux derniers messages",
  menu: "Agents et conversations",
  detail: "Détails de l’agent",
  readOnly: "Lecture seule",
  idle: "Disponible",
  working: "Travail en cours",
  requiresAction: "En attente de votre réponse",
  unavailable: "Vérifier la connexion à {harness}",
  notStarted: "Envoyez un message pour commencer",
  authenticationRequired: "Authentification IA requise",
  starting: "Démarrage de {harness}…",
  startFailed: "Échec du démarrage de {harness}",
  sourceUnavailable: "Impossible d’ouvrir la conversation {harness}",
  retirementUnknown:
    "Le processus {harness} précédent est peut-être encore actif",
  deliveryFailed: "Échec de l’envoi du message",
  deliveryUnconfirmed: "Envoi du message non confirmé",
  authenticationSettings: "Configurer l’authentification IA",
  stop: "Arrêter le travail",
  empty: "Sur quoi allons-nous travailler aujourd’hui ?",
  input: "Envoyer un message",
  codex: "Codex",
  loading: "Chargement de la conversation…",
  error: "Impossible d’actualiser la conversation",
  retry: "Relire",
  retryMessage: "Réessayer",
  deleteMessage: "Supprimer",
  deletingMessage: "Suppression…",
  deleteUnconfirmed: "Suppression non confirmée",
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
export function activityStatus(
  copy: ChatCopy,
  activity: ConversationActivity,
): string {
  return copy[activity].replace("{harness}", copy.codex);
}
export function preparationStatus(
  copy: ChatCopy,
  preparation?: InputPreparation,
): string | null {
  if (preparation?.type === "preparing") {
    return activityStatus(copy, "starting");
  }
  if (preparation?.type !== "failed") {
    return null;
  }
  return activityStatus(
    copy,
    preparation.reason === "sourceUnavailable"
      ? "sourceUnavailable"
      : preparation.reason === "retirementUnknown"
        ? "retirementUnknown"
        : "startFailed",
  );
}
export function inputStatus(
  copy: ChatCopy,
  input: Pick<RetainedInputView, "observation" | "preparation">,
): string {
  if (input.preparation?.type === "failed") {
    return (
      preparationStatus(copy, input.preparation) ?? copy[input.observation]
    );
  }
  // Possible forwarding is historical evidence, not a surviving processing owner.
  if (input.observation === "uncertain") {
    return copy.uncertain;
  }
  return preparationStatus(copy, input.preparation) ?? copy[input.observation];
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
