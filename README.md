<!-- qhud:languages -->
<p align="center">
  <strong>한국어</strong> · <a href="README.en.md">English</a> · <a href="README.zh-CN.md">简体中文</a>
</p>
<!-- /qhud:languages -->

<p align="center">
  <img src="docs/assets/qhud-banner.svg" alt="qhud — AI 사용량, 초기화, 세션을 한눈에" width="100%">
</p>

<p align="center">
  <strong>Claude Code, Codex, Antigravity</strong>를 위한 조용한 데스크톱 도우미.<br>
  계정 사용량, 초기화 시간, 지원하는 터미널 세션을 한곳에서 확인하세요.
</p>

<p align="center">
  <a href="https://github.com/chquandogong/qhud/releases/latest"><img src="https://img.shields.io/github/v/release/chquandogong/qhud?style=flat-square&color=28a99e" alt="최신 릴리스"></a>
  <a href="https://github.com/chquandogong/qhud/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/chquandogong/qhud/ci.yml?branch=main&style=flat-square&label=build" alt="Ubuntu 및 Windows 빌드 상태"></a>
  <img src="https://img.shields.io/badge/platforms-Linux%20%7C%20Windows-52677d?style=flat-square" alt="Linux 및 Windows">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-52677d?style=flat-square" alt="MIT 라이선스"></a>
</p>

<p align="center">
  <a href="#install">다운로드</a> · <a href="docs/i18n/ko/docs/GUIDE.md">사용자 가이드</a> · <a href="docs/i18n/ko/docs/README.md">문서</a> · <a href="docs/i18n/ko/CONTRIBUTING.md">기여하기</a>
</p>

---

<!-- qhud:anchor -->
<a id="one-view-for-your-ai-work"></a>

## AI 작업을 한눈에

qhud는 여러 명령행 인터페이스에 흩어진 계정 할당량과 세션 상태를 작은 데스크톱 위젯에 모읍니다. [qmonster](https://github.com/chquandogong/qmonster)의 관찰 파이프라인과 Rust/Tauri 백엔드, 가벼운 웹뷰를 사용합니다. 빌드에 Node.js나 npm이 필요하지 않습니다.

| 확인할 정보 | 제공 내용 |
| --- | --- |
| **사용량과 초기화** | 계정별 기간, 제공되는 모델별 풀, 초기화 카운트다운, 정확한 초기화 타임스탬프. |
| **계정 맥락** | 구분된 계정/작업공간 행, 설정 가능한 표시 이름, 재시작 후에도 시점이 표시되는 스냅샷. |
| **세션 활동** | 지원하는 Linux 환경에서 상태, 컨텍스트 사용 정도, 모델, 추론 노력, 브랜치, 작업 디렉터리, 충돌 표시. |
| **최신성** | 2초마다 로컬 관찰. 공급자 사용량 요청은 명시적으로 새로고침할 때 실행. |

<!-- qhud:anchor -->
<a id="a-compact-view-with-room-for-detail"></a>

## 작게 보고, 필요할 때 자세히

<table>
  <tr>
    <td align="center"><img src="docs/assets/widget-compact.png" width="330" alt="간결한 공급자 사용량 영역과 세션 타일을 보여 주는 Linux 데모"><br><sub>간결한 세션 개요</sub></td>
    <td align="center"><img src="docs/assets/widget-expanded.png" width="330" alt="세션과 충돌 상세 정보를 펼친 Linux 데모"><br><sub>필요할 때 펼치는 상세 정보</sub></td>
  </tr>
</table>

*스크린샷은 예시 값을 사용하는 Linux 데모 픽스처입니다. 현재 계정 배치와 표시 가능한 모델별 기간은 공급자, 로그인, 플랫폼에 따라 달라집니다.*

<!-- qhud:anchor -->
<a id="platform-support"></a>

## 플랫폼 지원

| 기능 | Linux x86_64 | Windows x64 |
| --- | --- | --- |
| 네이티브 데스크톱 위젯 | GTK / WebKitGTK | WebView2, WSL 불필요 |
| 계정 사용량 및 초기화 시간 | 지원 | 지원 |
| 여러 로그인 설정 디렉터리 | 지원 | 지원 |
| 터미널 창 분할 관찰 | tmux / herdr | 네이티브 Windows 터미널 탭은 지원하지 않음 |
| 데스크톱 통합 | 기준: Ubuntu 24.04, XWayland를 통한 GNOME/Wayland | 네이티브 창과 트레이. Linux 데스크톱 계층 동작은 보장하지 않음 |
| 배포 | 포터블 `.tar.gz` | 포터블 `.zip` |

다른 Linux 데스크톱은 가능한 범위에서 지원합니다. macOS 패키지는 공개하지 않습니다. 지원하는 멀티플렉서가 없으면 qhud는 실제 로컬 계정과 창 분할 0개를 표시합니다. 예시 데이터는 `--demo`로만 사용할 수 있습니다.

<!-- qhud:anchor -->
<a id="install"></a>

## 설치

**현재 릴리스: [v0.6.1](https://github.com/chquandogong/qhud/releases/tag/v0.6.1).**
두 플랫폼의 빌드/테스트 작업을 통과해야 공개합니다. 각 아카이브에는 SHA-256 파일이 있습니다.

| 다운로드 | 검증 |
| --- | --- |
| [Windows x64 ZIP](https://github.com/chquandogong/qhud/releases/download/v0.6.1/qhud-v0.6.1-windows-x86_64.zip) | [SHA-256](https://github.com/chquandogong/qhud/releases/download/v0.6.1/qhud-v0.6.1-windows-x86_64.zip.sha256) |
| [Linux x86_64 tarball](https://github.com/chquandogong/qhud/releases/download/v0.6.1/qhud-v0.6.1-linux-x86_64.tar.gz) | [SHA-256](https://github.com/chquandogong/qhud/releases/download/v0.6.1/qhud-v0.6.1-linux-x86_64.tar.gz.sha256) |

### Windows

ZIP을 풀고 버전 디렉터리 안의 `qhud.exe`를 실행합니다. Microsoft Edge WebView2 Runtime이 필요합니다. 포터블 패키지는 바로가기나 자동 시작을 등록하지 않습니다. 사전 요구사항, 소스 빌드, 설정은 [Windows 가이드](docs/i18n/ko/docs/05-ops/WINDOWS.md)를 참조하세요.

### Linux

아카이브와 SHA-256 파일을 같은 디렉터리에 다운로드한 뒤 실행합니다.

```sh
sha256sum -c qhud-v0.6.1-linux-x86_64.tar.gz.sha256
tar -xzf qhud-v0.6.1-linux-x86_64.tar.gz
install -Dm755 qhud-v0.6.1-linux-x86_64/qhud "$HOME/.local/bin/qhud"
"$HOME/.local/bin/qhud"
```

Ubuntu 24.04 런타임 의존성에는 `libwebkit2gtk-4.1-0`, `libgtk-3-0`, 트레이용 `libayatana-appindicator3-1`이 포함됩니다.
[소스 빌드 및 GNOME 자동 시작 →](docs/i18n/ko/docs/05-ops/RUNBOOK.md)

<!-- qhud:anchor -->
<a id="everyday-use"></a>

## 일상적인 사용

- **이동 및 크기 변경:** 머리글이나 바닥글을 드래그하고 모서리 손잡이로 크기를 변경합니다.
- **확인:** 계정 행이나 세션 타일을 선택합니다. 게이지에 마우스를 올리면 정확한 초기화 시간을 볼 수 있습니다.
- **새로고침:** 상단 ⟳로 모든 공급자를 새로고침하거나 계정 옆의 ⟳를 사용합니다.
- **미리 보기:** 트레이의 *Pin above windows*(창 위에 고정)를 사용합니다.
- **종료:** 트레이에서 *Quit qhud*(qhud 종료)를 선택합니다.

**이미 실행 중인** 위젯에는 다음 명령으로 같은 작업을 할 수 있습니다.

```sh
qhud --refresh-all
qhud --peek
```

[명령, 계정 설정, 문제 해결 →](docs/i18n/ko/docs/GUIDE.md)

<!-- qhud:anchor -->
<a id="understand-the-numbers"></a>

## 숫자의 의미

백분율은 **계정과 기간**에 속합니다. Spark 5H/7D를 포함한 모델별 한도는 공급자가 해당 로그인에 대해 반환하는 경우에만 나타납니다. 없는 한도를 임의의 0으로 채우지 않으며, 한 계정의 저장된 사용량을 다른 계정의 이름 아래 재사용하지 않습니다.

저장된 값에는 경과 시간이 표시됩니다. 초기화까지 남은 시간은 카운트다운이며, 게이지에 마우스를 올리면 정확한 날짜와 시간을 볼 수 있습니다. 로컬 관찰은 공급자 API를 요청하지 않습니다. 명시적 새로고침은 공급자 사용량 엔드포인트를 위해 기존 액세스 토큰을 읽을 수 있지만, qhud 자체는 OAuth refresh grant를 실행하지 않습니다. Codex는 대체 조회를 자체 CLI에 위임할 수 있으며, Antigravity는 실행 중인 CLI의 루프백 서비스를 사용합니다.

계정 설정은 저장소 밖에 보관합니다. [설정 및 데이터 가이드](docs/i18n/ko/docs/GUIDE.md#configuration-and-local-data)를 참조하세요.

<!-- qhud:anchor -->
<a id="explore-the-documentation"></a>

## 문서 살펴보기

설계 결정과 역사 기록을 포함한 모든 문서를 영어, 한국어, 중국어 간체로 제공합니다.

| 시작하기 | 더 알아보기 |
| --- | --- |
| [사용자 가이드](docs/i18n/ko/docs/GUIDE.md) | [아키텍처](docs/i18n/ko/docs/03-spec/ARCHITECTURE.md) |
| [Windows 설정](docs/i18n/ko/docs/05-ops/WINDOWS.md) · [Linux 운영](docs/i18n/ko/docs/05-ops/RUNBOOK.md) | [명세](docs/i18n/ko/docs/03-spec/SPEC.md) · [결정 로그](docs/i18n/ko/docs/02-decisions/DECISION_LOG.md) |
| [기여 안내](docs/i18n/ko/CONTRIBUTING.md) | [테스트 계획](docs/i18n/ko/docs/04-quality/TEST_PLAN.md) · [위험 등록부](docs/i18n/ko/docs/04-quality/RISK_REGISTER.md) |
| [변경 이력](docs/i18n/ko/CHANGELOG.md) | [전체 문서 색인](docs/i18n/ko/docs/README.md) |

<!-- qhud:anchor -->
<a id="contribute"></a>

## 기여하기

버그 보고, 문서 정정, 번역 개선은 **English, 한국어, 简体中文**으로 환영합니다. [기여 가이드](docs/i18n/ko/CONTRIBUTING.md)를 읽거나 [이슈를 작성](https://github.com/chquandogong/qhud/issues/new/choose)하세요.

qhud는 독립 프로젝트이며 Anthropic, OpenAI, Google과 제휴 관계가 없습니다. 공급자 이름은 호환 도구를 식별하기 위해 사용합니다.

[MIT 라이선스](LICENSE) · [qmonster](https://github.com/chquandogong/qmonster) 기반
