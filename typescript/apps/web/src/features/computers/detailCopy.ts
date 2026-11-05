const english = {
  computer: "Computer",
  on: "The Computer is on",
  off: "The Computer is off",
  unknown: "Power has not been confirmed",
  connection: "Computer connection",
  connectionHint:
    "Reported by the connection app inside the Computer, independently of its power.",
  management: "Power management connection",
  managementHint: "The path used to deliver start and stop requests.",
  storage: "Saved storage",
  storageHint: "Confirms the allocated storage, not file contents or backups.",
  operation: "Operation status",
  diagnostics: "Troubleshooting information",
  close: "Close",
};
export type ComputerDetailCopy = typeof english;
const korean: ComputerDetailCopy = {
  computer: "컴퓨터",
  on: "컴퓨터가 켜져 있습니다",
  off: "컴퓨터가 꺼져 있습니다",
  unknown: "전원 상태를 확인하지 못했습니다",
  connection: "컴퓨터 연결",
  connectionHint:
    "컴퓨터 안의 연결 프로그램이 보내는 신호입니다. 전원 상태와는 다를 수 있습니다.",
  management: "전원 관리 연결",
  managementHint: "시작·중지 요청을 전달하는 관리 경로입니다.",
  storage: "저장 공간",
  storageHint:
    "할당된 저장 공간을 확인합니다. 파일 내용이나 백업을 검사한 것은 아닙니다.",
  operation: "작업 상태",
  diagnostics: "문제 진단용 정보",
  close: "닫기",
};
const japanese: ComputerDetailCopy = {
  computer: "コンピューター",
  on: "コンピューターは起動しています",
  off: "コンピューターは停止しています",
  unknown: "電源の状態を確認できません",
  connection: "コンピューターの接続",
  connectionHint:
    "コンピューター内の接続アプリが送る信号です。電源の状態とは異なる場合があります。",
  management: "電源管理の接続",
  managementHint: "起動・停止の要求を送るための管理経路です。",
  storage: "保存領域",
  storageHint:
    "割り当て済みの保存領域の確認です。ファイルの内容やバックアップは検査していません。",
  operation: "操作の状態",
  diagnostics: "診断情報",
  close: "閉じる",
};
const french: ComputerDetailCopy = {
  computer: "Ordinateur",
  on: "L’ordinateur est allumé",
  off: "L’ordinateur est arrêté",
  unknown: "L’alimentation n’a pas pu être confirmée",
  connection: "Connexion de l’ordinateur",
  connectionHint:
    "Signal de l’application de connexion dans l’ordinateur, indépendant de son alimentation.",
  management: "Connexion de gestion de l’alimentation",
  managementHint:
    "Canal utilisé pour transmettre les demandes de démarrage et d’arrêt.",
  storage: "Stockage persistant",
  storageHint:
    "Confirme le stockage alloué, pas le contenu des fichiers ni les sauvegardes.",
  operation: "État de l’opération",
  diagnostics: "Informations de diagnostic",
  close: "Fermer",
};
export function computerDetailCopy(locale: string): ComputerDetailCopy {
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
