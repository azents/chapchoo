import type { AgentPaletteKey } from "@chapchoo/web-core/computer";

export interface CreationCopy {
  name: string;
  handlePreserved: string;
  newAgent: string;
  companions: string;
  addCompanion: string;
  creationTitle: string;
  chooseProfile: string;
  emptyHint: string;
  handle: string;
  permanent: string;
  image: string;
  palette: string;
  edit: string;
  save: string;
  saveFailed: string;
  currentImage: string;
  close: string;
  randomPalette: string;
  randomFailed: string;
  preview: string;
  previewCaption: string;
  previewHint: string;
  open: string;
  checking: string;
  available: string;
  invalid: string;
  occupied: string;
  exhausted: string;
  unavailable: string;
  palettes: Record<AgentPaletteKey, string>;
}
const english: CreationCopy = {
  name: "Name",
  handlePreserved: "Renaming the profile keeps this handle.",
  newAgent: "New Agent",
  companions: "Your companions",
  addCompanion: "Meet a new Agent",
  creationTitle: "Give your Agent a name.",
  chooseProfile: "Choose a profile image and color, too.",
  emptyHint: "Start something new with an Agent.",
  handle: "Handle",
  permanent: "The handle cannot be changed after creation.",
  image: "Profile image",
  palette: "Color",
  edit: "Edit profile",
  save: "Save",
  saveFailed: "Could not save the profile. Your changes are still here.",
  currentImage: "Current image",
  close: "Close",
  randomPalette: "Random color",
  randomFailed: "Could not choose another color. Your draft is unchanged.",
  preview: "Profile preview",
  previewCaption: "Preview",
  previewHint: "Saving applies this profile to the Agent.",
  open: "Open",
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
  name: "이름",
  handlePreserved: "프로필 이름을 바꿔도 handle은 유지됩니다.",
  newAgent: "새 에이전트",
  companions: "함께하는 에이전트",
  addCompanion: "새로운 에이전트와 함께하기",
  creationTitle: "이름을 지어 주세요.",
  chooseProfile: "프로필 이미지와 색상도 골라 주세요.",
  emptyHint: "새로운 에이전트와 함께 시작해 보세요.",
  handle: "핸들",
  permanent: "핸들은 생성한 뒤 변경할 수 없습니다.",
  image: "프로필 이미지",
  palette: "에이전트 색상",
  edit: "프로필 편집",
  save: "저장",
  saveFailed: "프로필을 저장하지 못했습니다. 수정한 내용은 그대로 유지됩니다.",
  currentImage: "현재 이미지",
  close: "닫기",
  randomPalette: "랜덤",
  randomFailed: "다른 색상을 선택하지 못했습니다. 현재 입력은 유지됩니다.",
  preview: "프로필 미리보기",
  previewCaption: "미리보기",
  previewHint: "저장하면 에이전트 프로필에 반영됩니다.",
  open: "열기",
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
  name: "名前",
  handlePreserved: "名前を変更してもハンドルは変わりません。",
  newAgent: "新しいエージェント",
  companions: "一緒に働くエージェント",
  addCompanion: "新しいエージェントを迎える",
  creationTitle: "名前を付けましょう。",
  chooseProfile: "プロフィール画像とカラーも選んでください。",
  emptyHint: "新しいエージェントと一緒に始めましょう。",
  handle: "ハンドル",
  permanent: "作成後はハンドルを変更できません。",
  image: "プロフィール画像",
  palette: "カラー",
  edit: "プロフィールを編集",
  save: "保存",
  saveFailed: "保存できませんでした。編集内容は保持されています。",
  currentImage: "現在の画像",
  close: "閉じる",
  randomPalette: "ランダムなカラー",
  randomFailed: "別のカラーを選べませんでした。編集内容は保持されています。",
  preview: "プロフィールのプレビュー",
  previewCaption: "プレビュー",
  previewHint: "保存するとエージェントのプロフィールに反映されます。",
  open: "開く",
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
  name: "Nom",
  handlePreserved: "Renommer le profil ne modifie pas cet identifiant.",
  newAgent: "Nouvel Agent",
  companions: "Vos compagnons",
  addCompanion: "Accueillir un nouvel Agent",
  creationTitle: "Donnez un nom à votre Agent.",
  chooseProfile: "Choisissez aussi une image de profil et une couleur.",
  emptyHint: "Commencez avec un nouvel Agent.",
  handle: "Identifiant",
  permanent: "Cet identifiant ne peut plus être modifié après la création.",
  image: "Image de profil",
  palette: "Couleur",
  edit: "Modifier le profil",
  save: "Enregistrer",
  saveFailed:
    "Le profil n’a pas pu être enregistré. Vos modifications sont conservées.",
  currentImage: "Image actuelle",
  close: "Fermer",
  randomPalette: "Couleur aléatoire",
  randomFailed:
    "Impossible de choisir une autre couleur. Vos modifications sont conservées.",
  preview: "Aperçu du profil",
  previewCaption: "Aperçu",
  previewHint: "Enregistrer applique ce profil à l’Agent.",
  open: "Ouvrir",
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
