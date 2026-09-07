<!-- qhud:languages -->
<p align="center">
  <a href="../../../../01-discovery/RESEARCH_NOTES.md">English</a> · <a href="../../../ko/docs/01-discovery/RESEARCH_NOTES.md">한국어</a> · <strong>简体中文</strong>
</p>
<!-- /qhud:languages -->

<!-- qhud:anchor -->
<a id="research_notes"></a>

# 调研笔记

> 状态：完成 · 日期：2026-08-05 · 负责人：chquandogong

<!-- qhud:anchor -->
<a id="window-layering-on-linux-desktops"></a>

## Linux 桌面的窗口分层

- GNOME Mutter 对 layer-shell 协议的请求仍**开放且未实现**，第三方应用无法在 GNOME 使用原生 Wayland 小组件层：<https://gitlab.gnome.org/GNOME/mutter/-/work_items/973>
- `wlr-layer-shell` 协议（wlroots、KWin、Smithay 均已实现，只有 GNOME 没有）：<https://wayland.app/protocols/wlr-layer-shell-unstable-v1>
- 因此，在 GNOME 中只能通过 XWayland EWMH 状态（`_NET_WM_STATE_BELOW` + `_NET_WM_STATE_STICKY`）访问桌面小组件层；Mutter 会遵循这些状态，已实机验证，见 FEASIBILITY_REPORT。

<!-- qhud:anchor -->
<a id="tauri-capabilities"></a>

## Tauri 能力

- `always_on_bottom` 窗口选项和 `set_always_on_bottom`：<https://github.com/tauri-apps/tauri/commit/c1ec0f155118527361dd5645d920becbc8afd569>
- 底层 tao 实现（Linux 上的 GTK `set_keep_below`）：<https://github.com/tauri-apps/tao/pull/522>
- 窗口位置和层级管理**在 Wayland 后端会静默失效**，这也是 qhud 强制 `GDK_BACKEND=x11` 的原因：<https://github.com/tauri-apps/tauri/issues/14913>

<!-- qhud:anchor -->
<a id="qmonster-reuse-surface-rev-aa2bd39"></a>

## qmonster 可复用接口（rev aa2bd39）

- `qmonster::app::bootstrap::Context::new(config, source, notifier, sink)`：公开构造函数，以 `PaneSource` + `NotifyBackend` 为泛型。
- `qmonster::store::sink::NoopSink`：上游已有 qhud 所需的不写入审计接收器。
- `qmonster::app::event_loop::run_once_with_target`：执行一次观测，返回 `Vec<PaneReport>`；qhud 将其映射为小组件 JSON（schema v1）。
- 缺少 `.git` 时，`build.rs` 回退到 `v{CARGO_PKG_VERSION}-nogit`，因此可安全用作 cargo git 依赖。
- 仪表语义：压力是 0..1 的比例（`context_pressure`、`quota_5h_pressure`、`quota_weekly_pressure`）；重置时间是 Unix 秒（`quota_*_resets_at`）。
