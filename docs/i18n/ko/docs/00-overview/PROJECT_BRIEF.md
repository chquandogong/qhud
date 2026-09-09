<!-- qhud:languages -->
<p align="center">
  <strong>한국어</strong> · <a href="../../../../00-overview/PROJECT_BRIEF.md">English</a> · <a href="../../../zh-CN/docs/00-overview/PROJECT_BRIEF.md">简体中文</a>
</p>
<!-- /qhud:languages -->

<!-- qhud:anchor -->
<a id="project_brief--qhud"></a>

# 프로젝트 개요 — qhud

> 상태: v0.6.0 릴리스 준비 · 날짜: 2026-09-07 · 담당: chquandogong

<!-- qhud:anchor -->
<a id="problem"></a>

## 문제

Claude Code, Codex, Gemini, Antigravity를 나란히 실행하는 다중 에이전트 tmux 작업에서는 컨텍스트 포화, 5h/7d 사용 한도 소진, 승인 대기 중인 창 같은 운영 부담이 생깁니다. 하지만 운영자는 [qmonster](https://github.com/chquandogong/qmonster) TUI 창으로 _전환해야만_ 이를 확인할 수 있습니다. 실제 작업에 집중하는 동안 계기는 시야에서 사라집니다. “작업 도중 5h가 100%에 도달했다” 같은 갑작스러운 한도 초과는 바로 모니터를 _보고 있지 않을 때_ 일어납니다.

<!-- qhud:anchor -->
<a id="solution"></a>

## 해결책

qhud는 **상시 표시 데스크톱 HUD**입니다. 모든 창 아래, 배경화면 위에 놓이고 작업 공간을 옮겨도 유지되는 작은 위젯으로, 모니터의 빈 구석에 qmonster의 창 타일과 CTX / 5H / 7D 계기를 표시합니다. 벽시계처럼 한눈에 확인할 수 있어 포커스나 창을 바꿀 필요가 없습니다.

qhud는 qmonster 크레이트를 연결해 tmux/herdr 관찰, 공급자 데이터 분석, 정책 파이프라인을 재사용합니다. 실행 중인 터미널이 없어도 계정 사용 한도는 유용합니다. 로컬 계정 정보와 시각이 표시된 저장 결과를 계속 보여 주고, 명시적으로 새로고침하면 현재 사용량을 요청합니다.

<!-- qhud:anchor -->
<a id="first-user"></a>

## 첫 사용자

qmonster 작성자의 다중 모니터 Ubuntu GNOME 워크스테이션입니다. 다음 대상은 Linux/X11 또는 GNOME Wayland에서 qmonster를 사용하는 운영자입니다. v0.6.0에서는 계정 사용량과 초기화 시점을 확인하는 Windows x64 네이티브 WebView2 위젯을 추가합니다. Windows 터미널 탭의 네이티브 모니터링은 포함하지 않습니다.

<!-- qhud:anchor -->
<a id="success-criteria-v060"></a>

## 성공 기준 (v0.6.0)

1. GNOME Wayland에서 XWayland를 통해 위젯이 데스크톱 계층(keep-below + sticky)에 유지됩니다. `_NET_WM_STATE`로 검증합니다.
2. 모니터 사이 어디로든 이동하고 크기를 조절할 수 있으며 위치를 저장합니다.
3. 디자인 목업과 시각적으로 일치합니다. 데모 페이로드를 일치 검증용 데이터로도 사용합니다.
4. tmux/herdr를 사용할 수 있으면 실시간 데이터를, 그렇지 않으면 실제 로컬 계정 행과 시각이 표시된 사용 한도를 보여 줍니다. 예시 창은 `--demo`에서만 나타납니다.
5. 관찰 전용입니다. TUI가 관리하는 `~/.qmonster`에 **어떤 데이터도 쓰지 않습니다**.
6. Windows에서 네이티브 계정 사용량, 공급자가 반환하는 모델별 한도, 읽기 쉬운 초기화 시간, 재시작 후 데이터 유지를 지원합니다.
7. Ubuntu 소스 빌드 경로와 Linux tarball 릴리스를 유지합니다. 두 플랫폼이 모두 릴리스 검증을 통과한 뒤에만 Windows ZIP을 공개합니다.

<!-- qhud:anchor -->
<a id="non-goals-v060"></a>

## 비목표 (v0.6.0)

- 제어 동작과 알림을 제공하지 않습니다. 경고는 TUI와 공급자가 담당합니다.
- Windows 터미널 탭의 네이티브 관찰이나 macOS를 지원하지 않습니다.
- npm/deb/AppImage/MSI 패키지를 제공하지 않습니다. Linux tarball과 Windows ZIP을 제공합니다.
- 공급자가 반환하지 않는 모델별 한도를 만들어 내지 않습니다.

<!-- qhud:anchor -->
<a id="current-verification"></a>

## 현재 검증 상태

2026-09-07 원격 Ubuntu와 Windows CI 작업 모두 QHUD 테스트 98/98개와 릴리스 빌드를 통과했으며 Ubuntu fmt/clippy도 통과했습니다. 로컬 Windows 빌드는 네이티브 화면 표시와 계정 새로고침 스모크 테스트를 통과했습니다. 근거와 남은 Ubuntu 데스크톱 검증은 [CI 34117359064](https://github.com/chquandogong/qhud/actions/runs/34117359064) 및 TEST_PLAN을 참고하세요.

<!-- qhud:anchor -->
<a id="documents"></a>

## 문서

Quetzalcoatl 방식의 의사결정 문서는 `docs/` 아래에 있습니다. 탐색(`01-discovery/`), 결정(`02-decisions/`), 사양(`03-spec/`), 품질(`04-quality/`), 운영(`05-ops/`)으로 구성됩니다.
