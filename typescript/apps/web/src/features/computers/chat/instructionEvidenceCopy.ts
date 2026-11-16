const english = {
  title: "System prompt",
  supplied: "Applied to this conversation",
  prepared: "Not yet applied to this conversation",
  empty: "No system prompt has been prepared for this conversation.",
  back: "Back to conversation",
  product: "Product instructions",
  function: "Function instructions",
  owner: "Owner system prompt",
  execution: "Execution instructions",
};
export type InstructionEvidenceCopy = typeof english;
const korean: InstructionEvidenceCopy = {
  title: "시스템 프롬프트",
  supplied: "이 대화에 적용됨",
  prepared: "아직 이 대화에 적용되지 않음",
  empty: "이 대화에 준비된 시스템 프롬프트가 없습니다.",
  back: "대화로 돌아가기",
  product: "제품 지시문",
  function: "기능 지시문",
  owner: "소유자 시스템 프롬프트",
  execution: "실행 지시문",
};
const japanese: InstructionEvidenceCopy = {
  title: "システムプロンプト",
  supplied: "この会話に適用済み",
  prepared: "この会話にはまだ適用されていません",
  empty: "この会話のシステムプロンプトはまだ準備されていません。",
  back: "会話に戻る",
  product: "製品の指示",
  function: "機能の指示",
  owner: "所有者のシステムプロンプト",
  execution: "実行の指示",
};
const french: InstructionEvidenceCopy = {
  title: "Prompt système",
  supplied: "Appliqué à cette conversation",
  prepared: "Pas encore appliqué à cette conversation",
  empty: "Aucun prompt système n’a été préparé pour cette conversation.",
  back: "Retour à la conversation",
  product: "Instructions du produit",
  function: "Instructions de la fonction",
  owner: "Prompt système du propriétaire",
  execution: "Instructions de l’exécution",
};
export function instructionEvidenceCopy(
  locale: string,
): InstructionEvidenceCopy {
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
