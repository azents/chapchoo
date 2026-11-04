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
  teamAddress: string;
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
  teamAddress: "Team address",
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
  teamHandle: "핸들",
  teamAddress: "팀 주소",
  cancel: "취소",
  close: "닫기",
  create: "만들기",
  unavailable: "공간 목록을 불러오지 못했습니다. 다시 시도해 주세요.",
  retry: "다시 시도",
  back: "뒤로",
  handleConflict: "이미 사용 중인 핸들입니다.",
  invalidTeam: "팀 이름과 핸들을 확인해 주세요.",
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
  teamHandle: "ハンドル",
  teamAddress: "チームのアドレス",
  cancel: "キャンセル",
  close: "閉じる",
  create: "作成",
  unavailable: "ワークスペースを読み込めませんでした。もう一度お試しください。",
  retry: "再試行",
  back: "戻る",
  handleConflict: "このハンドルはすでに使われています。",
  invalidTeam: "チーム名とハンドルを確認してください。",
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
  teamAddress: "Adresse de l’équipe",
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

interface TeamHandleCopy {
  permanent: string;
  checking: string;
  available: string;
  invalid: string;
  occupied: string;
  exhausted: string;
  unavailable: string;
}
export function teamHandleCopy(language: string): TeamHandleCopy {
  const copy = shellCopy(language);
  const common = { occupied: copy.handleConflict };
  if (language.startsWith("ko"))
    return {
      ...common,
      permanent: "핸들은 팀을 만든 뒤 변경할 수 없습니다.",
      checking: "사용 가능 여부 확인 중…",
      available: "사용 가능",
      invalid:
        "영문 소문자·숫자·하이픈으로 3~16자를 입력하고, 처음과 끝은 영문 또는 숫자로 작성해 주세요.",
      exhausted:
        "사용 가능한 핸들을 찾지 못했습니다. 다른 핸들을 입력해 주세요.",
      unavailable:
        "사용 가능 여부를 확인하지 못했습니다. 핸들을 수정해 다시 시도해 주세요.",
    };
  if (language.startsWith("ja"))
    return {
      ...common,
      permanent: "作成後はハンドルを変更できません。",
      checking: "使用できるか確認中…",
      available: "使用できます",
      invalid:
        "小文字の英字・数字・ハイフンで3〜16文字を入力し、先頭と末尾は英字または数字にしてください。",
      exhausted:
        "使用できるハンドルが見つかりませんでした。別のハンドルを入力してください。",
      unavailable:
        "確認できませんでした。ハンドルを編集して再試行してください。",
    };
  if (language.startsWith("fr"))
    return {
      ...common,
      permanent: "Cet identifiant ne peut plus être modifié après la création.",
      checking: "Vérification de la disponibilité…",
      available: "Disponible",
      invalid:
        "Utilisez 3 à 16 lettres minuscules, chiffres ou tirets, avec une lettre ou un chiffre à chaque extrémité.",
      exhausted: "Aucun identifiant disponible trouvé. Saisissez-en un autre.",
      unavailable:
        "Vérification impossible. Modifiez l’identifiant pour réessayer.",
    };
  return {
    ...common,
    permanent: "The handle cannot be changed after creation.",
    checking: "Checking availability…",
    available: "Available",
    invalid:
      "Use 3–16 lowercase letters, digits or hyphens, with a letter or digit at each end.",
    exhausted: "Could not find an available handle. Enter another one.",
    unavailable: "Could not check availability. Edit the handle to try again.",
  };
}
