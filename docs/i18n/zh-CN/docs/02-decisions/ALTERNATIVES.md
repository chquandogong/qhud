<!-- qhud:languages -->
<p align="center">
  <a href="../../../../02-decisions/ALTERNATIVES.md">English</a> · <a href="../../../ko/docs/02-decisions/ALTERNATIVES.md">한국어</a> · <strong>简体中文</strong>
</p>
<!-- /qhud:languages -->

<!-- qhud:anchor -->
<a id="alternatives--desktop-widget-implementation"></a>

# 备选方案 — 桌面小组件实现

> 状态：完成 · 日期：2026-08-05 · 负责人：chquandogong
> 选择：**A（Tauri v2）**，见 DECISION_LOG D-002。

| # | 方案 | 与设计稿的 UI 一致性 | qmonster 集成 | 常驻成本 | GNOME Wayland 上的桌面层 | 失效条件 |
| --- | --- | --- | --- | --- | --- | --- |
| **A** | **Tauri v2**（Rust + WebKitGTK） | ★★★ 原样迁移 HTML | ★★★ 直接链接库（同语言） | 约 15 MB 二进制，中等 RSS | `alwaysOnBottom` 通过 XWayland 下的 GTK keep-below 实现，**已实机验证** | WebKitGTK 透明渲染缺陷 |
| B | Electron | ★★★ 原样迁移 HTML | ★☆ 只能使用子进程/IPC | 约 250 MB RSS，对常驻小组件而言过重 | `type:'desktop'` / below 提示，生态成熟 | 资源开销不可接受 |
| C | GNOME Shell 扩展（GJS/Clutter） | ★☆ 无 webview，需全面重写 UI | ★★ 文件/JSON 交接 | 很低 | **唯一真正的原生层**，在概览中也干净 | 每次 GNOME 发布的 API 变动；仅 GNOME |
| D | 原生 Rust GUI（GTK4-rs / egui / iced） | ★☆ 重新实现 CSS | ★★★ 直接链接库 | 很低 | GTK4-rs keep-below 可用；基于 winit 的 egui/iced 没有 keep-below | UI 重写成本失控 |
| E | 将现有 TUI 固定在透明终端 | ☆ 是 TUI，不是设计稿 | 已完成 | 很低 | 通过 wmctrl/xdotool 使用相同 EWMH 技巧 | 属于验证用临时方案，不能作为产品 |

说明

- E 实际上已作为编码前探针执行，使用相同的 EWMH 机制，随后弃用。
- 若两个已接受的 GNOME 问题（出现在概览中、依赖 XWayland）变得足够重要，C 仍是长期解决方案。
- 若 WebKitGTK 渲染缺陷无法修复，B 仍作为备选方案。
