<!-- qhud:languages -->
<p align="center">
  <strong>한국어</strong> · <a href="../../../../01-discovery/OFFICE_HOURS.md">English</a> · <a href="../../../zh-CN/docs/01-discovery/OFFICE_HOURS.md">简体中文</a>
</p>
<!-- /qhud:languages -->

<!-- qhud:anchor -->
<a id="office_hours--pressure-review-result"></a>

# 검토 회의 — 압박 요인 검토 결과

> 상태: 완료 · 날짜: 2026-08-05 · 담당: chquandogong

<!-- qhud:anchor -->
<a id="1-problem-sharpened"></a>

## 1. 문제 구체화

- **불편**: 운영자가 다른 창의 실제 작업에 집중하는 동안 사용 한도와 컨텍스트 압박을 볼 수 없습니다. qmonster TUI는 모든 질문에 답하지만 tmux 창 안에서 그것을 _직접 볼 때만_ 그렇습니다.
- **최근 사례**: CLI가 작업을 거부한 뒤에야 5h 기간이 88%라는 것을 발견했습니다. 초기화 카운트다운은 편집기 뒤의 창에 있었습니다.
- **빈도**: 다중 에이전트 세션 중 지속적으로 발생합니다.
- **현재 우회 방법**: 모니터 한쪽 구석을 TUI 실행 터미널에 할당합니다. 터미널 하나가 필요하고 alt-tab 포커스를 차지하며 최대화한 창 아래로 사라집니다.

<!-- qhud:anchor -->
<a id="2-narrowest-customer"></a>

## 2. 가장 좁은 고객 범위

모니터 2대의 GNOME Wayland 워크스테이션을 쓰는 qmonster 작성자입니다. 다음은 기존 qmonster 운영자(Linux, tmux, 여러 CLI)입니다.

<!-- qhud:anchor -->
<a id="3-why-now"></a>

## 3. 지금 해야 하는 이유

- qmonster v3.2.0은 이미 전체 데이터 파이프라인을 Rust 라이브러리로 제공합니다(`lib.rs`가 adapters/domain/policy/tmux를 공개). 위젯은 표현 계층_만_ 필요합니다.
- 목업(“Qmonster · AI CLI 모니터”)은 순수 HTML/CSS이므로 webview 껍데기로 거의 변환 비용 없이 재현할 수 있습니다.
- 2026-08-05 GNOME Wayland가 XWayland의 keep-below + sticky 조합을 수용하는 것을 검증했습니다(실기 실험, FEASIBILITY_REPORT 참고).

<!-- qhud:anchor -->
<a id="4-smallest-useful-wedge"></a>

## 4. 가장 작은 유용한 기능

기존 qmonster 파이프라인의 창 타일 + CTX/5H/7D 계기를 표시하고 이동/크기 조절/저장을 지원하는 읽기 전용 위젯입니다. tmux가 없으면 데모로 대체합니다. 알림, 제어 동작, 설정 UI는 없습니다.

<!-- qhud:anchor -->
<a id="5-framing-shake--exactly-the-same-uiux"></a>

## 5. 방향 재검토 — “정확히 같은 UI/UX”

목업은 960px 문서형 페이지이며 목업 전용 설명 요소(모드 설명, 범례)를 포함합니다. 그대로 복사하면 좋은 위젯이 되지 못합니다. **재해석을 채택했습니다**. 색상, 타일, 계기, 상태 배지, 심각도 구간 같은 시각 언어는 100% 유지하고 위젯 크기(약 360–420px)에 맞게 다시 배치하며 ↑/↓ 선택을 클릭 확장으로 바꾸고 설명용 장식은 제거합니다. 위젯이 키보드 포커스를 가져가면 안 되므로 포인터 조작만 제공합니다.

<!-- qhud:anchor -->
<a id="6-10-star-sketch-later"></a>

## 6. 이상적인 모습 (향후)

타일 클릭 → 해당 tmux 창으로 이동. 중요한 사용 한도는 시야 가장자리에서 맥동합니다. 배경화면에 맞는 테마, 모니터별 여러 위젯 배치, 크로스 플랫폼(Windows WorkerW / macOS 데스크톱 계층)을 지원합니다.

<!-- qhud:anchor -->
<a id="verdict"></a>

## 결론

진행합니다. 최소 유용 기능에는 1주가 필요하며 가장 위험한 미지수인 GNOME Wayland의 데스크톱 계층은 코드를 작성하기 전에 실기 실험으로 검증했습니다.
