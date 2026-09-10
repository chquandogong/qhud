<!-- qhud:languages -->
<p align="center">
  <strong>한국어</strong> · <a href="../../../../05-ops/WINDOWS.md">English</a> · <a href="../../../zh-CN/docs/05-ops/WINDOWS.md">简体中文</a>
</p>
<!-- /qhud:languages -->

<!-- qhud:anchor -->
<a id="windows-native-build-and-run"></a>

# Windows 네이티브 빌드와 실행

qhud는 v0.6.0부터 Windows x64 WebView2 위젯을 제공합니다. WSL 없이 계정 사용량을 표시합니다. 프런트엔드는 저장소의 `ui/` 파일을 사용하므로 Node.js나 npm이 필요하지 않습니다.

<!-- qhud:anchor -->
<a id="install-a-release"></a>

## 릴리스 설치

[GitHub Releases](https://github.com/chquandogong/qhud/releases)에서 `qhud-v0.6.2-windows-x86_64.zip`과 체크섬 파일을 받습니다. 압축을 풀고 버전명이 붙은 디렉터리 안의 `qhud.exe`를 실행합니다. Microsoft Edge WebView2 Runtime이 필요합니다.

ZIP은 휴대용 배포판입니다. 설치 프로그램, 로그인 자동 시작, 바로가기를 자동 등록하지 않습니다. 업데이트할 때는 트레이의 **Quit qhud**로 종료한 뒤 실행 파일을 교체합니다. 설정과 공급자 로그인 파일은 별도 위치에 유지됩니다.

<!-- qhud:anchor -->
<a id="build-from-source"></a>

## 소스 빌드

- Git for Windows, PATH에 `git.exe` 필요.
- Rust 1.88 이상과 `x86_64-pc-windows-msvc` 툴체인.
- **Desktop development with C++** 및 Windows SDK가 설치된 Visual Studio Build Tools.
- Microsoft Edge WebView2 Runtime.

Developer PowerShell for Visual Studio에서 저장소로 이동해 실행합니다.

```powershell
.\scripts\Build-Windows.ps1 -Test
```

개발 셸 밖에서는 C++ 도구 설치 경로를 지정합니다.

```powershell
.\scripts\Build-Windows.ps1 -Test -VisualStudioPath 'C:\BuildTools'
```

결과는 `target\x86_64-pc-windows-msvc\release\qhud.exe`입니다. `-Run`은 빌드 후 실행, `-DebugBuild`는 debug 프로필 선택, `-PrepareOnly`는 컴파일 없이 도구와 의존성 준비입니다. `-Test`는 같은 빌드 설정으로 테스트를 먼저 실행합니다. `-PrepareOnly`와 `-Run` 또는 `-Test`를 함께 사용하지 마세요.

스크립트는 PATH의 도구와 기존 Visual Studio 개발 환경을 우선합니다. 저장소 옆의 `tools/rust/cargo/bin`, `tools/cargo/bin`, `tools/vs-buildtools`도 확인합니다. 도구를 설치하거나 전역 PATH를 바꾸지 않습니다.

Windows 빌드는 형제 `qhud-deps/qmonster` 디렉터리에 고정 revision을 준비하고 `patches/qmonster-windows.patch`를 적용합니다. 수정된 다른 checkout은 덮어쓰지 않습니다. 해당 Cargo 명령의 임시 설정으로만 패치를 적용하고 잠시 수정한 lockfile은 성공 여부와 관계없이 원래 바이트로 복원합니다. 같은 checkout에서 다른 Cargo 명령을 동시에 실행하지 마세요. 일반 Linux `cargo build --release --locked`는 고정 Git 의존성을 사용하며 이 준비 단계가 필요하지 않습니다.

<!-- qhud:anchor -->
<a id="run-and-interact"></a>

## 실행과 조작

```powershell
Start-Process -FilePath '.\target\x86_64-pc-windows-msvc\release\qhud.exe'
& '.\target\x86_64-pc-windows-msvc\release\qhud.exe' --peek
& '.\target\x86_64-pc-windows-msvc\release\qhud.exe' --refresh-all
```

상단이나 푸터를 드래그해 이동하고 모서리로 크기를 조절합니다. 트레이의 **Quit qhud**로 종료합니다. 실행 중이면 이후 `--peek`와 `--refresh-all` 요청을 기존 인스턴스로 전달합니다. 사용량은 ⟳ 또는 명시적 새로고침 명령으로 조회합니다. 로컬 관찰 주기는 네트워크 새로고침 주기가 아닙니다.

<!-- qhud:anchor -->
<a id="account-and-model-usage"></a>

## 계정 및 모델 사용량

새 Windows 설치는 `%APPDATA%\qhud`에 설정을 저장합니다. `accounts.json`은 표시 이름과 등록한 계정 경로, `fetched-usage.json`은 조회 시각을 포함한 사용량을 담습니다. 개인 설정과 조회 결과는 Git 저장소나 릴리스에 포함하지 않습니다.

`QHUD_CONFIG_DIR`이 설정 위치를 재정의합니다. `XDG_CONFIG_HOME`이나 기존 `%USERPROFILE%\.config\qhud`를 존중합니다. 공급자 로그인 정보는 기본적으로 `%USERPROFILE%\.codex`와 `%USERPROFILE%\.claude`에서 읽으며 `CODEX_HOME`, `CLAUDE_CONFIG_DIR`로 바꿀 수 있습니다. Codex는 로컬 `auth.json`의 ID 토큰에서 이메일을 읽으므로 일반적으로 새 컴퓨터에 이메일 매핑이 필요하지 않습니다. 이메일이 없거나 형식이 잘못되면 계정 ID를 표시합니다. `accounts.json`의 `labels`는 계정/작업 공간 식별자를 바꾸지 않고 이메일 표시보다 우선합니다. 스키마는 [운영 가이드](RUNBOOK.md)를 참고하세요.

GPT-5.3-Codex-Spark 5H/7D 같은 모델 한도는 서버가 반환할 때 표시합니다. 긴 이름은 줄바꿈하고 항목이 많으면 사용량 영역을 스크롤합니다. 오른쪽 시간은 초기화까지 남은 시간이며 계기에 마우스를 올리면 정확한 초기화 날짜와 시각을 확인할 수 있습니다. 누락된 모델 한도를 0%로 가정하거나 다른 계정의 수치를 대신 표시하지 않습니다.

<!-- qhud:anchor -->
<a id="support-and-verification"></a>

## 지원과 검증

Linux GTK 프레임 복구, X11 창 처리, tmux/herdr 관찰을 유지합니다. Windows Terminal, PowerShell, WezTerm 탭의 네이티브 관찰은 구현하지 않았습니다. 관찰 가능한 멀티플렉서가 없으면 실제 로컬 계정과 `panes = 0`을 표시하며 예시 데이터에는 `--demo`가 필요합니다. Linux `/proc` 프로세스 지표는 Windows에서 없을 수 있습니다.

CI는 Ubuntu 24.04와 Windows에서 테스트 및 릴리스 빌드를 실행하며 모두 성공해야 공개합니다. 단위 테스트는 Spark 5H/7D 사용률과 초기화 보존, 모델별 수치만 있는 스냅샷 복원, 동시 저장 결과 보존을 검증합니다. 자동 빌드와 테스트가 모든 데스크톱 또는 공급자 계정의 실제 동작을 입증하지는 않습니다.
