export interface ChatCopy {
  activity: string;
  close: string;
  message: string;
  send: string;
  attach: string;
  latest: string;
  menu: string;
  detail: string;
  readOnly: string;
  idle: string;
  working: string;
  stopped: string;
  stop: string;
  empty: string;
  input: string;
  today: string;
  user: string;
  response: string;
  after: string;
  followup: string;
  eyebrow: string;
  planTitle: string;
  planSummary: string;
  planFooter: string;
  route: readonly { time: string; title: string; note: string }[];
  taskTitle: string;
  taskDescription: string;
  tasks: readonly string[];
  tools: string;
  search: string;
  read: string;
  complete: string;
  inProgress: string;
  query: string;
  page: string;
  hours: string;
  codex: string;
}
const en: ChatCopy = {
  activity: "Activity",
  close: "Close",
  message: "Message",
  send: "Send",
  attach: "Attach file",
  latest: "Jump to latest",
  menu: "Agents and conversations",
  detail: "Agent details",
  readOnly: "Read-only",
  idle: "Ready to chat",
  working: "Finding cafés",
  stopped: "Work stopped",
  stop: "Stop work",
  empty: "What shall we work on today?",
  input: "Send a message",
  today: "Today, 13:10",
  user: "I'd like a relaxed walk in Seoul on Saturday afternoon. A quiet café too, without packing too much in!",
  response:
    "Let's take it slowly this time. I picked a route with room to stop whenever somewhere catches your eye.",
  after: "Shall I find a few quieter cafés?",
  followup: "Yes, find two quiet cafés that open on Saturday.",
  eyebrow: "SATURDAY WALK",
  planTitle: "Slowly, through Yeonnam",
  planSummary: "About 3 hours · 2.4 km · Three relaxed stops",
  planFooter: "Feel free to choose just the places you like.",
  route: [
    {
      time: "14:00",
      title: "Gyeongui Line Forest Park",
      note: "An easy walk in the shade",
    },
    {
      time: "15:00",
      title: "A quiet neighborhood café",
      note: "Coffee and a few pages by the window",
    },
    {
      time: "16:00",
      title: "The lanes of Yeonhui",
      note: "Small shops, whenever you feel like stopping",
    },
  ],
  taskTitle: "This task",
  taskDescription: "Checking Saturday hours and the atmosphere.",
  tasks: [
    "Find candidate cafés",
    "Check Saturday opening hours",
    "Choose two to recommend",
  ],
  tools: "Tool calls",
  search: "Web search",
  read: "Read page",
  complete: "Completed",
  inProgress: "In progress",
  query: "Quiet cafés in Yeonnam",
  page: "Café information · Location and hours",
  hours: "Saturday opening hours",
  codex: "Codex",
};
const ko: ChatCopy = {
  activity: "작업 상태",
  close: "닫기",
  message: "메시지",
  send: "보내기",
  attach: "파일 첨부",
  latest: "최신 대화로 이동",
  menu: "에이전트와 대화",
  detail: "에이전트 상세",
  readOnly: "읽기 전용",
  idle: "대화 가능",
  working: "카페 찾는 중",
  stopped: "작업 중지",
  stop: "작업 중지",
  empty: "오늘은 어떤 일을 함께 해볼까요?",
  input: "메시지 보내기",
  today: "오늘, 오후 1:10",
  user: "토요일 오후에 서울에서 좀 걷고 싶어. 조용한 카페도 들르고, 너무 빡빡하지 않게!",
  response:
    "좋아요. 이번엔 많이 하는 것보다, 느긋하게. 걷다가 마음에 드는 곳에서 쉬어갈 수 있게 골라봤어요.",
  after: "카페는 조용한 곳 위주로 더 찾아볼까요?",
  followup: "응, 토요일에 여는 조용한 카페로 두 곳만 더 찾아줘.",
  eyebrow: "SATURDAY WALK",
  planTitle: "연남동에서, 천천히",
  planSummary: "약 3시간 · 걷기 2.4km · 여유롭게 세 곳",
  planFooter: "마음에 드는 곳만 골라도 괜찮아요.",
  route: [
    {
      time: "14:00",
      title: "경의선숲길 산책",
      note: "나무 그늘 따라, 가벼운 발걸음으로",
    },
    {
      time: "15:00",
      title: "골목 안 작은 카페",
      note: "창가에서 커피 한 잔과 책 몇 장",
    },
    {
      time: "16:00",
      title: "연희동 골목 구경",
      note: "작은 가게들을 구경하다가, 내키는 곳에 들러요",
    },
  ],
  taskTitle: "이번 작업",
  taskDescription: "토요일 영업 여부와 분위기를 확인하고 있어요.",
  tasks: ["후보 카페 찾기", "토요일 영업 정보 확인", "두 곳 골라 안내하기"],
  tools: "도구 호출",
  search: "웹 검색",
  read: "페이지 읽기",
  complete: "완료",
  inProgress: "진행 중",
  query: "연남동 조용한 카페",
  page: "카페 안내 · 위치와 영업시간",
  hours: "토요일 영업 정보",
  codex: "Codex",
};
const ja: ChatCopy = {
  activity: "作業状況",
  close: "閉じる",
  message: "メッセージ",
  send: "送信",
  attach: "ファイルを添付",
  latest: "最新の会話へ",
  menu: "エージェントと会話",
  detail: "エージェントの詳細",
  readOnly: "閲覧のみ",
  idle: "会話できます",
  working: "カフェを探しています",
  stopped: "作業を停止しました",
  stop: "作業を停止",
  empty: "今日は何を一緒にしましょうか？",
  input: "メッセージを送る",
  today: "今日 13:10",
  user: "土曜の午後、ソウルをのんびり歩きたい。静かなカフェにも寄って、予定は詰めすぎずに！",
  response:
    "今回は、ゆっくり過ごしましょう。気になる場所で休める、余裕のあるコースを選びました。",
  after: "静かなカフェをもう少し探しましょうか？",
  followup: "うん、土曜に開いている静かなカフェを二つ探して。",
  eyebrow: "SATURDAY WALK",
  planTitle: "延南洞を、ゆっくり",
  planSummary: "約3時間 · 徒歩2.4km · 気ままに3か所",
  planFooter: "気に入った場所だけ選んでも大丈夫です。",
  route: [
    {
      time: "14:00",
      title: "京義線の森公園を散歩",
      note: "木陰を気軽な足取りで",
    },
    {
      time: "15:00",
      title: "路地の小さなカフェ",
      note: "窓辺でコーヒーと読書",
    },
    {
      time: "16:00",
      title: "延禧洞の路地巡り",
      note: "気になる小さなお店に寄り道",
    },
  ],
  taskTitle: "今回の作業",
  taskDescription: "土曜の営業時間と雰囲気を確認しています。",
  tasks: [
    "カフェの候補を探す",
    "土曜の営業時間を確認する",
    "おすすめを二つ選ぶ",
  ],
  tools: "ツール呼び出し",
  search: "ウェブ検索",
  read: "ページを読む",
  complete: "完了",
  inProgress: "実行中",
  query: "延南洞 静かなカフェ",
  page: "カフェ情報 · 場所と営業時間",
  hours: "土曜の営業時間",
  codex: "Codex",
};
const fr: ChatCopy = {
  activity: "Activité",
  close: "Fermer",
  message: "Message",
  send: "Envoyer",
  attach: "Joindre un fichier",
  latest: "Aller aux derniers messages",
  menu: "Agents et conversations",
  detail: "Détails de l’agent",
  readOnly: "Lecture seule",
  idle: "Disponible",
  working: "Recherche de cafés",
  stopped: "Travail arrêté",
  stop: "Arrêter le travail",
  empty: "Sur quoi allons-nous travailler aujourd’hui ?",
  input: "Envoyer un message",
  today: "Aujourd’hui, 13 h 10",
  user: "J’aimerais me promener à Séoul samedi après-midi, avec un café au calme, sans trop charger le programme !",
  response:
    "Prenons notre temps. Voici un parcours qui laisse la liberté de s’arrêter dès qu’un endroit vous plaît.",
  after: "Voulez-vous d’autres cafés au calme ?",
  followup: "Oui, trouve deux cafés tranquilles ouverts le samedi.",
  eyebrow: "BALADE DU SAMEDI",
  planTitle: "Doucement, à Yeonnam",
  planSummary: "Environ 3 h · 2,4 km à pied · Trois étapes tranquilles",
  planFooter: "Choisissez simplement les endroits qui vous plaisent.",
  route: [
    {
      time: "14:00",
      title: "Parc forestier de Gyeongui",
      note: "Une balade à l’ombre des arbres",
    },
    {
      time: "15:00",
      title: "Un petit café du quartier",
      note: "Un café et quelques pages près de la fenêtre",
    },
    {
      time: "16:00",
      title: "Les ruelles de Yeonhui",
      note: "De petites boutiques, au gré de vos envies",
    },
  ],
  taskTitle: "Cette tâche",
  taskDescription: "Vérification des horaires du samedi et de l’ambiance.",
  tasks: [
    "Trouver des cafés",
    "Vérifier les horaires du samedi",
    "En recommander deux",
  ],
  tools: "Appels d’outils",
  search: "Recherche web",
  read: "Lire la page",
  complete: "Terminé",
  inProgress: "En cours",
  query: "Cafés tranquilles à Yeonnam",
  page: "Informations du café · Adresse et horaires",
  hours: "Horaires du samedi",
  codex: "Codex",
};
export function chatCopy(locale: string): ChatCopy {
  return locale.startsWith("ko")
    ? ko
    : locale.startsWith("ja")
      ? ja
      : locale.startsWith("fr")
        ? fr
        : en;
}
