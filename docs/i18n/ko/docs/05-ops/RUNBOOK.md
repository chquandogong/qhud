<!-- qhud:languages -->
<p align="center">
  <a href="../../../../05-ops/RUNBOOK.md">English</a> · <strong>한국어</strong> · <a href="../../../zh-CN/docs/05-ops/RUNBOOK.md">简体中文</a>
</p>
<!-- /qhud:languages -->

<!-- qhud:anchor -->
<a id="runbook"></a>

# 운영 가이드

> 상태: 지속 갱신 · 날짜: 2026-09-07 · 담당: chquandogong

<!-- qhud:anchor -->
<a id="install"></a>

## 설치

**Windows x64:** [WINDOWS](WINDOWS.md)에 설명한 ZIP 릴리스 또는 저장소의 빌드 스크립트를 사용합니다. 아래 Linux 안내의 적용 범위는 그대로 유지합니다.

**릴리스 tarball**(Linux x86_64):

```bash
gh release download --repo chquandogong/qhud --pattern '*linux-x86_64.tar.gz'
tar -xzf qhud-v*-linux-x86_64.tar.gz && cd qhud-v*/
./qhud &
```

런타임 의존성(Ubuntu 24.04 이름): `libwebkit2gtk-4.1-0`, `libgtk-3-0`, `libayatana-appindicator3-1`(트레이, 선택 사항).

**소스 빌드**:

```bash
sudo apt-get install -y libwebkit2gtk-4.1-dev libgtk-3-dev \
  libayatana-appindicator3-dev librsvg2-dev pkg-config
git clone https://github.com/chquandogong/qhud && cd qhud
cargo build --release --locked # binary at target/release/qhud
```

<!-- qhud:anchor -->
<a id="run--quit"></a>

## 실행 / 종료

- 시작: `./qhud &` — 위젯이 데스크톱 계층에 나타납니다. 멀티플렉서가 없으면 실제 로컬 계정 스냅샷과 창 0개를 표시합니다. `--demo`는 예시 데이터를 명시적으로 선택합니다.
- 이동: 위쪽 막대 또는 푸터를 드래그합니다. 크기 조절: ◢ 손잡이를 드래그합니다.
- 종료: 트레이 아이콘 → _Quit qhud_ 또는 `pkill qhud`를 사용합니다(설계상 제목 표시줄 없음).
- **글꼴 크기**: Ctrl을 누른 채 위젯 위에서 마우스 휠을 스크롤합니다(70–160%, 저장됨).
- **Peek(잠시 맨 앞으로 표시)**: 트레이 → _Pin above windows_를 선택하고 다시 선택하면 뒤로 보냅니다. 또는 `~/.local/bin/qhud --peek`를 실행합니다. 키보드 단축키는 GNOME Settings → Keyboard → Custom Shortcuts에 `/home/USER/.local/bin/qhud --peek` 명령을 등록합니다(예: Super+Q). WebKitGTK가 예약하므로 qhud에 Unix 시그널을 보내면 안 됩니다(D-012).
- 이미 실행 중일 때 `qhud`를 시작하면 기존 인스턴스가 흡수합니다(단일 인스턴스 가드).
- **수치가 의심스러운가요?** `~/.local/bin/qhud --dump`가 위젯이 표시하는 정확한 페이로드를 출력합니다(관찰 한 주기, 보기 좋은 JSON).
- **사용 한도 행에서 비용이나 초기화 카운트다운이 사라졌나요?** 보조 파일 귀속은 설계상 세 지점에서 조용히 거부됩니다. `QMONSTER_SIDEFILE_DIAG=1 qhud --dump 2>&1 >/dev/null`로 어느 지점인지 확인합니다. cwd 불일치, 60초 동일 cwd 모호성 검사, 자손 CLI 불일치 중 하나를 보고합니다.
- **위젯이 실제로 렌더링하나요?** 생성한 내용을 stderr로 보고합니다. `strip: N sections, M rows, K gauges`, 모든 행의 표시 텍스트(`labels(...)`), 실제 클릭마다 `ptr:`/`sel:`/`qsel:`, 프런트엔드 예외의 `js-error …`입니다. 하지만 추적 로그는 픽셀이 아닌 로직을 증명합니다. 오류가 없다는 사실은 무언가 그려졌다는 증거가 **아닙니다**. 실제 프레임 정지 중 클릭과 조회가 며칠간 보이지 않게 실행되며 배운 교훈입니다. 픽셀은 검증할 수 있습니다. `xwd -id <qhud window> | md5sum`을 몇 초 간격으로 두 번 실행하면 달라야 하며(푸터 시계가 매초 갱신됨), xwd는 볼 수 있는 이미지로 디코딩됩니다. `framestall:` 로그는 내장 감시기가 멈춘 프레임 시계를 감지해 복구 중이라는 뜻입니다(흔들기 → 재실행).
- **클릭 없는 조회 경로**(keep-below 위젯은 합성 포인터 입력을 받지 않음, D-010): `qhud --refresh-all`, `qhud --refresh-claude`, `qhud --fetch-codex`는 단일 인스턴스 채널로 실행 중인 위젯에 전달되며 단축키에 연결할 수 있습니다. `qhud --claude-usage`, `qhud --codex-usage`, `qhud --agy-usage`는 같은 조회를 독립 실행하고 JSON을 출력하며 클릭과 마찬가지로 조회 저장소에 기록합니다. `qhud
--codex-appserver`는 만료 토큰 대체 경로를 요청 시 실행합니다. `QHUD_EXTRA_DIAG=1 qhud --claude-usage`는 형식 변경 확인을 위해 실제 응답에서 식별 정보 없는 `extra_usage`/`spend` 하위 객체를 출력합니다.
- **⟳가 “usage response did not parse: …”를 표시하나요?** Endpoint의 형식이 바뀐 것입니다. v0.5.3부터 이후 부분은 serde 자체의 필드/타입 메시지이므로 읽으면 필드를 알 수 있습니다. 선례(2026-09-01, v0.5.3): `extra_usage.used_credits`가 `4997` 대신 `4997.0`으로 오기 시작해 선택적 필드 하나가 이틀간 응답 전체를 실패시켰습니다. 이제 정수 의미의 금액 필드는 정수값인 실수를 허용합니다. 새 정수형 필드도 같은 유연한 읽기 함수(`usage_cache.rs`의 `lenient_i64`/`lenient_u8`, D-019)를 거쳐야 다음 이틀짜리 장애를 막을 수 있습니다. 메시지가 기간이나 백분율을 가리키면 qhud가 표시하는 필드의 변경이므로 유연한 처리보다 코드 수정이 필요합니다.
- **계정과 플랜**은 이 공개 저장소 밖의 `~/.config/qhud/accounts.json`에 있습니다. `labels` / `plans` / `workspace_names` / `workspace_plans`는 표시 텍스트, `known[]`은 과거 연결 계정 목록, `forgotten`은 자리 표시자 숨김에 사용합니다(활성 계정은 숨기지 않음). 표시 이름은 운영자가 제공하며 응답의 `plan_type`으로 “수정”하면 안 됩니다. `prolite`는 ChatGPT Pro 5x, `team`은 ChatGPT Business로 표시합니다.
- **공급자당 여러 계정**(D-015): 추가 계정을 각각 자체 디렉터리에서 로그인 상태로 유지한 뒤 디렉터리를 등록합니다.

  ```bash
  CLAUDE_CONFIG_DIR=~/claude-personal claude   # sign in once, keep it
  CODEX_HOME=~/.codex-dogu codex login          # same idea for codex
  ```

  `accounts.json`에는 유효한 JSON을 저장합니다(주석과 마지막 쉼표는 지원하지 않음).

  ```json
  {
    "claude_config_dirs": ["~/claude-personal"],
    "codex_homes": ["~/.codex-dogu"]
  }
  ```

  Claude 디렉터리마다 자체 행(계정 정보 + 자체 스냅샷 - ⟳)을 표시하고 추가 Codex 홈은 모든 자격 증명 스캔에 참여합니다. 행 식별자는 **(계정, 조직)**입니다. 하나의 claude.ai 로그인이 팀 좌석과 개인 조직 모두에 속할 수 있습니다. 브라우저 OAuth 뒤 CLI의 조직 선택 단계에서 원하는 조직을 고르세요. 브라우저는 활성 세션으로 자동 진행하므로 실제 선택은 조직 단계에서 이루어집니다. 기본 (계정, 조직)과 같은 디렉터리는 중복 표시하지 않고 건너뜁니다.

- **qhud 자체 ⟳ 결과**는 `~/.config/qhud/fetched-usage.json`에 저장합니다(저장소 밖에 두는 같은 개인정보 규칙; 임시 파일 후 이름 변경). 삭제해도 다음 ⟳가 다시 만듭니다.

<!-- qhud:anchor -->
<a id="autostart--app-launcher-gnome"></a>

## 자동 시작 + 앱 실행기 (GNOME)

먼저 안정적인 경로에 바이너리를 설치합니다. 자동 시작이 `target/release/`를 가리키면 다음 `cargo clean`에서 깨집니다.

```bash
install -Dm755 target/release/qhud ~/.local/bin/qhud
install -Dm644 src-tauri/icons/128x128.png \
  ~/.local/share/icons/hicolor/128x128/apps/qhud.png

mkdir -p ~/.config/autostart ~/.local/share/applications
cat > ~/.config/autostart/qhud.desktop <<EOF
[Desktop Entry]
Type=Application
Name=qhud
Comment=Ambient desktop HUD for AI CLI sessions
Exec=$HOME/.local/bin/qhud
Icon=qhud
Terminal=false
Categories=System;Monitor;
StartupNotify=false
StartupWMClass=qhud
X-GNOME-Autostart-enabled=true
X-GNOME-Autostart-Delay=3
EOF
cp ~/.config/autostart/qhud.desktop ~/.local/share/applications/qhud.desktop
```

`applications` 사본은 GNOME 앱 목록에도 qhud를 추가합니다. 3초 자동 시작 지연으로 데스크톱과 트레이용 AppIndicator 확장이 먼저 준비됩니다. 중복 실행은 기존 인스턴스가 흡수합니다(단일 인스턴스 가드, v0.3.0).

**새 버전을 다시 빌드한 뒤** 설치 사본을 갱신합니다. `install -m755 target/release/qhud ~/.local/bin/qhud && pkill -x qhud && ~/.local/bin/qhud &`

<!-- qhud:anchor -->
<a id="troubleshooting"></a>

## 문제 해결

| 증상 | 해결 |
| --- | --- |
| 위젯이 비어 있음 / 투명하지 않음 / **픽셀 정지**(클릭과 추적 로그는 작동하지만 화면이 바뀌지 않음; 밤새 DPMS 후 흔함) | v0.5.1부터 qhud가 불안정한 두 WebKitGTK 경로를 자체 비활성화합니다(`WEBKIT_DISABLE_DMABUF_RENDERER=1` + `WEBKIT_DISABLE_COMPOSITING_MODE=1`; 디스플레이 절전 때 스레드 컴포지터 프레임 시계가 멈추며 시작 시 libEGL DRI3 오류가 징후). 자체 복구도 수행합니다. Rust 픽셀 가드가 약 28초마다 푸터 띠를 해시하고 두 정지 샘플에서 `frame freeze detected`를 기록한 뒤 창을 unmap/remap해 복구하고 여전히 정적이면 재실행합니다. `QHUD_KEEP_DMABUF=1` / `QHUD_KEEP_COMPOSITING=1`을 설정했다면 해제합니다. 정지 확인: `xwd -id $(xdotool search --class qhud \| tail -1) \| md5sum`을 몇 초 간격으로 두 번 실행하여 해시가 같으면 정지입니다(푸터 시계는 매초 갱신). |
| Claude 사용 한도 행이 오래된 ⟳에 머무름(경과 시간이 계속 증가) 및 상단 ⟳ 오류 툴팁 | stderr를 읽습니다. `usage response did not parse: <serde message>`는 endpoint 형식 변경(진단 절, D-019), `Claude token rejected (401)`는 해당 디렉터리에서 `claude`를 다시 실행해야 한다는 뜻입니다. 만료된 추가 계정은 기본 계정 수치를 숨기지 않으므로 행 하나만 오래됐다면 해당 디렉터리만 문제입니다. |
| 위젯이 창 위로 올라옴 | XWayland를 확인합니다. 창의 `xprop WM_CLASS`가 응답해야 합니다. `QHUD_NO_X11_FORCE=1`을 설정했다면 계층 처리는 컴포지터가 담당합니다. |
| 모니터 분리 후 잘못된 모니터에 표시 | 기하 복원이 사라진 모니터를 가리킵니다. `~/.config/xyz.dogu.qhud/`의 window-state 파일을 삭제하고 재시작합니다. |
| 트레이 아이콘 없음 | AppIndicator 확장이 없습니다. 위젯은 계속 실행되며 `pkill qhud`로 종료합니다. |
| HiDPI에서 흐림 | GNOME 46의 소수 배율 + XWayland가 X11 클라이언트를 흐리게 만듭니다. 정수 배율을 쓰거나 GNOME 47+로 업그레이드합니다(`xwayland-native-scaling`). |
| tmux가 실행 중인데 창이 감지되지 않음 | qhud는 10초마다 탐색합니다. TUI와 같은 설정(`~/.qmonster/config/qmonster.toml`의 `[mux]/[tmux]` 대상)을 확인합니다. 예시 데이터를 명시적으로 선택했다면 `--demo` 없이 재시작합니다. |
| 창 바깥쪽 약 10px에서 드래그/크기 조절이 무시됨 | 해당 띠는 tao 내장 가장자리 처리기 소유입니다(D-008). 이동은 상단/푸터 안쪽, 크기 조절은 ◢ 글리프를 잡습니다. |
| 위젯은 보이지만 실제 마우스 입력을 모두 무시함(합성/xdotool은 작동) | Ubuntu Desktop Icons NG 확장이 데스크톱 계층의 실제 입력을 가로챕니다(D-010). `gnome-extensions disable ding@rastersoft.com`으로 끕니다. 아이콘 사용자에게 필요한 보조 확장 공존은 백로그입니다. `qhud ui:` stderr 추적 로그로 클릭이 도착하는지 확인합니다. |

<!-- qhud:anchor -->
<a id="update-the-qmonster-pipeline"></a>

## qmonster 파이프라인 갱신

`src-tauri/Cargo.toml`의 `rev`를 올리고 `cargo build`를 실행합니다. `view.rs`/`poll.rs`에서 컴파일러가 지적하는 문제를 수정하고 TEST_PLAN 수동 체크리스트를 다시 실행합니다.

<!-- qhud:anchor -->
<a id="release-procedure"></a>

## 릴리스 절차

1. 공개 전에 README, CHANGELOG, 플랫폼 안내, `docs/05-ops/releases/v<version>.md`를 갱신합니다. 패키지, Tauri 설정, Cargo lock의 패키지 버전을 맞춥니다.
2. `codex/` 준비 브랜치를 push하고 Ubuntu와 Windows CI 모두 통과해야 합니다. Linux는 기준 고정 Git 의존성을 사용합니다. Windows는 `scripts/Build-Windows.ps1 -Test`를 실행하고 manifest/lock 복원을 검증합니다. 해당 Windows checkout에서 Cargo 명령을 동시에 실행하지 않습니다.
3. 무관한 커밋을 덮어쓰지 않고 검증한 revision을 `main`에 통합합니다. 릴리스 revision에 주석 있는 `v<version>` 태그를 만들고 push합니다.
4. Release 워크플로는 두 플랫폼을 테스트/빌드하고 기존 Linux x86_64 tarball 및 Windows x86_64 ZIP을 SHA-256 체크섬과 함께 패키징하고 출처를 증명합니다. 공개 작업은 두 빌드가 성공해야 실행되며 저장소의 릴리스 노트를 사용합니다.
5. 워크플로 성공과 모든 플랫폼 다운로드/체크섬 첨부를 확인합니다. CI 빌드 통과는 실제 데스크톱 검증을 대체하지 않습니다. 실행해 보지 않은 데스크톱 통합 항목을 기록합니다.
