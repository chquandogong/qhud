<!-- qhud:languages -->
<p align="center">
  <a href="../../../../00-overview/DASHBOARD.md">English</a> · <strong>한국어</strong> · <a href="../../../zh-CN/docs/00-overview/DASHBOARD.md">简体中文</a>
</p>
<!-- /qhud:languages -->

<!-- qhud:anchor -->
<a id="dashboard--qhud"></a>

# 대시보드 — qhud

> 상태: v0.6.1 출시 · 날짜: 2026-09-07 · 담당: chquandogong
> 유일한 기준 정보는 이 Git 저장소입니다. 이 보드는 작업 인계 지점입니다. 다른 세션이나 에이전트에서 작업을 재개할 때 먼저 읽으세요.

<!-- qhud:anchor -->
<a id="state"></a>

## 현황

| 항목 | 값 |
| --- | --- |
| 버전 | [v0.6.1](https://github.com/chquandogong/qhud/releases/tag/v0.6.1) — SPEC/ARCHITECTURE 전면 개정 및 날짜가 명시된 Ubuntu 검증 근거. 실행 동작은 [v0.6.0](https://github.com/chquandogong/qhud/releases/tag/v0.6.0)과 동일합니다(Linux x86_64 tarball + Windows x86_64 ZIP). |
| 파이프라인 의존성 | qmonster @ `6a21c44`; Linux에서는 기준 Git 의존성, Windows에서는 명령 범위에만 적용하는 패치와 lockfile 복원을 사용합니다. |
| 플랫폼 | Ubuntu 24.04 / GNOME 및 Windows x64 네이티브 / WebView2. Windows 터미널 탭 관찰은 계속 지원하지 않습니다. |
| 실행 검증 근거 | **Ubuntu 2026-09-07, 공개된 v0.6.0 Linux 배포 파일**(체크섬 검증 및 설치): Ubuntu 24.04.3 · GNOME Shell 46.0 Wayland · kernel 7.0.0-28 · eDP-1 2560×1600 + HDMI-1 3840×2160(scale 1) · webkit2gtk 2.52.6 · herdr 0.7.5에서 창 8개 실시간 관찰. `_NET_WM_STATE` = BELOW+STICKY+SKIP_PAGER+SKIP_TASKBAR, `_NET_WM_DESKTOP` = 0xFFFFFFFF, `WM_CLASS` = qhud/Qhud. 3초 간격의 두 `xwd` 해시가 달라 실제 화면 갱신을 확인했습니다. 프레임 가드가 활성화되었고 세 공급자 모두 ⟳에 응답했습니다. Windows v0.6.0에서는 네이티브 표시, 이메일, 초기화 시간, Codex 새로고침 및 app-server 대체 경로를 검증했습니다. |
| 품질 기준 | Ubuntu fmt/clippy/테스트 **98/98**/릴리스 빌드 통과; Windows 테스트 **98/98**/릴리스 빌드 통과 — [CI 34117359064](https://github.com/chquandogong/qhud/actions/runs/34117359064). 2026-09-07 Ubuntu에서 v0.6.0 트리로 로컬 재실행: fmt 이상 없음, clippy `-D warnings` 이상 없음, 98/98, 형제 checkout 없이 고정 Git 의존성으로 릴리스 빌드 6분 21초. |
| 입력 검증 | Ubuntu에서는 **컴포지터 경로만 허용**합니다(Mutter RemoteDesktop 주입 또는 사람이 직접 조작; XTEST 불허, D-010). |
| 교차 검증 | Codex/GPT — AGREE-WITH-CHANGES, CV-1..4 채택(CROSS_VALIDATION_LOG). |
| 프레임 가드 현장 결과 | 08-26 → 09-07(journal에 수집된 기간): **정지 28회, remap 복구 28회, 재실행 0회, 사용자에게 드러난 사고 0회**(D-017). v0.5.1/v0.5.2 수치는 stderr가 journal에 전달되지 않은 터미널 연결 인스턴스에서 얻었으므로 다시 산출할 수 없습니다. |
| 알려진 실사용 미해결 사항 | 2026-09-07에도 미해결: `~/claude-personal` 자격 증명이 2026-08-17에 만료되어 해당 행은 ⟳마다 401을 반환합니다. 운영자가 로그인해야만 해결되며 릴리스는 자격 증명을 갱신하지 않습니다. 기본 계정은 영향받지 않으며 부분 실패는 부분 실패로 유지됩니다. |

<!-- qhud:anchor -->
<a id="decision-index-full-entries-in-decision_log"></a>

## 결정 색인 (전체 항목은 DECISION_LOG)

D-001 두 번째 프런트엔드 · D-002 Tauri v2 · D-003 XWayland/EWMH 계층 · D-004 qmonster 라이브러리 + NoopSink · D-005 demo=시각 일치 검증 데이터 · D-006 tarball 릴리스 · D-007 mux 백엔드 팩터리(herdr) · D-008 직접 제어하는 창 기하 · D-009 pointerdown 선택 · D-010 DING 가로채기 및 검증 절차 · D-011 사실의 범위에 맞는 표시 · D-012 확대/축소, peek 및 시그널 금지 · D-013 로컬 계정 식별 · D-014 기본은 수동 관찰, 요청 시 네트워크 · D-015 계정별 CLI 설정 디렉터리를 통한 다중 계정 · D-016 위임된 조회 경로(codex app-server, agy loopback RPC) · D-017 위젯이 자체 픽셀을 검사하는 프레임 가드 · D-018 행 식별자는 (계정, 조직) · D-019 유연한 응답 숫자 처리 및 거부된 응답의 필드 표시 · D-020 Windows 변경의 범위를 한정하고 사용량의 계정과 기간을 보존.

<!-- qhud:anchor -->
<a id="work-board"></a>

## 작업 보드

| 작업 | 상태 | 담당 |
| --- | --- | --- |
| v0.1.0 최소 유용 기능(위젯 + 연결 계층 + 데모 + 문서 + CI + 릴리스) | 완료 2026-08-05 | claude+chquandogong |
| herdr 백엔드 실시간 동작(D-007) · 입력 구조(D-008/9/10) | 완료 | claude+chquandogong |
| 사실의 범위에 맞는 표시(D-011, v0.2.0) | 완료 2026-08-06 | claude+chquandogong |
| 확대/축소 · peek · 단일 인스턴스(D-012, v0.3.0) 및 밝은 트레이 아이콘 | 완료 2026-08-06 | claude+chquandogong |
| 신뢰할 수 있는 사용 한도: 귀속 수정, 계정 식별, ⟳, 작업 공간(v0.4.0) | 완료 2026-08-07 | claude+chquandogong |
| 모든 계정을 한눈에: 다중 계정, 저장, 추가 사용량, agy RPC, codex app-server, 전체 새로고침(v0.5.0, D-015/D-016) | 완료 2026-08-10 | claude+chquandogong |
| v0.5.0 태그 및 릴리스(e6e31ed 이후 처음으로 CI 정상화) | 완료 2026-08-10 | claude+chquandogong |
| “선택이 작동하지 않음” 해결 과정: ptr/qsel 입력 추적 → 사용량 행 선택, ⟳에서만 네트워크 → 픽셀 프레임 가드(v0.5.1, D-017) | 완료 2026-08-14 | claude+chquandogong |
| 행 식별자 = (계정, 조직) — 하나의 로그인, 두 조직, 별도 한도(v0.5.2) | 완료 2026-08-17 | claude+chquandogong |
| 실수형 `used_credits`로 이틀간 중단된 Claude ⟳: 유연한 응답 숫자 처리 및 필드를 알려 주는 분석 오류(v0.5.3, D-019) | 완료 2026-09-04 | claude+chquandogong |
| Windows 네이티브 계정 위젯; mux 없는 로컬 대체 화면; 모델/초기화 표시 및 소유 계정 검증(v0.6.0) | 구현 완료; 로컬 테스트 98/98 | codex+chquandogong |
| 다른 환경에서도 사용할 수 있는 Linux 의존성 manifest; Windows 스크립트 범위 패치; Ubuntu + Windows CI/릴리스 기준 | 두 플랫폼 CI 작업 통과 | codex+chquandogong |
| v0.6.0 공개 및 다운로드 검증 | [Release 34127575141 통과](https://github.com/chquandogong/qhud/actions/runs/34127575141); 두 압축 파일의 체크섬 일치 | codex+chquandogong |
| 공개 v0.6.0 빌드의 Ubuntu 재검증으로 “이 Windows 호스트에서 재검증하지 않음” 해소 | 완료 2026-09-07 | claude+chquandogong |
| v0.6.0/v0.6.1에 맞춰 SPEC과 ARCHITECTURE 전면 개정; 현장 근거에 따라 RISK_REGISTER와 ASSUMPTIONS 갱신 | 완료 2026-09-07 | claude+chquandogong |
| ⏳ TEST_PLAN 미완료 항목(개요 / 잠금 / 절전 / 핫플러그 / 전체 화면) | **예정 — 첫 실기 검증** | operator |
| 일반 tmux 서버의 실사용 검증(대체 경로) | 예정 | operator |
| `~/claude-personal`에 개인 조직 로그인(CLI 조직 선택 단계에서 PERSONAL 조직 선택; registry 연결 완료; OAuth가 계속 팀 세션을 자동 선택함) | 예정 — 운영자가 원하는 시점 | operator |
| 타일 클릭 → 터미널에서 해당 창에 포커스 | 백로그 | — |
| GNOME Shell 확장: 개요 화면에서 자연스러운 고정 및 DING 공존 | 백로그 | — |
| qmonster에서 `ObserveSnapshot`을 업스트림으로 공개한 뒤 고정 해제 | 백로그 | — |
| `.deb` 패키지(CV 보류 항목) | 백로그 | — |
| agy 다중 계정(OS 키링 역공학) | 백로그 | — |

<!-- qhud:anchor -->
<a id="decision-queue-human"></a>

## 사람의 결정이 필요한 항목

_열린 항목이 없습니다._ 기준 장비에서는 DING을 비활성화한 상태로 유지합니다(`~/Desktop`이 비어 있으며 운영자 승인 완료). 보조 확장이 구현되기 전에 다시 활성화하면 위젯 조작이 작동하지 않습니다.

<!-- qhud:anchor -->
<a id="resume-point"></a>

## 작업 재개 지점

v0.6.1은 문서 및 검증 릴리스입니다. 실행 동작은 v0.6.0과 동일하며, `docs/03-spec/SPEC.md`와 `docs/03-spec/ARCHITECTURE.md`는 v0.6.0 소스를 기준으로 전면 개정했고 RISK_REGISTER / ASSUMPTIONS는 날짜가 명시된 현장 근거에 맞춰 갱신했습니다.

**v0.6.0에 남아 있던 Ubuntu 검증 공백을 해소했습니다.** v0.6.0은 Windows 호스트에서 출시되어 “Ubuntu 데스크톱 통합을 다시 검증하지 않았다”고 기록했습니다. 2026-09-07 공개 Linux 배포 파일의 체크섬을 검증하고 기준 Ubuntu 장비에 설치하여 실행했습니다. 데스크톱 계층 상태, 실제 픽셀 갱신, herdr 창 8개 관찰, 세 공급자의 새로고침을 모두 확인했습니다. 위 실행 검증 근거와 TEST_PLAN의 날짜별 항목을 참고하세요.

이번 세션에서 추가로 확인한 내용: v0.5.3의 응답 형식 변경 수정은 2026-09-04 이후 유지되고 있습니다(모든 Claude ⟳ 성공). v0.6.0에서 릴리스 프로필을 workspace 루트로 옮기면서 Linux 바이너리는 25.13 MiB에서 15.07 MiB로 줄었습니다. `[profile.release]`가 루트가 아닌 workspace 멤버에 있어 cargo가 무시하고 있었으므로 이전 모든 릴리스에서는 `strip`/`lto`/`codegen-units = 1`이 적용되지 않았습니다.

다음으로 진행할 유의미한 작업은 순서대로 다음과 같습니다.

1. **운영자 검증** — 실제 장비에서 사람이 확인해야 하는 TEST_PLAN ⏳ 항목(GNOME 개요, 잠금/해제, 절전/복귀, 모니터 핫플러그, 전체 화면), 일반 tmux 백엔드 확인, `~/claude-personal`에 개인 조직 로그인입니다. Registry는 이미 연결되어 있습니다. CLI 조직 선택 단계에서 PERSONAL 조직을 선택하면 해당 행의 지속적인 401도 해결됩니다.
2. **Windows 현장 검증 확대** — Windows 근거는 호스트 한 대의 스모크 테스트 한 번입니다. 터미널 창 귀속은 설계상 구현하지 않았으므로 Windows 지원 범위는 “세션 HUD”가 아닌 “계정 사용 한도 위젯”으로 명확히 유지합니다.
3. **백로그** — 타일에서 창 포커스로 이동하는 기능이 가장 가치 있는 소규모 작업입니다. 이후 GNOME Shell 확장, 업스트림 `ObserveSnapshot` 공개 및 고정 해제, `.deb` 패키지, agy 다중 계정 순입니다.
