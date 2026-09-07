<!-- qhud:languages -->
<p align="center">
  <a href="../../../../02-decisions/CROSS_VALIDATION_LOG.md">English</a> · <a href="../../../ko/docs/02-decisions/CROSS_VALIDATION_LOG.md">한국어</a> · <strong>简体中文</strong>
</p>
<!-- /qhud:languages -->

<!-- qhud:anchor -->
<a id="cross_validation_log"></a>

# 交叉验证日志

> 状态：完成 · 日期：2026-08-05 · 负责人：chquandogong

<!-- qhud:anchor -->
<a id="session-1--architecture-decision-pre-ship"></a>

## 会话 1 — 架构决策（发布前）

- **问题**：“Tauri v2 + 强制 XWayland keep-below + 直接复用 qmonster 库并使用不写入的接收器”是否是 GNOME 46 Wayland 上 qhud v0.1 的正确架构？
- **提出者**：Claude（Fable 5），以目标机器上的两个实机探针为证据。
- **独立评审者**：OpenAI Codex CLI 0.146（GPT），任务 `019fd114-4dbb-7522-ab71-e8a130634a49`，对固定的 qmonster rev `aa2bd39` 进行源码级评审。（评审者的本地命令执行被沙箱阻止，因此通过只读 GitHub 访问同一 revision 验证，提供了文件和行号引用。）

<!-- qhud:anchor -->
<a id="verdict-agree-with-changes"></a>

### 结论：**AGREE-WITH-CHANGES（同意，但需修改）**

<!-- qhud:anchor -->
<a id="where-the-reviewer-independently-confirmed-the-design"></a>

### 评审者独立确认的设计

- `Context::new(config, source, notifier, sink)` 是正确的公开构造入口；字段私有，无法使用结构体字面量。与实现一致。
- 上游 `NoopSink` 加上只有一个方法的自定义 `NotifyBackend` 已足够；`Context::new` 将所有持久化接收器保留为 `None`，因此循环对 `~/.qmonster` 只读。已对照 `event_loop.rs:362–654` 验证，与实现一致。
- **不得调用 `build_startup_runtime` / `with_anomaly_sink`**：它们会创建 qmonster 目录布局、打开 sqlite、执行保留策略。qhud 没有调用它们。
- `PaneReport` 未实现 `Serialize`，且含有 `pub(crate)` 字段，因此必须使用 qhud 自有的 DTO。与 `view.rs` schema v1 一致。
- 演示回退必须显式并带水印，不能静默发生。与 `DEMO` 标签和 `source` 字段一致。
- 上游提供导出契约前，须使用精确 `rev=` 固定版本。与实现一致。

<!-- qhud:anchor -->
<a id="changes-adopted-from-the-review"></a>

### 采纳的评审修改

| ID | 修改 | 位置 |
| --- | --- | --- |
| CV-1 | 托盘新增 **Reset position**，用于位置恢复到已断开的显示器时恢复 | `main.rs`；RISK R10 |
| CV-2 | 前端新增**过期看门狗**：超过 8 s 未收到数据时，页脚变为警告并显示“stalled”，避免看似合理的数字冻结不动 | `app.js`、`style.css` |
| CV-3 | 扩展验收矩阵：GNOME 概览、锁定/解锁、挂起/恢复、全屏、显示器热插拔，等待首次检查 | TEST_PLAN |
| CV-4 | 风险登记新增：透明区域拦截点击（R9）、位置恢复的显示器拓扑风险（R10） | RISK_REGISTER |

<!-- qhud:anchor -->
<a id="reviewer-suggestions-deferred-with-rationale"></a>

### 延后的评审建议及理由

| 建议 | 处理 |
| --- | --- |
| 以 `.deb` 为主要发布产物 | **延后到路线图。** v0.1 遵循 qmonster 家族惯例：二进制 tarball 加运行时依赖说明。`.deb` 需要 tauri-cli 打包，排在首次外部用户需求之后。 |
| 在 qmonster 上游提供带版本的 `ObserveSnapshot` 外观接口 | **已在计划中**，见 DECISION_LOG D-004 升级路径。需要上游发布，不在 qhud v0.1 范围内。 |
| 若不变量失效，以 GNOME Shell 扩展回退 | 已记录为 ALTERNATIVES C / 路线图；当前实机证据表明 XWayland 不变量成立。 |
| 专用轮询工作线程，轮次不重叠 | **已满足**：单线程严格按 tick → sleep 顺序执行；轮询较慢只会拉长周期，不会重叠。 |
| 单独设置 `_NET_WM_STATE_SKIP_PAGER` 提示 | **已满足**：实机 `xprop` 显示已设置 SKIP_PAGER；Tauri 的 skipTaskbar 同时映射到两个 GTK 提示。 |
| 未加载 Pricing/ClaudeSettings，因此未完全达到 TUI 功能一致性 | 接受为 v0.1 限制：ctx/5h/7d 仪表来自适配器，不受影响；Codex `cost` 可能缺失。已在 SPEC 非目标中注明。 |

<!-- qhud:anchor -->
<a id="method-note-honesty"></a>

### 方法说明（如实记录）

这是实际进行的双模型验证：Claude 提出方案，GPT 作为对抗评审者，遵循先独立提出方法的协议，并非模拟验证。最终裁决依据是证据（实机 `xprop` 状态、固定 revision 的源码引用），不是模型身份。
