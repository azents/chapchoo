import type { ComputerStatus } from "./computer.js";

const english = {
  signIn: "Sign in",
  accessUnavailable:
    "Your account could not be verified. Refresh or sign in again to view this page.",
  noResources: "No managed resource reservations.",
  intent: "Requested state",
  resources: "Managed resources",
  namespace: "Namespace",
  resourceName: "Resource name",
  dockerDaemon: "Docker daemon",
  dockerContainer: "Docker container",
  dockerVolume: "Docker volume",
  containerId: "Full container ID",
  volumeCreatedAt: "Volume creation identity",
  presence: "Presence",
  present: "Present",
  absent: "Confirmed absent",
  uid: "UID",
  execution: "Execution",
  pod: "Pod",
  pvc: "PVC",
  bootstrapSecret: "Bootstrap Secret",
  agents: "Agents",
  computerStatus: "Computer status",
  accounts: "Account and spaces",
  inventory: "Computer inventory",
  providers: "Computer Providers",
  create: "Create Agent",
  recoverCreation: "Check creation result",
  canCreate: "Creation supported",
  cannotCreate: "Creation not supported",
  retryPreparation: "Retry preparation",
  retryPreparationWarning:
    "Retry this Agent’s preparation without creating another Agent or replacing any allocated saved environment.",
  name: "Agent name",
  provider: "Computer Provider",
  chooseProvider: "Choose a Computer Provider",
  noProviders: "No Computer Provider is available for creation.",
  noAgents: "No Agents yet.",
  noComputers: "No managed Computers.",
  power: "Computer power",
  management: "Host controls",
  connector: "Computer Connector",
  saved: "Saved environment",
  phase: "Current operation",
  running: "Running",
  stopped: "Stopped",
  unknown: "Unknown",
  connected: "Connected",
  disconnected: "Disconnected",
  verified: "Verified",
  missing: "Missing",
  accepted: "Accepted",
  retiring: "Stopping",
  starting: "Starting",
  deleting: "Deleting",
  completed: "Completed",
  blocked: "Operation blocked",
  blockedHint:
    "This operation cannot continue. Check host controls and saved storage in Computer details.",
  uncertain: "Checking completion",
  checkingStop: "Verifying shutdown",
  startAfterStop: "Restarting",
  checkingStart: "Verifying startup",
  checkingRestart: "Restarting",
  checkingDelete: "Verifying deletion",
  checkingStartHint:
    "Startup has not been verified. Check host controls and the last power observation in Computer details before submitting another request.",
  checkingDeleteHint:
    "Deletion has not been verified. Resources or saved data may remain; check Computer details before deciding whether to force remove the Agent.",
  start: "Start",
  stop: "Stop",
  restart: "Restart",
  delete: "Delete Agent",
  force: "Force remove",
  cancel: "Cancel",
  confirm: "Confirm",
  stopWarning:
    "This interrupts running processes. Files in the saved environment remain; live process state is not preserved.",
  deleteWarning: "This deletes the Agent, its Computer and all saved data.",
  forceWarning:
    "Remove this Agent now? Its Computer and data may remain. The background cleanup request is retained, but completion is not confirmed.",
  delayed:
    "Starting is taking longer than expected. This does not mean the Computer has failed.",
  stale:
    "Could not load the latest state. Showing the last-known information; current power and cleanup completion are not confirmed.",
  unavailable:
    "The request could not be completed. Review the current state before trying again.",
  lostSubmission:
    "The request outcome is not confirmed. Refresh before sending another operation; it may already have been accepted.",
  removed: "The Agent created by this request has already been removed.",
  retry: "Refresh",
  retryRead: "Try again",
  back: "Back to Agents",
  next: "Next page",
  first: "First page",
  logout: "Log out",
  observed: "Last power observation",
  neverObserved: "Power state has not been verified yet",
  lastKnown: "Last-known power",
  residual: "Cleanup remains",
  cleanupCompleted: "Cleanup confirmed",
  unsupported: "This action is not supported.",
  conflicting: "Another operation is in progress.",
  managementUnavailable: "Host controls are unavailable.",
  powerUnknown: "Current power is not verified.",
  requiresStopped: "The Computer must be stopped.",
  requiresRunning: "The Computer must be running.",
  savedUnverified: "The original saved environment is not verified.",
  readOnly: "Read-only inventory",
  replay: "Replay character animation",
  status: "Status",
  details: "Computer details",
};
export type ComputerCopy = Record<keyof typeof english, string>;
const korean: ComputerCopy = {
  signIn: "로그인",
  accessUnavailable:
    "로그인 상태를 확인하지 못했습니다. 다시 시도하거나 로그인해 주세요.",
  noResources: "예약된 관리 리소스가 없습니다.",
  intent: "요청한 상태",
  resources: "관리 리소스",
  namespace: "Namespace",
  resourceName: "리소스 이름",
  dockerDaemon: "Docker 데몬",
  dockerContainer: "Docker 컨테이너",
  dockerVolume: "Docker 볼륨",
  containerId: "전체 컨테이너 ID",
  volumeCreatedAt: "볼륨 생성 식별값",
  presence: "존재 여부",
  present: "존재함",
  absent: "없음 확인",
  uid: "UID",
  execution: "실행",
  pod: "Pod",
  pvc: "PVC",
  bootstrapSecret: "부트스트랩 Secret",
  agents: "에이전트",
  computerStatus: "컴퓨터 상태",
  accounts: "계정 및 공간",
  inventory: "컴퓨터 목록",
  providers: "Computer Provider",
  create: "에이전트 만들기",
  recoverCreation: "생성 결과 확인",
  canCreate: "에이전트 생성 가능",
  cannotCreate: "에이전트 생성 불가",
  retryPreparation: "준비 다시 시도",
  retryPreparationWarning:
    "같은 에이전트의 준비를 다시 시도합니다. 새 에이전트를 만들거나 이미 할당된 저장 환경을 바꾸지 않습니다.",
  name: "에이전트 이름",
  provider: "Computer Provider",
  chooseProvider: "Computer Provider 선택",
  noProviders: "에이전트를 만들 수 있는 Computer Provider가 없습니다.",
  noAgents: "아직 만든 에이전트가 없습니다.",
  noComputers: "관리 중인 컴퓨터가 없습니다.",
  power: "컴퓨터 전원",
  management: "호스트 제어",
  connector: "컴퓨터 연결",
  saved: "저장 환경",
  phase: "현재 작업",
  running: "실행 중",
  stopped: "중지됨",
  unknown: "확인되지 않음",
  connected: "연결됨",
  disconnected: "연결 끊김",
  verified: "확인됨",
  missing: "없음",
  accepted: "요청 접수됨",
  retiring: "종료 중",
  starting: "시작 중",
  deleting: "삭제 중",
  completed: "완료",
  blocked: "작업을 진행할 수 없음",
  blockedHint:
    "작업을 계속할 수 없습니다. 컴퓨터 상세 정보에서 관리 연결과 저장 공간 상태를 확인해 주세요.",
  uncertain: "완료 여부 확인 중",
  checkingStop: "종료 확인 중",
  startAfterStop: "재시작 중",
  checkingStart: "시작 확인 중",
  checkingRestart: "재시작 중",
  checkingDelete: "삭제 확인 중",
  checkingStartHint:
    "시작 완료를 확인하지 못했습니다. 요청을 반복하기 전에 컴퓨터 상세 정보에서 관리 연결과 마지막 전원 확인 시각을 확인해 주세요.",
  checkingDeleteHint:
    "삭제 완료를 확인하지 못했습니다. 컴퓨터나 저장 데이터가 남아 있을 수 있습니다. 강제 제거 여부를 결정하기 전에 컴퓨터 상세 정보를 확인해 주세요.",
  start: "시작",
  stop: "중지",
  restart: "다시 시작",
  delete: "에이전트 삭제",
  force: "강제 제거",
  cancel: "취소",
  confirm: "확인",
  stopWarning:
    "실행 중인 프로세스가 중단됩니다. 저장 환경의 파일은 유지되지만, 실행 중인 프로세스 상태는 보존되지 않습니다.",
  deleteWarning: "에이전트와 컴퓨터, 저장된 데이터가 모두 삭제됩니다.",
  forceWarning:
    "지금 에이전트를 강제로 제거할까요? 컴퓨터와 데이터가 남을 수 있습니다. 백그라운드 정리 요청은 유지되지만, 정리가 끝났는지는 확인되지 않았습니다.",
  delayed:
    "시작에 시간이 걸리고 있습니다. 이것만으로 컴퓨터 실행에 실패했다고 단정할 수는 없습니다.",
  stale:
    "최신 상태를 불러오지 못해 마지막으로 확인한 정보를 표시합니다. 현재 전원이나 정리 완료 여부는 확인되지 않았습니다.",
  unavailable:
    "요청을 완료하지 못했습니다. 현재 상태를 확인한 뒤 다시 시도해 주세요.",
  lostSubmission:
    "요청이 처리됐는지 확인하지 못했습니다. 이미 접수됐을 수 있으니 다시 요청하기 전에 현재 상태를 확인해 주세요.",
  removed: "이 요청으로 만든 에이전트는 이미 제거되었습니다.",
  retry: "새로고침",
  retryRead: "다시 시도",
  back: "에이전트 목록",
  next: "다음 페이지",
  first: "첫 페이지",
  logout: "로그아웃",
  observed: "마지막 전원 확인",
  neverObserved: "전원 상태를 아직 확인하지 못했습니다.",
  lastKnown: "마지막으로 확인한 전원",
  residual: "정리 필요",
  cleanupCompleted: "정리 완료",
  unsupported: "지원되지 않는 작업입니다.",
  conflicting: "다른 작업이 진행 중입니다.",
  managementUnavailable: "호스트를 제어할 수 없습니다.",
  powerUnknown: "현재 전원 상태를 확인하지 못했습니다.",
  requiresStopped: "컴퓨터가 중지된 상태에서만 할 수 있습니다.",
  requiresRunning: "컴퓨터가 실행 중일 때만 할 수 있습니다.",
  savedUnverified: "기존 저장 환경을 확인하지 못했습니다.",
  readOnly: "조회만 가능",
  replay: "캐릭터 동작 다시 보기",
  status: "상태",
  details: "컴퓨터 상세 정보",
};
const japanese: ComputerCopy = {
  signIn: "ログイン",
  accessUnavailable:
    "ログイン状態を確認できませんでした。再試行するか、もう一度ログインしてください。",
  noResources: "予約済みの管理リソースはありません。",
  intent: "要求された状態",
  resources: "管理リソース",
  namespace: "Namespace",
  resourceName: "リソース名",
  dockerDaemon: "Docker デーモン",
  dockerContainer: "Docker コンテナー",
  dockerVolume: "Docker ボリューム",
  containerId: "完全なコンテナー ID",
  volumeCreatedAt: "ボリュームの作成識別値",
  presence: "存在の確認",
  present: "存在",
  absent: "不在確認済み",
  uid: "UID",
  execution: "実行",
  pod: "Pod",
  pvc: "PVC",
  bootstrapSecret: "ブートストラップ Secret",
  agents: "エージェント",
  computerStatus: "コンピューターの状態",
  accounts: "アカウントとスペース",
  inventory: "コンピューター一覧",
  providers: "Computer Provider",
  create: "エージェントを作成",
  recoverCreation: "作成結果を確認",
  canCreate: "エージェントを作成可能",
  cannotCreate: "エージェントを作成不可",
  retryPreparation: "準備を再試行",
  retryPreparationWarning:
    "同じエージェントの準備を再試行します。新しいエージェントを作成したり、割り当て済みの保存環境を置き換えたりはしません。",
  name: "エージェント名",
  provider: "Computer Provider",
  chooseProvider: "Computer Provider を選択",
  noProviders:
    "エージェントの作成に利用できる Computer Provider がありません。",
  noAgents: "エージェントはまだありません。",
  noComputers: "管理対象のコンピューターはありません。",
  power: "コンピューターの電源",
  management: "ホスト操作",
  connector: "コンピューターの接続",
  saved: "保存環境",
  phase: "現在の操作",
  running: "実行中",
  stopped: "停止",
  unknown: "未確認",
  connected: "接続済み",
  disconnected: "切断",
  verified: "確認済み",
  missing: "見つかりません",
  accepted: "受付済み",
  retiring: "実行を停止中",
  starting: "起動中",
  deleting: "削除中",
  completed: "完了",
  blocked: "操作を進められません",
  blockedHint:
    "操作を続けられません。コンピューターの詳細でホスト操作の接続と保存環境の状態を確認してください。",
  uncertain: "完了を確認中",
  checkingStop: "停止を確認中",
  startAfterStop: "再起動中",
  checkingStart: "起動を確認中",
  checkingRestart: "再起動中",
  checkingDelete: "削除を確認中",
  checkingStartHint:
    "起動の完了を確認できていません。要求を再送する前に、コンピューターの詳細でホスト操作の接続と最後の電源確認時刻を確認してください。",
  checkingDeleteHint:
    "削除の完了を確認できていません。コンピューターや保存データが残っている可能性があります。強制削除を選ぶ前に、コンピューターの詳細を確認してください。",
  start: "起動",
  stop: "停止",
  restart: "再起動",
  delete: "エージェントを削除",
  force: "強制削除",
  cancel: "キャンセル",
  confirm: "確認",
  stopWarning:
    "実行中のプロセスを中断します。保存環境のファイルは残りますが、プロセスの実行状態は保持されません。",
  deleteWarning: "エージェントとコンピューター、保存データをすべて削除します。",
  forceWarning:
    "今すぐエージェントを強制削除しますか？コンピューターやデータが残る場合があります。バックグラウンドでのクリーンアップ要求は維持されますが、完了は確認できていません。",
  delayed:
    "起動に時間がかかっています。これだけではコンピューターの起動に失敗したとは判断できません。",
  stale:
    "最新の状態を取得できないため、最後に確認した情報を表示しています。現在の電源やクリーンアップの完了状況は未確認です。",
  unavailable:
    "要求を完了できませんでした。現在の状態を確認してから再試行してください。",
  lostSubmission:
    "リクエストが処理されたか確認できませんでした。すでに受け付けている可能性があるため、再送する前に現在の状態を確認してください。",
  removed: "このリクエストで作成したエージェントはすでに削除されています。",
  retry: "再読み込み",
  retryRead: "再試行",
  back: "エージェント一覧",
  next: "次のページ",
  first: "最初のページ",
  logout: "ログアウト",
  observed: "最後の電源確認",
  neverObserved: "電源の状態はまだ確認できていません。",
  lastKnown: "最後に確認した電源",
  residual: "クリーンアップが必要",
  cleanupCompleted: "クリーンアップ完了",
  unsupported: "この操作は未対応です。",
  conflicting: "別の操作が進行中です。",
  managementUnavailable: "ホストを操作できません。",
  powerUnknown: "現在の電源は未確認です。",
  requiresStopped: "コンピューターが停止している場合のみ操作できます。",
  requiresRunning: "コンピューターが実行中の場合のみ操作できます。",
  savedUnverified: "元の保存環境は未確認です。",
  readOnly: "読み取り専用",
  replay: "キャラクターの動きを再生",
  status: "状態",
  details: "コンピューターの詳細",
};
const french: ComputerCopy = {
  signIn: "Se connecter",
  accessUnavailable:
    "Votre connexion n’a pas pu être vérifiée. Réessayez ou reconnectez-vous pour consulter cette page.",
  noResources: "Aucune réservation de ressource gérée.",
  intent: "État demandé",
  resources: "Ressources gérées",
  namespace: "Namespace",
  resourceName: "Nom de la ressource",
  dockerDaemon: "Démon Docker",
  dockerContainer: "Conteneur Docker",
  dockerVolume: "Volume Docker",
  containerId: "ID complet du conteneur",
  volumeCreatedAt: "Identifiant de création du volume",
  presence: "Présence",
  present: "Présente",
  absent: "Absence confirmée",
  uid: "UID",
  execution: "Exécution",
  pod: "Pod",
  pvc: "PVC",
  bootstrapSecret: "Secret de démarrage",
  agents: "Agents",
  computerStatus: "État de l’ordinateur",
  accounts: "Compte et espaces",
  inventory: "Ordinateurs",
  providers: "Computer Providers",
  create: "Créer un Agent",
  recoverCreation: "Vérifier le résultat de création",
  canCreate: "Création d’Agents disponible",
  cannotCreate: "Création d’Agents indisponible",
  retryPreparation: "Réessayer la préparation",
  retryPreparationWarning:
    "La préparation du même Agent sera relancée. Aucun autre Agent ne sera créé et l’environnement persistant déjà alloué ne sera pas remplacé.",
  name: "Nom de l’Agent",
  provider: "Computer Provider",
  chooseProvider: "Choisir un Computer Provider",
  noProviders: "Aucun Computer Provider n’est disponible pour créer un Agent.",
  noAgents: "Aucun Agent pour le moment.",
  noComputers: "Aucun ordinateur géré.",
  power: "Alimentation de l’ordinateur",
  management: "Commandes de l’hôte",
  connector: "Connexion de l’ordinateur",
  saved: "Environnement persistant",
  phase: "Opération en cours",
  running: "En cours d’exécution",
  stopped: "Arrêté",
  unknown: "Non confirmé",
  connected: "Connecté",
  disconnected: "Déconnecté",
  verified: "Vérifié",
  missing: "Absent",
  accepted: "Acceptée",
  retiring: "Arrêt de l’exécution",
  starting: "Démarrage",
  deleting: "Suppression",
  completed: "Terminée",
  blocked: "Opération bloquée",
  blockedHint:
    "Cette opération ne peut pas continuer. Vérifiez les commandes de l’hôte et le stockage dans les détails de l’ordinateur.",
  uncertain: "Vérification de l’achèvement",
  checkingStop: "Vérification de l’arrêt",
  startAfterStop: "Redémarrage en cours",
  checkingStart: "Vérification du démarrage",
  checkingRestart: "Redémarrage en cours",
  checkingDelete: "Vérification de la suppression",
  checkingStartHint:
    "Le démarrage n’est pas confirmé. Consultez les commandes de l’hôte et la dernière observation de l’alimentation dans les détails avant de renouveler la demande.",
  checkingDeleteHint:
    "La suppression n’est pas confirmée. Des ressources ou des données peuvent subsister. Consultez les détails avant de décider de retirer immédiatement l’Agent.",
  start: "Démarrer",
  stop: "Arrêter",
  restart: "Redémarrer",
  delete: "Supprimer l’Agent",
  force: "Retirer immédiatement",
  cancel: "Annuler",
  confirm: "Confirmer",
  stopWarning:
    "Les processus en cours seront interrompus. Les fichiers de l’environnement persistant seront conservés, mais pas l’état des processus en cours.",
  deleteWarning:
    "L’Agent, son ordinateur et toutes les données enregistrées seront supprimés.",
  forceWarning:
    "Retirer cet Agent maintenant ? Son ordinateur et ses données peuvent subsister. La demande de nettoyage en arrière-plan est maintenue, mais son achèvement n’est pas confirmé.",
  delayed:
    "Le démarrage prend plus de temps que prévu. Cela ne suffit pas à conclure que l’ordinateur n’a pas pu démarrer.",
  stale:
    "Impossible de charger l’état actuel. Les dernières informations connues sont affichées ; l’alimentation actuelle et la fin du nettoyage ne sont pas confirmées.",
  unavailable:
    "La demande n’a pas pu aboutir. Vérifiez l’état actuel avant de réessayer.",
  lostSubmission:
    "Le résultat n’a pas pu être confirmé. Vérifiez l’état actuel avant de soumettre une nouvelle demande : la précédente a peut-être déjà été acceptée.",
  removed: "L’Agent associé à cette demande de création a déjà été retiré.",
  retry: "Actualiser",
  retryRead: "Réessayer",
  back: "Liste des Agents",
  next: "Page suivante",
  first: "Première page",
  logout: "Déconnexion",
  observed: "Dernière observation de l’alimentation",
  neverObserved: "L’état de l’alimentation n’a pas encore été vérifié.",
  lastKnown: "Dernier état connu de l’alimentation",
  residual: "Nettoyage restant",
  cleanupCompleted: "Nettoyage confirmé",
  unsupported: "Cette opération n’est pas prise en charge.",
  conflicting: "Une autre opération est en cours.",
  managementUnavailable: "Les commandes de l’hôte sont indisponibles.",
  powerUnknown: "L’alimentation actuelle n’est pas vérifiée.",
  requiresStopped: "L’ordinateur doit être arrêté pour effectuer cette action.",
  requiresRunning:
    "L’ordinateur doit être en cours d’exécution pour effectuer cette action.",
  savedUnverified: "L’environnement persistant d’origine n’a pas été vérifié.",
  readOnly: "Inventaire en lecture seule",
  replay: "Rejouer l’animation du personnage",
  status: "État",
  details: "Détails de l’ordinateur",
};
export function computerCopy(locale: string): ComputerCopy {
  switch (locale) {
    case "ko-KR":
      return korean;
    case "ja-JP":
      return japanese;
    case "fr-FR":
      return french;
    default:
      return english;
  }
}

/** Explain the current operation without converting intent into observed power. */
export function computerOperation(
  computer: Pick<ComputerStatus, "operation" | "intent" | "phase">,
  copy: ComputerCopy,
): { label: string; hint: string | null } {
  if (computer.phase === "blocked") {
    return { label: copy.blocked, hint: copy.blockedHint };
  }
  if (
    computer.operation === "stop" &&
    computer.intent === "running" &&
    (computer.phase === "retiring" || computer.phase === "uncertain")
  ) {
    return {
      label: copy.startAfterStop,
      hint: null,
    };
  }
  if (computer.operation === "restart" && computer.phase !== "completed") {
    return { label: copy.checkingRestart, hint: null };
  }
  if (computer.phase === "uncertain") {
    if (computer.operation === "stop") {
      return { label: copy.checkingStop, hint: null };
    }
    switch (computer.intent) {
      case "stopped":
        return { label: copy.checkingStop, hint: null };
      case "running":
        return { label: copy.checkingStart, hint: copy.checkingStartHint };
      case "deleting":
        return { label: copy.checkingDelete, hint: copy.checkingDeleteHint };
    }
  }
  return { label: copy[computer.phase], hint: null };
}
