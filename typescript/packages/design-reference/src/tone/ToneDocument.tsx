import { useEffect, useState } from "react";
import {
  Alert,
  Badge,
  Button,
  Checkbox,
  Group,
  MantineProvider,
  Modal,
  Radio,
  SegmentedControl,
  Select,
  Stack,
  Switch,
  Tabs,
  Textarea,
  TextInput,
} from "@mantine/core";
import { EmailField, PasswordField } from "@chapchoo/ui/fields";
import {
  EmptyFeedback,
  ErrorFeedback,
  LoadingFeedback,
} from "@chapchoo/ui/Feedback";
import { ConfirmationDialog } from "@chapchoo/ui/ConfirmationDialog";
import { mascots, platformFonts, toneColors } from "./tokens";
import { makeReferenceTheme, referenceVariables } from "./theme";
import type { ReactElement, ReactNode } from "react";
import type { ReferencePlatform } from "./tokens";
import "@mantine/core/styles.css";
import "./tone.css";

export type ToneExample =
  | "foundations"
  | "components"
  | "states"
  | "library"
  | "planner"
  | "settings"
  | "mobile"
  | "mascots";

function Mascot({
  index = 1,
  size = "small",
}: {
  index?: number;
  size?: "small" | "large";
}): ReactElement {
  const pet = mascots[index] ?? mascots[1];
  return (
    <img
      className={`cc-mascot cc-mascot-${size}`}
      src={`./mascots/${pet?.file ?? "02-corgi.png"}`}
      alt={pet?.name ?? "코기"}
    />
  );
}

function Section({
  title,
  note,
  children,
}: {
  title: string;
  note?: string;
  children: ReactNode;
}): ReactElement {
  return (
    <section className="cc-section">
      <div className="cc-section-heading">
        <h2>{title}</h2>
        {note ? <p>{note}</p> : null}
      </div>
      {children}
    </section>
  );
}

function Status({
  children,
  intent = "neutral",
}: {
  children: ReactNode;
  intent?: "neutral" | "success" | "warning" | "danger";
}): ReactElement {
  return <span className={`cc-status cc-status-${intent}`}>{children}</span>;
}

function Foundations({
  platform,
}: {
  platform: ReferencePlatform;
}): ReactElement {
  return (
    <>
      <div className="cc-intro">
        <div>
          <div className="cc-kicker">CHAPCHOO / VISUAL FOUNDATIONS</div>
          <h1>다정한 표정, 선명한 정보.</h1>
          <p>작은 친구처럼 밝고 말랑하게. 일하는 화면은 차분하고 또렷하게.</p>
        </div>
        <Mascot size="large" />
      </div>
      <Section
        title="색은 역할을 갖습니다"
        note="크림색은 바탕, 버터색은 중요한 행동. 의미 있는 글자는 충분히 진하게."
      >
        <div className="cc-swatches">
          {Object.entries(toneColors)
            .slice(0, 13)
            .map(([key, value]) => (
              <div className="cc-swatch" key={key}>
                <div style={{ background: value }} />
                <b>{key}</b>
                <code>{value}</code>
              </div>
            ))}
        </div>
      </Section>
      <Section
        title="플랫폼에 맞는 글자"
        note={
          platform === "web"
            ? "웹: Pretendard. 화면이 좁아져도 웹 폰트는 유지합니다."
            : "iOS 표현: Helvetica + Apple SD Gothic Neo. Apple 기기에서 실폰트를 확인해야 합니다."
        }
      >
        <div className="cc-type-samples">
          <div>
            <span>Heading / 24 · 600</span>
            <h2>해야 할 일이, 가벼워지는 순간</h2>
          </div>
          <div>
            <span>Body / 16 · 400 · 1.7</span>
            <p>
              한 번에 다 끝내지 않아도 괜찮아요. 지금 필요한 정보와 다음 행동을
              또렷하게 보여줍니다.
            </p>
          </div>
          <div>
            <span>Label / 14 · 500</span>
            <b>이름 · 저장 위치 · 마지막 변경</b>
          </div>
          <code className="cc-font-stack">{platformFonts[platform]}</code>
        </div>
      </Section>
      <Section
        title="부드러운 형태와 단정한 간격"
        note="4 / 8 / 12 / 16 / 24 / 32. 모바일은 장식보다 정보량을 먼저 생각합니다."
      >
        <div className="cc-shape-row">
          <div className="cc-shape control">Control · 12</div>
          <div className="cc-shape surface">Surface · 18</div>
          <div className="cc-density-note">
            <strong>본문은 화면 폭을 충분히</strong>
            <p>
              작은 프로필은 위쪽에. 빈 좌측 열 대신 읽을 수 있는 공간을
              남깁니다.
            </p>
          </div>
        </div>
      </Section>
    </>
  );
}

interface DemoState {
  name: string;
  email: string;
  password: string;
  enabled: boolean;
  selected: string;
  checked: boolean;
  modal: "none" | "confirm" | "details";
  notice: string;
}

interface DemoProps {
  state: DemoState;
  onChange: (patch: Partial<DemoState>) => void;
}

function Components({ state, onChange }: DemoProps): ReactElement {
  return (
    <>
      <div className="cc-page-heading">
        <div className="cc-kicker">REFERENCE COMPONENTS</div>
        <h1>익숙하게 쓰고, 편하게 읽어요.</h1>
        <p>
          기본 상태와 조작 예시입니다. 실제 계정이나 제품 기능에는 연결되지
          않습니다.
        </p>
      </div>
      <Section
        title="행동"
        note="중요한 행동은 버터색, 보조 행동은 조용하게. 위험 행동은 별도로 구분합니다."
      >
        <div className="cc-panel">
          <Group gap="md" wrap="wrap">
            <Button
              onClick={() =>
                onChange({ notice: "데모: 변경 내용을 저장했습니다." })
              }
            >
              변경 저장
            </Button>
            <Button
              variant="default"
              onClick={() => onChange({ modal: "details" })}
            >
              자세히 보기
            </Button>
            <Button
              variant="subtle"
              color="dark"
              onClick={() =>
                onChange({ notice: "데모: 이전 단계로 돌아갑니다." })
              }
            >
              나중에
            </Button>
            <Button color="red" onClick={() => onChange({ modal: "confirm" })}>
              항목 삭제
            </Button>
            <Button disabled>선택 필요</Button>
            <Button loading>저장 중</Button>
          </Group>
        </div>
      </Section>
      <Section
        title="입력"
        note="레이블은 항상 남기고, 오류는 고칠 위치 가까이에 둡니다."
      >
        <div className="cc-fields">
          <TextInput
            label="표시 이름"
            value={state.name}
            onChange={(event) => onChange({ name: event.currentTarget.value })}
            placeholder="이름을 입력하세요"
          />
          <EmailField
            label="이메일"
            value={state.email}
            onValueChange={(email) => onChange({ email })}
            placeholder="you@example.com"
          />
          <PasswordField
            label="비밀번호"
            value={state.password}
            onValueChange={(password) => onChange({ password })}
            visibilityToggleLabel="비밀번호 표시 전환"
          />
          <TextInput
            label="오류 상태"
            defaultValue="hello@"
            error="이메일 주소를 확인해 주세요."
          />
          <Select
            label="저장 위치"
            defaultValue="personal"
            allowDeselect={false}
            data={[
              { value: "personal", label: "개인 공간" },
              { value: "team", label: "팀 공간" },
            ]}
          />
          <Textarea
            label="짧은 메모"
            placeholder="기억해둘 내용을 적어주세요"
            autosize
            minRows={2}
          />
        </div>
      </Section>
      <Section title="선택과 구분">
        <div className="cc-panel">
          <Group gap="xl" wrap="wrap">
            <Checkbox
              label="완료한 항목 함께 보기"
              checked={state.checked}
              onChange={(event) =>
                onChange({ checked: event.currentTarget.checked })
              }
            />
            <Switch
              label="새 소식 알림"
              checked={state.enabled}
              onChange={(event) =>
                onChange({ enabled: event.currentTarget.checked })
              }
            />
            <Radio.Group
              label="공개 범위"
              value={state.selected}
              onChange={(selected) => onChange({ selected })}
            >
              <Group mt="sm">
                <Radio value="private" label="나만" />
                <Radio value="team" label="팀원" />
              </Group>
            </Radio.Group>
          </Group>
          <div className="cc-inline-space">
            <SegmentedControl
              defaultValue="all"
              data={[
                { label: "전체", value: "all" },
                { label: "저장됨", value: "saved" },
                { label: "보관됨", value: "archived" },
              ]}
            />
          </div>
        </div>
      </Section>
      <Section title="내비게이션과 상태">
        <div className="cc-panel">
          <Tabs defaultValue="overview">
            <Tabs.List>
              <Tabs.Tab value="overview">개요</Tabs.Tab>
              <Tabs.Tab value="activity">활동</Tabs.Tab>
            </Tabs.List>
            <Tabs.Panel value="overview" pt="lg">
              <Group gap="sm">
                <Status>초안</Status>
                <Status intent="success">✓ 저장 완료</Status>
                <Status intent="warning">! 확인 필요</Status>
                <Status intent="danger">× 연결 실패</Status>
              </Group>
            </Tabs.Panel>
            <Tabs.Panel value="activity" pt="lg">
              데모 활동: 메모 2개를 정리했습니다.
            </Tabs.Panel>
          </Tabs>
        </div>
      </Section>
    </>
  );
}

function States({ onChange }: DemoProps): ReactElement {
  return (
    <>
      <div className="cc-page-heading">
        <div className="cc-kicker">FEEDBACK & RECOVERY</div>
        <h1>무슨 일인지, 다음엔 뭘 할지.</h1>
        <p>색만으로 구분하지 않고, 상태와 다음 행동을 함께 보여줍니다.</p>
      </div>
      <Section title="불러오는 중">
        <div className="cc-panel">
          <LoadingFeedback message="저장된 자료를 불러오고 있어요." />
        </div>
      </Section>
      <Section title="아직 비어 있을 때">
        <div className="cc-empty">
          <Mascot index={0} />
          <EmptyFeedback
            title="첫 자료를 담아볼까요?"
            message="다시 보고 싶은 링크나 짧은 메모를 저장해보세요."
            action={
              <Button
                onClick={() =>
                  onChange({ notice: "데모: 자료 추가 화면입니다." })
                }
              >
                자료 추가
              </Button>
            }
          />
        </div>
      </Section>
      <Section title="문제가 생겼을 때">
        <ErrorFeedback
          title="자료를 불러오지 못했어요"
          message="기존 자료는 그대로 있어요. 연결 상태를 확인하고 다시 시도해 주세요."
          retryLabel="다시 시도"
          onRetry={() =>
            onChange({ notice: "데모: 다시 불러오기를 요청했습니다." })
          }
        />
      </Section>
      <Section title="확인이 필요한 행동">
        <div className="cc-panel">
          <p>
            삭제되는 항목과 영향을 먼저 알려줍니다. 평범한 편집마다 확인창을
            띄우지는 않습니다.
          </p>
          <Button
            color="red"
            variant="outline"
            onClick={() => onChange({ modal: "confirm" })}
          >
            삭제 확인 예시
          </Button>
        </div>
      </Section>
    </>
  );
}

const readingItems = [
  {
    title: "좋은 인터페이스의 작은 디테일",
    kind: "아티클",
    info: "8분 읽기 · 오늘 저장",
    tone: "warm",
    tag: "디자인",
  },
  {
    title: "서울에서 천천히 걷기 좋은 길",
    kind: "장소 모음",
    info: "5개 장소 · 어제 저장",
    tone: "green",
    tag: "주말",
  },
  {
    title: "이번 달에 읽고 싶은 책들",
    kind: "메모",
    info: "책 4권 · 10월 28일",
    tone: "cream",
    tag: "읽을거리",
  },
  {
    title: "혼자서도 꾸준히 만드는 방법",
    kind: "아티클",
    info: "12분 읽기 · 10월 26일",
    tone: "blue",
    tag: "만들기",
  },
  {
    title: "작업실의 작은 식물 기록",
    kind: "메모",
    info: "사진 3장 · 10월 24일",
    tone: "green",
    tag: "일상",
  },
];

function Library({ onChange }: DemoProps): ReactElement {
  return (
    <>
      <div className="cc-product-heading">
        <div>
          <div className="cc-kicker">LITTLE SHELF / FICTIONAL PRODUCT</div>
          <h1>다시 보고 싶은 것들</h1>
          <p>좋았던 발견들을, 한곳에 차곡차곡.</p>
        </div>
        <Button
          onClick={() => onChange({ notice: "데모: 새 자료를 추가합니다." })}
        >
          + 자료 담기
        </Button>
      </div>
      <div className="cc-toolbar">
        <TextInput
          aria-label="자료 찾기"
          placeholder="제목이나 태그로 찾기"
          value={undefined}
          onChange={(event) =>
            onChange({
              notice: event.currentTarget.value
                ? `데모 검색어: ${event.currentTarget.value}`
                : "",
            })
          }
        />
        <SegmentedControl
          defaultValue="all"
          data={[
            { label: "전체 12", value: "all" },
            { label: "읽는 중 3", value: "reading" },
          ]}
        />
      </div>
      <div className="cc-list-label">
        <span>최근 저장한 자료</span>
        <span>5개 예시</span>
      </div>
      <div className="cc-library-list">
        {readingItems.map((item, i) => (
          <button
            className="cc-library-row"
            key={item.title}
            onClick={() => onChange({ modal: "details", notice: "" })}
          >
            <div
              className={`cc-object cc-object-${item.tone}`}
              aria-hidden="true"
            >
              {i === 1 ? "↗" : i === 2 ? "≡" : "▤"}
            </div>
            <div className="cc-row-content">
              <strong>{item.title}</strong>
              <span>
                {item.kind} · {item.info}
              </span>
            </div>
            <span className="cc-tag">{item.tag}</span>
            <span className="cc-row-arrow" aria-hidden="true">
              ↗
            </span>
          </button>
        ))}
      </div>
      <div className="cc-helper">
        <Mascot index={4} />
        <p>
          다 읽지 않아도 괜찮아요.
          <br />
          <strong>다음에 떠올릴 수 있으면 충분하니까요.</strong>
        </p>
      </div>
    </>
  );
}

function Planner({ state, onChange }: DemoProps): ReactElement {
  return (
    <>
      <div className="cc-product-heading">
        <div>
          <div className="cc-kicker">SMALL STEPS / FICTIONAL PRODUCT</div>
          <h1>조금씩, 이번 주</h1>
          <p>해야 할 일 사이에 쉬어갈 시간도 남겨요.</p>
        </div>
        <Status intent="success">✓ 변경 저장됨</Status>
      </div>
      <div className="cc-week">
        <span>10월 26일 — 11월 1일</span>
        <div>
          <Button
            variant="default"
            size="xs"
            onClick={() => onChange({ notice: "데모: 이전 주" })}
          >
            ←
          </Button>
          <Button
            variant="default"
            size="xs"
            onClick={() => onChange({ notice: "데모: 다음 주" })}
          >
            →
          </Button>
        </div>
      </div>
      <div className="cc-planner">
        <div className="cc-task-group">
          <h2>
            오늘 <span>금요일, 30일</span>
          </h2>
          <label className="cc-task">
            <Checkbox
              aria-label="책상 위 정리하기 완료"
              checked={state.checked}
              onChange={(event) =>
                onChange({ checked: event.currentTarget.checked })
              }
            />
            <div>
              <strong>책상 위 정리하기</strong>
              <small>10분이면 충분해요</small>
            </div>
            <span className="cc-tag">일상</span>
          </label>
          <label className="cc-task">
            <Checkbox aria-label="읽던 글 마무리하기 완료" />
            <div>
              <strong>읽던 글 마무리하기</strong>
              <small>좋았던 문장 하나 남기기</small>
            </div>
            <span className="cc-tag">읽기</span>
          </label>
          <button
            className="cc-add-row"
            onClick={() => onChange({ notice: "데모: 할 일을 추가합니다." })}
          >
            + 작은 할 일 추가
          </button>
        </div>
        <div className="cc-task-group">
          <h2>
            주말에 <span>서두르지 않아도 되는 일</span>
          </h2>
          <label className="cc-task">
            <Checkbox aria-label="동네 한 바퀴 걷기 완료" />
            <div>
              <strong>동네 한 바퀴 걷기</strong>
              <small>날씨가 좋으면, 카페도 들르기</small>
            </div>
            <span className="cc-tag">쉬기</span>
          </label>
          <label className="cc-task">
            <Checkbox aria-label="다음 주의 나에게 메모 완료" />
            <div>
              <strong>다음 주의 나에게 메모</strong>
              <small>잊고 싶지 않은 생각 세 가지</small>
            </div>
          </label>
        </div>
      </div>
      <div className="cc-inline-note">
        <Mascot index={2} />
        <p>
          <strong>오늘 할 일 두 개.</strong> 전부 채우는 것보다, 하나씩 가볍게
          시작해봐요.
        </p>
      </div>
    </>
  );
}

function Settings({ state, onChange }: DemoProps): ReactElement {
  return (
    <>
      <div className="cc-page-heading">
        <div className="cc-kicker">COZY SPACE / FICTIONAL PRODUCT</div>
        <h1>내 공간을 편하게</h1>
        <p>나에게 맞는 작은 설정들을 골라주세요.</p>
      </div>
      <Section title="기본 정보">
        <div className="cc-settings-form">
          <TextInput
            label="공간 이름"
            value={state.name}
            onChange={(event) => onChange({ name: event.currentTarget.value })}
          />
          <EmailField
            label="알림 받을 이메일"
            value={state.email}
            onValueChange={(email) => onChange({ email })}
          />
          <div className="cc-settings-actions">
            <Button
              onClick={() => onChange({ notice: "데모: 설정을 저장했습니다." })}
            >
              변경 저장
            </Button>
            <span>이 화면은 가상 제품 예시입니다.</span>
          </div>
        </div>
      </Section>
      <Section title="알림">
        <div className="cc-setting-row">
          <div>
            <strong>일주일 돌아보기</strong>
            <p>저장한 것들과 작은 기록들을 모아봐요.</p>
          </div>
          <Switch
            aria-label="일주일 돌아보기"
            checked={state.enabled}
            onChange={(event) =>
              onChange({ enabled: event.currentTarget.checked })
            }
          />
        </div>
        <div className="cc-setting-row">
          <div>
            <strong>새로운 기능 소식</strong>
            <p>필요한 변화가 있을 때만 알려드려요.</p>
          </div>
          <Switch aria-label="새로운 기능 소식" />
        </div>
      </Section>
      <Section title="데이터 관리">
        <div className="cc-setting-row">
          <div>
            <strong>자료 내보내기</strong>
            <p>저장한 자료를 파일로 보관해요.</p>
          </div>
          <Button
            variant="default"
            onClick={() => onChange({ notice: "데모: 파일 내보내기" })}
          >
            내보내기
          </Button>
        </div>
      </Section>
    </>
  );
}

function ContentComposition(): ReactElement {
  return (
    <>
      <div className="cc-page-heading">
        <div className="cc-kicker">SLOW WEEKEND / FICTIONAL COMPOSITION</div>
        <h1>도시에서, 느긋한 하루.</h1>
        <p>주말의 발견들을 모아보는 가상의 화면입니다.</p>
      </div>
      <div className="cc-message">
        <div className="cc-identity">
          <Mascot />
          <div>
            <strong>코기</strong>
            <span>방금 정리한 내용</span>
          </div>
        </div>
        <div className="cc-message-body">
          <p>
            이번 주말엔 많이 하기보다 천천히 걸어볼까요? 저장해둔 곳 중에서
            쉬어가기 좋은 세 가지를 모았어요.
          </p>
          <div className="cc-compact-output">
            <h2>서두르지 않는 토요일</h2>
            <div>
              <strong>01</strong>
              <p>
                나무 그늘을 따라 걷기
                <span>
                  경의선숲길에서 시작해요. 벤치에서 잠깐 쉬어가도 좋아요.
                </span>
              </p>
            </div>
            <div>
              <strong>02</strong>
              <p>
                골목 안 작은 카페
                <span>읽던 책 한 권과 커피. 창가 자리가 있다면 더 좋겠죠.</span>
              </p>
            </div>
            <div>
              <strong>03</strong>
              <p>
                다음 주의 나에게 엽서
                <span>작은 문구점에 들러 오늘의 기분을 한 줄 남겨봐요.</span>
              </p>
            </div>
          </div>
          <p>세 곳을 다 가지 않아도 괜찮아요. 마음이 가는 곳만 골라보세요.</p>
          <details className="cc-details">
            <summary>저장한 메모 3개</summary>
            <ul>
              <li>그늘이 있는 산책길</li>
              <li>다시 가고 싶은 카페</li>
              <li>골목의 작은 가게들</li>
            </ul>
          </details>
        </div>
      </div>
    </>
  );
}

function MascotFamily(): ReactElement {
  return (
    <>
      <div className="cc-page-heading">
        <div className="cc-kicker">SMALL COMPANIONS</div>
        <h1>같은 말랑함, 다섯 가지 표정.</h1>
        <p>견종은 색과 귀로만. 커다란 주둥이나 과한 털감 없이 한 덩어리로.</p>
      </div>
      <div className="cc-mascot-grid">
        {mascots.map((pet, index) => (
          <div className="cc-mascot-tile" key={pet.key}>
            <Mascot index={index} size="large" />
            <h2>{pet.name}</h2>
            <p>{pet.key}</p>
          </div>
        ))}
      </div>
      <div className="cc-panel">
        <h2>화면에 필요한 만큼만</h2>
        <p>
          화자·정체성·의미 있는 빈 상태에 사용합니다. 정보가 많은 모바일
          화면에서는 작은 프로필이면 충분합니다.
        </p>
        <p>
          선택한 래스터 시안이며, 최종 벡터·애니메이션 라이브러리나 견종별 역할
          설정은 아닙니다.
        </p>
      </div>
    </>
  );
}

function Example({
  example,
  platform,
  state,
  onChange,
}: DemoProps & {
  example: ToneExample;
  platform: ReferencePlatform;
}): ReactElement {
  switch (example) {
    case "foundations":
      return <Foundations platform={platform} />;
    case "components":
      return <Components state={state} onChange={onChange} />;
    case "states":
      return <States state={state} onChange={onChange} />;
    case "library":
      return <Library state={state} onChange={onChange} />;
    case "planner":
      return <Planner state={state} onChange={onChange} />;
    case "settings":
      return <Settings state={state} onChange={onChange} />;
    case "mobile":
      return <ContentComposition />;
    case "mascots":
      return <MascotFamily />;
  }
}

/** Small reference-only interaction container, separate from pure compositions. */
export default function ToneDocument({
  example,
  platform,
}: {
  example: ToneExample;
  platform: ReferencePlatform;
}): ReactElement {
  const [state, setState] = useState<DemoState>({
    name: "나의 작은 공간",
    email: "hello@example.com",
    password: "",
    enabled: true,
    selected: "private",
    checked: false,
    modal: "none",
    notice: "",
  });
  useEffect(() => {
    const previous = document.documentElement.lang;
    document.documentElement.lang = "ko";
    return (): void => {
      document.documentElement.lang = previous;
    };
  }, []);
  const onChange = (patch: Partial<DemoState>): void =>
    setState((current) => ({ ...current, ...patch }));
  const product =
    example === "library" || example === "planner" || example === "settings";
  return (
    <MantineProvider
      theme={makeReferenceTheme(platform)}
      cssVariablesResolver={referenceVariables}
      forceColorScheme="light"
    >
      <div className="tone-page" data-platform={platform} lang="ko">
        <div className="cc-review-bar">
          <span>CHAPCHOO DESIGN REFERENCE</span>
          <span>
            {platform === "ios"
              ? "iOS typography · Apple device required"
              : "Web · Pretendard"}{" "}
            · {product ? "가상 제품" : "디자인 예시"}
          </span>
        </div>
        <div className={product ? "cc-product-layout" : "cc-reference-layout"}>
          {product ? (
            <aside className="cc-product-nav">
              <a
                className="cc-brand"
                href="?id=chapchoo-foundations--overview&viewMode=story&tone-document"
              >
                ✳ <strong>chapchoo</strong>
              </a>
              <div className="cc-nav-caption">가상 제품 예시</div>
              {[
                ["library", "자료 모아보기"],
                ["planner", "작은 계획"],
                ["settings", "공간 설정"],
              ].map(([key, label]) => (
                <a
                  key={key}
                  className={example === key ? "active" : ""}
                  aria-current={example === key ? "page" : undefined}
                  href={`?id=chapchoo-products--${key ?? "library"}&viewMode=story&tone-document`}
                >
                  {label}
                </a>
              ))}
              <div className="cc-nav-pet">
                <Mascot />
                <p>조금 가벼운 하루를.</p>
              </div>
            </aside>
          ) : null}
          <main className="cc-content">
            <Example
              example={example}
              platform={platform}
              state={state}
              onChange={onChange}
            />
            {state.notice ? (
              <Alert
                className="cc-notice"
                role="status"
                title="샘플 피드백"
                withCloseButton
                onClose={() => onChange({ notice: "" })}
              >
                {state.notice}
              </Alert>
            ) : null}
          </main>
        </div>
        <ConfirmationDialog
          opened={state.modal === "confirm"}
          title="이 항목을 삭제할까요?"
          message="가상 항목 하나가 목록에서 사라집니다. 실제 저장된 자료에는 영향을 주지 않는 확인창 예시입니다."
          confirmLabel="삭제"
          cancelLabel="취소"
          destructive
          onConfirm={() =>
            onChange({ modal: "none", notice: "데모: 항목을 삭제했습니다." })
          }
          onCancel={() => onChange({ modal: "none" })}
        />
        <Modal
          opened={state.modal === "details"}
          onClose={() => onChange({ modal: "none" })}
          title="자료 미리보기"
          closeButtonProps={{ "aria-label": "닫기" }}
          centered
        >
          <Stack>
            <Badge variant="light" color="gray">
              가상 제품 샘플
            </Badge>
            <p>좋은 인터페이스의 작은 디테일</p>
            <p>
              본문·보조 정보·다음 행동의 위계를 살펴보기 위한 정적 예시입니다.
            </p>
            <Button onClick={() => onChange({ modal: "none" })}>닫기</Button>
          </Stack>
        </Modal>
      </div>
    </MantineProvider>
  );
}
