<!-- qhud:languages -->
<p align="center">
  <strong>한국어</strong> · <a href="../../../../03-spec/SPEC.md">English</a> · <a href="../../../zh-CN/docs/03-spec/SPEC.md">简体中文</a>
</p>
<!-- /qhud:languages -->

<!-- qhud:anchor -->
<a id="spec--qhud"></a>

# 사양 — qhud

> 상태: 지속 갱신(v0.6.0 소스를 기준으로 전면 개정) · 날짜: 2026-09-07 · 담당: chquandogong
> qhud가 해야 하는 일을 정의합니다. ARCHITECTURE는 구현 방식을, DECISION_LOG는 이유를 설명합니다. FR-1 … FR-24는 다른 문서가 인용하는 번호를 유지합니다. FR-25 … FR-34는 v0.5.1 → v0.6.0에서 출시되었지만 사양에 행이 없었던 요구 사항입니다.

<!-- qhud:anchor -->
<a id="product-statement"></a>

## 제품 정의

다른 곳으로 전환하지 않고 한 가지 질문에 답합니다. **각 AI CLI 계정에 사용 가능한 여유가 얼마나 남아 있으며 언제 초기화되는가?** 위젯이 표시하는 다른 모든 정보는 이를 위한 것입니다. 터미널 멀티플렉서를 사용할 수 있다면 창별 화면은 더 좁은 두 번째 질문, 즉 실행 중인 각 세션이 지금 무엇을 하는지에 답합니다.

이 우선순위는 중요하며 이전과 달라졌습니다. v0.1은 사용 한도도 보여 주는 창 모니터였습니다. v0.5.0부터 운영자가 명시한 목적은 공급자의 웹페이지를 다시 열지 않고 모든 계정과 작업 공간을 한눈에 확인하는 것입니다. v0.6.0의 대표 신규 플랫폼은 창을 전혀 관찰하지 않습니다.

<!-- qhud:anchor -->
<a id="provider-vocabulary"></a>

## 공급자 용어

일부 공급자는 이름이 세 가지이며 코드가 기준입니다. 다른 모든 문서는 이 표의 대응 관계를 따라야 합니다.

| 페이로드 값 | 구역 제목 | 본문 이름 | 비고 |
| --- | --- | --- | --- |
| `claude` | CLAUDE | Claude Code | |
| `codex` | CODEX | Codex | 하나의 로그인이 여러 작업 공간을 가질 수 있습니다. |
| `agy` | AGY | Antigravity | |
| `gemini` | GEMINI | Gemini | 별도 공급자 문자열인 **동시에** agy 조회 결과 안의 한도 이름입니다. `gemini-*` 버킷은 agy의 기본 계기에 들어가고 다른 한도는 범위별 칩이 됩니다. |

<!-- qhud:anchor -->
<a id="functional-requirements"></a>

## 기능 요구 사항

<!-- qhud:anchor -->
<a id="the-widget-as-an-object-on-the-desktop"></a>

### 데스크톱 위의 위젯

| ID | 요구 사항 | 상태 |
| --- | --- | --- |
| FR-1 | 테두리 없고 투명하며 모서리가 둥근 데스크톱 계층 창: 모든 창 아래, 모든 작업 공간에 고정, 작업 표시줄과 pager에 나타나지 않음 | 완료, Linux — 2026-09-07 `_NET_WM_STATE`로 검증 |
| FR-2 | 위쪽 및 푸터 막대를 드래그해 모니터 사이 어디로든 이동 | 완료(직접 제어, D-008) |
| FR-3 | ◢ 손잡이로 크기를 조절하고 내용 재배치 | 완료(직접 제어, D-008) |
| FR-4 | 재시작 후 위치와 크기 유지 | 완료(window-state 플러그인, 약 30초마다 저장) |
| FR-10 | 트레이: 표시/숨김, 창 위 고정, 위치 초기화, 종료; 트레이가 없어도 위젯 실행 유지 | 완료(최선의 호환성) |
| FR-11 | Ctrl+휠로 70–160% 확대/축소, 저장, 포인터 전용 | 완료(D-012) |
| FR-12 | 트레이 체크와 `qhud --peek`로 계층 전환; 중복 실행 흡수 | 완료(D-012) |
| FR-26 | 위젯이 자체 화면 정지를 감지하고 운영자 개입 없이 복구: 자체 픽셀 샘플링, 숨김/표시, 이후 재실행 | 완료(v0.5.1, D-017) — Linux 전용 |
| FR-29 | Windows 경로와 숨겨진 보조 프로세스를 사용하는 Windows x64 네이티브 위젯이 WebView2로 동일한 계정 사용량 표시 | 완료(v0.6.0, D-020) — 창 관찰은 명시적으로 제외 |

<!-- qhud:anchor -->
<a id="what-it-renders"></a>

### 표시하는 내용

| ID | 요구 사항 | 상태 |
| --- | --- | --- |
| FR-5 | qmonster 파이프라인을 2초마다 관찰; 상태 배지와 컨텍스트 계기가 있는 창 타일 및 공급자별 계정 범위 사용 한도 영역 표시 | 완료(D-011) |
| FR-6 | 타일 클릭으로 설정 칩과 충돌 배너 확장; 다시 클릭하면 접힘; 선택 상태 저장 | 완료 |
| FR-7 | 계기의 심각도 구간: 60 미만 양호, 60–74 주의, 75–84 경고, 85 이상 위험 | 완료 |
| FR-8 | 관찰 사이에 초기화 카운트다운과 유휴 경과 배지를 로컬에서 갱신 | 완료 |
| FR-14 | 공급자별 사용량 영역: 공급자를 구역 제목으로, 계정과 플랜을 식별 정보 행에, 기간별 계기를 각 행에 표시 | 완료(v0.4.0) |
| FR-25 | 사용량 행을 선택하면 세부 정보가 같은 위치에 확장됨; 행 선택은 네트워크 작업을 수행하지 않음 | 완료(v0.5.1) |
| FR-31 | 행이 창을 넘으면 사용량 영역 스크롤; 긴 모델명은 자르지 않고 줄바꿈; 계기와 확장 행에서 정확한 초기화 날짜와 시각 제공 | 완료(v0.6.0) |
| FR-9 | 멀티플렉서가 없으면 실제 로컬 계정과 시각이 표시된 한도 및 창 0개 표시; 예시 데이터는 `--demo`와 DEMO 배지가 필요함; 실시간 연결은 10초마다 재시도 | 완료(v0.6.0에서 데모 대체 동작 교체, D-005) |
| FR-30 | mux 없이 시작해도 사용량을 만들어 내지 않음: 누락 값은 그대로 두고, 저장된 값이 없는 계정도 새로고침 버튼에 접근할 수 있도록 표시 | 완료(v0.6.0) |

<!-- qhud:anchor -->
<a id="whose-numbers-these-are"></a>

### 수치가 속한 계정

| ID | 요구 사항 | 상태 |
| --- | --- | --- |
| FR-13 | 모든 사용 한도 행에 소유 계정 표시: 이메일 또는 ID, 조직, 플랜, 팀 좌석의 두 가지 등급을 로컬 파일에서만 읽음 | 완료(D-013) |
| FR-15 | 연결 이력은 있지만 유효한 자격 증명이 없는 계정은 시각이 표시된 자리 표시자로 한 줄 뒤에 접어 표시하고 제거 가능; 유효한 계정은 숨기지 않음 | 완료(D-013) |
| FR-22 | registry 설정 디렉터리를 통해 여러 Claude 계정을 동시에 표시; 계정마다 자체 스냅샷과 새로고침이 있는 한 행; 조회 부분 실패는 부분 실패로 유지 | 완료(D-015) |
| FR-27 | 행 식별자는 (계정, 조직): 팀 좌석과 개인 조직을 가진 하나의 로그인은 별도 한도의 두 행이며, 새로고침 결과는 설정 디렉터리와 계정 ID로 행에 연결 | 완료(v0.5.2, D-018) — 실제 두 번째 조직 행은 현장 미검증 |
| FR-34 | 저장된 스냅샷은 현재 로그인한 계정에만 사용; 시각보다 소유 계정을 먼저 확인하여 이전 로그인의 더 최신 수치가 현재 로그인의 오래된 수치를 가리지 않음 | 완료(v0.6.0) |

D-015부터 유지되는 제한: 창의 계정을 특정할 수 없으므로 창이 제공한 계기는 항상 기본 계정 행에 속합니다.

<!-- qhud:anchor -->
<a id="getting-the-numbers"></a>

### 수치 가져오기

| ID | 요구 사항 | 상태 |
| --- | --- | --- |
| FR-16 | Claude 모델별 사용량은 명시적 버튼 또는 `--refresh-claude`로 새로고침; 타이머나 OAuth refresh grant는 사용하지 않음 | 완료(D-014) |
| FR-17 | 명시적 요청으로 Codex 작업 공간별 한도 조회; 다른 작업 공간의 응답은 잘못 표시하지 않고 제외 | 완료(v0.4.0) |
| FR-19 | Claude 사용 크레딧 지출을 공급자가 제공하는 심각도와 함께 최소 화폐 단위로 계정 행에 표시하고 해당 기능이 없는 플랜에서는 숨김 | 완료(v0.5.0) |
| FR-20 | 명시적 새로고침 결과를 재시작 후에도 보존하고 실제 출처와 경과 시간을 표시; 실시간 창 데이터 우선 | 완료(v0.5.0) |
| FR-21 | 한 버튼으로 모든 공급자를 동시에 새로고침하고 전체 상태 반영; 공급자별 버튼 유지; `--refresh-all`은 단축키에서 전달 | 완료(v0.5.0) |
| FR-23 | CLI 자체 loopback RPC로 명시적 agy 한도 새로고침: 토큰 없음, 장비 내부 통신, 운영체제로 포트 탐색; 마지막 조회 저장 | 완료(v0.5.0) |
| FR-24 | 활성 Codex 로그인의 직접 조회가 실패하면 짧게 실행되는 `codex app-server` 자식 프로세스가 응답하여 CLI가 토큰 회전을 관리하고 이 경로에서 qhud는 토큰 자체를 사용하지 않음 | 완료(v0.5.0, D-016) |
| FR-33 | 동시에 실행되는 공급자 새로고침이 서로의 저장 결과를 보존 | 완료(v0.6.0) |

<!-- qhud:anchor -->
<a id="being-trustworthy-about-it"></a>

### 신뢰성

| ID | 요구 사항 | 상태 |
| --- | --- | --- |
| FR-18 | webview 밖에서도 잘못된 출력을 확인할 수 있음: 위젯은 렌더링 구조, 표시한 텍스트와 레이블, 모든 실제 포인터 이벤트, 프런트엔드 예외를 보고하며 모든 조회 경로에 대응 CLI 명령 제공 | 완료(v0.4.0, v0.5.1 확장) |
| FR-28 | 정수 의미의 응답 숫자를 의미에 맞게 읽고, 선택적 필드 하나가 응답 전체를 실패시키지 않으며, 거부된 응답에서 형식이 바뀐 필드를 표시 | 완료(v0.5.3, D-019) |
| FR-32 | 모델별 수치만 있는 결과도 각 기간의 사용률과 초기화 시각을 재시작 후 보존; 공급자의 현재 응답에 없는 모델을 0%로 만들지 않음 | 완료(v0.6.0) |

<!-- qhud:anchor -->
<a id="non-functional-requirements"></a>

## 비기능 요구 사항

**관찰 전용.** qmonster 자체 디렉터리 아래 어디에도 쓰지 않으며 알림도 보내지 않습니다. 해당 상태와 알림은 TUI가 담당합니다(D-004).

**기본은 수동 관찰, 네트워크는 요청할 때만**(D-013, D-014, D-016). 2초 루프는 로컬 파일과 멀티플렉서를 읽으며 공급자 API 요청을 보내지 않고 관찰에 토큰을 사용하지 않은 채 계정 필드를 선택합니다. 다른 작업은 약 30초마다 창 기하를 저장하고 Linux에서 약 28초마다 픽셀을 샘플링하는 것뿐입니다. 외부에 접근하는 모든 작업은 운영자의 명시적 조작 또는 대응 CLI 명령에서 실행되며 OAuth refresh grant는 실행하지 않습니다. 조회 경로에서 qhud는 Claude와 Codex의 액세스 토큰을 읽고 agy에서는 자격 증명을 읽지 않으며 Codex 대체 경로에서는 인증을 CLI에 위임합니다.

**선택은 조회가 아닙니다.** v0.5.1부터 선택, 확장, 행 클릭은 네트워크 작업을 수행하지 않습니다. 새로고침 버튼과 전달용 플래그만 수행합니다.

**크기.** v0.6.0 Linux 릴리스 바이너리는 25 MB 예산(D-014) 대비 15.07 MiB입니다. 두 가지 기록을 바로잡습니다. 이전 수치는 모두 로컬 빌드 측정치였고, 루트가 아닌 workspace manifest의 릴리스 프로필을 cargo가 무시하여 v0.5.3에서는 25.13 MiB로 예산을 조용히 초과했습니다. 로컬 빌드가 아니라 릴리스 파일을 측정해야 합니다. Windows 수치는 기록하지 않았습니다.

**프런트엔드.** 정적 HTML, CSS, JavaScript입니다. 번들러와 node_modules가 없고 어느 플랫폼의 빌드에도 Node나 npm이 필요하지 않습니다.

**규모 범위.** 워크스테이션 한 대입니다. 창 화면은 1–12개 창에 맞춰 설계되었으며 그 이상에서는 기존 방식을 늘리기보다 상호작용 모델을 바꿔야 합니다. 계정 화면은 사용량 영역이 창보다 커질 수 있도록 설계되었습니다. 세 공급자의 계정 8개가 한 화면에 들어가지 않아 v0.6.0에서 스크롤을 추가했고, 창 0개는 이제 오류가 아닌 정상 지원 상태입니다.

<!-- qhud:anchor -->
<a id="the-payload-contract"></a>

## 페이로드 계약

Tauri 이벤트 `qhud://report` 하나를 2초마다 보냅니다. 기준은 `src-tauri/src/view.rs`이며 schema 버전은 1이고 모든 변경은 추가 방식입니다.

```jsonc
{
  "schema": 1,
  "source": "live",        // "live" | "local" | "demo"
  "backend": "herdr",      // "herdr" | "tmux" | null
  "generated_at_ms": 1788790282397,
  "poll_secs": 2,

  "quotas": [{             // one row per (provider, account, organization)
    "provider": "claude",  // claude | codex | agy | gemini
    "h5": { "pct": 49, "source": "providerofficial",
            "reset_unix": 1788438599, "of_tokens": null },
    "d7": { "pct": 7, "source": "providercache",
            "reset_unix": 1788976799, "of_tokens": null },
    "from_label": "claude:1:main",   // pane whose reading won; "" if synthesized
    "session": "dogu-3d-studio",
    "account": {
      "display": "chquan@dogu.xyz", // precomputed label → email → id
      "label": null, "email": "…", "account_id": "…",
      "org": "…", "org_type": "claude_team", "org_id": "…",
      "tiers": [{ "kind": "org", "tier": "max_5x" }],
      "plan": "team", "config_dir": null   // null ⇒ the default account
    },
    "origin": "pane",              // pane | cache | fetched
    "cache_fetched_at_ms": 1788790243688,
    "scoped": [{ "kind": "weekly_scoped", "scope": "Fable",
                 "pct": 12, "reset_unix": 1788976799 }],
    "extra": { "enabled": true, "used_minor": 4997, "currency": "USD",
               "exponent": 2, "limit_minor": null, "percent": 0,
               "severity": "normal", "limit_reached": false }
  }],

  "panes": [{
    "pane_id": "wC:p1", "label": "claude:1:main", "session": "…",
    "provider": "claude", "status": "active", "status_label": "active",
    "elapsed_secs": null, "cli_version": null, "update_hint": null,
    "model": "…", "effort": "max", "branch": null, "cwd": "~/qhud",
    "mem": "165 KB", "cost_usd": 0.0, "flags": [],
    "gauges": { "ctx": { … }, "h5": null, "d7": { … } },
    "conflicts": [{ "reason": "…", "severity": "warning",
                    "paths": ["…"], "peers": ["codex:1:review"] }]
  }],

  "summary": { "panes": 8, "conflicts": 0, "max_5h_pct": 6 },

  "account_placeholders": [{ "provider": "claude", "key": "personal-free",
                             "label": "…", "hint": "…", "plan": "free" }],
  "workspace_names":  { "<account_id>": "business" },
  "workspace_plans":  { "<account_id>": "ChatGPT Business" },
  "codex_workspaces": [{ "account_id": "…", "name": null,
                         "plan_type": "prolite", "credits_balance": "0",
                         "active": true,
                         "windows": [{ "label": "weekly", "used_percent": 41,
                                       "reset_unix": 1789147159,
                                       "scope": null }] }],
  "codex_fetched_at_ms": 1788790245717
}
```

보통 비어 있는 필드는 null로 보내지 않고 생략합니다. `account`, `origin`, `cache_fetched_at_ms`, `scoped`, `extra`, `account_placeholders`, `workspace_names`, `workspace_plans`, `codex_workspaces`, `codex_fetched_at_ms`가 이에 해당합니다. `qhud --dump`는 바로 이 페이로드를 출력합니다.

**계약 규칙.**

- **사실은 그 사실이 성립하는 범위에 표시합니다**(D-011). 사용 한도는 계정 정보이므로 계정 행마다 한 번 표시합니다. 컨텍스트와 상태는 창 정보이므로 타일에 표시합니다. 창별 `gauges.h5`/`d7`는 집계에 필요해 페이로드에 남지만 타일에서는 표시하지 않습니다.
- **단위 변환은 이 경계에서 한 번만 수행합니다.** 압박 정도는 정수 백분율, 초기화 시각은 Unix 초로 내보내며 카운트다운 문자열은 프런트엔드가 담당합니다. webview는 qmonster 타입을 보지 않습니다.
- **기간 레이블은 응답 값입니다**(`5h`, `daily`, `weekly`, `30d`, `yearly`). 인터페이스는 모든 곳에서 5H, 1D, 7D, 30D라는 동일한 기간 용어를 표시합니다. “weekly”와 “7D”는 같은 7일 이동 기간이며 하나의 사실에 이름이 두 개여서는 안 됩니다.
- **`scoped[].kind`**는 기본이 아닌 기간에 대한 공급자 용어입니다. Claude의 `session`, `weekly_all`, `weekly_scoped`와 각 공급자 추가 한도의 `pool_<label>`이 있으며 `scope`에 모델 또는 한도 이름을 담습니다. 알 수 없는 종류는 사라지는 대신 툴팁 행으로 표시됩니다.
- **`origin`은 신뢰성을 나타내는 필드입니다.** `pane`은 실시간 수치, `cache`는 CLI 자체 디스크 사본, `fetched`는 qhud의 마지막 명시적 새로고침입니다. `pane` 이외에는 경과 시간을 표시해야 하며 출처가 없으면 해당 행은 수치의 상태를 주장하지 않습니다.
- **금액은 최소 화폐 단위**에 통화와 지수를 더한 값이므로 실수형 금액 왕복 변환이 없습니다.

<!-- qhud:anchor -->
<a id="verification"></a>

## 검증

두 플랫폼에서 자동 검증을 수행하고 모두 통과해야 릴리스를 공개합니다. 형식 검사, 경고를 오류로 취급하는 린트, 전체 테스트, 릴리스 빌드가 포함됩니다. v0.6.0 테스트는 98개입니다.

조작 검증에는 절차가 있습니다(D-010). 합성 X11 입력만으로는 인정하지 않습니다. 실제 입력을 가로채던 지점인 컴포지터의 표면 선택을 우회하기 때문입니다. 컴포지터 경로 주입이나 사람의 직접 조작을 위젯 자체 추적 로그로 확인해야 합니다.

렌더링 검증에는 별도 절차가 있습니다(D-017). 추적 로그는 로직만 증명하며 실제 그리기를 증명하지 않습니다. 픽셀은 **검증할 수 있습니다**. 몇 초 간격으로 창 해시를 두 번 비교하거나 프레임 가드의 자체 로그를 읽습니다. 오류가 없다는 사실은 무언가 그려졌다는 증거가 아닙니다.

수동 체크리스트와 날짜별 근거, 아직 남은 항목은 TEST_PLAN에 있습니다.

<!-- qhud:anchor -->
<a id="out-of-scope"></a>

## 범위 밖

- **제어 동작.** qhud는 창에 키를 보내거나 CLI를 재시작하거나 공급자 설정을 바꾸지 않습니다.
- **알림.** 경고는 TUI와 공급자가 담당합니다. 두 번째 알림 주체는 중복 알림을 일으킵니다(D-004).
- **설정 UI.** 표시 이름과 등록 계정은 운영자가 소유한 파일 하나를 직접 편집합니다.
- **macOS.**
- **Windows 터미널 창의 네이티브 관찰.** Windows 위젯은 계정 사용량을 표시하며 Windows Terminal이나 PowerShell 창의 탐색 및 귀속은 구현하지 않았습니다(D-020). Linux `/proc` 기반 프로세스 지표는 Windows에서 비어 있을 수 있습니다.
- **설치 프로그램과 패키지 형식.** Linux tarball과 Windows ZIP을 제공합니다. npm, deb, AppImage, MSI가 없으며 Windows에서는 설치 프로그램, 자동 시작, 바로가기 등록을 제공하지 않습니다.
- **계기별 임계값 설정.** 심각도 구간은 디자인 목업에서 가져왔으며 설정할 수 없습니다.
- **만들어 낸 데이터.** 공급자가 반환하지 않은 한도를 생성하지 않으며 누락된 수치를 0으로 표시하지 않습니다.
