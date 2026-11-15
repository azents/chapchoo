const english = {
  title: "Prompt sources",
  execution: "Execution",
  supplied: "Supplied",
  prepared: "Prepared",
  loading: "Loading prompt sources…",
  error: "Prompt sources could not be loaded.",
  empty: "No retained prompt sources are available for this conversation.",
  absent: "This source was not supplied.",
  retry: "Try again",
  more: "Load earlier executions",
  moreError: "Earlier executions could not be loaded.",
  product: "Product instructions",
  function: "Function instructions",
  owner: "Owner system prompt",
  executionSource: "Execution instructions",
};
export type InstructionEvidenceCopy = typeof english;
const korean: InstructionEvidenceCopy = {
  title: "프롬프트 출처",
  execution: "실행",
  supplied: "제공됨",
  prepared: "준비됨",
  loading: "프롬프트 출처를 불러오는 중…",
  error: "프롬프트 출처를 불러오지 못했습니다.",
  empty: "이 대화에 보존된 프롬프트 출처가 없습니다.",
  absent: "이 출처는 제공되지 않았습니다.",
  retry: "다시 시도",
  more: "이전 실행 불러오기",
  moreError: "이전 실행을 불러오지 못했습니다.",
  product: "제품 지시문",
  function: "기능 지시문",
  owner: "소유자 시스템 프롬프트",
  executionSource: "실행 지시문",
};
const japanese: InstructionEvidenceCopy = {
  title: "プロンプトの出典",
  execution: "実行",
  supplied: "提供済み",
  prepared: "準備済み",
  loading: "プロンプトの出典を読み込み中…",
  error: "プロンプトの出典を読み込めませんでした。",
  empty: "この会話には保存されたプロンプトの出典がありません。",
  absent: "この出典は提供されていません。",
  retry: "再試行",
  more: "以前の実行を読み込む",
  moreError: "以前の実行を読み込めませんでした。",
  product: "製品の指示",
  function: "機能の指示",
  owner: "所有者のシステムプロンプト",
  executionSource: "実行の指示",
};
const french: InstructionEvidenceCopy = {
  title: "Sources du prompt",
  execution: "Exécution",
  supplied: "Fourni",
  prepared: "Préparé",
  loading: "Chargement des sources du prompt…",
  error: "Impossible de charger les sources du prompt.",
  empty: "Aucune source de prompt n’est conservée pour cette conversation.",
  absent: "Cette source n’a pas été fournie.",
  retry: "Réessayer",
  more: "Charger les exécutions précédentes",
  moreError: "Impossible de charger les exécutions précédentes.",
  product: "Instructions du produit",
  function: "Instructions de la fonction",
  owner: "Prompt système du propriétaire",
  executionSource: "Instructions de l’exécution",
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
