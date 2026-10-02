<!-- qhud:languages -->
<p align="center">
  <strong>한국어</strong> · <a href="../../../../00-overview/DASHBOARD.md">English</a> · <a href="../../../zh-CN/docs/00-overview/DASHBOARD.md">简体中文</a>
</p>
<!-- /qhud:languages -->

<!-- qhud:anchor -->
<a id="dashboard--qhud"></a>

# 대시보드 — qhud

> 상태: v0.7.4 출시 · 갱신: 2026-10-02 · 담당: chquandogong
> 공개 프로젝트 현황입니다. Git 이력, 태그가 붙은 릴리스, workflow 실행이 기준 기록이며 아래 날짜별 관찰은 당시 검증 범위를 유지합니다.

<!-- qhud:anchor -->
<a id="state"></a>

## 현황

| 항목 | 값 |
| --- | --- |
| 현재 릴리스 | [v0.7.4](https://github.com/chquandogong/qhud/releases/tag/v0.7.4) — 좁은 시스템 표시줄 수정(속도 겹침 해소), Tauri 2.12.1, `sysinfo` 0.38.4, 소스 빌드 최소 Rust 1.90 선언. Linux x86_64 tarball, Windows x86_64 ZIP, CycloneDX SBOM과 각각의 SHA-256 파일 및 출처 증명. 이전: [v0.7.3](https://github.com/chquandogong/qhud/releases/tag/v0.7.3) — 알려진 Codex 요금제 값을 **Pro 100**, **Pro 200**, **Pro 500**으로 표시하며, 운영자가 명시한 override가 우선하고 알 수 없는 값은 배지에서 숨깁니다. v0.7.2 이후 완료한 보안, 의존성, 릴리스 workflow, 번역 운영 작업도 패키지에 담습니다. Linux x86_64 tarball, Windows x86_64 ZIP, CycloneDX SBOM을 제공하며 각각 SHA-256 파일과 출처 증명이 있습니다. |
| 릴리스 이후 소스 | v0.7.4는 v0.7.3 태그 이후의 모든 소스 변경을 담습니다: [#28](https://github.com/chquandogong/qhud/pull/28) v0.7.3 Windows 검증 기록, [#29](https://github.com/chquandogong/qhud/pull/29) Tauri 2.12.1·Rust 1.90·최소 버전 CI 작업, [#30](https://github.com/chquandogong/qhud/pull/30) `sysinfo` 0.38.4, [#31](https://github.com/chquandogong/qhud/pull/31) 좁은 표시줄 수정과 네이티브 README 스크린샷. 이전: v0.7.3은 v0.7.2 태그 이후의 모든 소스 변경을 패키지에 담습니다: [#22](https://github.com/chquandogong/qhud/pull/22) 의존성과 rustls 보안 갱신, [#23](https://github.com/chquandogong/qhud/pull/23) Rust 의존성 정책 게이트, [#24](https://github.com/chquandogong/qhud/pull/24) 번역 workflow, [#25](https://github.com/chquandogong/qhud/pull/25) 릴리스 강화와 SBOM, [#26](https://github.com/chquandogong/qhud/pull/26) 형식화된 비식별 오류, 이번 릴리스의 Codex 요금제 이름 매핑. |
| 요금제 레이블 자동 검사 | **v0.7.3 로컬 소스 트리, 2026-10-01:** Node 테스트 8/8 통과. 요금제 레이블 테스트 2개가 `prolite` / `pro` / `promax` 매핑, 정규화, 명시적 override 우선순위, 알 수 없는 값 숨김을 다룹니다. 이는 합성 입력을 사용한 자동화 근거이며 실제 공급자나 데스크톱 표시 검증이 아닙니다. |
| 파이프라인 의존성 | qmonster @ `6a21c44`; Linux 기준 Git 의존성, Windows 명령 범위 패치 및 lockfile 복원. |
| 지원 플랫폼 | Ubuntu 24.04 / GNOME 및 Windows x64 네이티브 / WebView2. Windows 터미널 탭 네이티브 관찰은 지원 범위 밖입니다. |
| 현재 실행 근거 | **공개 v0.7.4 Windows 자산, 2026-10-02 (게시 후):** 내려받은 ZIP의 SHA-256은 `d1c5bcbd7177075fcf00d48cd1d9a2239f8415f97b3ec945eaeeaa59a3a098cf`와 일치했고, 실행 파일과 About 대화상자는 v0.7.4를 보고했습니다. 네이티브 WebView2 빌드는 합성 값, 새로 만든 격리된 공급자·설정 홈, 새로운 100% 확대 비율 프로필을 사용해 격리된 `--demo` 모드로 실행했습니다. 기본 확대 비율에서 CPU/MEM/GPU/DISK/NET 표시줄은 검증한 일반 394픽셀 및 좁은 361픽셀 창 너비에서 겹치지 않았습니다. Windows Graphics Capture로 394×522 이미지 세 장을 보관했으며 About은 홈페이지와 GitHub 링크를 표시했습니다. 실제 계정과 나머지 Windows 상호작용 체크리스트는 검증하지 않았습니다. **게시 전 근거:** **Ubuntu 2026-10-02, v0.7.4 트리(Tauri 2.12.1):** 네이티브 릴리스 빌드를 Xvfb 데모 모드로 실행했을 때 메인 창과 트레이가 생성되고 v0.7.2와 동일하게 렌더링되었으며 화면이 계속 다시 그려졌습니다. 시스템 표시줄은 300, 360, 450, 600픽셀 창에서 겹침 없이 확인했습니다. 실제 GNOME 세션이나 Windows 실기에서는 아직 실행하지 않았습니다. 이전 근거: **공개 v0.7.3 Windows 자산, 2026-10-01:** 내려받은 ZIP의 SHA-256은 `d6268bfd696e2a9c8b17d77a6590bfb11df18d73498e9d96f627998114183d4f`와 일치했고, 압축을 푼 실행 파일은 버전 0.7.3을 보고했습니다. 실제 Codex 계정에서 원시 요금제 값 `pro`는 **Pro 200**으로 표시되었고, About은 v0.7.3을 표시했으며, Windows 네이티브 WebView2 위젯에는 CPU, 메모리, GPU, 디스크, 네트워크 지표가 표시되었습니다. 비공개 로컬 스크린샷을 확인한 뒤 삭제했으며 실제 계정 이미지는 보관하지 않았습니다. 이전 근거: **Ubuntu 2026-10-01, v0.7.2 트리:** 빌드 후 `~/.local/bin`에 설치하고 실제 GNOME 세션에서 재시작했습니다. 주간 0%를 리셋 시각 없이 보고하는 유휴 Codex 패인이 있는 상태에서 `--dump`가 그날 Codex 조회 결과의 리셋 시각을 담았고, 위젯은 v0.7.1이 카운트다운 없이 `0%`만 보이던 자리에 `7D 6% · 6d 20h`를 표시했습니다. 이 패치는 같은 Windows 병합 코드를 썼지만 v0.7.3 검증 전에는 실제 기기에서 다시 확인하지 않았습니다. 이전 근거: **Ubuntu 2026-09-11, v0.7.1:** Intel Arc(Meteor Lake, `i915`, gt0+gt1)를 측정했습니다. `--system-dump` 24.8%와 같은 구간의 독립 rc6 유휴 잔류 계산 24.8%가 일치했고 실행 위젯이 CPU, 메모리, GPU, 디스크, 네트워크 기록을 표시했습니다. v0.7.1은 Linux GPU 샘플러만 바꾸므로 이 패치에서 Windows 실행은 다시 확인하지 않았습니다. |
| 이전 플랫폼 근거 | **공개 v0.6.0 Linux 배포 파일, Ubuntu 2026-09-07:** 체크섬 일치, GNOME/Wayland 계층 상태, 픽셀 갱신, herdr 창 8개, 세 공급자 새로고침 통과. **Windows v0.6.0:** 네이티브 표시, 계정 이메일, 초기화 시간, Codex 새로고침, app-server 대체 경로 통과. 정확한 환경과 해시는 TEST_PLAN을 참고하세요. |
| 품질 근거 | **v0.7.4 Ubuntu 로컬, 2026-10-02:** 형식과 clippy `-D warnings` 이상 없음, Rust 테스트 141/141, Node 테스트 8/8, `cargo +1.90.0 check` 통과, cargo-deny 이상 없음, `--locked` 릴리스 빌드 2분 27초. 이전 근거: **v0.7.2 Ubuntu 로컬, 2026-10-01:** 형식과 clippy `-D warnings` 이상 없음, Rust 테스트 129/129, Node 테스트 6/6, `--locked` 릴리스 빌드 2분 00초. 이전 근거: **v0.7.1 Ubuntu 로컬, 2026-09-11:** 형식과 clippy `-D warnings` 이상 없음, Rust 테스트 127/127, Node 테스트 6/6, 릴리스 빌드 2분 08초. 이전 v0.6.0 Ubuntu·Windows CI는 Rust 테스트 98/98과 릴리스 빌드를 통과했습니다: [CI 34117359064](https://github.com/chquandogong/qhud/actions/runs/34117359064). |
| 입력 검증 원칙 | Linux 조작 주장은 컴포지터 경로 주입이나 사람이 직접 조작해야 하며 XTEST만으로는 인정하지 않습니다(D-010). |
| 프레임 가드 현장 근거 | 2026-08-26 → 09-07 journal 기간: 정지 28회, remap 복구 28회, 재실행 0회, 운영자에게 보인 사고 0회(D-017). 이전 터미널 연결 인스턴스의 stderr는 journal에 없어 당시 수치를 재산출할 수 없습니다. |
| 추가 현장 검증 | 시스템 지표의 직접 근거는 Intel/i915 Linux 장비 한 대와 Windows 네이티브 장비 한 대입니다. AMD, NVIDIA, 추가 Windows 하드웨어, 일반 tmux 서버, 실제 두 번째 조직 행, 여러 데스크톱 수명 주기는 더 넓은 현장 근거가 필요합니다. 하드웨어 근거가 없을 때는 자동 검증을 기준으로 사용합니다. |

<!-- qhud:anchor -->
<a id="decision-index"></a>

## 결정 색인

D-001 두 번째 프런트엔드 · D-002 Tauri v2 · D-003 XWayland/EWMH 계층 · D-004 qmonster 라이브러리 + NoopSink · D-005 demo=시각 일치 픽스처 · D-006 tarball 릴리스 · D-007 mux 백엔드 팩터리(herdr) · D-008 직접 제어하는 기하 · D-009 pointerdown 선택 · D-010 입력 검증 절차 · D-011 범위에 맞는 표시 · D-012 확대/축소 + peek + 시그널 금지 · D-013 로컬 계정 식별 · D-014 기본은 수동 관찰, 요청 시 네트워크 · D-015 계정별 CLI 설정 디렉터리를 통한 다중 계정 · D-016 위임 조회 경로 · D-017 프레임 가드 · D-018 행 식별자=(계정, 조직) · D-019 유연한 응답 숫자 · D-020 범위가 한정된 Windows 적응 · D-021 형식화하고 비식별화한 평상시 진단.

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
| v0.7.1 이후 `main`: 브랜치·태그 보호, 보안 의존성 패치, 평상시 로그 비식별화 | 2026-09-14 병합; 바이너리는 이후 버전에서 공개 예정 |

<!-- qhud:anchor -->
<a id="open-verification-and-backlog"></a>

## 남은 검증과 백로그

| 항목 | 상태 |
| --- | --- |
| TEST_PLAN의 GNOME 개요, 잠금/해제, 절전/복귀, 모니터 핫플러그, 전체 화면 | 실제 장비 검증 대기 |
| 일반 tmux 실시간 관찰 대체 경로 | 현장 검증 대기 |
| 실제 다중 조직 로그인의 두 번째 조직 행 | 현장 검증 대기; 자격 증명과 장비 경로는 저장소 밖에 보관 |
| AMD, NVIDIA 및 추가 Windows 시스템 지표 | 추가 하드웨어 근거 대기 |
| [glib 0.18 권고 사항 마이그레이션](https://github.com/chquandogong/qhud/issues/13) | GTK 3 의존성 제약으로 추적 중 |
| [열린 codex 브랜치의 CI 중복 실행](https://github.com/chquandogong/qhud/issues/15) | P2 저장소 운영 백로그 |
| [남은 라이브러리 오류 원문 분류](https://github.com/chquandogong/qhud/issues/16) | P2 개인정보 보호 보강; #14에서 확인한 노출은 수정됨 |
| 타일 → 터미널 창 포커스 이동 | 백로그 |
| 개요 화면 고정과 데스크톱 아이콘 공존을 위한 GNOME Shell 확장 | 백로그 |
| 업스트림 `ObserveSnapshot`, `.deb` 패키지, agy 다중 계정 | 백로그 |

<!-- qhud:anchor -->
<a id="evidence-notes"></a>

## 근거 설명

v0.7.0은 계정 및 창 소유 규칙을 바꾸지 않고 시스템 영역과 About을 추가했습니다. 샘플링은 기존 2초 관찰 스레드에서 실행하고 기록은 메모리에 최대 30개만 보존하며 위젯을 숨기거나 최소화하면 멈춥니다. 누락 값을 만들어 낸 0 대신 공백으로 표시합니다. v0.7.1은 Linux Intel 측정을 추가했습니다. 위 24.8% 비교는 i915 호스트 한 대를 검증하며 모든 어댑터를 검증했다는 뜻은 아닙니다.

2026-09-07 공개 배포 파일 검증은 Windows 호스트에서 출시한 v0.6.0이 남긴 Ubuntu 데스크톱 공백을 해소했습니다. 계층 상태, 화면 갱신, herdr 관찰, 공급자 새로고침에 대한 유효한 과거 근거로 유지하며 v0.7.1 관찰을 현재 시스템 지표 근거로 사용합니다. 정확한 절차와 미완료 항목은 [TEST_PLAN](../04-quality/TEST_PLAN.md), 현재 요구 사항과 구현 경계는 [SPEC](../03-spec/SPEC.md) 및 [ARCHITECTURE](../03-spec/ARCHITECTURE.md)에 있습니다.
