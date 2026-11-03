export interface ShellCopy {
  account: string;
  settings: string;
  profile: string;
  login: string;
  logout: string;
  navigation: string;
  chooseContext: string;
  yourSpace: string;
  teams: string;
  createTeam: string;
  administration: string;
  agents: string;
  teamSettings: string;
  invitations: string;
  signupLinks: string;
  resetLinks: string;
  computerProviders: string;
  serverSettings: string;
  teamName: string;
  teamHandle: string;
  cancel: string;
  close: string;
  create: string;
  unavailable: string;
  retry: string;
  back: string;
  handleConflict: string;
  invalidTeam: string;
  creationUncertain: string;
}
const en: ShellCopy = {
  account: "Account and settings",
  settings: "Settings",
  profile: "Profile",
  login: "Sign in",
  logout: "Sign out",
  navigation: "Open navigation",
  chooseContext: "Switch workspace",
  yourSpace: "Your space",
  teams: "Teams",
  createTeam: "Create team",
  administration: "Administration",
  agents: "Agents",
  teamSettings: "Team settings",
  invitations: "Invitations",
  signupLinks: "Signup links",
  resetLinks: "Password reset links",
  computerProviders: "Computer Providers",
  serverSettings: "Server settings",
  teamName: "Team name",
  teamHandle: "Handle",
  cancel: "Cancel",
  close: "Close",
  create: "Create",
  unavailable: "Could not load your workspaces. Try again.",
  retry: "Retry",
  back: "Back",
  handleConflict: "This handle is already in use.",
  invalidTeam: "Check the team name and handle.",
  creationUncertain:
    "We could not confirm whether the team was created. Check the team list before submitting again.",
};
const ko: ShellCopy = {
  account: "계정 및 설정",
  settings: "설정",
  profile: "프로필",
  login: "로그인",
  logout: "로그아웃",
  navigation: "메뉴 열기",
  chooseContext: "공간 선택",
  yourSpace: "내 공간",
  teams: "팀",
  createTeam: "팀 만들기",
  administration: "관리자 페이지",
  agents: "에이전트",
  teamSettings: "팀 설정",
  invitations: "초대",
  signupLinks: "가입 링크",
  resetLinks: "비밀번호 재설정 링크",
  computerProviders: "Computer Provider",
  serverSettings: "서버 설정",
  teamName: "팀 이름",
  teamHandle: "팀 ID",
  cancel: "취소",
  close: "닫기",
  create: "만들기",
  unavailable: "공간 목록을 불러오지 못했습니다. 다시 시도해 주세요.",
  retry: "다시 시도",
  back: "뒤로",
  handleConflict: "이미 사용 중인 팀 ID입니다.",
  invalidTeam: "팀 이름과 ID를 확인해 주세요.",
  creationUncertain:
    "팀이 만들어졌는지 확인하지 못했습니다. 다시 만들기 전에 팀 목록을 확인해 주세요.",
};
const ja: ShellCopy = {
  account: "アカウントと設定",
  settings: "設定",
  profile: "プロフィール",
  login: "ログイン",
  logout: "ログアウト",
  navigation: "メニューを開く",
  chooseContext: "ワークスペースを切り替え",
  yourSpace: "自分のスペース",
  teams: "チーム",
  createTeam: "チームを作成",
  administration: "管理者ページ",
  agents: "エージェント",
  teamSettings: "チーム設定",
  invitations: "招待",
  signupLinks: "登録リンク",
  resetLinks: "パスワード再設定リンク",
  computerProviders: "Computer Provider",
  serverSettings: "サーバー設定",
  teamName: "チーム名",
  teamHandle: "チームID",
  cancel: "キャンセル",
  close: "閉じる",
  create: "作成",
  unavailable: "ワークスペースを読み込めませんでした。もう一度お試しください。",
  retry: "再試行",
  back: "戻る",
  handleConflict: "このチームIDはすでに使われています。",
  invalidTeam: "チーム名とIDを確認してください。",
  creationUncertain:
    "チームが作成されたか確認できませんでした。もう一度作成する前にチーム一覧を確認してください。",
};
const fr: ShellCopy = {
  account: "Compte et paramètres",
  settings: "Paramètres",
  profile: "Profil",
  login: "Se connecter",
  logout: "Se déconnecter",
  navigation: "Ouvrir le menu",
  chooseContext: "Changer d’espace de travail",
  yourSpace: "Votre espace",
  teams: "Équipes",
  createTeam: "Créer une équipe",
  administration: "Administration",
  agents: "Agents",
  teamSettings: "Paramètres de l’équipe",
  invitations: "Invitations",
  signupLinks: "Liens d’inscription",
  resetLinks: "Liens de réinitialisation du mot de passe",
  computerProviders: "Computer Providers",
  serverSettings: "Paramètres du serveur",
  teamName: "Nom de l’équipe",
  teamHandle: "Identifiant",
  cancel: "Annuler",
  close: "Fermer",
  create: "Créer",
  unavailable: "Impossible de charger vos espaces de travail. Réessayez.",
  retry: "Réessayer",
  back: "Retour",
  handleConflict: "Cet identifiant est déjà utilisé.",
  invalidTeam: "Vérifiez le nom et l’identifiant de l’équipe.",
  creationUncertain:
    "La création de l’équipe n’a pas pu être confirmée. Vérifiez la liste des équipes avant de réessayer.",
};
export function shellCopy(language: string): ShellCopy {
  return language.startsWith("ko")
    ? ko
    : language.startsWith("ja")
      ? ja
      : language.startsWith("fr")
        ? fr
        : en;
}
