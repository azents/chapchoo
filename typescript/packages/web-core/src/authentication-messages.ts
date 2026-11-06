export interface AuthenticationCopy {
  title: string;
  connect: string;
  codex: string;
  personal: string;
  team: string;
  emptyTitle: string;
  emptyHint: string;
  manage: string;
  choose: string;
  change: string;
  chatgpt: string;
  apiKey: string;
  registered: string;
  unverified: string;
  billing: string;
  name: string;
  created: string;
  key: string;
  issueKey: string;
  save: string;
  cancel: string;
  close: string;
  rename: string;
  replace: string;
  reauthenticate: string;
  remove: string;
  removeHint: string;
  replaceHint: string;
  inUse: string;
  unavailable: string;
  retry: string;
  recover: string;
  uncertain: string;
  notObserved: string;
  deviceHint: string;
  openDevice: string;
  check: string;
  checking: string;
  expired: string;
  cancelled: string;
  failed: string;
  superseded: string;
  storedOnly: string;
  unselected: string;
  ownerOnly: string;
  more: string;
  previous: string;
  first: string;
  environment: string;
  authentication: string;
  profile: string;
  environmentTitle: string;
  authenticationTitle: string;
  authenticationHint: string;
  next: string;
  newAgent: string;
  cloud: string;
  environmentHint: string;
  selected: string;
  remaining: string;
  keyVisibility: string;
  execution: string;
  recordUnavailable: string;
}
const english: AuthenticationCopy = {
  first: "First page",
  recordUnavailable:
    "The connection was saved, but is no longer available to select. No replacement was selected.",
  execution: "Execution",
  keyVisibility: "Show or hide API key",
  cloud: "Cloud",
  environmentHint: "Choose the Computer your Agent will use.",
  selected: "Selected",
  remaining: "Code expires in",
  title: "AI connections",
  connect: "Connect account",
  codex: "Codex",
  personal: "Personal",
  team: "Team",
  emptyTitle: "Connect Codex",
  emptyHint: "Connect ChatGPT or register an OpenAI API key.",
  manage: "Manage",
  choose: "Choose authentication",
  change: "Change",
  chatgpt: "ChatGPT account",
  apiKey: "OpenAI API key",
  registered: "Registered",
  unverified: "Usage unverified",
  billing:
    "OpenAI API usage is billed separately from your ChatGPT subscription.",
  name: "Alias",
  created: "Created",
  key: "API key",
  issueKey: "Create an OpenAI API key",
  save: "Save",
  cancel: "Cancel",
  close: "Close",
  rename: "Rename",
  replace: "Replace key",
  reauthenticate: "Reconnect",
  remove: "Delete connection",
  removeHint:
    "Delete the credentials stored here? This does not revoke access at OpenAI.",
  replaceHint:
    "This replaces the shared stored key for every Agent selecting this connection. Their references stay the same. The API billing project may change.",
  inUse:
    "An Agent still selects this connection. Change its authentication before deleting.",
  unavailable:
    "Could not complete the request. Your current selection is unchanged.",
  retry: "Reload",
  recover: "Check request result",
  uncertain:
    "The result is not yet known. Check the request result before starting another operation.",
  notObserved:
    "No accepted result has been observed. This does not confirm cancellation. Check again.",
  deviceHint: "Enter this code at OpenAI to connect your account.",
  openDevice: "Open OpenAI",
  check: "Check connection",
  checking: "Connecting…",
  expired: "This code expired. Start a new connection.",
  cancelled: "Connection cancelled.",
  failed: "Connection could not be completed. You can try again.",
  superseded: "A newer change replaced this attempt.",
  storedOnly:
    "Saved authentication reference only; this does not confirm account access, billing or Computer setup.",
  unselected: "No authentication selected",
  ownerOnly: "Only Team Owners and Managers can manage Team connections.",
  more: "Next",
  previous: "Previous",
  environment: "Computer",
  authentication: "Codex authentication",
  profile: "Profile and name",
  environmentTitle: "Choose a Computer",
  authenticationTitle: "Connect Codex",
  authenticationHint: "Choose the authentication this Agent will use.",
  next: "Next",
  newAgent: "New Agent",
};
const korean: AuthenticationCopy = {
  first: "처음",
  recordUnavailable:
    "연결은 저장되었지만 지금은 선택할 수 없습니다. 다른 인증으로 바꾸지는 않았습니다.",
  execution: "실행",
  keyVisibility: "API 키 표시 또는 숨기기",
  cloud: "클라우드",
  environmentHint: "에이전트가 사용할 컴퓨터를 골라 주세요.",
  selected: "선택됨",
  remaining: "코드 유효 시간",
  title: "AI 연결",
  connect: "계정 연결",
  codex: "Codex",
  personal: "개인",
  team: "팀",
  emptyTitle: "Codex를 연결하세요",
  emptyHint: "ChatGPT 계정을 연결하거나 OpenAI API 키를 등록하세요.",
  manage: "관리",
  choose: "사용할 인증 선택",
  change: "변경",
  chatgpt: "ChatGPT 계정",
  apiKey: "OpenAI API 키",
  registered: "등록됨",
  unverified: "사용 확인 전",
  billing: "OpenAI API 사용 요금은 ChatGPT 구독과 별도로 청구됩니다.",
  name: "별칭",
  created: "생성 일자",
  key: "API 키",
  issueKey: "OpenAI API 키 발급",
  save: "저장",
  cancel: "취소",
  close: "닫기",
  rename: "이름 변경",
  replace: "키 교체",
  reauthenticate: "다시 연결",
  remove: "연결 삭제",
  removeHint:
    "여기에 저장된 인증 정보를 삭제할까요? OpenAI의 접근 권한은 해제되지 않습니다.",
  replaceHint:
    "이 연결을 선택한 모든 에이전트가 참조하는 저장 키를 교체합니다. 인증 선택은 유지되며 API 요금 청구 프로젝트는 바뀔 수 있습니다.",
  inUse:
    "이 연결을 선택한 에이전트가 있습니다. 해당 에이전트의 인증을 변경한 뒤 삭제하세요.",
  unavailable: "요청을 완료하지 못했습니다. 현재 선택은 유지됩니다.",
  retry: "새로고침",
  recover: "요청 결과 확인",
  uncertain:
    "요청 결과를 확인하지 못했습니다. 다른 작업을 시작하기 전에 결과를 확인하세요.",
  notObserved:
    "아직 처리 완료가 확인되지 않았습니다. 취소된 것은 아니므로 다시 확인해 주세요.",
  deviceHint: "OpenAI에서 이 코드를 입력해 계정을 연결하세요.",
  openDevice: "OpenAI 열기",
  check: "연결 확인",
  checking: "연결 중…",
  expired: "코드가 만료되었습니다. 새로 연결해 주세요.",
  cancelled: "연결을 취소했습니다.",
  failed: "연결을 완료하지 못했습니다. 다시 시도할 수 있습니다.",
  superseded: "더 최근 변경으로 이 요청이 대체되었습니다.",
  storedOnly:
    "저장된 인증 선택입니다. 계정 사용 가능 여부·요금·컴퓨터 설정 완료를 뜻하지 않습니다.",
  unselected: "선택한 인증 없음",
  ownerOnly: "팀 소유자와 관리자만 팀 연결을 관리할 수 있습니다.",
  more: "다음",
  previous: "이전",
  environment: "컴퓨터",
  authentication: "Codex 인증",
  profile: "프로필·이름",
  environmentTitle: "컴퓨터를 선택하세요",
  authenticationTitle: "Codex를 연결하세요",
  authenticationHint: "이 에이전트에서 사용할 인증을 선택하세요.",
  next: "다음",
  newAgent: "새 에이전트",
};
const japanese: AuthenticationCopy = {
  first: "最初のページ",
  recordUnavailable:
    "接続は保存されましたが、現在は選択できません。別の認証には変更していません。",
  execution: "実行",
  keyVisibility: "APIキーの表示を切り替え",
  cloud: "クラウド",
  environmentHint: "エージェントが使うコンピューターを選んでください。",
  selected: "選択中",
  remaining: "コードの残り時間",
  title: "AI接続",
  connect: "アカウントを接続",
  codex: "Codex",
  personal: "個人",
  team: "チーム",
  emptyTitle: "Codexを接続しましょう",
  emptyHint:
    "ChatGPTアカウントを接続するか、OpenAI APIキーを登録してください。",
  manage: "管理",
  choose: "認証を選択",
  change: "変更",
  chatgpt: "ChatGPTアカウント",
  apiKey: "OpenAI APIキー",
  registered: "登録済み",
  unverified: "利用未確認",
  billing:
    "OpenAI APIの利用料金はChatGPTのサブスクリプションとは別に請求されます。",
  name: "エイリアス",
  created: "作成日時",
  key: "APIキー",
  issueKey: "OpenAI APIキーを発行",
  save: "保存",
  cancel: "キャンセル",
  close: "閉じる",
  rename: "名前を変更",
  replace: "キーを交換",
  reauthenticate: "再接続",
  remove: "接続を削除",
  removeHint:
    "保存された認証情報を削除しますか？OpenAI側のアクセス権限は取り消されません。",
  replaceHint:
    "この接続を選択したすべてのエージェントが参照する保存済みキーを交換します。選択は維持されますが、APIの請求先プロジェクトが変わる場合があります。",
  inUse:
    "この接続を選択中のエージェントがあります。先にエージェントの認証を変更してください。",
  unavailable: "リクエストを完了できませんでした。現在の選択は維持されます。",
  retry: "再読み込み",
  recover: "処理結果を確認",
  uncertain: "処理結果が不明です。別の操作を始める前に結果を確認してください。",
  notObserved:
    "完了した処理はまだ確認できません。キャンセルの確認ではありません。再度確認してください。",
  deviceHint: "OpenAIでこのコードを入力してアカウントを接続してください。",
  openDevice: "OpenAIを開く",
  check: "接続を確認",
  checking: "接続中…",
  expired: "コードの有効期限が切れました。新しく接続してください。",
  cancelled: "接続をキャンセルしました。",
  failed: "接続できませんでした。再試行できます。",
  superseded: "新しい変更によりこの試行は置き換えられました。",
  storedOnly:
    "保存された認証の選択です。アカウントの利用可否、料金やコンピューターの設定完了を示すものではありません。",
  unselected: "認証未選択",
  ownerOnly: "チームの接続を管理できるのはオーナーと管理者のみです。",
  more: "次へ",
  previous: "前へ",
  environment: "コンピューター",
  authentication: "Codex認証",
  profile: "プロフィール・名前",
  environmentTitle: "コンピューターを選択",
  authenticationTitle: "Codexを接続しましょう",
  authenticationHint: "このエージェントで使用する認証を選んでください。",
  next: "次へ",
  newAgent: "新しいエージェント",
};
const french: AuthenticationCopy = {
  first: "Première page",
  recordUnavailable:
    "La connexion a été enregistrée, mais n’est plus disponible. Aucune autre authentification n’a été sélectionnée.",
  execution: "Exécution",
  keyVisibility: "Afficher ou masquer la clé API",
  cloud: "Cloud",
  environmentHint: "Choisissez le Computer que votre Agent utilisera.",
  selected: "Sélectionné",
  remaining: "Expiration du code dans",
  title: "Connexions IA",
  connect: "Connecter un compte",
  codex: "Codex",
  personal: "Personnel",
  team: "Équipe",
  emptyTitle: "Connectez Codex",
  emptyHint: "Connectez ChatGPT ou enregistrez une clé API OpenAI.",
  manage: "Gérer",
  choose: "Choisir l’authentification",
  change: "Modifier",
  chatgpt: "Compte ChatGPT",
  apiKey: "Clé API OpenAI",
  registered: "Enregistré",
  unverified: "Utilisation non vérifiée",
  billing:
    "L’utilisation de l’API OpenAI est facturée séparément de votre abonnement ChatGPT.",
  name: "Alias",
  created: "Date de création",
  key: "Clé API",
  issueKey: "Créer une clé API OpenAI",
  save: "Enregistrer",
  cancel: "Annuler",
  close: "Fermer",
  rename: "Renommer",
  replace: "Remplacer la clé",
  reauthenticate: "Reconnecter",
  remove: "Supprimer la connexion",
  removeHint:
    "Supprimer les identifiants enregistrés ici ? Cela ne révoque pas l’accès chez OpenAI.",
  replaceHint:
    "La clé partagée enregistrée est remplacée pour tous les Agents qui sélectionnent cette connexion. Leurs références sont conservées. Le projet facturé pour l’API peut changer.",
  inUse:
    "Un Agent utilise encore cette connexion. Modifiez son authentification avant de la supprimer.",
  unavailable: "La demande n’a pas abouti. Votre sélection est conservée.",
  retry: "Actualiser",
  recover: "Vérifier le résultat",
  uncertain:
    "Le résultat est inconnu. Vérifiez-le avant de commencer une autre opération.",
  notObserved:
    "Aucun résultat accepté n’est encore observé. Cela ne confirme pas une annulation. Vérifiez à nouveau.",
  deviceHint: "Saisissez ce code sur OpenAI pour connecter votre compte.",
  openDevice: "Ouvrir OpenAI",
  check: "Vérifier la connexion",
  checking: "Connexion…",
  expired: "Ce code a expiré. Recommencez la connexion.",
  cancelled: "Connexion annulée.",
  failed: "La connexion a échoué. Vous pouvez réessayer.",
  superseded: "Une modification plus récente a remplacé cette tentative.",
  storedOnly:
    "Référence d’authentification enregistrée uniquement : ni accès au compte, ni facturation, ni configuration du Computer ne sont confirmés.",
  unselected: "Aucune authentification sélectionnée",
  ownerOnly:
    "Seuls les propriétaires et gestionnaires peuvent gérer les connexions de l’équipe.",
  more: "Suivant",
  previous: "Précédent",
  environment: "Computer",
  authentication: "Authentification Codex",
  profile: "Profil et nom",
  environmentTitle: "Choisissez un Computer",
  authenticationTitle: "Connectez Codex",
  authenticationHint: "Choisissez l’authentification de cet Agent.",
  next: "Suivant",
  newAgent: "Nouvel Agent",
};
export function authenticationCopy(locale: string): AuthenticationCopy {
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
