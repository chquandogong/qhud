<!-- qhud:languages -->
<p align="center">
  <a href="../../../../01-discovery/FEASIBILITY_REPORT.md">English</a> · <a href="../../../ko/docs/01-discovery/FEASIBILITY_REPORT.md">한국어</a> · <strong>简体中文</strong>
</p>
<!-- /qhud:languages -->

<!-- qhud:anchor -->
<a id="feasibility_report"></a>

# 可行性报告

> 状态：完成 · 日期：2026-08-05 · 负责人：chquandogong

<!-- qhud:anchor -->
<a id="environment-under-test"></a>

## 测试环境

| 项目 | 值 |
| --- | --- |
| 操作系统 | Ubuntu 24.04.3 LTS |
| 桌面 | GNOME 46，**Wayland** 会话 |
| 显示器 | HDMI-1 3840×2160 @ (0,0) · eDP-1 2560×1600 @ (3840,560)，均为 scale=1 |
| Rust | 1.94.1（qmonster 要求 1.88+） |

<!-- qhud:anchor -->
<a id="the-core-constraint"></a>

## 核心约束

GNOME Mutter **没有**实现 `wlr-layer-shell`，其他主要合成器均用该 Wayland 协议承载桌面小组件。原生 Wayland 客户端也不能在全局坐标中自行定位窗口。**XWayland** 客户端可通过 EWMH 提示实现这两项功能，Mutter 会遵循这些提示。

<!-- qhud:anchor -->
<a id="spike-1--gtk-keep-below-2026-08-05-pre-code"></a>

## 技术探针 1 — GTK keep-below（2026-08-05，编码前）

在 `GDK_BACKEND=x11` 下运行 GJS/GTK3 窗口，使用 `set_keep_below` + `stick` + skip 提示，与 Tauri 的 `tao` 在 Linux 上的调用相同：

```text
_NET_WM_STATE(ATOM) = _NET_WM_STATE_SKIP_PAGER, _NET_WM_STATE_SKIP_TASKBAR,
                      _NET_WM_STATE_BELOW, _NET_WM_STATE_STICKY, _NET_WM_STATE_FOCUSED
_NET_WM_DESKTOP(CARDINAL) = 4294967295   # all workspaces
move  (300,300) → (4200,800)             # cross-monitor: OK
resize 320×180 → 480×300                 # programmatic resize: OK
```

<!-- qhud:anchor -->
<a id="spike-2--the-real-qhud-binary-2026-08-05-post-build"></a>

## 技术探针 2 — 实际 qhud 二进制（2026-08-05，构建后）

`target/release/qhud`（Tauri v2，`alwaysOnBottom` + 运行时断言）：

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

## 评分

| 维度 | 得分（1–5） | 说明 |
| --- | --- | --- |
| 需求价值 | 4 | 解决作者日常无法一眼掌握状态的问题 |
| 技术可行性 | 5 | 两个探针均在目标机器上通过 |
| 实施可行性 | 5 | 复用 qmonster 数据层；最小切入点约需 1 周 |
| 风险 | 4 | 剩余风险：WebKitGTK 渲染问题、在 GNOME 概览中出现（已接受） |

<!-- qhud:anchor -->
<a id="decision-gate-1"></a>

## 决策关卡 1

**继续构建。** 备选方案及完整理由见 `../02-decisions/ALTERNATIVES.md` 和 `DECISION_LOG.md`。
