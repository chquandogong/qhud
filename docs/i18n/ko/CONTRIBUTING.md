<!-- qhud:languages -->
<p align="center">
  <strong>한국어</strong> · <a href="../../../CONTRIBUTING.md">English</a> · <a href="../zh-CN/CONTRIBUTING.md">简体中文</a>
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

`main`의 평상시 stderr 추적 로그는 식별 값을 생략합니다. 공개된 v0.7.1 바이너리는
이 보안 변경 전의 빌드입니다. `--dump`, `--codex-usage` 같은 명시적 JSON 명령은
어느 리비전에서든 운영자가 요청한 전체 페이로드를 출력하므로 공유 전에
내용을 확인하고 가려야 합니다.

<!-- qhud:anchor -->
<a id="build-and-check"></a>

## 빌드 및 검사

저장소는 Rust 1.88 이상을 선언하며 CI는 stable Rust를 사용합니다. 앱 빌드는 일반 HTML/CSS/JavaScript를 사용하므로 Node.js나 npm이 필요하지 않습니다. Node.js 22는 별도의 프런트엔드 시스템 지표 회귀 테스트에만 사용합니다. qmonster는 `src-tauri/Cargo.toml`의 리비전으로 고정되어 있습니다.

개발 플랫폼과 관계없이 다음 검사를 실행합니다.

```sh
node --test tests/system-metrics.test.cjs
```

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

기본 `README.md`는 한국어입니다. 영어는 `README.en.md`, 중국어 간체는 `README.zh-CN.md`에서 제공합니다. 기존 링크를 위해 `README.ko.md`는 `README.md`와 동일하게 유지하세요. 나머지 영어 원문은 저장소 루트와 `docs/` 아래에 있습니다. 전체 한국어 및 중국어 간체 미러는 `docs/i18n/ko/`와 `docs/i18n/zh-CN/` 아래에 있으며, 원문 기준 상대 문서 경로를 유지합니다.

<!-- qhud:anchor -->
<a id="which-documents-are-mirrored"></a>

### 미러 대상 문서

- **세 언어 전체 미러.** `README.en.md`와 그 한국어판 `README.md`, 중국어판 `README.zh-CN.md`; `CHANGELOG.md`, `CODE_OF_CONDUCT.md`, `CONTRIBUTING.md`, `SECURITY.md`, `SUPPORT.md`; 날짜가 붙은 연구, 결정, 회고, 릴리스 노트를 포함한 `docs/` 아래의 모든 Markdown 파일. 각 미러는 요약이 아닌 전체 번역입니다.
- **하나의 기준 언어, 미러 없음.** `AGENTS.md`와 `.github/copilot-instructions.md`는 코딩 에이전트를 위한 영어 지침입니다. PR 템플릿과 이슈 양식은 한 파일 안에 세 언어를 함께 담습니다. `LICENSE`는 원래 영어 문구를 유지합니다. 소스 주석, 커밋 메시지, 워크플로 파일은 영어로 작성합니다.
- **기준 기록과 요약.** 현재 이 등급을 쓰는 문서는 없습니다. 날짜가 붙은 내부 기록(오피스 아워, 연구 노트, 타당성 보고서, 대안, 교차 검증 로그, 회고)만 후보입니다. 문서를 이 등급으로 옮기려면 결정 로그 항목과 `scripts/check-repository.mjs`의 등록 목록 변경이 필요하며, 그 전까지는 전체 미러로 유지합니다.

저장소 검사기가 이 등록 목록을 가지고 있으며, 분류되지 않은 새 루트 Markdown 문서를 거부합니다.

<!-- qhud:anchor -->
<a id="keep-facts-mechanically-comparable"></a>

### 기계적으로 비교할 수 있는 사실 유지

문서를 변경할 때는 같은 PR에서 두 번역을 함께 갱신하세요. 문장은 번역하되 사실은 번역하지 않습니다. `node scripts/check-repository.mjs`는 모든 미러에서 다음 항목을 영어 원문과 비교하고, 하나라도 다르면 실패합니다.

- 코드 블록 밖의 요구사항·결정·위험·가정 ID(`FR-`, `NFR-`, `D-`, `CV-`, `R1`, `A1` 형식)
- `YYYY-MM-DD` 날짜와 릴리스 버전 집합
- 들여쓰기를 제거한 뒤 줄 단위로 비교하는 펜스 코드 블록
- 표 행 수와 제목 수

명령 문법, API 식별자, 인라인 코드는 바꾸지 말고 표의 모든 행을 유지하세요. 역사 기록은 해당 시점을 설명하므로 과거 관찰을 현재 사실처럼 조용히 바꾸지 마세요. MIT 라이선스 문구는 원문 그대로 유지합니다.

문서만 변경했다면 `node scripts/check-repository.mjs`로 언어별 파일 목록, 번역 사실, 버전,
릴리스 노트 링크, 로컬 Markdown 링크를 검사하세요. 다운로드에 표시하는 버전은
최신 공개 릴리스에 맞추고, 새 바이너리 태그 전의 `main` 동작은 미출시로 구분합니다.

각 문서의 실제 디렉터리에서 상대 링크와 섹션 앵커를 확인하세요. 기존 스크린샷이나 데모라고 명확히 표시한 스크린샷을 사용하고, 예시 사용량을 실시간 계정 값처럼 제시하지 마세요. 문서 색인은 모든 페이지를 [영어](../../../docs/README.md), [한국어](docs/README.md), [중국어 간체](../zh-CN/docs/README.md)로 나열합니다.

<!-- qhud:anchor -->
<a id="pull-request-translation-signal"></a>

### PR 번역 신호

PR에서는 `docs · release metadata` 검사가 `--base origin/<기준 브랜치>`로 검사기를 한 번 더 실행합니다. PR이 영어 원문을 바꾸었는데 두 미러를 모두 바꾸지 않았다면 실패합니다. 푸시하기 전에 같은 비교를 실행하세요.

```sh
node scripts/check-repository.mjs --base origin/main
```

오타, 깨진 링크, 형식 수정처럼 번역에는 해당하지 않는 영어 전용 변경이 맞다면 PR의 아무 커밋 메시지에나 트레일러를 넣어 확인 표시를 하세요. 경로와 이유를 적습니다.

```text
Translation-Exempt: docs/GUIDE.md fix an English misspelling
```

그러면 검사는 통과하고 확인 표시를 알림으로 보고합니다. 의미 변경, 사실 추가·삭제, 버전 갱신에는 트레일러를 쓰지 마세요. 이런 변경에는 두 번역이 모두 필요합니다. 번역만 고치는 변경에는 트레일러가 필요 없습니다.

<!-- qhud:anchor -->
<a id="reviewer-checklist"></a>

### 검토자 체크리스트

문서 변경을 승인하기 전에 각 언어판에서 다음을 확인하세요.

1. **의미.** 미러가 영어 원문과 같은 주장, 조건, 한계, 주의 사항을 담고 있으며 추가, 누락, 완화가 없습니다. 날짜가 붙은 관찰은 모든 언어에서 날짜를 유지합니다.
2. **검사기가 보지 못하는 사실.** 본문의 숫자와 단위, 제품·명령·설정 이름, 인용한 UI 레이블이 원문과 일치합니다.
3. **링크.** 상대 링크와 앵커가 번역 파일의 실제 디렉터리에서 해석됩니다. 번역된 제목은 `<!-- qhud:anchor -->`와 `<a id>`로 영어 앵커를 유지하고, 릴리스 노트의 언어 링크는 태그에 고정된 절대 URL로 유지합니다.
4. **개인정보.** 어떤 언어판에도 이메일 주소, 계정·조직 ID, 토큰, 쿠키, 세션 내용, 로컬 경로, 가리지 않은 진단이 새로 들어가지 않으며, 스크린샷은 모든 언어에서 데모이거나 가려져 있습니다.
5. **확인 표시.** 각 `Translation-Exempt` 트레일러가 올바른 경로를 가리키고 실제로 영어에만 해당하는 변경을 설명합니다.

CI가 이를 적용하는 방식과 첫 시험 적용의 비용은 [저장소 운영 가이드](docs/05-ops/REPOSITORY.md#translation-maintenance)에 기록되어 있습니다.

<!-- qhud:anchor -->
<a id="submit-a-pull-request"></a>

## 풀 리퀘스트 제출

변경 내용과 이유, 관련 검증, 남은 플랫폼 또는 화면 검증 공백을 설명하세요. 파서, 범위, 영속화, 식별 정보 동작을 바꿀 때는 회귀 테스트를 추가합니다. 문서만 변경했다면 앱 재빌드 대신 링크, 번역, 렌더링 검사가 필요합니다.

집중된 브랜치에서 `main` 대상 PR을 열고, 릴리스 태그를 검토 대신 푸시하지 마세요.
보호된 브랜치는 Linux·Windows 빌드/테스트, 저장소 무결성·의존성 검토,
Rust·JavaScript/TypeScript·Actions CodeQL 검사를 요구합니다. 검토 대화를
해결하고 검사 결과가 오래됐다면 브랜치를 갱신하세요. 1인 운영에는 필수 승인이
0건이며 팀 저장소라면 독립 검토자를 요구해야 합니다. 설정과 private/public
변형은 [저장소 운영 가이드](docs/05-ops/REPOSITORY.md)를 참조하세요.

빌드 통과가 데스크톱 동작을 입증하지는 않습니다. 창/계층을 변경할 때는 [테스트 계획](docs/04-quality/TEST_PLAN.md)을 따르고 자동 검사와 실제 컴포지터/픽셀 검증을 구분하세요. 합성 픽스처만으로 실제 공급자 테스트를 수행했다고 주장하지 마세요.

<!-- qhud:anchor -->
<a id="release-maintenance"></a>

## 릴리스 유지보수

버전 아카이브는 두 플랫폼 작업이 모두 통과한 뒤에만 기존 릴리스 워크플로를 통해 공개합니다. 패키지 버전, Tauri 설정, lockfile, 변경 이력, 릴리스 노트를 일치시키세요. 문서와 디자인 갱신만으로는 새 바이너리 릴리스가 필요하지 않습니다. 전체 절차는 [실행 지침서](docs/05-ops/RUNBOOK.md#release-procedure)에 있습니다.
