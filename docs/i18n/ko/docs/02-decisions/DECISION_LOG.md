<!-- qhud:languages -->
<p align="center">
  <strong>한국어</strong> · <a href="../../../../02-decisions/DECISION_LOG.md">English</a> · <a href="../../../zh-CN/docs/02-decisions/DECISION_LOG.md">简体中文</a>
</p>
<!-- /qhud:languages -->

# DECISION_LOG

> 상태: 계속 갱신 · 날짜: 2026-09-07 · 담당: chquandogong

형식: 맥락 → 선택지 → 결정 → 근거 → 잔여 위험.

---

<!-- qhud:anchor -->
<a id="d-001--build-a-second-frontend-not-a-tui-feature"></a>

## D-001 · TUI 기능이 아닌 두 번째 프런트엔드 구축

- **맥락**: 주변 시야에서 상태를 확인할 수 없는 문제(OFFICE_HOURS)는 터미널을 항상 보이게 두는 방법으로도 보완할 수 있다.
- **선택지**: (a) TUI를 실행하는 전용 터미널을 항상 표시, (b) 새 독립형 위젯 앱, (c) qmonster를 GUI로 포크.
- **결정**: (b) — qmonster를 라이브러리로 링크하는 독립형 `qhud`.
- **근거**: (a)는 터미널과 alt-tab 슬롯을 차지하며 목업과 같아질 수 없다. (c)는 유지보수 부담을 분기한다. (b)는 데이터 파이프라인을 100% 재사용하면서 qmonster를 그대로 둔다.
- **잔여 위험**: 두 프런트엔드가 하나의 암묵적 계약을 공유한다. D-004 참조.

<!-- qhud:anchor -->
<a id="d-002--tauri-v2-over-electron--gnome-extension--native-gtk"></a>

## D-002 · Electron / GNOME 확장 / 네이티브 GTK 대신 Tauri v2

- **맥락**: ~15 MB의 상시 실행 위젯에서 순수 HTML/CSS 목업과 정확히 같은 시각적 결과가 필요하다.
- **선택지**: `ALTERNATIVES.md`의 비교표(A–E).
- **결정**: Tauri v2, 테두리 없는 투명 창, 정적 프런트엔드, `withGlobalTauri`(번들러 없음).
- **근거**: HTML 목업을 거의 그대로 이식할 수 있다. Rust 백엔드는 qmonster 크레이트를 직접 링크하므로 IPC 계층이 없다. ~15 MB 바이너리와 적당한 RSS는 상시 위젯에서 Electron보다 낫다. 결정하기 전에 keep-below 메커니즘을 실기로 검증했다.
- **잔여 위험**: WebKitGTK 렌더링 특성(투명도 / dmabuf). 완화책: 실행 지침서의 환경 변수 대체 경로. Electron을 문서화된 대안으로 유지한다.

<!-- qhud:anchor -->
<a id="d-003--force-gdk_backendx11-xwayland-on-gnome"></a>

## D-003 · GNOME에서 `GDK_BACKEND=x11`(XWayland) 강제

- **맥락**: Mutter에는 layer-shell이 없다. Wayland 최상위 창은 전역 위치를 지정하거나 아래에 유지할 수 없다(tauri#14913은 동작하지 않음).
- **선택지**: (a) XWayland + EWMH below/sticky, (b) 계층을 소유하는 GNOME Shell 확장, (c) 네이티브 Wayland를 사용하고 일반 창을 수용, (d) wlr-layer-shell(GNOME 외 환경만).
- **결정**: (a). 탈출구로 `QHUD_NO_X11_FORCE=1`을 제공하고, 미리 설정된 `GDK_BACKEND`를 존중한다.
- **근거**: 현재 기본 GNOME에서 below+sticky+전역 위치 지정을 모두 제공하는 선택지는 (a)뿐이다. 대상 컴퓨터에서 두 번 검증했다.
- **잔여 위험**: 나중에 분수 배율을 켜면 XWayland가 흐려질 수 있다(A3). GNOME 개요 화면에 위젯이 나타난다(허용한 특성. (b)로만 해결 가능하며 로드맵 선택지로 유지).

<!-- qhud:anchor -->
<a id="d-004--link-qmonster-at-a-pinned-rev-observe-only-noopsink"></a>

## D-004 · 고정 리비전의 qmonster 링크, 관찰 전용(`NoopSink`)

- **맥락**: TUI는 `~/.qmonster` 아래에 sqlite/감사/아카이브를 기록한다. 두 번째 기록자가 생기면 경합한다. 업스트림 라이브러리 API는 안정성을 보장하지 않는다.
- **선택지**: (a) 하위 프로세스로 `qmonster --once`를 실행하고 파싱, (b) 공유 sqlite 읽기, (c) 쓰지 않는 sink로 라이브러리 직접 링크, (d) 먼저 업스트림에 JSON 내보내기 계약 요청.
- **결정**: 현재는 (c) — `Context::new(..., Box::new(NoopSink))` + `SilentNotify`, 의존성은 `rev = "aa2bd39…"`로 고정한다. 안정화되면 (d)로 승격한다.
- **근거**: (a)는 현재 기계 판독 가능한 출력이 없다. (b)는 비공개 스키마와 읽기 시점 결합을 만든다. (c)는 경합 없이 파서를 재사용한다. 설정은 `~/.qmonster/config/qmonster.toml`에서 **읽기 전용**으로 공유한다.
- **잔여 위험**: 리비전 갱신으로 컴파일이 깨질 수 있다. 조용히 잘못되는 대신 명백히 실패하게 하는 것이 고정의 목적이다. 로드맵: 두 프런트엔드가 공유하는 업스트림 `--emit-json` / 버전이 있는 내보내기.

<!-- qhud:anchor -->
<a id="d-005--demo-payload-mirrors-the-mockup-exactly"></a>

## D-005 · 데모 페이로드는 목업을 정확히 재현

- **맥락**: tmux 서버가 없으면 렌더링할 것도 없으며, 시각적 동등성 검증에는 픽스처가 필요하다.
- **결정**: `demo.rs`가 목업을 창 분할 단위까지 재현하고 UI에 `DEMO`를 표시한다. 동등성 검사 역할도 겸한다.

<!-- qhud:anchor -->
<a id="d-006--release--plain-binary-tarball-via-tag-driven-ci"></a>

## D-006 · 릴리스 = 태그 기반 CI의 일반 바이너리 tarball

- **맥락**: qmonster는 npm + 바이너리로 배포한다. qhud v0.1에는 신뢰할 만한 최소 릴리스가 필요하다.
- **결정**: 태그 푸시 시 GitHub Actions가 빌드하고 `qhud-vX.Y.Z-linux-x86_64.tar.gz` + sha256 + 빌드 출처 증명을 업로드한다. 아직 npm/deb/AppImage는 제공하지 않는다.
- **잔여 위험**: 사용자가 webkit2gtk 런타임을 설치해야 한다. RUNBOOK에 문서화했다.

<!-- qhud:anchor -->
<a id="d-007--mux-backend-follows-qmonsters-factory-widget-auto-probes-herdr--tmux"></a>

## D-007 · mux 백엔드는 qmonster 팩터리를 따르며 위젯 auto는 herdr → tmux 순으로 탐색

- **맥락**: v0.1.0은 tmux `PollingSource`를 하드코딩했으므로 herdr 환경(이 컴퓨터의 주 mux)이 실시간 상태로 전환되지 않았다. qmonster의 `[mux] backend = "auto"`는 `HERDR_ENV`/`HERDR_SOCKET_PATH`로 판단한다. 이 환경 변수들은 herdr 창 분할 *안에만* 존재하며, 데스크톱 위젯은 보통 그 안에 있지 않다.
- **선택지**: (a) 사용자가 `backend = "herdr"`를 명시하도록 요구, (b) qhud가 백엔드 감지를 재구현, (c) `app::tmux_source::build_tmux_source`를 재사용하되 설정이 `auto`이고 herdr 환경을 상속하지 않았으면 herdr를 먼저 탐색한 뒤 tmux 탐색.
- **결정**: (c). 명시적 `tmux`/`herdr` 설정은 그대로 전달한다. 푸터의 백엔드 레이블은 설정값이 아니라 *실제로 결정된* 소스에서 가져온다.
- **근거**: (a)는 공유 설정의 약속(같은 파일은 두 프런트엔드에서 같은 의미)을 깨뜨린다. (b)는 나중에 달라질 업스트림 로직을 복제한다. 탐색 비용은 최악에도 실패한 CLI 호출 한 번이다.
- **증거**: 2026-08-05에 실시간 전환을 검증했다. 실제 에이전트 작업공간이 있는 herdr 0.7.5 서버에서 `qhud: live via herdr (4 panes: …)`를 확인했다. tmux 대체 경로도 유지했다.
- **잔여 위험**: herdr와 tmux가 *모두* 실행 중이면 위젯 auto에서 herdr가 우선한다(문서화됨. 재정의하려면 `[mux] backend`를 명시).

<!-- qhud:anchor -->
<a id="d-008--all-window-geometry-interaction-is-self-driven-no-compositor-interactive-ops"></a>

## D-008 · 모든 창 위치·크기 상호작용을 자체 구동(컴포지터 대화형 작업 없음)

- **맥락**: 실행 중인 위젯에서 선택, 이동, 크기 변경이 모두 안 된다는 사용자 보고. 2026-08-05에 컴퓨터에서 계측했다(setTitle 비콘 + 합성 입력).
- **발견**(각각 검증했으며 각각만으로도 치명적):
  1. wry 드래그 영역 / `startDragging` / `startResizeDragging` — 컴포지터 측 대화형 이동/크기 변경 — 은 GNOME의 keep-below XWayland 창에서 신뢰할 수 없다(무동작 또는 부분 동작).
  2. tao의 보이지 않는 테두리 없는 크기 변경 안쪽 영역이 창 테두리에서 ~10px 이내의 모든 포인터 입력을 삼킨다(측정: ≤8px는 소실, ≥12px는 전달). 크기 변경 손잡이가 전부 그 안에 있었다.
  3. 창 자체가 이동하는 동안 WebKitGTK `event.screenX/Y`가 오래된 값이 된다(일정한 지연이 있는 변화량).
  4. tao `outerPosition()`/`outerSize()`가 이 무장식 창에 가상의 ~37px 프레임을 보고한다(y는 37 작고 높이는 37 큼).
- **결정**: 위치·크기를 전부 자체 제어한다. 포인터 이벤트는 Tauri의 전역 `cursorPosition()`을 읽는 rAF 루프를 시작/종료만 한다. setPosition/setSize는 지정값 그대로 적용한다. 잡은 위치의 측정값은 tao 외곽 치수가 아닌 DOM(`clientX/Y`, `innerWidth/Height`)에서 얻는다. 손잡이 입력 영역을 tao 안쪽 영역보다 충분히 안쪽으로 넓힌다. 권한은 cursor-position + set-position + set-size만 남긴다.
- **증거**: 두 모니터에서 합성 드래그가 픽셀 단위로 정확했다(이동 Δ=(70,55)/(70,55), 크기 변경 Δ=(52,36)/(52,36), 축소 Δ=(-45,-36)). 양쪽 모두 DOM 클릭 전달을 비콘으로 확인했다.
- **같은 조사에서 드러난 추가 수정**: `generate_context!`가 `../ui`를 cargo 변경 추적에 등록하지 않는다(오래된 자산으로 빌드됨. 이제 build.rs가 rerun-if-changed를 출력). window-state는 정상 종료 때만 저장했다(이제 폴링 루프에서 30 s마다 체크포인트 기록).
- **잔여 위험**: 드래그 중 rAF 루프가 프레임마다 IPC 왕복 한 번을 사용한다(무시할 수준). Tauri의 `document.title`은 WM_NAME이 아니므로 향후 디버깅에는 `setTitle`을 써야 한다.

<!-- qhud:anchor -->
<a id="d-009--tile-selection-binds-to-pointerdown-not-click"></a>

## D-009 · 타일 선택을 click이 아닌 pointerdown에 연결

- **맥락**: D-008 이후 이동/크기 변경은 되지만 선택은 여전히 안 된다는 사용자 보고(2026-08-06).
- **발견**: 이 WebKitGTK/X11 웹뷰는 `pointerdown`을 확실히 전달하지만(비콘으로 입증), down/up 쌍으로부터 **`click`을 전혀 합성하지 않는다**. 선택 핸들러가 한 번도 실행되지 않았다. 비콘 빌드로 확인했다. pointerdown에 연결한 선택은 전환되고 렌더링된다(`sel:wC:p1:R → sel:none:- → sel:wD:p1:R`).
- **의심 해소**: localStorage를 의심했지만(저장 디렉터리 없음), 쓰기는 정상이다. 디렉터리가 없었던 이유는 클릭 핸들러가 한 번도 실행되지 않았기 때문이다. 그래도 영속화는 try/catch로 감싸고 `render()` 뒤에 수행하여 저장이 UI를 막지 못하게 했다.
- **결정**: 모든 위젯 상호작용을 포인터 이벤트(`pointerdown`/`pointerup`)에 연결하고, 합성된 `click`에는 연결하지 않는다.
- **잔여 위험**: 사용자가 드래그를 의도해도 pointerdown 선택이 발생한다. 타일은 드래그 영역이 아니므로(상단/하단 막대만 해당) 현재 충돌은 없다. 나중에 타일을 드래그할 수 있게 되면 재검토한다.

<!-- qhud:anchor -->
<a id="d-010--ubuntu-ding-intercepts-real-pointer-input-verification-must-use-the-compositor-path"></a>

## D-010 · Ubuntu DING이 실제 포인터 입력을 가로챔; 검증은 컴포지터 경로 필수

- **맥락**: D-009 이후 선택이 합성(XTEST) 테스트를 통과했지만 사용자의 실제 마우스로는 여전히 동작하지 않았다(2026-08-06 보고).
- **방법론 실패 인정**: xdotool/XTEST는 XWayland 내부에 이벤트를 주입하여 실제 입력이 라우팅되는 계층인 **Mutter의 표면 선택을 우회한다**. 이전의 모든 “검증 완료”에는 같은 맹점이 있었다.
- **새 도구**: `org.gnome.Mutter.RemoteDesktop`를 통한 컴포지터 경로 입력 주입(절대 좌표용 ScreenCast 스트림 포함). 이벤트가 하드웨어와 똑같이 Mutter로 들어간다(`scratchpad rd_abs_click.py`, 지속되는 D-Bus 연결 하나 사용. 생성자의 연결이 끝나면 세션도 종료됨).
- **발견(A/B로 입증)**: Ubuntu의 **Desktop Icons NG(DING)** 확장 창이 위젯 위의 모든 실제 포인터 입력을 삼킨다. DING 끔 → 컴포지터 경로 클릭으로 선택 전환. DING 켬 → 똑같은 클릭 소실. 위젯 영역에서 XWayland의 포인터 보기가 멈춘 현상도 qhud 대신 Wayland 표면이 선택되고 있음을 독립적으로 확인했다.
- **처리**: 기준 컴퓨터에서 DING을 비활성화했다(`gnome-extensions disable ding@rastersoft.com`). `~/Desktop`이 비어 있어 이곳에서는 아이콘을 하나도 렌더링하지 않았다. 운영자는 위젯 상호작용을 포기하고 다시 활성화할 수 있다(RUNBOOK 행). 아이콘 사용자와의 공존은 동반 GNOME Shell 확장으로 해결한다(백로그).
- **코드 변경**: 영구 stderr 추적 로그(`ui_event` 명령, `qhud ui: sel:…` 줄)를 추가하여 WM_NAME을 오염시키지 않고도 로그로 실제 입력 동작을 검증할 수 있게 했다.
- **최종 증거**(v0.1.4, 컴포지터 경로, DING 끔): stderr의 `sel:none:-` → `sel:wC:p3:R` 전환 왕복.

<!-- qhud:anchor -->
<a id="d-011--scope-correct-display-quota-is-an-account-fact-shown-once-per-provider"></a>

## D-011 · 범위에 맞는 표시: 할당량은 계정의 사실이며 공급자별 한 번 표시

- **맥락**: 운영자의 지적(2026-08-06) — 두 `claude:1:main` 타일에 *서로 다른* 5H/7D 값이 표시되었다. 지적이 맞다. 5h/7d 기간은 **계정** 범위인데, v0.1은 각 창 분할의 보조 파일 스냅샷을 마치 할당량이 창 분할 범위인 것처럼 렌더링했다. 유휴 세션은 오래된 스냅샷을 보관하므로 같은 계정에 서로 다른 숫자가 나타났다. 오해를 부르고, 중복 레이블도 구별할 수 없었다. 원래 목업 자체에 이 의미 오류가 있었으며 충실히 이식하면서 그대로 보존했다.
- **결정**: 사실이 성립하는 범위에서 표시한다.
  1. **공급자 사용량 영역**(상단 막대 아래, 공급자별 한 행): 가장 최신 스냅샷의 5H/7D 게이지. 최신성 규칙: 할당량 기간 내 사용량은 증가만 하므로, **공급자의 창 분할 전체에서 최대 백분율이 가장 최신 값**이다(각 창 분할의 스냅샷은 하한값). 출처 창 분할은 툴팁에 표시한다.
  2. **타일은 창 분할의 사실만 표시**: 상태 배지 + CTX(+ 펼친 설정/충돌). 창 분할별 할당량 행은 제거한다.
  3. **작업공간 배지**(`@workspace`)로 작업공간 간 같은 레이블을 구별한다.
- **페이로드**: 스키마 v1에 추가 — `quotas[]`(provider, h5, d7, from_label, session), `panes[].session`.
- **알려진 제약**: 컴퓨터의 공급자별 계정 하나를 가정한다(다중 계정에는 공급자 표면의 계정 식별 정보가 필요한데 현재 노출되지 않음). 집계는 단위 테스트했다(`provider_quotas_takes_max_snapshot_per_window`).
- **증거**: 새 배치에서 컴포지터 경로 선택을 다시 실기 검증했다(`sel:wC:p1:R`). README 스크린샷을 다시 생성했다.

<!-- qhud:anchor -->
<a id="d-012--font-zoom-via-ctrlwheel-layer-peek-via-single-instance-argv-signals-are-forbidden"></a>

## D-012 · Ctrl+휠 글꼴 확대; 단일 인스턴스 argv로 계층 전환(시그널 금지)

- **맥락**: 운영자가 (1) 조절 가능한 글꼴 크기와 (2) 설계상 맨 뒤에 있는 위젯을 볼 방법을 요청했다.
- **글꼴 크기**: 위젯 위 Ctrl+휠로 웹뷰 페이지 확대를 조절한다(70–160%, 10% 단위). localStorage에 저장한다. 포인터만 사용하므로 키보드 포커스를 가져오지 않는 계약을 지킨다.
- **미리 보기**: 트레이 체크 항목 “Pin above windows” + 두 번째 프로세스의 `qhud --peek`를 tauri-plugin-single-instance가 실행 중 인스턴스에 전달한다. GNOME 사용자 지정 단축키를 `~/.local/bin/qhud --peek`에 연결한다. 위에 있을 때 푸터는 `pinned ·`를 표시하며, 다시 전환하면 below+sticky를 재설정한다.
- **뼈아픈 교훈(세그멘테이션 오류 3×로 검증)**: **Tauri/WebKitGTK 프로세스에 Unix 시그널 핸들러를 절대 설치하지 않는다.** 첫 설계는 SIGUSR1 + signal-hook를 사용했다. 첫 시그널에서 핸들러 스레드가 로그를 남기기도 전에 SIGSEGV로 종료되었다. WebKitGTK의 JavaScriptCore는 스레드 일시 중단용으로 SIGUSR1을 예약하며, 이를 가로채면 VM 스레드 제어가 손상된다. Wayland의 XWayland 클라이언트는 앱 전역 단축키도 사용할 수 없다. 따라서 단일 인스턴스 argv 전달을 선택했다. 충돌 없이 동작하고 우발적 이중 실행도 흡수한다(단일 인스턴스 백로그 항목 해결).
- **증거**: `--peek` 왕복 검증 — BELOW → ABOVE(`layer:pinned`) → BELOW+STICKY(`layer:below`). 중복 실행은 흡수됨(프로세스 1개), 충돌 없음.

<!-- qhud:anchor -->
<a id="d-013--quota-rows-carry-account-identity-identity-reads-stay-local"></a>

## D-013 · 할당량 행에 계정 식별 정보 표시; 식별 정보는 로컬에서 읽기

- **맥락**: 운영자가 한 컴퓨터에서 공급자별 여러 로그인을 사용한다(agy Google 계정 두 개, `auth.json` 파일 교체로 Codex 자격 증명 두 개, Claude 팀 좌석). “누구의 할당량인가?”라고 물었다. D-011에 기록된 알려진 제약은 다중 계정에 “공급자 표면의 계정 식별 정보가 필요한데 현재 노출되지 않는다”고 주장했다. 이 주장은 **틀렸다**. 모든 CLI가 자격 증명 옆에 로그인 신원을 평문으로 저장한다.
- **결정**: 식별 정보는 로컬 파일에서만 읽는다. `~/.claude.json:oauthAccount`, `~/.codex/auth.json:tokens.account_id`, `~/.gemini/google_accounts.json:active`. 네트워크나 토큰을 사용하지 않으며 자격 증명 필드 자체는 열지 않는다. 입력이 없거나 잘못되어도 틱을 실패시키지 않고 “레이블 없음”으로 처리한다.
- **등급은 하나가 아닌 둘**: Claude 팀 좌석에는 조직 풀과 구성원 자신의 좌석이 *모두* 있고, 각각 자체 요청 한도 등급(`organizationRateLimitTier` / `userRateLimitTier`)이 있다. 따라서 `tiers`는 목록이다. 하나로 합치면 풀 하나를 숨기게 된다.
- **표시 이름**: `~/.config/qhud/accounts.json`의 선택적 운영자 목록을 사용하며, 키는 `<provider>:<account_id-or-email>`이다. qhud는 공개 저장소이고 이 키들은 개인 식별자이므로 의도적으로 저장소 **밖에** 둔다. 매핑되지 않은 계정은 이메일, 그다음 계정 id로 대체한다.
- **페이로드**: 스키마 v1에 추가 — `quotas[].account`, 알 수 없으면 생략.
- **알려진 제약**: *활성* 계정에만 레이블을 붙인다. 모든 계정의 잔여 할당량을 동시에 보려면 계정별 조회가 필요하고, 보관된 자격 증명은 refresh grant 실행이 필요하다. 이는 보류하며 명시적인 운영자 승인을 요구한다. Codex 갱신 토큰은 일회용이고 회전하므로 저장 실패 시 `codex login`이 깨지기 때문이다. Codex는 평문 이메일을 노출하지 않으므로 목록 키는 UUID다.
- **증거**: `cargo test` 17개 통과. 실기 `--dump`에 두 등급을 포함한 `claude → dogu/team <chquan@dogu.xyz> DOGU (claude_team)`과 `codex → 3f13fa37…`가 표시된다.

<!-- qhud:anchor -->
<a id="d-014--binary-budget-20-mb--25-mb-network-is-opt-in-not-ambient"></a>

## D-014 · 바이너리 예산 20 MB → 25 MB; 네트워크는 상시가 아닌 선택적 사용

- **맥락**: Codex의 작업공간별 사용량을 추가하려면 HTTP 클라이언트가 필요하다. `reqwest` + `rustls`로 릴리스 바이너리가 17.9 MB에서 22.8 MB로 늘어 `SPEC.md`의 비기능 예산 20 MB를 초과했다.
- **결정**: 예산을 **25 MB**로 올린다. 20 MB는 v0.1 명세의 초기 추정이며 측정된 제약이 아니다. 패키징, 다운로드, 메모리 제한 중 여기에 의존하는 것이 없고 `strip` + `lto`도 이미 켜져 있었다. 따라서 2.8 MB는 낭비가 아니라 기능의 실제 비용이다. 기각한 대안: `native-tls`(크기 대신 OpenSSL 링크 의존성을 얻어 tarball 릴리스에 더 불리)와 기능 제외(운영자가 명시적으로 요청).
- **이 결정에서 더 중요한 절반**: “네트워크 없음”을 “**기본은 수동 관찰, 요청 시에만 네트워크 사용**”으로 바꾼다. 2 s 폴링 루프는 여전히 소켓을 열거나 자격 증명을 건드리지 않는다. 외부 호출 하나는 운영자 클릭으로 실행되며 OAuth refresh grant를 수행하지 않는다. 이 구분이 네트워크 코드가 전혀 없던 시절과 같은 수준으로 위젯의 평상시 상태를 안전하게 유지한다.
- **증거**: 22.8 MB 측정. 같은 변경에서 `SPEC.md` 비기능 섹션 갱신.

<!-- qhud:anchor -->
<a id="d-015--multi-account--per-account-cli-config-dirs-not-qhud-owned-logins"></a>

## D-015 · 다중 계정 = qhud가 관리하는 로그인이 아닌 계정별 CLI 설정 디렉터리

- **맥락**: 운영자가 밝힌 qhud의 목적(2026-08-10)은 공급자별 여러 계정의 사용량과 초기화 시간을 한눈에 보고, 명시적으로 새로고침하여 공급자 웹페이지를 다시 열지 않는 것이다. v0.4.0의 알려진 제약은 “공급자별 활성 로그인 하나”였으나, 이는 컴퓨터의 한계가 아니라 기본 자격 증명 경로만 읽은 결과였다. Claude Code는 `CLAUDE_CONFIG_DIR`별로, Codex는 `CODEX_HOME`별로 완전한 식별 정보 + 사용량 캐시 + 자격 증명을 유지한다.
- **결정**: 레지스트리(`~/.config/qhud/accounts.json`)에 `claude_config_dirs`와 `codex_homes`를 추가한다. 각 Claude 디렉터리는 자체 사용량 행을 제공한다. 식별 정보는 해당 디렉터리의 `.claude.json`, 숫자는 해당 사용량 캐시와 qhud의 마지막 ⟳ 중 더 최신 값에서 얻는다. ⟳ 한 번으로 모든 계정을 순회하고 각각 자체 저장 키로 기록한다. Codex 추가 홈은 기존의 모든 자격 증명 검색에 포함한다. 로그인과 토큰 갱신은 CLI에 맡기며, qhud는 여전히 refresh grant를 실행하지 않는다.
- **기각**: qhud가 자체 계정별 OAuth 로그인(기기 흐름)을 보유하는 방법. CLI로부터 완전히 독립하는 유일한 방법이지만, 자격 증명 보관, 갱신 실패(“로그인 파손”), 정책 위험을 HUD로 옮긴다. 설정 디렉터리 방식이 실제로 부족한 것으로 확인될 때만 재검토한다.
- **알려진 제약**: 창 분할의 계정을 특정할 수 없으므로 창 분할에서 받은 게이지는 항상 기본 계정 행에 놓인다. agy 다중 계정에는 OS 키링 역공학이 필요하며 시도하지 않았다.
- **증거**: 두 번째 설정 디렉터리로 실기 `--dump`를 실행하면 자체 등급/출처/경과 시간을 가진 claude 행 두 개가 나타난다. `fetch_all`은 자격 증명이 없는 디렉터리에 부분 오류를 기록하면서도 기본 계정 조회는 성공한다.

<!-- qhud:anchor -->
<a id="d-016--delegated-fetch-paths-the-providers-own-process-may-do-the-talking"></a>

## D-016 · 위임형 조회 경로: 공급자의 자체 프로세스가 응답할 수 있음

- **맥락**: 직접 경로로 해결할 수 없는 실패 두 가지가 있다. 만료된 Codex 액세스 토큰은 401을 반환한다(그리고 qhud는 회전하는 일회용 refresh grant를 절대 실행해서는 안 됨). agy에는 저장된 자격 증명으로 qhud가 호출할 HTTP 사용량 엔드포인트가 전혀 없다(활성 토큰은 OS 키링에 있음).
- **결정**: D-014의 “요청 시에만 네트워크 사용”에 세 번째 요청형 경로, 공급자의 자체 프로세스에 묻는 방법을 추가한다. 활성 로그인 대체 경로로 `codex -s read-only -a untrusted app-server`(stdio를 통한 JSON-RPC, **v0.6.0 수정: 승인 옵션은 이제 지원되는 `-a on-request`이며 샌드박스는 `read-only` 유지**, `account/rateLimits/read`)를 사용한다. agy에는 루프백 Connect RPC `RetrieveUserQuotaSummary`를 사용한다(토큰 불필요, 컴퓨터 로컬, /proc로 포트 탐색). 두 경로 모두 자격 증명 보관과 갱신을 전적으로 CLI에 맡기며, qhud는 토큰을 읽지 않는다.
- **순서**: Codex는 직접 HTTP를 먼저 사용하고(빠름), 활성 로그인 실패 시에만 app-server를 사용한다. agy는 직접 경로가 없으므로 루프백이 기본이다. 둘 다 클릭 시에만 실행하며 2 s 폴링 루프는 그대로다.
- **증거**: `--codex-appserver`가 요금제/크레딧/기간을 포함한 활성 작업공간을 반환한다. `--agy-usage`는 실기에서 포트를 찾고 풀 네 개를 모두 파싱했다.

<!-- qhud:anchor -->
<a id="d-017--the-widget-audits-its-own-pixels-frame-guard"></a>

## D-017 · 위젯이 자체 픽셀을 점검(프레임 가드)

- **맥락**: 로그에는 모든 클릭이 동작한다고 나타나는 위젯에 대해 사흘간 “선택이 안 됨” 보고가 이어졌다. JS, 입력, IPC가 실행되는 동안 화면은 몇 시간 전 프레임에서 멈춰 있었다. 디스플레이 절전 시 Mutter가 keep-below 창의 프레임 예약을 멈추고 GTK 프레임 시계가 다시 시작하지 않는다. 메커니즘 수준의 모든 수정이 실기에서 실패했다. DMABUF 끄기는 같은 날 재발했고, 합성 끄기와 JS rAF 감시기를 함께 써도 두 시간 안에 감시기가 알아차리지 못한 채 재발했다(소프트웨어 모드의 rAF는 화면과 분리되어 계속 호출됨). 이 창은 외부 1 px 크기 변경을 무시하므로(D-008) 흔들기 복구도 무효다. unmap/remap은 실기에서 그리기 재개를 입증했다.
- **결정**: 메커니즘에 기대는 대신 증상을 측정한다. Rust 측 가드가 위젯 자체 창의 푸터 픽셀 띠를 ~28 s마다 해시한다(그곳의 시계는 매초 다시 그려짐). 두 표본이 동일 ⇒ 멈춤 ⇒ 숨기기+표시 후 계층 상태 재설정. 한 표본 뒤에도 정지 상태 ⇒ `--respawned`로 재실행(자식이 종료 중인 프로세스를 기다려 단일 인스턴스 가드에 흡수되지 않게 함). 시작 시 “frame guard armed” 한 줄로 샘플러 자체를 확인하며, 모든 감지와 복구를 기록한다.
- **기각**: 환경 변수만으로 완화(저비용 보완책으로 `WEBKIT_DISABLE_DMABUF_RENDERER`, `WEBKIT_DISABLE_COMPOSITING_MODE`, 선택 해제 `QHUD_KEEP_*`는 유지하지만 단독으로 불충분함을 입증). 페이지 내부 rAF 감시기(구조적으로 감지 불가). 크기 흔들기(이 창에서는 무동작).
- **RUNBOOK에 기록한 결론**: 추적 로그는 논리를 입증할 뿐, 실제 그리기를 입증하지 않는다. 픽셀은 검증할 수 있다. `xwd | md5sum`을 두 번 수행하거나 가드 자체 로그를 보면 된다. D-010의 “오류가 없다고 무언가 그려졌다는 증거는 아니다”에 이제 강제 확인 수단이 생겼다.
- **증거**: 단계별 결정 로직 단위 테스트, 배포 시 “armed” 줄 확인. 2026-08-13→14 현장 결과: 멈춤 3건, 1분 이내 remap 복구 3건, 재실행 0건, 운영자가 알아차린 문제 0건.
- **위 수치를 대체하는 현장 집계**(저널 수집 기간 2026-08-26 → 09-07): **멈춤 28건, 첫 단계 복구 28건, 재실행 0건, 운영자가 알아차린 문제 0건.** 멈춤은 디스플레이 절전 뒤 수분 간격으로 몰리므로 가드가 초기 며칠의 관측보다 훨씬 자주 실행된다. 남겨 둘 결과 두 가지: Restart 단계는 여전히 현장에서 한 번도 실행되지 않았다. 이전 집계는 stderr가 저널에 도달하지 않은 터미널 연결 인스턴스에서 얻었으므로 재산출할 수 없다. v0.6.0부터 Linux 전용(D-020). 같은 실패 유형이 없는 Windows에서는 가드를 컴파일에서 제외한다.

<!-- qhud:anchor -->
<a id="d-018--a-quota-rows-identity-is-account-organization"></a>

## D-018 · 할당량 행의 식별자는 (계정, 조직)

> v0.5.2에 배포(2026-08-14/17, `dab68af`). 2026-09-04 문서 감사에서 결정이 CHANGELOG, DASHBOARD, RUNBOOK에는 반영됐지만 이 로그에는 없음을 발견해 여기에 기록했다.

- **맥락**: 운영자가 “두 번째 계정”이라고 부른 대상을 연결하면서 실제로는 두 번째 계정이 없음을 알았다. 같은 이메일과 같은 `accountUuid`의 claude.ai 로그인 하나가 두 조직에 속한다. 팀 좌석과 개인 무료 조직이며 각각 자체 할당량 풀이 있다. CLI 로그인은 설정 디렉터리당 조직 하나로 한정되고, 조직은 브라우저가 아니라 브라우저 OAuth 이후 CLI의 조직 선택 단계에서 정한다. D-015의 중복 제거는 계정 id만 키로 삼아 두 번째 조직을 “기본 계정의 중복”으로 버렸다.
- **결정**: 식별자는 둘의 쌍이다. `AccountLabel`이 계정 id와 함께 `org_id`(`organizationUuid`)를 전달하고 `config_dir`를 노출한다. 중복 제거는 (account, org)를 키로 삼는다. 프런트엔드도 같은 방식으로 사용량 행 키를 정하고, 각 행의 ⟳ 결과는 **설정 디렉터리로** 매칭한다. ⟳ 결과를 계정 id로 매칭하면 한 조직의 숫자가 같은 로그인의 두 행에 모두 들어간다. 이를 막기 위해 쌍을 사용하는 것이다.
- **기각**: 설정 디렉터리만으로 행 키 지정(같은 (계정, 조직)을 가진 두 디렉터리는 실제로 한 행이며, 재로그인이 바로 이런 경우를 만듦). 이메일로 키 지정(이메일은 할당량 범위가 아님).
- **알려진 제약**: 실제 두 번째 조직 행의 현장 검증은 CLI 조직 단계에서 개인 조직을 선택하는 로그인을 기다리고 있다. OAuth 흐름이 브라우저의 활성 팀 세션으로 계속 자동 진행한다.
- **증거**: 추가 디렉터리에 팀으로 재로그인해도 행이 늘지 않는다(같은 계정, 같은 조직으로 올바르게 중복 제거). 추가 디렉터리 자체의 `.claude.json`이 네트워크 없이 식별 정보를 제공한다.

<!-- qhud:anchor -->
<a id="d-019--wire-numbers-are-read-for-their-meaning-a-rejected-body-must-name-its-field"></a>

## D-019 · 전송 숫자는 의미대로 읽고, 본문 거부 시 해당 필드를 명시

- **맥락**: 2026-09-01에 `/api/oauth/usage`가 `extra_usage.used_credits`를 `4997` 대신 `4997.0`으로 직렬화하기 시작했다. serde는 `i64`에 부동소수점을 허용하지 않아 선택적 대체 필드 하나가 5h/7d 기간을 포함한 전체 응답을 실패시켰다. 이틀간 모든 Claude ⟳가 실패했다. 오류 문자열 “usage response did not parse”는 아무 대상도 밝히지 않아 죽은 토큰이나 빈 본문과 구별할 수 없었다. 공급자의 자체 클라이언트도 같은 주에 이 엔드포인트의 빈 본문과 필드 없는 본문을 응답 내부에서 처리하는 기능을 출시했다. 따라서 숫자 형식은 qhud가 의지할 계약이 아니다.
- **결정**: 두 규칙을 적용한다. (1) 의미상 정수인 금액 필드(`used_credits`, `decimal_places`, `amount_minor`, `exponent`)는 관대한 리더로 파싱한다. 정수값 부동소수점은 그 정수로 읽는다. 소수부가 있는 부동소수점은 단위가 모호하므로 버린다(기존 배율 추측 금지 원칙. $50를 $0.50로 표시해서는 안 됨). 어느 결과도 바깥 본문을 실패시켜서는 안 된다. 선택 필드는 끝까지 선택적이어야 하며 필드 하나 때문에 운영자가 기간별 정보를 잃어서는 안 된다. (2) 파싱 거부에 위치를 포함한 serde의 필드·타입 메시지를 담는다. 이 메시지는 구조상 식별 정보를 포함하지 않는다. 알 수 없는 필드는 타입 지정 없이 건너뛰고, 타입이 있는 모든 필드는 숫자, 불리언, 기간/요금제 문자열이다. 따라서 본문 로깅을 금지한 D-013 규칙도 지킨다.
- **기각**: 필드마다 `#[serde(untagged)]` 열거형 사용(문자열까지 조용히 삼키는 포괄 처리). 전체 본문을 `serde_json::Value`로 파싱하고 수동 순회(이전 Codex 형식 변화를 잡아낸 타입 계약 상실). 소수부가 있는 부동소수점을 최소 단위로 반올림(v0.5.0부터 금지한 배율 추측). 응답 본문을 오류에 다시 출력(식별 정보 유출).
- **증거**: 테스트 세 개 — 09-03 실제 본문 그대로, 정수값/소수값 쌍, 메시지가 불일치를 명시하고 본문을 반복하지 않음을 단언하는 거부 테스트. 수정 후 실기에서 위젯 자체 ⟳가 `claude usage ok [default] (5h 49%, 7d 7%, 3 scoped)`를 기록했다.

<!-- qhud:anchor -->
<a id="d-020--windows-adaptation-stays-scoped-usage-keeps-its-account-and-window"></a>

## D-020 · Windows 적응은 범위를 한정하고, 사용량은 계정과 기간을 유지

- **맥락**: Windows 위젯 추가로 일반 Linux 클론이 개발자의 형제 디렉터리에 의존해서는 안 된다. 모델 전용 할당량 응답도 없는 한도를 만들어 내지 않으면서 직렬화와 표시를 거쳐 유지되어야 한다.
- **결정**: Linux는 고정 Git 의존성, GTK 프레임 가드, X11/tmux/herdr 경로를 유지한다. Windows 빌드 스크립트는 명령 범위 Cargo 설정으로 저장소에 포함된 의존성 패치를 적용하고, 이후 정본 lockfile 바이트를 복원한다. CI는 두 플랫폼을 테스트/빌드하며, 릴리스 공개는 둘 다 끝날 때까지 기다린다. 공통 할당량 수정은 계정 식별 정보, 모델 범위, 각각의 초기화 시점을 보존하고 null 가능한 한도 맵을 허용하며, 서버가 반환하지 않은 모델을 절대 생성하지 않는다.
- **공통 동작 변경**: mux 없이 시작하면 실제 로컬 계정과 창 분할 0개를 표시한다. 데모 데이터는 명시적으로 선택한다. 홈/설정 환경 재정의를 존중하며, 일치하는 계정 식별 정보가 없는 스냅샷은 새로고침이 필요하다.
- **경계**: 네이티브 Windows 터미널 창 분할 귀속은 구현하지 않았다. Windows 지원은 Linux 데스크톱 통합을 대체하지 않는다.
