<!-- qhud:languages -->
<p align="center">
  <strong>한국어</strong> · <a href="../../../../00-overview/DASHBOARD.md">English</a> · <a href="../../../zh-CN/docs/00-overview/DASHBOARD.md">简体中文</a>
</p>
<!-- /qhud:languages -->

<!-- qhud:anchor -->
<a id="dashboard--qhud"></a>

# 대시보드 — qhud

> 상태: v0.7.1 출시 · 갱신: 2026-09-14 · 담당: chquandogong
> 공개 프로젝트 현황입니다. Git 이력, 태그가 붙은 릴리스, workflow 실행이 기준 기록이며 아래 날짜별 관찰은 당시 검증 범위를 유지합니다.

<!-- qhud:anchor -->
<a id="state"></a>

## 현황

| 항목 | 값 |
| --- | --- |
| 현재 릴리스 | [v0.7.1](https://github.com/chquandogong/qhud/releases/tag/v0.7.1) — [v0.7.0](https://github.com/chquandogong/qhud/releases/tag/v0.7.0)에서 도입한 조건부 GPU 지원을 완성하는 Linux Intel GPU 측정. Linux x86_64 tarball 및 Windows x86_64 ZIP. |
| 파이프라인 의존성 | qmonster @ `6a21c44`; Linux 기준 Git 의존성, Windows 명령 범위 패치 및 lockfile 복원. |
| 지원 플랫폼 | Ubuntu 24.04 / GNOME 및 Windows x64 네이티브 / WebView2. Windows 터미널 탭 네이티브 관찰은 지원 범위 밖입니다. |
| 현재 실행 근거 | **Ubuntu 2026-09-11, v0.7.1:** Intel Arc(Meteor Lake, `i915`, gt0+gt1)를 측정했습니다. `--system-dump` 24.8%와 같은 구간의 독립 rc6 유휴 잔류 계산 24.8%가 일치했고 실행 위젯이 CPU, 메모리, GPU, 디스크, 네트워크 기록을 표시했습니다. v0.7.1은 Linux GPU 샘플러만 바꾸므로 이 패치에서 Windows 실행은 다시 확인하지 않았습니다. |
| 이전 플랫폼 근거 | **공개 v0.6.0 Linux 배포 파일, Ubuntu 2026-09-07:** 체크섬 일치, GNOME/Wayland 계층 상태, 픽셀 갱신, herdr 창 8개, 세 공급자 새로고침 통과. **Windows v0.6.0:** 네이티브 표시, 계정 이메일, 초기화 시간, Codex 새로고침, app-server 대체 경로 통과. 정확한 환경과 해시는 TEST_PLAN을 참고하세요. |
| 품질 근거 | **v0.7.1 Ubuntu 로컬, 2026-09-11:** 형식과 clippy `-D warnings` 이상 없음, Rust 테스트 127/127, Node 테스트 6/6, 릴리스 빌드 2분 08초. 이전 v0.6.0 Ubuntu·Windows CI는 Rust 테스트 98/98과 릴리스 빌드를 통과했습니다: [CI 34117359064](https://github.com/chquandogong/qhud/actions/runs/34117359064). |
| 입력 검증 원칙 | Linux 조작 주장은 컴포지터 경로 주입이나 사람이 직접 조작해야 하며 XTEST만으로는 인정하지 않습니다(D-010). |
| 프레임 가드 현장 근거 | 2026-08-26 → 09-07 journal 기간: 정지 28회, remap 복구 28회, 재실행 0회, 운영자에게 보인 사고 0회(D-017). 이전 터미널 연결 인스턴스의 stderr는 journal에 없어 당시 수치를 재산출할 수 없습니다. |
| 추가 현장 검증 | 시스템 지표의 직접 v0.7.1 근거는 Intel/i915 Linux 장비 한 대입니다. AMD, NVIDIA, WDDM, 일반 tmux 서버, 실제 두 번째 조직 행, 여러 데스크톱 수명 주기는 더 넓은 현장 근거가 필요합니다. 하드웨어 근거가 없을 때는 자동 검증을 기준으로 사용합니다. |

<!-- qhud:anchor -->
<a id="decision-index"></a>

## 결정 색인

D-001 두 번째 프런트엔드 · D-002 Tauri v2 · D-003 XWayland/EWMH 계층 · D-004 qmonster 라이브러리 + NoopSink · D-005 demo=시각 일치 픽스처 · D-006 tarball 릴리스 · D-007 mux 백엔드 팩터리(herdr) · D-008 직접 제어하는 기하 · D-009 pointerdown 선택 · D-010 입력 검증 절차 · D-011 범위에 맞는 표시 · D-012 확대/축소 + peek + 시그널 금지 · D-013 로컬 계정 식별 · D-014 기본은 수동 관찰, 요청 시 네트워크 · D-015 계정별 CLI 설정 디렉터리를 통한 다중 계정 · D-016 위임 조회 경로 · D-017 프레임 가드 · D-018 행 식별자=(계정, 조직) · D-019 유연한 응답 숫자와 필드별 오류 · D-020 범위가 한정된 Windows 적응.

전체 항목과 근거는 [DECISION_LOG](../02-decisions/DECISION_LOG.md)에 있습니다.

<!-- qhud:anchor -->
<a id="delivery-record"></a>

## 배포 기록

| 배포 | 상태 |
| --- | --- |
| v0.1.0–v0.3.0: 최소 위젯, herdr/tmux 관찰, 입력 구조, 확대/축소, peek, 단일 인스턴스 | 2026-08-05/06 출시 |
| v0.4.0–v0.5.3: 범위가 정확한 한도, 계정 식별, 다중 계정, 저장, 위임 조회, 프레임 가드, 조직 식별, 응답 형식 변경 복구 | 2026-08-07 → 09-04 출시 |
| v0.6.0–v0.6.2: Windows 네이티브 계정 위젯, mux 없는 실제 계정 화면, 소유 계정 보호, 포터블 패키지, 핵심 문서 개정, Codex 이메일 자동 표시 | 2026-09-07/08 출시 |
| v0.7.0: CPU, 메모리, 조건부 GPU, 디스크, 네트워크의 제한된 시스템 기록 및 접근 가능한 About | 2026-09-10 출시 |
| v0.7.1: Intel i915/xe 유휴 잔류 GPU 샘플러 및 같은 구간 현장 비교 | 2026-09-11 출시 및 검증 |

<!-- qhud:anchor -->
<a id="open-verification-and-backlog"></a>

## 남은 검증과 백로그

| 항목 | 상태 |
| --- | --- |
| TEST_PLAN의 GNOME 개요, 잠금/해제, 절전/복귀, 모니터 핫플러그, 전체 화면 | 실제 장비 검증 대기 |
| 일반 tmux 실시간 관찰 대체 경로 | 현장 검증 대기 |
| 실제 다중 조직 로그인의 두 번째 조직 행 | 현장 검증 대기; 자격 증명과 장비 경로는 저장소 밖에 보관 |
| Windows, AMD, NVIDIA 시스템 지표 | 추가 하드웨어 근거 대기 |
| 타일 → 터미널 창 포커스 이동 | 백로그 |
| 개요 화면 고정과 데스크톱 아이콘 공존을 위한 GNOME Shell 확장 | 백로그 |
| 업스트림 `ObserveSnapshot`, `.deb` 패키지, agy 다중 계정 | 백로그 |

<!-- qhud:anchor -->
<a id="evidence-notes"></a>

## 근거 설명

v0.7.0은 계정 및 창 소유 규칙을 바꾸지 않고 시스템 영역과 About을 추가했습니다. 샘플링은 기존 2초 관찰 스레드에서 실행하고 기록은 메모리에 최대 30개만 보존하며 위젯을 숨기거나 최소화하면 멈춥니다. 누락 값을 만들어 낸 0 대신 공백으로 표시합니다. v0.7.1은 Linux Intel 측정을 추가했습니다. 위 24.8% 비교는 i915 호스트 한 대를 검증하며 모든 어댑터를 검증했다는 뜻은 아닙니다.

2026-09-07 공개 배포 파일 검증은 Windows 호스트에서 출시한 v0.6.0이 남긴 Ubuntu 데스크톱 공백을 해소했습니다. 계층 상태, 화면 갱신, herdr 관찰, 공급자 새로고침에 대한 유효한 과거 근거로 유지하며 v0.7.1 관찰을 현재 시스템 지표 근거로 사용합니다. 정확한 절차와 미완료 항목은 [TEST_PLAN](../04-quality/TEST_PLAN.md), 현재 요구 사항과 구현 경계는 [SPEC](../03-spec/SPEC.md) 및 [ARCHITECTURE](../03-spec/ARCHITECTURE.md)에 있습니다.
