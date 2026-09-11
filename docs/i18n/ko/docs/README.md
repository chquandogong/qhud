<!-- qhud:languages -->
<p align="center">
  <strong>한국어</strong> · <a href="../../../README.md">English</a> · <a href="../../zh-CN/docs/README.md">简体中文</a>
</p>
<!-- /qhud:languages -->

<!-- qhud:anchor -->
<a id="documentation"></a>

# 문서


qhud의 전체 문서를 최초 설치부터 아키텍처, 결정, 릴리스 이력까지 세 언어 모두로 제공합니다.

<!-- qhud:anchor -->
<a id="start-using-qhud"></a>

## qhud 사용 시작

| 문서 | 다루는 내용 |
| --- | --- |
| [프로젝트 소개](../../../../README.md) | 기능, 스크린샷, 플랫폼 지원, 다운로드. |
| [사용자 가이드](GUIDE.md) | 사용량/초기화 의미, 명령, 다중 계정, 설정, 문제 해결. |
| [Windows 설정](05-ops/WINDOWS.md) | WebView2 요구사항, 포터블 설치, 네이티브 MSVC 빌드. |
| [운영 지침서](05-ops/RUNBOOK.md) | Linux 설정, GNOME 통합, 진단, 릴리스 절차. |
| [기여 안내](../CONTRIBUTING.md) | 버그 보고, 개발 검사, 번역, 풀 리퀘스트. |
| [변경 이력](../CHANGELOG.md) | 버전별 변경과 날짜가 있는 검증 기록. |

<!-- qhud:anchor -->
<a id="understand-the-implementation"></a>

## 구현 이해

| 문서 | 다루는 내용 |
| --- | --- |
| [프로젝트 대시보드](00-overview/DASHBOARD.md) | 현재 릴리스, 검증 증거, 작업 보드, 재개 지점. |
| [프로젝트 개요](00-overview/PROJECT_BRIEF.md) | 목적, 지원 범위, 요구사항, 비목표. |
| [명세](03-spec/SPEC.md) | 요구사항, CLI 플래그, 환경 변수, 페이로드 계약. |
| [아키텍처](03-spec/ARCHITECTURE.md) | 데이터 흐름, 모듈, 식별 정보, 공급자 조회 경로, 플랫폼 경계. |
| [결정 로그](02-decisions/DECISION_LOG.md) | 설계 결정과 그 근거. |
| [테스트 계획](04-quality/TEST_PLAN.md) | 자동 검사와 날짜가 있는 데스크톱 검증. |
| [위험 등록부](04-quality/RISK_REGISTER.md) | 알려진 위험, 완화책, 관찰된 실패. |
| [가정](01-discovery/ASSUMPTIONS.md) | 현재 전제와 검증 상태. |

<!-- qhud:anchor -->
<a id="explore-the-project-history"></a>

## 프로젝트 이력 살펴보기

이 문서들은 당시 날짜의 조사와 결정을 보존합니다. 초기 제안이나 과거 검증 공백은 역사적 진술이며 반드시 현재 제품 동작을 뜻하지는 않습니다. 현재 상태는 명세와 대시보드부터 확인하세요.

| 문서 | 기록 |
| --- | --- |
| [상담 기록](01-discovery/OFFICE_HOURS.md) | 초기 문제 정의와 논의. |
| [조사 노트](01-discovery/RESEARCH_NOTES.md) | 초기 기술 참고 자료와 관찰. |
| [타당성 보고서](01-discovery/FEASIBILITY_REPORT.md) | 초기 구현 선택지와 타당성. |
| [대안](02-decisions/ALTERNATIVES.md) | 검토한 접근법과 장단점. |
| [교차 검증 로그](02-decisions/CROSS_VALIDATION_LOG.md) | 외부 검토와 채택한 변경. |
| [회고](05-ops/RETRO.md) | 개발 과정에서 기록한 교훈. |
| [v0.6.0 릴리스 노트](05-ops/releases/v0.6.0.md) | 네이티브 Windows 지원과 할당량/초기화 수정. |
| [v0.6.1 릴리스 노트](05-ops/releases/v0.6.1.md) | 문서 재작성과 Ubuntu 검증. |
| [v0.6.2 릴리스 노트](05-ops/releases/v0.6.2.md) | 로컬 로그인 정보에서 Codex 이메일 자동 표시. |
| [v0.7.0 릴리스 노트](05-ops/releases/v0.7.0.md) | 작은 시스템 사용량 기록, 조건부 GPU 측정, About 정보. |

<!-- qhud:anchor -->
<a id="languages-and-source-fidelity"></a>

## 언어 및 원문 충실성

저장소의 기본 화면은 한국어 `README.md`입니다. 영어는 `README.en.md`에서 제공하며, 기존 링크를 위해 `README.ko.md`도 동일한 한국어 사본으로 유지합니다. 나머지 영어 페이지는 원래 경로를 유지합니다. 한국어와 중국어 미러도 언어 디렉터리 안에서 같은 경로를 보존합니다. 모든 페이지는 다른 언어의 대응 페이지로 연결되며, 번역된 제목에는 원문 문서의 링크를 위한 앵커가 유지됩니다.

번역은 개요만이 아닌 문서 전체를 포함합니다. 명령 블록, API/스키마 식별자, 요구사항/결정 ID, 날짜, 검증 수치를 보존합니다. [MIT 라이선스](../../../../LICENSE)는 영어 원문을 유지합니다.

정정 사항은 [이슈](https://github.com/chquandogong/qhud/issues/new/choose)를 작성하거나 [기여 가이드](../CONTRIBUTING.md)를 따르세요. 세 언어 모두 환영합니다.
