<!-- qhud:languages -->
<p align="center">
  <strong>한국어</strong> · <a href="../../../../01-discovery/FEASIBILITY_REPORT.md">English</a> · <a href="../../../zh-CN/docs/01-discovery/FEASIBILITY_REPORT.md">简体中文</a>
</p>
<!-- /qhud:languages -->

<!-- qhud:anchor -->
<a id="feasibility_report"></a>

# 실현 가능성 보고서

> 상태: 완료 · 날짜: 2026-08-05 · 담당: chquandogong

<!-- qhud:anchor -->
<a id="environment-under-test"></a>

## 검증 환경

| 항목 | 값 |
| --- | --- |
| OS | Ubuntu 24.04.3 LTS |
| 데스크톱 | GNOME 46, **Wayland** 세션 |
| 모니터 | HDMI-1 3840×2160 @ (0,0) · eDP-1 2560×1600 @ (3840,560), 모두 scale=1 |
| Rust | 1.94.1 (qmonster 요구 사항은 1.88+) |

<!-- qhud:anchor -->
<a id="the-core-constraint"></a>

## 핵심 제약

GNOME Mutter는 다른 주요 컴포지터가 데스크톱 위젯에 사용하는 Wayland 프로토콜인 `wlr-layer-shell`을 구현하지 **않습니다**. 네이티브 Wayland 클라이언트는 창의 전역 위치도 직접 지정할 수 없습니다. **XWayland** 클라이언트에는 두 기능이 모두 EWMH 힌트로 제공되며 Mutter가 이를 따릅니다.

<!-- qhud:anchor -->
<a id="spike-1--gtk-keep-below-2026-08-05-pre-code"></a>

## 실험 1 — GTK keep-below (2026-08-05, 코드 작성 전)

`GDK_BACKEND=x11`에서 실행한 GJS/GTK3 창입니다. `set_keep_below` + `stick` + skip 힌트를 사용했으며, 이는 Linux에서 Tauri의 `tao`가 사용하는 호출과 같습니다.

```text
_NET_WM_STATE(ATOM) = _NET_WM_STATE_SKIP_PAGER, _NET_WM_STATE_SKIP_TASKBAR,
                      _NET_WM_STATE_BELOW, _NET_WM_STATE_STICKY, _NET_WM_STATE_FOCUSED
_NET_WM_DESKTOP(CARDINAL) = 4294967295   # all workspaces
move  (300,300) → (4200,800)             # cross-monitor: OK
resize 320×180 → 480×300                 # programmatic resize: OK
```

<!-- qhud:anchor -->
<a id="spike-2--the-real-qhud-binary-2026-08-05-post-build"></a>

## 실험 2 — 실제 qhud 바이너리 (2026-08-05, 빌드 후)

`target/release/qhud`(Tauri v2, `alwaysOnBottom` + 런타임 재설정):

```text
_NET_WM_STATE(ATOM) = _NET_WM_STATE_SKIP_PAGER, _NET_WM_STATE_SKIP_TASKBAR,
                      _NET_WM_STATE_BELOW, _NET_WM_STATE_STICKY, _NET_WM_STATE_FOCUSED
_NET_WM_DESKTOP(CARDINAL) = 4294967295
WM_CLASS(STRING) = "qhud", "Qhud"
move (50,50) → (4300,700) → (3400,60)    # both monitors: OK
binary size: 15 MB (release, LTO, stripped)
```

<!-- qhud:anchor -->
<a id="scorecard"></a>

## 평가표

| 기준 | 점수 (1–5) | 설명 |
| --- | --- | --- |
| 필요성 | 4 | 작성자가 매일 겪는 한눈에 확인하기 어려운 문제를 해결합니다. |
| 실현 가능성 | 5 | 정확한 대상 장비에서 두 실험 모두 통과했습니다. |
| 실행 타당성 | 5 | qmonster의 데이터 계층 재사용; 최소 유용 기능까지 약 1주입니다. |
| 위험 | 4 | 남은 위험: WebKitGTK 렌더링 특성, GNOME 개요 노출(수용). |

<!-- qhud:anchor -->
<a id="decision-gate-1"></a>

## 결정 단계 1

**구현을 진행합니다.** 대안과 전체 근거는 `../02-decisions/ALTERNATIVES.md` 및 `DECISION_LOG.md`에 있습니다.
