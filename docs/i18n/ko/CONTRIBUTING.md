<!-- qhud:languages -->
<p align="center">
  <a href="../../../CONTRIBUTING.md">English</a> · <strong>한국어</strong> · <a href="../zh-CN/CONTRIBUTING.md">简体中文</a>
</p>
<!-- /qhud:languages -->

<!-- qhud:anchor -->
<a id="contributing-to-qhud"></a>

# qhud에 기여하기

qhud를 더 명확하고 신뢰할 수 있게 만드는 데 참여해 주셔서 감사합니다. 이슈, 풀 리퀘스트, 문서 개선은 **English, 한국어, 简体中文**으로 환영합니다.

<!-- qhud:anchor -->
<a id="before-you-start"></a>

## 시작하기 전에

- [사용자 가이드](docs/GUIDE.md)에서 지원 동작과 알려진 제약을 읽어 보세요.
- 새 보고를 작성하기 전에 [기존 이슈](https://github.com/chquandogong/qhud/issues)를 검색하세요. 가능하다면 작고 재현 가능한 예시를 포함합니다.
- 동작이나 아키텍처를 크게 바꾸려면 광범위한 구현에 착수하기 전에 이슈로 논의하세요. 작은 수정과 번역은 바로 PR을 제출해도 됩니다.
- 각 변경의 범위를 집중하세요. 문제, 변경 후 동작, 검증 방법을 설명합니다.

<!-- qhud:anchor -->
<a id="report-a-bug"></a>

## 버그 보고

[버그 보고 양식](https://github.com/chquandogong/qhud/issues/new?template=bug_report.yml)을 사용합니다. qhud 버전, OS/데스크톱, 설치 방법, 공급자, 재현 단계, 기대 결과, 실제 결과를 포함하세요. 새 공급자 데이터와 저장된 스냅샷을 구분합니다. 화면 문제라면 데스크톱 잠금이 해제되어 있는지, 트레이·새로고침·창 조작이 반응하는지 설명하세요.

진단에는 이메일, 계정 ID, 로컬 경로, 세션 내용, 자격 증명이 포함될 수 있습니다. 공유 전에 내용을 확인하고 민감한 부분을 가리세요. `auth.json`, `.credentials.json`, 공급자 설정 디렉터리 전체를 첨부하지 마세요. 자격 증명 노출 보고라면 공개 이슈에 자격 증명을 게시하지 않고 문제를 설명하세요.

<!-- qhud:anchor -->
<a id="build-and-check"></a>

## 빌드 및 검사

저장소는 Rust 1.88 이상을 선언하며 CI는 stable Rust를 사용합니다. 프런트엔드는 일반 HTML/CSS/JavaScript로 작성되어 npm이 필요하지 않습니다. qmonster는 `src-tauri/Cargo.toml`의 리비전으로 고정되어 있습니다.

### Linux

[실행 지침서](docs/05-ops/RUNBOOK.md)에 문서화된 개발 의존성을 설치한 뒤 실행합니다.

```sh
cargo fmt --all --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
cargo build --locked --release
```

### Windows

C++ 워크로드와 Windows SDK를 설치한 Visual Studio용 Developer PowerShell을 사용합니다.

```powershell
.\scripts\Build-Windows.ps1 -Test
git diff --exit-code -- Cargo.toml Cargo.lock
```

스크립트는 고정된 qmonster 체크아웃을 준비하고 자체 Cargo 명령에만 Windows 패치를 적용합니다. 이후 정본 lockfile을 복원합니다. 같은 체크아웃에서 다른 Cargo 명령을 동시에 실행하지 마세요. 도구 탐색과 명시적 경로는 [Windows 가이드](docs/05-ops/WINDOWS.md)를 참조하세요.

<!-- qhud:anchor -->
<a id="protect-the-data-contract"></a>

## 데이터 계약 보호

- 각 할당량이 계정, 조직, 모델 범위, 기간 길이, 초기화 시점과 연결된 상태를 유지하세요. 누락된 데이터를 임의의 0으로 바꾸면 안 됩니다.
- 스냅샷의 경과 시간과 소유권을 보존하세요. 파싱 성공이 저장된 값이 현재 로그인에 속한다는 증거는 아닙니다.
- 공급자 요청은 명시적 새로고침 동작 뒤에 두세요. qhud는 공급자 로그인을 소유하거나 자체 OAuth refresh grant를 실행하지 않습니다.
- Windows 전용 적응은 범위를 한정하세요. Linux 빌드는 개발자의 형제 의존성 디렉터리 없이 일반 클론에서 계속 동작해야 합니다.
- Linux에서는 Tauri/WebKitGTK 프로세스에 Unix 시그널 핸들러를 설치하지 마세요. [D-012](docs/02-decisions/DECISION_LOG.md)를 참조하세요.

[명세](docs/03-spec/SPEC.md), [아키텍처](docs/03-spec/ARCHITECTURE.md), [결정 로그](docs/02-decisions/DECISION_LOG.md)에 이러한 제약을 설명합니다.

<!-- qhud:anchor -->
<a id="documentation-and-translations"></a>

## 문서 및 번역

영어 원문은 저장소 루트와 `docs/` 아래에 있습니다. `README.ko.md`와 `README.zh-CN.md`가 현지화된 진입점입니다. 전체 한국어 및 중국어 간체 미러는 `docs/i18n/ko/`와 `docs/i18n/zh-CN/` 아래에 있으며, 원문 기준 상대 문서 경로를 유지합니다.

문서를 변경할 때는 대응하는 번역도 갱신하세요. 명령 문법, API 식별자, 날짜, 요구사항/결정 ID, 표의 모든 행을 보존합니다. 역사 기록은 해당 시점을 설명하므로 과거 관찰을 현재 사실처럼 조용히 바꾸지 마세요. MIT 라이선스 문구는 원문 그대로 유지합니다.

각 문서의 실제 디렉터리에서 상대 링크와 섹션 앵커를 확인하세요. 기존 스크린샷이나 데모라고 명확히 표시한 스크린샷을 사용하고, 예시 사용량을 실시간 계정 값처럼 제시하지 마세요. 문서 색인은 모든 페이지를 [영어](../../../docs/README.md), [한국어](docs/README.md), [중국어 간체](../zh-CN/docs/README.md)로 나열합니다.

<!-- qhud:anchor -->
<a id="submit-a-pull-request"></a>

## 풀 리퀘스트 제출

변경 내용과 이유, 관련 검증, 남은 플랫폼 또는 화면 검증 공백을 설명하세요. 파서, 범위, 영속화, 식별 정보 동작을 바꿀 때는 회귀 테스트를 추가합니다. 문서만 변경했다면 앱 재빌드 대신 링크, 번역, 렌더링 검사가 필요합니다.

빌드 통과가 데스크톱 동작을 입증하지는 않습니다. 창/계층을 변경할 때는 [테스트 계획](docs/04-quality/TEST_PLAN.md)을 따르고 자동 검사와 실제 컴포지터/픽셀 검증을 구분하세요. 합성 픽스처만으로 실제 공급자 테스트를 수행했다고 주장하지 마세요.

<!-- qhud:anchor -->
<a id="release-maintenance"></a>

## 릴리스 유지보수

버전 아카이브는 두 플랫폼 작업이 모두 통과한 뒤에만 기존 릴리스 워크플로를 통해 공개합니다. 패키지 버전, Tauri 설정, lockfile, 변경 이력, 릴리스 노트를 일치시키세요. 문서와 디자인 갱신만으로는 새 바이너리 릴리스가 필요하지 않습니다. 전체 절차는 [실행 지침서](docs/05-ops/RUNBOOK.md#release-procedure)에 있습니다.
