<!-- qhud:languages -->
<p align="center">
  <a href="../../../ko/docs/00-overview/PROJECT_BRIEF.md">한국어</a> · <a href="../../../../00-overview/PROJECT_BRIEF.md">English</a> · <strong>简体中文</strong>
</p>
<!-- /qhud:languages -->

<!-- qhud:anchor -->
<a id="project_brief--qhud"></a>

# 项目简介 — qhud

> 状态：v0.6.0 发布准备 · 日期：2026-09-07 · 负责人：chquandogong

<!-- qhud:anchor -->
<a id="problem"></a>

## 问题

在 tmux 中同时运行多个智能体（Claude Code、Codex、Gemini、Antigravity）会带来运行压力：上下文逐渐填满、5h/7d 配额不断消耗、窗格静默等待批准。操作者只有切换到 [qmonster](https://github.com/chquandogong/qmonster) TUI 窗格，才能看到这些信息。专注工作时，仪表却不在视野里。“任务中途 5h 用量达到 100%”之类的意外，恰恰发生在你没有查看监视器的时候。

<!-- qhud:anchor -->
<a id="solution"></a>

## 方案

qhud 是一个**常驻桌面的信息面板（HUD）**：它位于所有窗口下方、壁纸上方，并固定显示在各个工作区，在显示器的空闲角落呈现 qmonster 窗格卡片和 CTX / 5H / 7D 仪表。像看挂钟一样一眼掌握情况，无须切换焦点或窗格。

qhud 链接 qmonster crate，复用其 tmux/herdr 观测、供应商解析与策略流水线。即使没有正在运行的终端，账户配额仍有价值：本地身份和带时间的已保存读数保持可见，显式刷新则请求当前用量。

<!-- qhud:anchor -->
<a id="first-user"></a>

## 首位用户

qmonster 作者自己的多显示器 Ubuntu GNOME 工作站。
其次是 Linux/X11 或 GNOME Wayland 上的其他 qmonster 用户。
v0.6.0 新增原生 Windows x64 WebView2 小组件，用于查看账户用量和重置时间。不包含原生 Windows 终端标签页监控。

<!-- qhud:anchor -->
<a id="success-criteria-v060"></a>

## 成功标准（v0.6.0）

1. 在 GNOME Wayland 上通过 XWayland 保持桌面层位置（keep-below + sticky），并以 `_NET_WM_STATE` 验证。
2. 可跨显示器移动、调整大小，并保存位置。
3. 与设计稿视觉一致，演示数据同时作为视觉一致性测试夹具。
4. 存在 tmux/herdr 时显示实时数据；否则显示真实本地账户行和带时间的配额。只有 `--demo` 才显示示例窗格。
5. 仅观测：对 `~/.qmonster` **零写入**，该目录由 TUI 管理。
6. Windows 原生账户用量显示，供应商返回时显示模型独立配额池，重置时间清晰可读，并在重启后保留。
7. 保留 Ubuntu 源码构建路径和 Linux tarball 发布。只有两种平台均通过发布门槛后，才发布 Windows ZIP。

<!-- qhud:anchor -->
<a id="non-goals-v060"></a>

## 非目标（v0.6.0）

- 不执行控制操作，不发送通知；告警由 TUI 和供应商负责。
- 不提供原生 Windows 终端标签页观测或 macOS 支持。
- 不提供 npm/deb/AppImage/MSI 打包；提供 Linux tarball 和 Windows ZIP。
- 不为供应商未返回的模型配额池虚构配额。

<!-- qhud:anchor -->
<a id="current-verification"></a>

## 当前验证

2026-09-07，远程 Ubuntu 和 Windows CI 作业均通过 98/98 项 QHUD 测试及 release 构建；Ubuntu fmt/clippy 也已通过。本地 Windows 构建通过原生显示和账户刷新冒烟检查。证据及剩余 Ubuntu 桌面检查见 [CI 34117359064](https://github.com/chquandogong/qhud/actions/runs/34117359064) 和 TEST_PLAN。

<!-- qhud:anchor -->
<a id="documents"></a>

## 文档

Quetzalcoatl 风格的决策文档位于 `docs/`：调研（`01-discovery/`）、决策（`02-decisions/`）、规格（`03-spec/`）、质量（`04-quality/`）、运维（`05-ops/`）。
