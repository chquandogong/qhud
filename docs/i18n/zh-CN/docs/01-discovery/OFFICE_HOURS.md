<!-- qhud:languages -->
<p align="center">
  <a href="../../../../01-discovery/OFFICE_HOURS.md">English</a> · <a href="../../../ko/docs/01-discovery/OFFICE_HOURS.md">한국어</a> · <strong>简体中文</strong>
</p>
<!-- /qhud:languages -->

<!-- qhud:anchor -->
<a id="office_hours--pressure-review-result"></a>

# OFFICE_HOURS — 压力评审结果

> 状态：完成 · 日期：2026-08-05 · 负责人：chquandogong

<!-- qhud:anchor -->
<a id="1-problem-sharpened"></a>

## 1. 聚焦问题

- **痛点**：操作者在其他窗口专注实际工作时，看不到配额和上下文压力。qmonster TUI 能回答所有问题，但必须去看位于 tmux 窗格中的它。
- **近期实例**：直到 CLI 拒绝继续工作，才发现 5h 窗口已用 88%；重置倒计时在编辑器后面的窗格里。
- **频率**：任何多智能体会话期间都会持续发生。
- **现有替代方法**：在一台显示器的角落固定一个运行 TUI 的终端，但这会占用终端、增加 alt-tab 焦点切换，并被最大化窗口遮住。

<!-- qhud:anchor -->
<a id="2-narrowest-customer"></a>

## 2. 最窄客户群

在双显示器 GNOME Wayland 工作站上的 qmonster 作者；随后是 qmonster 现有用户（Linux、tmux、多个 CLI）。

<!-- qhud:anchor -->
<a id="3-why-now"></a>

## 3. 为什么是现在

- qmonster v3.2.0 已将完整数据流水线作为 Rust 库发布（`lib.rs` 导出 adapters/domain/policy/tmux），小组件**只是**展示层。
- 设计稿（“Qmonster · AI CLI 모니터”）是纯 HTML/CSS；webview 外壳几乎无需转换就能复现。
- 2026-08-05 已验证 GNOME Wayland 接受 XWayland keep-below + sticky 组合，实机探针见 FEASIBILITY_REPORT。

<!-- qhud:anchor -->
<a id="4-smallest-useful-wedge"></a>

## 4. 最小有用切入点

一个只读小组件，从现有 qmonster 流水线显示窗格卡片和 CTX/5H/7D 仪表，可移动、可调整大小、可保存状态，tmux 不存在时回退到演示。不包含告警、操作和设置界面。

<!-- qhud:anchor -->
<a id="5-framing-shake--exactly-the-same-uiux"></a>

## 5. 重新审视“完全相同的 UI/UX”

设计稿是宽 960px 的文档式页面，包含仅用于设计稿的解释元素（模式说明、图例）。照搬会得到一个不好用的小组件。**采用重新诠释**：100% 保留视觉语言（配色、卡片、仪表、状态标签、严重程度分级），改为约 360–420px 的小组件尺度，将 ↑/↓ 选择改为点击展开，移除解释性装饰。小组件不得抢占键盘焦点，只使用指针交互。

<!-- qhud:anchor -->
<a id="6-10-star-sketch-later"></a>

## 6. 十星体验草图（以后）

点击卡片直接跳转到对应 tmux 窗格。临界配额在视野边缘轻微闪动。随壁纸调整主题。多小组件布局，每台显示器各自配置。跨平台（Windows WorkerW / macOS 桌面层）。

<!-- qhud:anchor -->
<a id="verdict"></a>

## 结论

继续推进。该切入点需一周工作；在开始编码前，最大的未知项（GNOME Wayland 的桌面层）已通过实机探针验证，降低了风险。
