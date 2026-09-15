<!-- qhud:languages -->
<p align="center">
  <strong>한국어</strong> · <a href="../../../../04-quality/TEST_PLAN.md">English</a> · <a href="../../../zh-CN/docs/04-quality/TEST_PLAN.md">简体中文</a>
</p>
<!-- /qhud:languages -->

<!-- qhud:anchor -->
<a id="test_plan"></a>

# 테스트 계획

> 상태: v0.7.1 자동 검증 및 Linux 시스템 지표 근거 기록; 데스크톱 미검증 사항 유지 · 날짜: 2026-09-14 · 담당: chquandogong

<!-- qhud:anchor -->
<a id="automated-gates"></a>

## 자동 검증 기준

CI는 `main`과 `codex/**` push 및 pull request에서 실행합니다. Ubuntu 24.04는 기준 고정 Git 의존성으로 다음을 실행합니다.

```bash
cargo fmt --all --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
node --test tests/system-metrics.test.cjs
cargo build --locked --release
```

Windows MSVC는 빌드 스크립트를 통해 범위가 한정된 의존성 패치를 사용합니다.

```powershell
./scripts/Build-Windows.ps1 -Test -VisualStudioPath '<Visual Studio installation>'
git diff --exit-code -- Cargo.toml Cargo.lock
```

CI는 Node 22를 쓰는 별도의 Ubuntu `repository-integrity` 작업도 실행합니다.

```bash
node scripts/check-repository.mjs
```

이 명령은 패키지·Tauri·lockfile 버전 일치, 한국어 README 사본의 바이트 단위 일치, 현재 릴리스 노트와 설치 예시, 영어·한국어·중국어 간체 문서 파일 집합과 릴리스 노트 언어 링크, 로컬 Markdown 링크 대상을 검사합니다. 문서만 바꿨다면 Rust를 컴파일하지 않고 로컬에서 이 명령을 실행할 수 있습니다. push 또는 pull request에서는 CI가 모든 작업을 실행합니다.

`-Test`는 같은 의존성 환경에서 qhud Rust 테스트 후 빌드합니다. Node 22에서 `node --test tests/system-metrics.test.cjs`를 실행해 프런트엔드 시스템 지표 도우미도 검사합니다. 스크립트는 Windows 명령 후 기준 lockfile을 복원합니다. Linux 빌드에는 형제 checkout이나 Windows 패치가 필요하지 않습니다. Ubuntu와 Windows 테스트 및 빌드가 모두 성공해야 릴리스를 공개합니다.

2026-09-07: `1514e09`의 [CI 34117359064](https://github.com/chquandogong/qhud/actions/runs/34117359064)가 통과했습니다. Ubuntu fmt/clippy, **98/98 테스트**, 릴리스 빌드; Windows **98/98 테스트**, 릴리스 빌드, 기준 의존성 파일 검사가 통과했습니다. 로컬 Windows 스크립트도 98/98 테스트와 릴리스 빌드를 통과했습니다.

2026-09-11, v0.7.1 Ubuntu 로컬 검증에서 형식과 clippy 이상 없음, **Rust 테스트 127/127**, **Node 테스트 6/6**, 릴리스 빌드 2분 08초를 통과했습니다. 이는 로컬 트리 근거이며 릴리스 workflow 실행이 공개 기록입니다.

2026-09-07, 공개 v0.6.0 Linux 배포 파일의 Ubuntu 재검증입니다. v0.6.0에서 누락으로 기록한 검증을 수행했습니다. 릴리스 tarball을 내려받아 SHA-256이 제공 파일과 일치함을 확인하고 그 정확한 바이너리를 `~/.local/bin/qhud`에 설치해 배포 파일과 해시를 비교한 뒤 기준 장비에서 재실행했습니다. 환경: Ubuntu 24.04.3, GNOME Shell 46.0 Wayland, kernel 7.0.0-28, eDP-1 2560×1600 + HDMI-1 3840×2160 모두 scale 1, webkit2gtk 2.52.6, herdr 0.7.5, rustc 1.94.1. 같은 트리의 로컬 검증도 fmt 이상 없음, clippy `-D warnings` 이상 없음, 98/98 테스트를 통과했고 형제 checkout 없이 고정 Git 의존성으로 릴리스 빌드를 6분 21초에 완료했습니다.

<!-- qhud:anchor -->
<a id="unit-coverage"></a>

## 단위 테스트 범위

- `view.rs`: 백분율 범위 제한/반올림, 사람이 읽기 쉬운 바이트 표시, `~` 축약, 레이블 자르기. 전체 `PaneReport` 생성은 업스트림 비공개이므로 매핑은 아래 실사용 체크리스트로 검증합니다.
- 계정 전환 후 저장 한도의 소유 계정, mux가 실행되지 않을 때의 로컬 계정 행, 재시작 후 범위별 수치만 또는 추가 사용량만 있는 스냅샷.
- Codex 모델 한도는 두 공급자 파서와 계정 행 병합을 거쳐 모델명, 5H/7D 기간, 사용량, 초기화 시각을 보존합니다. 검증 데이터는 현재 실제 계정에 그 한도가 있음을 증명하지 않습니다.
- 시스템 지표 테스트는 준비 단계, 카운터 초기화, 제한되고 시각이 있는 기록, 긴 공백의 속도 거부 및 기록 초기화 도우미, 디스크·네트워크 집계 필터, 어댑터 선택, 플랫폼 GPU 파서를 다룹니다. `tests/system-metrics.test.cjs`는 null과 0의 구별, 고정 기록 슬롯, 적응형 속도 축을 검사합니다.
- About 테스트는 제작자와 저장소라는 이름 외의 모든 링크 대상을 거부합니다. 버전은 Cargo 빌드 메타데이터에서 가져옵니다.

<!-- qhud:anchor -->
<a id="manual-verification-checklist--system-metrics-and-about"></a>

## 수동 검증 체크리스트 — 시스템 지표와 About

| 확인 항목 | 동작 | 통과 기준 — 근거 날짜 |
| --- | --- | --- |
| 시스템 진단 | `qhud --system-dump` 실행 | 두 샘플을 기다린 뒤 시스템 전용 JSON 출력; 사용할 수 없는 카운터는 0이 아니라 null |
| 기록과 세부 정보 | 60초간 관찰하고 표시된 지표를 포인터와 키보드로 각각 선택 | 최대 30개 순서 있는 지점; 올바른 단위/세부 정보; 두 번째 선택으로 닫힘 |
| 숨김/재개 | 10초 넘게 숨기거나 최소화한 뒤 표시 | 이전 기록과 속도 기준이 지워지고 재개 급증이 없음 |
| 선택 GPU | 지원 어댑터와 독립 OS/드라이버 카운터 비교 | 같은 어댑터와 비슷한 구간; 미지원 GPU는 숨김 |
| About | qhud 이름을 열고 닫기 경로와 두 링크 확인 | 내장 버전이 정확하고 포커스가 돌아오며 홈페이지/저장소만 열림 |

2026-09-11 Ubuntu에서 설치한 v0.7.1 빌드가 CPU, 메모리, Intel GPU, 디스크, 네트워크 기록을 표시했습니다. `--system-dump`의 Intel Arc GPU 24.8%와 같은 구간의 독립 i915 rc6 유휴 잔류 계산 24.8%가 일치했습니다. 이는 Intel/i915 호스트 한 대의 근거이며 모든 어댑터나 OS를 검증하지는 않습니다.

<!-- qhud:anchor -->
<a id="manual-verification-checklist--ubuntu-desktop-layer"></a>

## 수동 검증 체크리스트 — Ubuntu 데스크톱 계층

창 계층을 바꾼 뒤 대상 장비에서 실행합니다.

| 확인 항목 | 명령 / 동작 | 통과 기준 — 근거 날짜 |
| --- | --- | --- |
| Below + sticky + skip | `xprop -id $(xdotool search --class qhud \| tail -1) _NET_WM_STATE _NET_WM_DESKTOP` | 네 상태가 모두 있고 desktop은 `4294967295` — **2026-09-07 공개 v0.6.0 배포 파일에서 통과** |
| 창 아래 유지 | 아무 앱이나 위젯 위로 드래그 | 위젯이 위로 올라오지 않음 |
| 작업 공간 고정 | 작업 공간 전환 | 모든 작업 공간에서 위젯 표시 |
| 모니터 사이 이동 | 상단 막대를 다른 모니터로 드래그 | 재시작 후 위치 유지 |
| 크기 조절 | ◢ 손잡이 드래그 | 내용 재배치; 재시작 후 유지 |
| 로컬 대체 | 활성 tmux/herdr 서버 중지 | 실제 계정과 시각이 표시된 한도 유지; 예시 창이나 `DEMO` 배지 없음 |
| 기본은 실제 데이터 | `qhud --dump` 및 `qhud --demo --dump` | 플래그가 없으면 `source`는 실제 창을 담은 `live`/`local`, 있으면 `source`는 목업 창 3개의 `demo` — **2026-09-07 통과** |
| 명시적 데모 | 기존 인스턴스를 닫은 뒤 `qhud --demo` 실행 | `DEMO` 배지; 목업과 같은 타일 |
| 실시간 복구 | tmux + AI CLI 시작 | ≤12초 안에 실시간 데이터(10초 재탐색 + 2초 관찰) — 2026-08-05 |
| 실시간 관찰 | 위젯 stderr의 시작 행 읽기 | 창 레이블 없이 창 수만 포함한 `live via herdr` — **2026-09-07 통과**(창 8개) |
| 모든 공급자 새로고침 | `qhud --refresh-all` 후 위젯 stderr 읽기 | 세 공급자 모두 오류 없이 응답하며 로그에는 집계 수만 있고 계정 ID·사용량 값이 없음 |
| 실제 픽셀 갱신 | `xwd -id <window> \| md5sum`을 몇 초 간격으로 두 번 | 두 해시가 **다름**(푸터 시계는 매초 갱신) — **2026-09-07 통과**; 같으면 프레임 정지(D-017) |
| 프레임 가드 활성 | 위젯 stderr에서 `frame guard armed` 검색 | 프로세스 시작마다 정확히 한 행 — **2026-09-07 통과**; 없으면 샘플러 실패로 정지를 감지하지 못함 |
| 시각적 일치 | `docs/assets/widget-*.png`와 비교 | 색상/타일/계기/배지가 목업과 일치 |
| GNOME 개요 | Activities / 작업 공간 제스처 열기 | ⏳ 위젯이 창으로 나타날 수 있음(수용한 특성 R2); 이후 아래 계층으로 복귀해야 함 |
| 잠금 / 해제 | 화면 잠금 후 해제 | ⏳ 아래 계층 + sticky 유지(`xprop` 재확인) |
| 절전 / 복귀 | 절전 후 복귀 | ⏳ 잠금/해제와 같음 |
| 전체 화면 앱 | 위젯 모니터에서 창을 전체 화면으로 전환 | ⏳ 위젯이 비쳐 나오지 않음 |
| 모니터 핫플러그 | 외부 모니터 분리/재연결 | ⏳ 트레이 → Reset position으로 복구 가능 |

⏳ 행은 Codex 교차 검증(CV 로그)에서 추가했으며 첫 실기 검증을 기다리고 있습니다. 그중 잠금/해제와 절전/복귀는 체크리스트가 아니라 현장에서 발견한 D-017 화면 정지를 일으키는 바로 그 디스플레이 절전 조건입니다. 실행 전까지 해당 경로의 유일한 근거는 프레임 가드 현장 집계입니다. journal 수집 기간 2026-08-26 → 09-07에 **감지 28회, 첫 단계 복구 28회, 재실행 0회, 사용자에게 드러난 사고 0회**입니다.

2026-08-05(D-008): 직접 제어하는 기하 구현으로 두 모니터에서 클릭 전달, 드래그 이동, 손잡이 크기 조절을 픽셀 단위로 재검증했습니다(합성 입력 근거는 DECISION_LOG D-008).

<!-- qhud:anchor -->
<a id="manual-verification-checklist--windows-and-account-usage"></a>

## 수동 검증 체크리스트 — Windows와 계정 사용량

- 네이티브 WebView2 앱을 시작해 이동하고 크기를 조절하며 트레이, 기존 인스턴스의 `--peek`, 명시적 새로고침 버튼을 확인합니다.
- tmux/herdr 백엔드 없이 창 0개와 실제 계정 행을 확인합니다. 재시작 후 새로고침 결과에 시각이 유지되며 예시 사용량으로 대체하지 않습니다.
- 창을 줄이거나 계정/모델 행을 여러 개 사용합니다. 스크롤로 모든 사용 한도와 초기화 시간에 접근할 수 있고 창 및 푸터 조작부가 남아 있어야 합니다.
- 읽을 수 있는 모델명과 초기화 카운트다운을 확인합니다. 서버에 없는 모델 한도는 표시하지 않으며 0 채우기나 다른 계정의 사용량 표시가 없어야 합니다.
- 백그라운드 CLI 탐색이 콘솔 창을 열지 않는지 확인합니다. Windows 터미널 탭의 네이티브 모니터링은 현재 지원 범위 밖입니다.

설치 후 로컬 Windows v0.6.0 화면, 계정 이메일, 모델별/초기화 행, 명시적 Codex 새로고침, 실제 app-server 대체 경로를 검증했습니다. 설치 실행 파일 SHA-256: `3D688DB2F90E6D4780C0F016CB0CED1572189AC83FC1658F73A6688D1477F9FF`. 이는 별도 GitHub 릴리스 파일이 아닌 로컬 MSVC 빌드를 식별합니다.

최종 태그 `v0.6.0`(`cfdd850`)은 [main CI](https://github.com/chquandogong/qhud/actions/runs/34127575130)와 [Release](https://github.com/chquandogong/qhud/actions/runs/34127575141)를 통과했습니다. 공개 압축 파일 두 개를 내려받아 각각 SHA-256 파일과 일치함을 확인했습니다. 공개 Windows 실행 파일을 설치했으며 SHA-256은 `1233E4E7D58B8E6A0C855D3B528C7810FFD99DA476509F9C19509A3B5F094917`입니다. 설치된 공개 프로세스는 응답하며 계정 행을 내보냅니다. Windows 세션이 잠금 화면으로 전환되어 최종 컴포지터 스크린샷은 미완료이며 앞선 로컬 빌드 시각 확인이 직접 픽셀 근거로 남아 있습니다.

<!-- qhud:anchor -->
<a id="ubuntu-input-verification-protocol-mandatory-since-d-010"></a>

## Ubuntu 입력 검증 절차 (D-010부터 필수)

**조작이 작동한다는 주장에 XTEST(xdotool)만 사용하는 것은 인정하지 않습니다**. XWayland 내부에 주입되어 실제 입력을 가로채던 지점인 Mutter의 표면 선택을 우회하기 때문입니다(D-010). 모든 “조작이 작동한다”는 주장에는 **컴포지터 경로 주입**이 필요합니다. Mutter RemoteDesktop 절대 포인터 클릭(D-010에 기록한 `rd_abs_click.py` 방법) 또는 사람이 직접 조작한 뒤 `qhud ui:` stderr 추적 로그로 확인해야 합니다.

2026-08-06(D-010, v0.1.4): DING 비활성 상태에서 컴포지터 경로의 선택 전환 왕복(`sel:none:-` → `sel:wC:p3:R`)을 검증했습니다. 같은 클릭도 DING을 활성화하면 사라집니다(A/B).

<!-- qhud:anchor -->
<a id="non-regression-invariants"></a>

## 회귀 방지 불변 조건

- qhud가 `~/.qmonster` 아래 파일을 생성/수정하지 않습니다(R5). 10분 실행 뒤 `find ~/.qmonster -newer /tmp/mark` 결과는 TUI가 만든 파일만이어야 합니다.
- Linux에서는 포인터 선택과 이동/크기 조절이 컴포지터 경로 입력으로 작동합니다. 시스템 지표 버튼은 Tab으로 이동한 뒤 Enter·Space로도 활성화되며 About을 닫으면 열기 버튼으로 포커스가 돌아옵니다.
