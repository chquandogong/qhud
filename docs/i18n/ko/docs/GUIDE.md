<!-- qhud:languages -->
<p align="center">
  <strong>한국어</strong> · <a href="../../../GUIDE.md">English</a> · <a href="../../zh-CN/docs/GUIDE.md">简体中文</a>
</p>
<!-- /qhud:languages -->

<!-- qhud:anchor -->
<a id="user-guide"></a>

# 사용자 가이드

qhud 설치, 기존 CLI 계정 연결, 저장된 스냅샷과 현재 응답을 구별하며 사용량을 읽는 방법을 안내합니다.

<!-- qhud:anchor -->
<a id="install-and-start"></a>

## 설치 및 시작

[릴리스](https://github.com/chquandogong/qhud/releases/latest)에서 Windows ZIP 또는 Linux tarball을 선택합니다. 각 아카이브에는 버전이 붙은 디렉터리가 있으며, 별도 SHA-256 파일이 함께 제공됩니다.

| 플랫폼 | 시작 방법 | 요구사항 |
| --- | --- | --- |
| Windows x64 | ZIP을 풀고 추출된 디렉터리 안의 `qhud.exe`를 실행합니다. | Microsoft Edge WebView2 Runtime. [Windows 설정](05-ops/WINDOWS.md)을 참조하세요. |
| Linux x86_64 | tarball을 풀고 버전 디렉터리 안의 바이너리를 설치합니다. | GTK/WebKitGTK. 기준 시스템은 Ubuntu 24.04입니다. [Linux 운영](05-ops/RUNBOOK.md)을 참조하세요. |

Linux v0.8.1에서 두 파일을 모두 다운로드한 뒤 실행합니다.

```sh
sha256sum -c qhud-v0.8.1-linux-x86_64.tar.gz.sha256
tar -xzf qhud-v0.8.1-linux-x86_64.tar.gz
install -Dm755 qhud-v0.8.1-linux-x86_64/qhud "$HOME/.local/bin/qhud"
"$HOME/.local/bin/qhud"
```

Windows 다운로드는 표시된 해시를 `.sha256` 파일과 비교합니다.

```powershell
Get-FileHash -Algorithm SHA256 .\qhud-v0.8.1-windows-x86_64.zip
Get-Content .\qhud-v0.8.1-windows-x86_64.zip.sha256
```

현재 release workflow로 게시한 아카이브에는 GitHub 빌드 출처 증명도 있습니다. GitHub CLI를 설치하고 실제 다운로드한 파일명으로 바꿔 `.sha256` 제공 파일이 아닌 아카이브 자체를 검증합니다.

```sh
gh attestation verify ./qhud-vX.Y.Z-linux-x86_64.tar.gz --repo chquandogong/qhud
```

```powershell
gh attestation verify .\qhud-vX.Y.Z-windows-x86_64.zip --repo chquandogong/qhud
```

체크섬은 함께 제공된 파일을 기준으로 전송 손상이나 교체를 찾고, 증명은 GitHub Actions가 이 저장소를 위해 아카이브를 빌드했는지 확인합니다. 증명 기능을 켜기 전에 만든 릴리스는 검증되지 않습니다.

v0.7.2 이후 릴리스에는 CycloneDX 1.5 JSON 소프트웨어 자재 명세서(SBOM) `qhud-vX.Y.Z-sbom.cdx.json`도 별도의 `.sha256` 파일, 빌드 출처 증명과 함께 제공됩니다. release workflow는 태그된 소스와 고정된 `Cargo.lock`에서 모든 대상 플랫폼 기준으로 SBOM을 만들므로 Linux와 Windows 빌드의 Rust 크레이트를 모두 담습니다. 프런트엔드는 qhud가 직접 작성한 HTML·CSS·JavaScript이며 외부 패키지가 없습니다. 게시 전에 로컬 빌드 경로를 제거합니다. SBOM 파일 자체를 검증하세요(Windows에서는 위와 같이 `Get-FileHash` 결과를 제공 파일과 비교합니다).

```sh
sha256sum -c qhud-vX.Y.Z-sbom.cdx.json.sha256
gh attestation verify ./qhud-vX.Y.Z-sbom.cdx.json --repo chquandogong/qhud
```

Codex와 Claude Code는 각 공급자의 CLI로 로그인합니다. Windows에서 Antigravity는 로그인한 앱을 열거나 `agy` CLI를 실행해 둡니다. qhud를 열고 공급자 이름 옆의 ⟳를 눌러 사용량을 조회하세요. 포터블 패키지는 바로가기나 자동 시작을 등록하지 않습니다.

<!-- qhud:anchor -->
<a id="read-the-widget"></a>

## 위젯 읽기

공급자별로 **이메일 주소** 아래에 **개인**, **팀** 순서로 행을 표시합니다. 플랜, 기간별 사용량, 초기화 시각은 각 작업공간이나 조직에 속합니다. 연결을 기다리는 프로필도 해당 이메일 아래에 표시합니다.

- **5H / 7D**는 초기화까지 남은 시간이 아니라 기간의 길이입니다.
- **오른쪽 시간**은 초기화까지 남은 시간입니다. 마우스를 올리면 정확한 현지 날짜와 시간을 볼 수 있습니다.
- **모델 이름**은 별도의 할당량 풀을 구별합니다. 공급자가 제공하는 경우에만 나타납니다.
- **스냅샷 경과 시간**은 저장된 사용량임을 알려 줍니다. qhud를 재시작해도 오래된 값이 새 응답으로 바뀌지 않습니다.
- **창 분할 0개**는 지원하는 터미널 소스가 없다는 뜻입니다. 계정 사용량은 계속 표시할 수 있습니다.

모델 할당량이 없는 것은 사용량 0과 다릅니다. 예를 들어 Spark 5H/7D는 현재 Codex 계정의 응답에 해당 한도가 있을 때만 표시합니다. 다른 계정을 새로고침해도 활성 계정의 한도를 확인한 것이 되지는 않습니다.

**초기화권 N개** 배지는 공급자가
제공한 횟수만 표시합니다. Codex는 공식 `rateLimitResetCredits.availableCount`를
사용합니다. 명시적인 `0`은 유효하지만 필드가 없으면 확인 불가로 남습니다. 이는
추가 사용 금액 및 초기화까지 남은 시간과 별개이며, qhud는 횟수를 표시할 뿐
초기화권을 사용하지 않습니다. Claude 사용량 응답에 수동 초기화 횟수가 없으면
펼친 상세에 정보가 없다고 표시합니다. 초기화 날짜, 사용률, 유형이 다른 초기화
제안으로 횟수를 만들어 내지 않습니다. 횟수에도 해당 계정 행의 스냅샷 경과 시간이
적용됩니다.

Claude의 기존 개인 `claude_max` 메타데이터와 `default_claude_max_20x` 조합은
[공식 Max 20x 이름 변경](https://support.claude.com/en/articles/17728952-what-is-the-ultra-plan)에
따라 **Ultra**로 표시합니다. `default_claude_max_5x`는 **Max 5x**로 표시합니다.
Team/Enterprise에서는 위 두 종류의 알려진 사용자 등급에 따라 구성원의 사용량 등급을
기본 배지에 **Max 5x** 또는 **Ultra**로 표시합니다. 별도의 개인/팀 구분과 펼친
상세의 **조직 플랜: Team/Enterprise**는 실제 조직 플랜을 유지합니다. 알 수 없는
구성원 등급을 조직 등급으로 대신하지 않으며, 사용자 등급을 알 수 없으면 배지는
Team/Enterprise로 유지합니다. 명시적 플랜 재정의를 우선합니다. 정확한
`claude_ultra` 또는 `ultra` 메타데이터도 Ultra로 표시하지만 알 수 없는 조직 값은
추측하지 않습니다. 새로운 Ultra 응답에서
5H 한도를 더 이상 제공하지 않는 경우를 포함하여, 없는 5H 한도는 표시하지 않습니다.

인식된 요금제 또는 좌석 배지 옆의 **지원 모델**을 선택하면 공급자 안내를 펼칩니다.
이는 **2026-11-23**에 검토한 공식 요금제 참고 정보이며, 해당 계정의 모델 접근 권한이나
남은 사용량을 실시간으로 확인한 결과는 아닙니다. 실제 선택 가능 여부는 공급자의
모델 선택기에서 로그인한 계정과 작업공간을 기준으로 확인하세요.

- **Codex:** Codex CLI에서 `/model`을 실행합니다. 제공 모델은 계정, 작업공간 권한,
  클라이언트 버전에 따라 달라지며 Team/Business 플랜 이름만으로 구성원의 좌석
  유형을 확인할 수 없습니다. [공식 Codex 모델 안내](https://developers.openai.com/codex/models)를 참고하세요.
- **Claude Code:** 터미널 또는 Claude Desktop의 Code 탭에서 `/model`을 실행합니다.
  Max·Ultra 플랜과 Team Premium 좌석은 한도 내 Fable 사용을 포함하지만,
  Team Standard 좌석에서 Fable을 사용하려면 사용 크레딧이 필요합니다.
  표시된 **Max 5x** 구성원 사용량 등급만으로 Premium 좌석을 확인한 것은 아닙니다.
  [Claude Code 모델 설정](https://code.claude.com/docs/en/model-config)과
  [Fable 이용 자격 및 한도](https://support.claude.com/en/articles/15424964-claude-fable-models-on-your-plan)를 참고하세요.
- **Antigravity:** 앱의 모델 선택기를 사용합니다. 유료 Google AI Pro와 체험 구독은
  접근 가능한 모델이 다를 수 있으며, 일부 모델은 유료 구독 또는 특정 자격을 갖춘
  체험 구독이 필요합니다. 업무용 계정에는 파트너 모델의 관리자 승인 등 조직의
  접근 규칙이 적용됩니다. [Antigravity 요금제 및 모델 조건](https://antigravity.google/pricing)을 참고하세요.

지원하는 Linux 환경에서는 세션 타일에 상태, 컨텍스트 사용 정도, 모델, 추론 노력, 브랜치, 작업 디렉터리, 충돌 표시가 추가됩니다. 이 릴리스는 Windows Terminal, PowerShell, WezTerm 탭을 네이티브로 관찰하지 않습니다.

<!-- qhud:anchor -->
<a id="system-usage-and-about"></a>

## 시스템 사용량과 정보

하단의 작은 그래프는 2초마다 측정하여 최근 약 60초의 최대 30개 표본을 표시합니다.
CPU, MEM, GPU, DISK, NET을 선택하면 상세 정보가 열립니다. 같은 항목을 다시
선택하면 닫히고 다른 항목을 선택하면 내용이 바뀝니다. CPU·메모리·GPU는 0–100%
고정 축을 사용하고, 디스크와 네트워크는 읽기+쓰기 또는 수신+송신 속도를 최근 트래픽에
맞춘 축으로 표시합니다. 마우스를 올리면 현재 값을 볼 수 있습니다.
키보드는 Tab으로 항목에 이동한 뒤 Enter 또는 Space로 조작합니다.

| 항목 | 측정 범위 |
| --- | --- |
| CPU | 논리 프로세서 전체의 CPU 사용률. |
| MEM | 사용 중인 물리 메모리 / 전체 물리 메모리. 상세 정보에 바이트 용량을 표시합니다. |
| GPU | 최초 유효 표본에서 가장 바쁜 지원 어댑터 하나를 골라 해당 측정 세션 동안 유지합니다. Windows는 같은 엔진의 프로세스 인스턴스를 합산한 뒤 가장 바쁜 엔진의 사용률을 표시합니다. |
| DISK | 물리 디스크 전체의 초당 읽기·쓰기 바이트. 파티션과 중첩 장치의 입출력을 중복 합산하지 않습니다. 상세 정보는 볼륨별 중복을 제거한 로컬 디스크 볼륨의 사용/전체 용량이며 30초마다 갱신합니다. |
| NET | 사용 가능한 IP 주소가 있는 비루프백 인터페이스 전체의 수신·송신 속도. Linux는 작동 상태가 내려간 인터페이스도 제외합니다. VPN·가상·물리 인터페이스에서 같은 트래픽이 중복 집계될 수 있으며 인터넷 회선 속도계는 아닙니다. |

GPU 측정 지원은 장치와 드라이버에 따라 달라집니다. Windows는 WDDM GPU Engine
카운터를 사용합니다. Linux는 AMD의 사용 가능한 busy-percent sysfs 값, Intel
i915·xe의 유휴 잔류 카운터를 뒤집은 값, 설치된 NVML 라이브러리를 통한 NVIDIA
측정을 지원합니다. VRAM 사용량은 Linux 측정원이
제공할 때만 표시하며 현재 Windows에는 표시하지 않습니다. 미지원 GPU는 숨깁니다.
이미 지원이 확인된 GPU의 측정이 일시 실패하면 칸을 유지하고 그래프에 빈 구간을 표시합니다.
첫 CPU·속도 표본, 카운터 초기화, 측정 불가 값은 `--` 또는 빈 구간이며 임의의 0이
아닙니다. 정상 측정된 유휴 상태는 0으로 표시합니다.

창을 숨기거나 최소화하면 시스템 측정을 중지합니다. 다시 표시하면 기록을 새로 시작하고
속도 카운터를 준비합니다. 측정 간격이 길어진 경우도 기준값을 초기화합니다. 기록은
메모리에만 보관하며 시스템 측정은 프로세스 스캔, 모니터링 명령 실행, 계정 자격 증명
읽기, 원격 측정 전송을 수행하지 않습니다. `--demo`에는 예시임을 밝힌 합성 값을 사용합니다.

왼쪽 위 **qhud**를 클릭하면 About에서 실행 중인 빌드의 버전, 제작자
**Chenghao Quan (chquandogong)**,
[제작자 홈페이지](https://chquandogong.github.io/CHENGHAO-QUAN/)를 확인할 수 있습니다.
×, Escape 또는 패널 바깥을 클릭하면 닫힙니다. 시스템 정보만 JSON으로 진단하려면
`qhud --system-dump`를 실행합니다. 두 표본을 기다린 뒤 계정 정보를 읽거나
위젯을 열지 않고 종료합니다.

<!-- qhud:anchor -->
<a id="controls-and-commands"></a>

## 조작 및 명령

머리글/바닥글을 드래그하여 위젯을 이동하고, 모서리 손잡이로 크기를 변경하며, Ctrl+휠로 확대 비율을 조절합니다. 계정이나 타일을 선택하면 상세 정보가 펼쳐집니다. 트레이에는 *Pin above windows*(창 위에 고정)와 *Quit qhud*(qhud 종료)가 있습니다.

다음 명령은 **이미 실행 중인** 인스턴스를 대상으로 합니다.

| 명령 | 동작 |
| --- | --- |
| `qhud --peek` | 창 위 고정과 데스크톱 계층 모드를 전환합니다. |
| `qhud --refresh-all` | 지원하는 모든 공급자의 새로고침을 요청합니다. |
| `qhud --refresh-claude` | Claude 사용량 새로고침을 요청합니다. |
| `qhud --fetch-codex` | Codex 작업공간 사용량 새로고침을 요청합니다. |

Windows에서는 `& '.\qhud.exe' --refresh-all`처럼 실행 파일 경로를 사용합니다. 전달용 플래그는 최초 실행 시 해당 동작을 수행하지 않습니다. 먼저 위젯을 시작한 뒤 사용하세요.

다음 진단 명령은 독립적으로 실행됩니다.

```sh
qhud --provider-health
qhud --refresh-health
qhud --dump
qhud --system-dump
qhud --claude-usage
qhud --codex-usage
qhud --agy-usage
qhud --codex-appserver
```

`--dump`는 로컬 관찰 페이로드 하나를 출력합니다. 세 `--*-usage` 명령은 조회한 뒤 JSON을 출력하고 결과를 저장합니다. `--codex-appserver`는 CLI 대체 경로의 응답을 저장하지 않고 출력합니다. 예시 데이터를 보려면 기존 인스턴스를 종료한 뒤 `qhud --demo`를 사용합니다. 진단에는 계정 식별 정보, 로컬 경로, 세션 상세 정보가 포함될 수 있으므로 이슈를 공유하기 전에 민감한 내용을 가리세요.

<!-- qhud:anchor -->
<a id="configuration-and-local-data"></a>

## 설정 및 로컬 데이터

qhud 설정은 소스 저장소 밖에 보관합니다.

| 항목 | 위치 또는 동작 |
| --- | --- |
| Linux 기본 설정 | `~/.config/qhud/` |
| Windows 신규 설치 | `%APPDATA%\qhud\` |
| 표시 설정과 등록 계정 | 설정 디렉터리 안의 `accounts.json` |
| 마지막 명시적 조회 사용량 | 설정 디렉터리 안의 `fetched-usage.json` |
| 기본 Codex 홈 | `~/.codex` 또는 `CODEX_HOME` |
| 기본 Claude 설정 디렉터리 | `~/.claude` 또는 `CLAUDE_CONFIG_DIR` |

설정 디렉터리 우선순위는 지정된 그대로의 `QHUD_CONFIG_DIR`, `XDG_CONFIG_HOME/qhud`, Windows의 기존 `%USERPROFILE%\.config\qhud`와 그다음 `%APPDATA%\qhud`, 마지막으로 `~/.config/qhud`입니다. 홈 디렉터리에서 Windows는 `USERPROFILE`을, Linux는 `HOME`을 우선합니다.

`accounts.json`은 주석이나 후행 쉼표가 없는 일반 JSON입니다. 아래 예시는 자리표시자 계정 ID를 사용하므로 CLI 또는 qhud 진단이 보고한 식별 정보로 바꾸세요. 다른 항목을 버리지 말고 기존 파일에 설정을 병합합니다.

```json
{
  "labels": {
    "codex:YOUR_ACCOUNT_ID": "work@example.com"
  },
  "plans": {
    "codex:YOUR_ACCOUNT_ID": "Pro 200"
  },
  "workspace_names": {
    "YOUR_OTHER_WORKSPACE_ID": "Work"
  },
  "workspace_plans": {
    "YOUR_OTHER_WORKSPACE_ID": "Pro 500"
  },
  "claude_config_dirs": ["~/claude-personal"],
  "codex_homes": ["~/.codex-personal"]
}
```

Codex는 로컬 `auth.json`의 ID 토큰에 있는 이메일을 자동으로 표시합니다. 이메일은 표시용 보조 정보이며 없거나 형식이 잘못되면 계정/작업공간 ID로 대체합니다. 새 컴퓨터에서도 일반적인 이메일 표시에 별도 레이블 파일이 필요하지 않습니다. 일치하는 `labels` 항목은 이메일보다 우선하며, 이메일과 레이블 모두 사용량을 구분하는 계정/작업공간 식별자를 바꾸지 않습니다.

Codex 플랜 배지에서 qhud는 검토한 전송 값 세 개에 현재 업스트림 상태 이름을 적용합니다. `prolite`는 **Pro 100**, `pro`는 **Pro 200**, `promax`는 **Pro 500**으로 표시합니다. 명시적인 `plans` 항목은 계정 배지를, `workspace_plans` 항목은 일치하는 작업공간 배지를 재정의합니다. 알 수 없는 전송 값은 확장 세부 정보와 툴팁에만 남고 표시 플랜 배지가 되지 않습니다.

<!-- qhud:anchor -->
<a id="more-than-one-account"></a>

## 여러 계정

추가 계정마다 공급자 디렉터리를 별도로 두고 로그인 상태를 유지한 뒤 `claude_config_dirs` 또는 `codex_homes`로 해당 디렉터리를 등록합니다.

```sh
CLAUDE_CONFIG_DIR=~/claude-personal claude
CODEX_HOME=~/.codex-personal codex login
```

Windows에서는 CLI 세션의 환경 변수를 설정합니다.

```powershell
$previousCodexHome = $env:CODEX_HOME
try {
  $env:CODEX_HOME = "$env:USERPROFILE\.codex-personal"
  codex login
} finally {
  $env:CODEX_HOME = $previousCodexHome
}
```

이 예시는 로그인 후 셸의 이전 `CODEX_HOME`을 복원합니다. 등록한 디렉터리 문자열에서는 맨 앞의 `~/` 또는 단독 `~`를 확장합니다. JSON 파일 안의 `%ENV%`, `$ENV`, `~\`는 확장하지 않습니다. 절대 경로도 사용할 수 있습니다.

Claude 행은 **계정과 조직**을 구별합니다. 두 디렉터리에 같은 계정과 조직으로 로그인해도 별도의 할당량 풀이 생기지 않습니다. Antigravity 다중 계정 지원은 구현하지 않았습니다. `known` 계정 목록은 qhud가 자동으로 학습하지 않으며 운영자가 관리합니다. 자리표시자를 잊기 처리해도 활성 계정이 숨겨지지는 않습니다.

<!-- qhud:anchor -->
<a id="windows-personal-and-team-profiles"></a>

### Windows 개인 및 팀 프로필

두 이메일의 개인·팀 범위를 Codex와 Claude에서 모두 표시하려면 `accounts.json`의
`profiles`에 연결할 항목 여덟 개를 등록합니다. 등록은 연결 대기 표시를 만들며,
인증 정보를 생성하거나 앱의 로그인을 전환하거나 계정 ID를 만들어 내지 않습니다.
아래 합성 예시의 이메일과 비공개 디렉터리를 자신의 것으로 바꾸고 기존 설정에
항목을 병합하세요.

```json
{
  "profiles": [
    { "key": "codex-first-personal", "provider": "codex", "label": "First personal", "email": "first@example.com", "scope": "personal", "directory": "~/qhud-accounts/codex-first" },
    { "key": "codex-first-team", "provider": "codex", "label": "First team", "email": "first@example.com", "scope": "team", "directory": "~/qhud-accounts/codex-first" },
    { "key": "codex-second-personal", "provider": "codex", "label": "Second personal", "email": "second@example.com", "scope": "personal", "directory": "~/qhud-accounts/codex-second" },
    { "key": "codex-second-team", "provider": "codex", "label": "Second team", "email": "second@example.com", "scope": "team", "directory": "~/qhud-accounts/codex-second" },
    { "key": "claude-first-personal", "provider": "claude", "label": "First personal", "email": "first@example.com", "scope": "personal", "directory": "~/qhud-accounts/claude-first-personal" },
    { "key": "claude-first-team", "provider": "claude", "label": "First team", "email": "first@example.com", "scope": "team", "directory": "~/qhud-accounts/claude-first-team" },
    { "key": "claude-second-personal", "provider": "claude", "label": "Second personal", "email": "second@example.com", "scope": "personal", "directory": "~/qhud-accounts/claude-second-personal" },
    { "key": "claude-second-team", "provider": "claude", "label": "Second team", "email": "second@example.com", "scope": "team", "directory": "~/qhud-accounts/claude-second-team" }
  ]
}
```

Codex 로그인 하나로 해당 이메일이 접근할 수 있는 개인·팀 작업공간을 조회할 수
있으므로 두 프로필이 하나의 격리된 Codex 홈을 공유할 수 있습니다. Claude는 선택한
개인·팀 조직별로 별도의 인증된 프로필이 필요합니다. Claude 로그인에서는 이메일뿐
아니라 요청한 조직도 선택하세요. 정확한 조직을 고정해야 하면 확인한
`expected_org_id`를 프로필에 추가하며 추측해서 넣지 않습니다. 두 이메일이 같은
Codex 팀 작업공간을 공유해도 이메일과 작업공간 ID로 구분합니다.

압축을 푼 Windows 패키지에서 `qhud.exe`가 있는 폴더 또는 소스 저장소 루트에 PowerShell을 엽니다. [연결 스크립트](../../../../scripts/Connect-Account.ps1)에 `-List`를 주면 등록된 프로필이 나옵니다. `-AllPending`으로 연결하거나 `-Profile codex-first-personal`로 예시의 프로필 하나를 선택하세요.

```powershell
.\scripts\Connect-Account.ps1 -List
.\scripts\Connect-Account.ps1 -AllPending
```

스크립트는 연결 대기 프로필을 처리하고 공급자·디렉터리 조합을 중복 제거하므로,
공유 Codex 홈에 두 번 로그인하지 않습니다. 로그인은 공급자의 자체 절차에서
사용자가 진행합니다. 격리된 프로필 디렉터리를 사용하며 기본 `~/.codex`와
`~/.claude` 로그인을 덮어쓰지 않고 보존합니다. 이후 qhud를 새로고침하세요.
미연결 프로필에는 수치 없이 **qhud 연결 필요**를 표시하고, 인증 만료·다른 이메일·다른
조직도 명시합니다. 일치하는 실제 계정/작업공간 행이 생기면 대기 항목을 대체합니다.
프로필을 추가해도 계정에 없는 개인·팀 작업공간 접근 권한이 생기지는 않습니다.

Claude 웹의 계정 메뉴에서 확인한 Free 플랜을 개인 프로필에
기록할 수 있습니다. `observed_plan`에 해당 프로필의 `email`, `plan` 값 `"free"`,
확인 시각 `verified_at_ms`(Unix 밀리초)를 저장합니다. 개인 행에는 **Free**와 함께
사용량이 제공되지 않는다고 표시합니다. 공급자가 해당 개인 조직의 사용량을
반환하면 그 사용량 행을 우선 표시합니다.

Codex 팀 프로필에는 작업공간의 관리자 Members 페이지에서 별도로 확인한 좌석
정보를 기록할 수 있습니다. qhud 연결 대기 중에도 **Standard · 웹 확인** 또는
**Premium · 웹 확인**으로 표시합니다. 이는 날짜가 있는 수동 확인 기록이며,
자동 갱신되는 API 값이나 qhud 연결 성공의 증거가 아닙니다. 툴팁과 펼친 상세에
출처와 확인 시각을 표시합니다. 사용량 행에는 이메일과 작업공간 ID가 모두 일치할
때만 붙이며, 공급자의 Business 등 실제 플랜을 대체하거나 해당 플랜에서 좌석
유형을 추정하지 않습니다.

<!-- qhud:anchor -->
<a id="network-and-freshness"></a>

## 네트워크 및 최신성

<!-- qhud:anchor -->
<a id="windows-provider-connection-status"></a>

### Windows 공급자 연결 상태

사용량이 없어도 Codex, Claude, Antigravity가 표시됩니다. **조회 준비**는 로컬
연결 조건을 찾았다는 뜻이고, **조회 성공**은 명시적인 새로고침에 성공했다는 뜻입니다.
공급자 상태를 펼치면 데이터 출처, 마지막 시도, 마지막 성공 스냅샷, 확인 가능한
인증 만료 시각과 복구 방법을 볼 수 있습니다. 인증 만료는 로컬 로그인 토큰의
만료이며 구독 만료가 아닙니다. 조회에 실패해도 기존 값은 경과 시간과 함께 남고,
없는 사용량을 0으로 표시하지 않습니다.

Codex는 활성 계정에 설치된 CLI의 `app-server`를 사용하며 Windows의
`Programs/OpenAI/Codex/bin` 설치 경로를 지원합니다. Claude는 Claude Code의
로그인을 사용합니다. Desktop 앱에 로그인해도 만료된 CLI 인증이 갱신되지는
않습니다. 안내에 따라 `claude auth login`을 완료하고 qhud를 새로고침하세요.
qhud는 로그인을 자동 조작하거나 자체적으로 refresh token을 교환하지 않습니다.

Antigravity는 기존 `agy` 리스너와 실행 중인 Windows 네이티브 앱의 로컬 데몬
등록을 지원합니다. 프로세스와 루프백 포트를 검증하고 앱이 반환한 계정에 사용량을
연결합니다. 이 로컬 인터페이스는 Antigravity 버전에 따라 달라질 수 있습니다.
저장된 결과와 현재 CLI 식별 정보가 일치하지 않으면 **마지막 조회 계정**으로
표시하며 현재 앱 로그인이라고 단정하지 않습니다. 로컬 서비스가 없으면 Antigravity를
실행하거나 열고 다시 조회하세요.

Antigravity 플랜 배지는 조회한 계정의 네이티브 `userTier.name`을 사용합니다.
예를 들어 **Google AI Pro**를 해당 계정의 사용량 스냅샷과 함께 저장합니다.
업그레이드 안내나 할당량 형태를 현재 플랜으로 간주하지 않습니다. 새 응답에 플랜
이름이 없으면 이전 배지를 지우며, 명시적 계정 `plans` 재정의는 계속 우선합니다.
공급자가 이름을 제공하지 않고 재정의도 없으면 배지를 표시하지 않습니다. 저장된
플랜 이름은 사용량 값과 같은 스냅샷 경과 시간 및 계정 소유 관계를 따릅니다.

`qhud --provider-health`는 이메일, 계정 ID, 경로, 인증 정보를 제외한 로컬 연결
진단을 출력합니다. `qhud --refresh-health`는 위젯과 같은 명령으로 세 공급자를
명시적으로 조회한 뒤 진단을 출력하므로 공급자 요청이 발생합니다. 두 명령 모두
추가 창을 열지 않으며 일반 백그라운드 관찰은 계속 수동적으로 동작합니다.

Windows 소스 빌드에서 `scripts/Build-Windows.ps1 -DependenciesRoot <path>`는
기존 수정된 qmonster 체크아웃을 덮어쓰지 않고 별도 의존성 디렉터리를 사용합니다.
기본 경로는 계속 인접한 `qhud-deps` 디렉터리입니다.

| 공급자 | 수동 관찰 | 명시적 새로고침 |
| --- | --- | --- |
| Claude | 가능한 경우 로컬 식별 정보, 상태줄, 사용량 캐시. | 기존 OAuth 액세스 토큰을 공급자 사용량 엔드포인트로 전송. |
| Codex | 로컬 식별 정보와 지원하는 창 분할 상태줄 데이터. | 활성 계정은 `codex app-server`, 대체 경로와 저장된 인증은 사용량/계정 엔드포인트. |
| Antigravity | 가능한 경우 로컬 계정, 네이티브 데몬 준비 상태와 창 분할 정보. | 검증된 Windows 네이티브 데몬 또는 실행 중인 `agy` 프로세스의 루프백 RPC. |

로컬 관찰은 2초마다 실행되며 공급자 API를 요청하지 않습니다. 자격 증명도 들어 있는 파일에서 식별 필드를 읽을 수 있습니다. 명시적 새로고침은 기존 액세스 토큰을 읽을 수 있으며, qhud 자체는 OAuth refresh grant를 실행하지 않습니다. 공급자 CLI가 인증과 위임된 토큰 갱신을 관리합니다.

조회 결과는 경과 시간과 함께 저장합니다. 계정 일치 확인으로 이전 로그인 스냅샷이 새 로그인 이름 아래 나타나는 것을 막습니다. 동시 새로고침도 서로의 저장 결과를 보존합니다. qhud는 qmonster 데이터베이스에 쓰지 않고 qmonster 파이프라인을 관찰합니다.

<!-- qhud:anchor -->
<a id="troubleshooting"></a>

## 문제 해결

| 증상 | 확인 사항 |
| --- | --- |
| 계정이 없음 | 공급자 CLI에 로그인되어 있고 qhud가 의도한 설정/홈 디렉터리를 읽는지 확인합니다. |
| 이메일 대신 ID 표시 | qhud가 v0.6.2 이상이며 의도한 Codex 홈을 읽는지 확인합니다. 로컬 ID 토큰에 사용 가능한 이메일이 없으면 일치하는 `labels` 항목으로 표시를 지정합니다. |
| Spark 또는 다른 모델이 없음 | 새로고침 후 현재 계정의 응답을 확인합니다. 없는 모델은 만들어 표시하지 않습니다. |
| 오래된 값 또는 401 오류 | 행의 경과 시간/오류를 읽고, 영향을 받은 디렉터리에서 해당 공급자 CLI로 다시 로그인합니다. |
| 터미널 창 분할이 없음 | Linux에서는 tmux/herdr를 확인합니다. 네이티브 Windows 터미널 탭은 이 릴리스의 지원 범위 밖입니다. |
| 위젯이 보이지 않음 | 트레이를 확인하고 실행 중 인스턴스에 `--peek`를 사용하며 데스크톱 잠금이 해제되어 있는지 확인합니다. |
| 행이 창 밖으로 넘어감 | 사용량 영역을 스크롤하거나 창 크기를 늘립니다. |
| 디스플레이 절전 후 Linux 픽셀이 멈춤 | [실행 지침서](05-ops/RUNBOOK.md#troubleshooting)의 프레임 가드 진단을 참조합니다. |

버그를 보고하기 전에 qhud 버전, OS, 공급자, 설치 방법, 재현 단계, 민감한 내용을 가린 오류를 기록합니다. 자동 검사와 실제 데스크톱 검증의 차이는 [기여 안내](../CONTRIBUTING.md)와 [테스트 계획](04-quality/TEST_PLAN.md)을 참조하세요.
