import type { McpTestFailure } from "@chapchoo/web-core/computer";

const en = {
  title: "MCP servers",
  add: "Add",
  addTitle: "Add MCP server",
  edit: "Edit",
  editTitle: "Edit MCP server",
  name: "Name",
  endpoint: "Server URL",
  auth: "Authentication",
  none: "None",
  bearer: "Bearer token",
  keep: "Leave blank to keep the saved token",
  test: "Test connection",
  testing: "Testing connection…",
  save: "Save",
  cancel: "Cancel",
  close: "Close",
  delete: "Delete",
  confirm: "Delete this MCP server?",
  check: "Check result",
  uncertain:
    "The change hasn’t been confirmed. Check its result before submitting another change.",
  rejected: "The change was rejected. Check the settings or reload the list.",
  empty: "No MCP servers",
  loading: "Loading MCP servers…",
  unavailable: "Couldn’t load MCP servers.",
  testUnavailable: "Couldn’t check the connection. Try again.",
  retry: "Try again",
  next: "Next",
  previous: "Previous",
  count: (n: number): string => `${n} ${n === 1 ? "tool" : "tools"} found`,
  failures: {
    destination: "Use a publicly reachable HTTPS server URL.",
    busy: "The connection test is busy. Try again.",
    unavailable: "Couldn’t reach the server. Check its URL and availability.",
    authentication:
      "Authentication was rejected. Check or replace the Bearer token.",
    unsupportedVersion:
      "The server uses an unsupported MCP version. Check for a compatible Streamable HTTP endpoint.",
    missingDiscovery:
      "The server doesn’t support MCP discovery. Check the server URL and configuration.",
    missingTools:
      "The server doesn’t offer MCP tools. Check its configuration.",
    invalidResponse:
      "The server returned an invalid response. Check its configuration.",
    incompleteCatalogue:
      "Couldn’t read the complete tool list. Check the server’s pagination.",
    timedOut:
      "The server took too long to respond. Check its availability and try again.",
  } satisfies Record<McpTestFailure, string>,
};
export type McpServersCopy = typeof en;
const ko: McpServersCopy = {
  title: "MCP 서버",
  add: "추가",
  addTitle: "MCP 서버 추가",
  edit: "수정",
  editTitle: "MCP 서버 수정",
  name: "이름",
  endpoint: "서버 URL",
  auth: "인증",
  none: "인증 없음",
  bearer: "Bearer 토큰",
  keep: "비워 두면 저장된 토큰을 사용합니다",
  test: "연결 테스트",
  testing: "연결을 확인하는 중…",
  save: "저장",
  cancel: "취소",
  close: "닫기",
  delete: "삭제",
  confirm: "이 MCP 서버를 삭제할까요?",
  check: "결과 확인",
  uncertain:
    "변경 결과가 확인되지 않았습니다. 다른 변경을 보내기 전에 결과를 확인해 주세요.",
  rejected: "변경이 거절되었습니다. 설정을 확인하거나 목록을 새로 불러오세요.",
  empty: "등록된 MCP 서버가 없습니다",
  loading: "MCP 서버를 불러오는 중…",
  unavailable: "MCP 서버를 불러오지 못했습니다.",
  testUnavailable: "연결을 확인하지 못했습니다. 다시 시도해 주세요.",
  retry: "다시 시도",
  next: "다음",
  previous: "이전",
  count: (n) => `도구 ${n}개 확인`,
  failures: {
    destination: "외부에서 접근 가능한 HTTPS 서버 URL을 입력하세요.",
    busy: "연결 테스트가 혼잡합니다. 잠시 후 다시 시도해 주세요.",
    unavailable:
      "서버에 연결하지 못했습니다. 서버 URL과 동작 상태를 확인하세요.",
    authentication:
      "인증이 거절되었습니다. Bearer 토큰을 확인하거나 교체하세요.",
    unsupportedVersion:
      "지원하지 않는 MCP 버전입니다. 호환되는 Streamable HTTP 주소가 있는지 확인하세요.",
    missingDiscovery:
      "서버가 MCP 탐색을 지원하지 않습니다. 서버 URL과 설정을 확인하세요.",
    missingTools:
      "서버가 MCP 도구를 제공하지 않습니다. 서버 설정을 확인하세요.",
    invalidResponse:
      "서버 응답 형식이 올바르지 않습니다. 서버 설정을 확인하세요.",
    incompleteCatalogue:
      "전체 도구 목록을 읽지 못했습니다. 서버의 페이지 나누기 설정을 확인하세요.",
    timedOut:
      "서버 응답이 늦어 테스트를 마치지 못했습니다. 서버 상태를 확인하고 다시 시도하세요.",
  },
};
const ja: McpServersCopy = {
  title: "MCP サーバー",
  add: "追加",
  addTitle: "MCP サーバーを追加",
  edit: "編集",
  editTitle: "MCP サーバーを編集",
  name: "名前",
  endpoint: "サーバー URL",
  auth: "認証",
  none: "認証なし",
  bearer: "Bearer トークン",
  keep: "空欄のままにすると、保存済みのトークンを使います",
  test: "接続をテスト",
  testing: "接続を確認中…",
  save: "保存",
  cancel: "キャンセル",
  close: "閉じる",
  delete: "削除",
  confirm: "この MCP サーバーを削除しますか？",
  check: "結果を確認",
  uncertain:
    "変更結果を確認できていません。別の変更を送信する前に結果を確認してください。",
  rejected:
    "変更は拒否されました。設定を確認するか、一覧を読み込み直してください。",
  empty: "MCP サーバーは登録されていません",
  loading: "MCP サーバーを読み込み中…",
  unavailable: "MCP サーバーを読み込めませんでした。",
  testUnavailable: "接続を確認できませんでした。もう一度お試しください。",
  retry: "再試行",
  next: "次へ",
  previous: "前へ",
  count: (n) => `ツールを ${n} 件確認しました`,
  failures: {
    destination: "公開 HTTPS サーバーの URL を入力してください。",
    busy: "接続テストが混み合っています。もう一度お試しください。",
    unavailable:
      "サーバーに接続できませんでした。URL と稼働状況を確認してください。",
    authentication:
      "認証が拒否されました。Bearer トークンを確認するか、差し替えてください。",
    unsupportedVersion:
      "未対応の MCP バージョンです。互換性のある Streamable HTTP エンドポイントがあるか確認してください。",
    missingDiscovery:
      "サーバーは MCP の検出に対応していません。URL と設定を確認してください。",
    missingTools:
      "サーバーが MCP ツールを提供していません。設定を確認してください。",
    invalidResponse:
      "サーバーからの応答が正しくありません。サーバーの設定を確認してください。",
    incompleteCatalogue:
      "ツール一覧を最後まで取得できませんでした。サーバーのページ分割を確認してください。",
    timedOut:
      "サーバーが時間内に応答しませんでした。稼働状況を確認して再試行してください。",
  },
};
const fr: McpServersCopy = {
  title: "Serveurs MCP",
  add: "Ajouter",
  addTitle: "Ajouter un serveur MCP",
  edit: "Modifier",
  editTitle: "Modifier le serveur MCP",
  name: "Nom",
  endpoint: "URL du serveur",
  auth: "Authentification",
  none: "Aucune",
  bearer: "Jeton Bearer",
  keep: "Laissez le champ vide pour conserver le jeton enregistré",
  test: "Tester la connexion",
  testing: "Vérification de la connexion…",
  save: "Enregistrer",
  cancel: "Annuler",
  close: "Fermer",
  delete: "Supprimer",
  confirm: "Supprimer ce serveur MCP ?",
  check: "Vérifier le résultat",
  uncertain:
    "La modification n’est pas confirmée. Vérifiez son résultat avant d’envoyer une autre modification.",
  rejected:
    "La modification a été refusée. Vérifiez les paramètres ou rechargez la liste.",
  empty: "Aucun serveur MCP",
  loading: "Chargement des serveurs MCP…",
  unavailable: "Impossible de charger les serveurs MCP.",
  testUnavailable: "Impossible de vérifier la connexion. Réessayez.",
  retry: "Réessayer",
  next: "Suivant",
  previous: "Précédent",
  count: (n) => `${n} ${n > 1 ? "outils trouvés" : "outil trouvé"}`,
  failures: {
    destination: "Utilisez l’URL d’un serveur HTTPS accessible publiquement.",
    busy: "Le service de test est occupé. Réessayez.",
    unavailable:
      "Le serveur est inaccessible. Vérifiez son URL et sa disponibilité.",
    authentication:
      "L’authentification a été refusée. Vérifiez ou remplacez le jeton Bearer.",
    unsupportedVersion:
      "Version MCP non prise en charge. Vérifiez si un point de terminaison Streamable HTTP compatible est disponible.",
    missingDiscovery:
      "Le serveur ne prend pas en charge la découverte MCP. Vérifiez son URL et sa configuration.",
    missingTools:
      "Le serveur ne propose pas d’outils MCP. Vérifiez sa configuration.",
    invalidResponse:
      "La réponse du serveur est invalide. Vérifiez sa configuration.",
    incompleteCatalogue:
      "Impossible de lire la liste complète. Vérifiez la pagination du serveur.",
    timedOut:
      "Le serveur n’a pas répondu à temps. Vérifiez sa disponibilité et réessayez.",
  },
};
export function mcpServersCopy(locale: string): McpServersCopy {
  switch (locale.split("-")[0]) {
    case "ko":
      return ko;
    case "ja":
      return ja;
    case "fr":
      return fr;
    default:
      return en;
  }
}
