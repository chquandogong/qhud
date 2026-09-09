<!-- qhud:languages -->
<p align="center">
  <strong>한국어</strong> · <a href="../../../../02-decisions/ALTERNATIVES.md">English</a> · <a href="../../../zh-CN/docs/02-decisions/ALTERNATIVES.md">简体中文</a>
</p>
<!-- /qhud:languages -->

<!-- qhud:anchor -->
<a id="alternatives--desktop-widget-implementation"></a>

# ALTERNATIVES — 데스크톱 위젯 구현 대안

> 상태: 완료 · 날짜: 2026-08-05 · 담당: chquandogong
> 선택: **A (Tauri v2)** — DECISION_LOG D-002 참조.

| # | 선택지 | 목업과의 UI 일치도 | qmonster 통합 | 상시 실행 비용 | GNOME Wayland의 데스크톱 계층 | 실패 조건 |
| ----- | ----- | ----- | ----- | ----- | ----- | ----- |
| **A** | **Tauri v2** (Rust + WebKitGTK) | ★★★ HTML을 그대로 이식 | ★★★ 직접 라이브러리 링크(같은 언어) | 바이너리 ~15 MB, 적당한 RSS | XWayland에서 GTK keep-below를 통한 `alwaysOnBottom` — **실기 검증 완료** | WebKitGTK 투명도 결함 |
| B | Electron | ★★★ HTML을 그대로 이식 | ★☆ 하위 프로세스/IPC만 가능 | RSS ~250 MB — 상시 위젯에는 무거움 | `type:'desktop'` / below 힌트, 성숙한 생태계 | 리소스 비용을 감당할 수 없음 |
| C | GNOME Shell 확장(GJS/Clutter) | ★☆ 웹뷰 없음 — UI 전체 재작성 | ★★ 파일/JSON 전달 | 최소 | **유일한 진정한 네이티브 계층**(개요 화면에서도 깔끔함) | 릴리스마다 변하는 GNOME API, GNOME 전용 |
| D | 네이티브 Rust GUI(GTK4-rs / egui / iced) | ★☆ CSS 재구현 | ★★★ 직접 라이브러리 링크 | 최소 | GTK4-rs keep-below는 가능, winit 기반(egui/iced)은 keep-below 없음 | UI 재작성 비용 급증 |
| E | 기존 TUI를 투명 터미널에 고정 | ☆ 목업이 아닌 TUI | 이미 구현됨 | 최소 | wmctrl/xdotool을 통한 같은 EWMH 방식 | 제품이 아닌 검증용 임시방편 |

참고

- E는 코딩 전 실험에서 사실상 수행되었으며(같은 EWMH 메커니즘), 이후 제외했다.
- 허용한 GNOME의 두 가지 특성(개요 화면 노출, XWayland 의존성)이 충분히 중요해진다면 C가 장기적인 해결책이다.
- WebKitGTK 렌더링 결함을 고칠 수 없는 것으로 판명되면 B가 대안으로 남는다.
