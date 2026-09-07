<!-- qhud:languages -->
<p align="center">
  <a href="../../../../02-decisions/CROSS_VALIDATION_LOG.md">English</a> · <strong>한국어</strong> · <a href="../../../zh-CN/docs/02-decisions/CROSS_VALIDATION_LOG.md">简体中文</a>
</p>
<!-- /qhud:languages -->

# CROSS_VALIDATION_LOG

> 상태: 완료 · 날짜: 2026-08-05 · 담당: chquandogong

<!-- qhud:anchor -->
<a id="session-1--architecture-decision-pre-ship"></a>

## 세션 1 — 아키텍처 결정(출시 전)

- **질문**: GNOME 46 Wayland의 qhud v0.1에 “Tauri v2 + 강제 XWayland keep-below + 쓰지 않는 sink를 사용한 qmonster 라이브러리 직접 재사용”이 올바른 아키텍처인가?
- **제안자**: Claude(Fable 5). 대상 컴퓨터에서 수행한 두 번의 실험을 근거로 제시했다.
- **독립 검토자**: OpenAI Codex CLI 0.146(GPT), 작업 `019fd114-4dbb-7522-ab71-e8a130634a49`. 고정한 qmonster 리비전 `aa2bd39`를 소스 수준에서 검토했다. (검토자의 로컬 명령 실행은 샌드박스에 차단되었다. 같은 리비전을 GitHub 읽기 전용 접근으로 확인했으며, 파일:줄 인용을 포함했다.)

<!-- qhud:anchor -->
<a id="verdict-agree-with-changes"></a>

### 판정: **AGREE-WITH-CHANGES**

<!-- qhud:anchor -->
<a id="where-the-reviewer-independently-confirmed-the-design"></a>

### 검토자가 설계를 독립적으로 확인한 사항

- `Context::new(config, source, notifier, sink)`가 올바른 공개 생성 경로다. 비공개 필드 때문에 구조체 리터럴은 사용할 수 없다. 구현과 일치한다.
- 업스트림 `NoopSink`와 메서드 하나만 구현한 사용자 정의 `NotifyBackend`면 충분하다. `Context::new`는 모든 영속화 sink를 `None`으로 두므로, 루프는 `~/.qmonster`에 대해 읽기 전용이다(`event_loop.rs:362–654`로 검증). 구현과 일치한다.
- **`build_startup_runtime` / `with_anomaly_sink`를 절대 호출하지 않는다.** 이들은 qmonster 디렉터리 구조를 만들고 sqlite를 열며 보존 처리를 실행한다. qhud는 호출하지 않는다.
- `PaneReport`는 `Serialize`가 아니며 `pub(crate)` 필드도 있다. 따라서 qhud 자체 DTO는 선택이 아닌 필수다. `view.rs` 스키마 v1과 일치한다.
- 데모 대체 모드는 명시적이어야 하고 표시가 있어야 하며, 조용히 전환해서는 안 된다. `DEMO` 배지와 `source` 필드가 이를 충족한다.
- 업스트림 내보내기 계약이 마련될 때까지 정확한 `rev=` 고정을 유지한다. 구현과 일치한다.

<!-- qhud:anchor -->
<a id="changes-adopted-from-the-review"></a>

### 검토에서 채택한 변경

| ID | 변경 | 위치 |
| ---- | ---- | ---- |
| CV-1 | 트레이에 **Reset position** 추가(사라진 모니터 위치로 복원될 때 복구) | `main.rs`; RISK R10 |
| CV-2 | 프런트엔드 **오래된 상태 감시기**: >8 s 동안 페이로드가 없으면 그럴듯한 숫자를 멈춘 채 두지 않고, 푸터를 경고 상태와 “stalled”로 표시 | `app.js`, `style.css` |
| CV-3 | 인수 행렬 확장: GNOME 개요, 잠금/해제, 절전/복귀, 전체 화면, 모니터 핫플러그(첫 검증 대기) | TEST_PLAN |
| CV-4 | 위험 등록부: 투명 영역 클릭 가로채기(R9), 화면 구성에 따른 위치 복원 위험(R10) | RISK_REGISTER |

<!-- qhud:anchor -->
<a id="reviewer-suggestions-deferred-with-rationale"></a>

### 검토자가 제안했으나 보류한 사항과 이유

| 제안 | 처리 |
| ---- | ---- |
| `.deb`를 기본 배포물로 제공 | **로드맵으로 보류.** v0.1은 qmonster 계열의 방식(바이너리 tarball + 런타임 의존성 문서)을 따른다. `.deb`에는 tauri-cli 번들링이 필요하므로 첫 외부 사용자 요구 뒤에 배치했다. |
| qmonster 업스트림에 버전이 있는 `ObserveSnapshot` 파사드 제공 | **이미 계획되어 있다**(DECISION_LOG D-004 승격 경로). 업스트림 릴리스가 필요하며 qhud v0.1 범위를 벗어난다. |
| 불변 조건이 깨지면 GNOME Shell 확장으로 대체 | ALTERNATIVES C / 로드맵에 기록했다. 오늘의 실기 증거로는 XWayland 불변 조건이 성립한다. |
| 전용 폴링 작업자, 겹치지 않는 틱 | **이미 충족한다**: 단일 스레드가 tick → sleep을 엄격히 순차 실행한다. 느린 폴링은 주기를 늘릴 뿐 중첩되지 않는다. |
| 별도 `_NET_WM_STATE_SKIP_PAGER` 힌트 | **이미 충족한다**: 실기 `xprop`에서 SKIP_PAGER 설정을 확인했다(Tauri의 skipTaskbar가 GTK 힌트 둘 모두에 대응). |
| Pricing/ClaudeSettings를 읽지 않으므로 완전한 TUI 동등성은 아님 | 허용한 v0.1 제약이다. 게이지(ctx/5h/7d)는 어댑터에서 오므로 영향이 없다. Codex `cost`는 없을 수 있다. SPEC의 범위 밖 항목에 기록했다. |

<!-- qhud:anchor -->
<a id="method-note-honesty"></a>

### 방법에 관한 참고(정직성)

이는 모의 검증이 아니라 실제 두 모델 간 검증이었다. Claude가 제안하고 GPT가 독립 접근을 먼저 수행하는 프로토콜로 반대 검토를 했다. 최종 판단 기준은 모델의 정체가 아니라 증거(실기 `xprop` 상태, 고정 리비전의 소스 인용)였다.
