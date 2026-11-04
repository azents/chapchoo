import type { AgentPaletteKey } from "@chapchoo/web-core/computer";

export interface CreationCopy {
  handle: string;
  permanent: string;
  image: string;
  palette: string;
  checking: string;
  available: string;
  invalid: string;
  occupied: string;
  exhausted: string;
  unavailable: string;
  palettes: Record<AgentPaletteKey, string>;
}
const english: CreationCopy = {
  handle: "Handle",
  permanent: "The handle cannot be changed after creation.",
  image: "Profile image",
  palette: "Color",
  checking: "Checking availability…",
  available: "Available",
  invalid:
    "Use 3–16 lowercase letters, digits or hyphens, with a letter or digit at each end.",
  occupied: "This handle is already in use.",
  exhausted: "Could not find an available handle. Enter another one.",
  unavailable: "Could not check availability. Edit the handle to try again.",
  palettes: {
    butter: "Butter",
    sage: "Sage",
    apricot: "Apricot",
    teal: "Teal",
    lilac: "Lilac",
    rose: "Rose",
  },
};
const korean: CreationCopy = {
  handle: "핸들",
  permanent: "핸들은 생성한 뒤 변경할 수 없습니다.",
  image: "프로필 이미지",
  palette: "색상",
  checking: "사용 가능 여부 확인 중…",
  available: "사용 가능",
  invalid:
    "영문 소문자·숫자·하이픈으로 3~16자를 입력하고, 처음과 끝은 영문 또는 숫자로 작성해 주세요.",
  occupied: "이미 사용 중인 핸들입니다.",
  exhausted: "사용 가능한 핸들을 찾지 못했습니다. 다른 핸들을 입력해 주세요.",
  unavailable:
    "사용 가능 여부를 확인하지 못했습니다. 핸들을 수정해 다시 시도해 주세요.",
  palettes: {
    butter: "버터",
    sage: "세이지",
    apricot: "살구",
    teal: "청록",
    lilac: "라일락",
    rose: "로즈",
  },
};
const japanese: CreationCopy = {
  handle: "ハンドル",
  permanent: "作成後はハンドルを変更できません。",
  image: "プロフィール画像",
  palette: "カラー",
  checking: "使用できるか確認中…",
  available: "使用できます",
  invalid:
    "小文字の英字・数字・ハイフンで3〜16文字を入力し、先頭と末尾は英字または数字にしてください。",
  occupied: "このハンドルは使用されています。",
  exhausted:
    "使用できるハンドルが見つかりませんでした。別のハンドルを入力してください。",
  unavailable: "確認できませんでした。ハンドルを編集して再試行してください。",
  palettes: {
    butter: "バター",
    sage: "セージ",
    apricot: "アプリコット",
    teal: "ティール",
    lilac: "ライラック",
    rose: "ローズ",
  },
};
const french: CreationCopy = {
  handle: "Identifiant",
  permanent: "Cet identifiant ne peut plus être modifié après la création.",
  image: "Image de profil",
  palette: "Couleur",
  checking: "Vérification de la disponibilité…",
  available: "Disponible",
  invalid:
    "Utilisez 3 à 16 lettres minuscules, chiffres ou tirets, avec une lettre ou un chiffre à chaque extrémité.",
  occupied: "Cet identifiant est déjà utilisé.",
  exhausted: "Aucun identifiant disponible trouvé. Saisissez-en un autre.",
  unavailable:
    "Vérification impossible. Modifiez l’identifiant pour réessayer.",
  palettes: {
    butter: "Beurre",
    sage: "Sauge",
    apricot: "Abricot",
    teal: "Sarcelle",
    lilac: "Lilas",
    rose: "Rose",
  },
};
export function creationCopy(locale: string): CreationCopy {
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
