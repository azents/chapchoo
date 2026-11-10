export interface SupplyCopy {
  title: string;
  managed: string;
  existing: string;
  path: string;
  invalidPath: string;
  saved: string;
  applied: string;
  unknown: string;
  nextStart: string;
  save: string;
  read: string;
  loading: string;
  error: string;
  savedNotice: string;
  uncertain: string;
  rejected: string;
  preparation: string;
  current: string;
  cached: string;
  authentication: string;
  authenticationMatches: string;
  authenticationDiffers: string;
}
const en: SupplyCopy = {
  preparation: "Applied preparation",
  current: "Fresh at start",
  cached: "Cached after lookup failure",
  authentication: "Applied authentication",
  authenticationMatches: "Using the saved authentication selection",
  authenticationDiffers:
    "The saved authentication differs from the running process. It applies on the next native start.",
  title: "Codex setup",
  managed: "Managed preparation",
  existing: "Existing Codex installation",
  path: "Codex executable path",
  invalidPath: "Enter an absolute executable path.",
  saved: "Saved selection",
  applied: "Observed applied selection",
  unknown: "Not observed",
  nextStart:
    "A saved change takes effect when a new native process starts, not on the next message.",
  save: "Save selection",
  read: "Check saved selection",
  loading: "Loading Codex setup…",
  error: "Codex setup could not be read",
  savedNotice: "Selection saved",
  uncertain:
    "Save is not confirmed. Check the saved selection before editing again.",
  rejected:
    "Selection was not saved. Review the current selection and try again.",
};
const ko: SupplyCopy = {
  preparation: "실행에 적용된 준비",
  current: "시작 시 최신 조회 확인",
  cached: "조회 실패로 보존된 준비 사용",
  authentication: "실행에 적용된 인증",
  authenticationMatches: "현재 저장된 인증을 사용 중입니다",
  authenticationDiffers:
    "저장된 인증이 실행 중인 인증과 다릅니다. 새 Codex 프로세스가 시작될 때 적용됩니다.",
  title: "Codex 실행 설정",
  managed: "자동 준비",
  existing: "기존 Codex 설치 사용",
  path: "Codex 실행 파일 경로",
  invalidPath: "실행 파일의 절대 경로를 입력해 주세요.",
  saved: "저장된 선택",
  applied: "적용이 확인된 선택",
  unknown: "확인되지 않음",
  nextStart:
    "저장한 변경은 다음 메시지가 아니라 새 Codex 프로세스가 시작될 때 적용됩니다.",
  save: "선택 저장",
  read: "저장된 선택 확인",
  loading: "Codex 실행 설정을 불러오는 중…",
  error: "Codex 실행 설정을 불러오지 못했습니다",
  savedNotice: "선택을 저장했습니다",
  uncertain:
    "저장 여부가 확인되지 않았습니다. 다시 편집하기 전에 저장된 선택을 확인해 주세요.",
  rejected:
    "선택을 저장하지 못했습니다. 현재 선택을 확인하고 다시 시도해 주세요.",
};
const ja: SupplyCopy = {
  preparation: "適用された準備",
  current: "起動時に最新を確認",
  cached: "取得失敗後にキャッシュを使用",
  authentication: "適用された認証",
  authenticationMatches: "保存されている認証を使用中",
  authenticationDiffers:
    "保存された認証は実行中の認証と異なります。次のネイティブ起動時に適用されます。",
  title: "Codex 実行設定",
  managed: "自動準備",
  existing: "既存の Codex を使用",
  path: "Codex 実行ファイルのパス",
  invalidPath: "実行ファイルの絶対パスを入力してください。",
  saved: "保存された選択",
  applied: "適用が確認された選択",
  unknown: "未確認",
  nextStart:
    "保存した変更は次のメッセージではなく、新しいネイティブプロセスの起動時に適用されます。",
  save: "選択を保存",
  read: "保存された選択を確認",
  loading: "Codex 実行設定を読み込み中…",
  error: "Codex 実行設定を読み込めませんでした",
  savedNotice: "選択を保存しました",
  uncertain:
    "保存を確認できません。再編集する前に保存された選択を確認してください。",
  rejected:
    "選択を保存できませんでした。現在の選択を確認して、もう一度お試しください。",
};
const fr: SupplyCopy = {
  preparation: "Préparation appliquée",
  current: "À jour au démarrage",
  cached: "Cache utilisé après un échec de recherche",
  authentication: "Authentification appliquée",
  authenticationMatches: "Utilise l’authentification enregistrée",
  authenticationDiffers:
    "L’authentification enregistrée diffère de celle en cours. Elle s’appliquera au prochain démarrage natif.",
  title: "Configuration de Codex",
  managed: "Préparation automatique",
  existing: "Installation Codex existante",
  path: "Chemin de l’exécutable Codex",
  invalidPath: "Saisissez le chemin absolu de l’exécutable.",
  saved: "Sélection enregistrée",
  applied: "Sélection appliquée observée",
  unknown: "Non observée",
  nextStart:
    "Une modification enregistrée s’applique au démarrage d’un nouveau processus natif, pas au message suivant.",
  save: "Enregistrer la sélection",
  read: "Vérifier la sélection enregistrée",
  loading: "Chargement de la configuration Codex…",
  error: "Impossible de lire la configuration Codex",
  savedNotice: "Sélection enregistrée",
  uncertain:
    "L’enregistrement n’est pas confirmé. Vérifiez la sélection enregistrée avant de la modifier.",
  rejected:
    "La sélection n’a pas été enregistrée. Vérifiez la sélection actuelle, puis réessayez.",
};
export function supplyCopy(locale: string): SupplyCopy {
  return locale.startsWith("ko")
    ? ko
    : locale.startsWith("ja")
      ? ja
      : locale.startsWith("fr")
        ? fr
        : en;
}
