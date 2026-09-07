# Windows 네이티브 빌드와 실행

qhud v0.6.0부터 Windows x64용 WebView2 위젯을 제공합니다. WSL 없이 계정별
사용량을 표시합니다. 저장소의 ui/ 파일을 사용하므로 Node.js나 npm은 필요하지 않습니다.

## 릴리스 설치

[GitHub Releases](https://github.com/chquandogong/qhud/releases)에서
`qhud-v0.6.0-windows-x86_64.zip`과 checksum 파일을 받고 압축을 푼 뒤 `qhud.exe`를
실행합니다. Microsoft Edge WebView2 Runtime이 필요합니다.

ZIP은 휴대용 실행 파일입니다. 설치 프로그램, 로그인 자동 실행, 바로가기를 자동으로
등록하지 않습니다. 업데이트할 때는 트레이의 **Quit qhud**로 종료한 다음 새 파일을
복사합니다. 설정과 계정 로그인 파일은 별도 위치에 유지됩니다.

## 소스 빌드

- Git for Windows (`git.exe`가 PATH에 있어야 합니다).
- Rust 1.88 이상과 `x86_64-pc-windows-msvc` 툴체인.
- Visual Studio Build Tools의 **Desktop development with C++** 워크로드와 Windows SDK.
- Microsoft Edge WebView2 Runtime.

Developer PowerShell for Visual Studio에서 저장소로 이동해 실행합니다.

```powershell
.\scripts\Build-Windows.ps1 -Test
```

개발 셸 밖에서는 C++ 도구 설치 경로를 지정할 수 있습니다.

```powershell
.\scripts\Build-Windows.ps1 -Test -VisualStudioPath 'C:\BuildTools'
```

결과는 `target\x86_64-pc-windows-msvc\release\qhud.exe`입니다. `-Run`은 빌드 뒤
실행, `-DebugBuild`는 debug 프로필, `-PrepareOnly`는 도구와 의존성 준비까지만
수행합니다. `-Test`는 같은 빌드 설정으로 테스트를 먼저 실행합니다. `-PrepareOnly`는
`-Run` 또는 `-Test`와 함께 쓰지 않습니다.

스크립트는 PATH의 도구와 기존 Visual Studio 개발 환경을 우선합니다. 도구가 없으면
저장소 상위의 `tools/rust/cargo/bin`, `tools/cargo/bin`, `tools/vs-buildtools`도
확인합니다. 도구 설치나 전역 PATH 변경은 하지 않습니다.

Windows 빌드는 저장소 상위 `qhud-deps/qmonster`에 고정 revision을 준비하고
`patches/qmonster-windows.patch`를 적용합니다. 변경된 다른 checkout은 덮어쓰지
않습니다. patch는 해당 Cargo 명령의 임시 설정으로만 적용하며, 잠시 바꾼 lockfile은
성공·실패 시 원래 바이트로 복원합니다. 같은 checkout에서 다른 Cargo 명령을 동시에
실행하지 마세요. Linux의 일반 `cargo build --release --locked`는 원래 고정 Git
의존성을 사용하고 이 준비 단계가 필요하지 않습니다.

## 실행과 조작

```powershell
Start-Process -FilePath '.\target\x86_64-pc-windows-msvc\release\qhud.exe'
& '.\target\x86_64-pc-windows-msvc\release\qhud.exe' --peek
& '.\target\x86_64-pc-windows-msvc\release\qhud.exe' --refresh-all
```

윗줄과 아랫줄을 드래그하여 이동하고 모서리에서 크기를 조절합니다. 트레이 메뉴의
**Quit qhud**로 종료합니다. 이미 실행 중이면 다음 실행의 `--peek`, `--refresh-all`
요청이 기존 인스턴스로 전달됩니다. 계정 사용량은 화면의 ⟳ 또는 명시적 새로고침
명령으로 조회합니다. 로컬 관찰 주기는 네트워크 새로고침 주기가 아닙니다.

## 계정·모델별 사용량

새 Windows 설치의 기본 설정 위치는 `%APPDATA%\qhud`입니다. `accounts.json`에는
표시 이름과 등록한 계정 경로, `fetched-usage.json`에는 조회 시각을 포함한 사용량이
저장됩니다. 개인 설정과 조회 결과는 Git 저장소 및 릴리스에 포함하지 않습니다.

`QHUD_CONFIG_DIR`은 설정 위치를 덮어씁니다. `XDG_CONFIG_HOME` 또는 기존
`%USERPROFILE%\.config\qhud`가 있으면 해당 경로를 존중합니다. 공급자 로그인은
기본적으로 `%USERPROFILE%\.codex`, `%USERPROFILE%\.claude`를 읽으며 `CODEX_HOME`,
`CLAUDE_CONFIG_DIR` 설정을 따릅니다. Codex가 로컬에 workspace ID만 제공하면
`accounts.json`의 `labels`로 이메일을 표시할 수 있습니다. 스키마는
[RUNBOOK](RUNBOOK.md)을 참고하세요.

Codex 모델별 한도(예: GPT-5.3-Codex-Spark 5H/7D)는 서버가 반환한 항목을 표시합니다.
긴 이름은 줄을 바꾸고 항목이 많으면 사용량 영역을 스크롤할 수 있습니다. 오른쪽 시간은
reset까지 남은 시간이며, 막대에 마우스를 올리면 정확한 초기화 날짜·시각을 확인합니다.
서버가 특정 모델의 한도를 반환하지 않으면 그 항목을 0%로 가정하거나 다른 계정의
수치를 대신 표시하지 않습니다.

## 지원 범위와 검증

기존 Linux의 GTK 프레임 복구, X11 창 처리, tmux/herdr 관찰은 유지합니다.
Windows Terminal·PowerShell·WezTerm 탭을 직접 관찰하는 Windows 백엔드는 없습니다.
관찰할 multiplexer가 없으면 실제 로컬 계정과 `panes = 0`을 표시하고 예시 데이터는
`--demo`에서만 사용합니다. Linux `/proc` 기반 프로세스 지표는 Windows에서 비어
있을 수 있습니다.

CI는 Ubuntu 24.04와 Windows에서 테스트 및 release 빌드를 수행하며, 두 플랫폼이
성공해야 릴리스를 공개합니다. 단위 테스트에는 Spark 5H/7D의 사용률·reset 보존,
모델별 한도만 있는 snapshot 복원, 동시 저장 결과 보존이 포함됩니다. 자동화된
빌드·테스트가 모든 데스크톱과 공급자 계정의 실제 동작을 보장하지는 않습니다.
