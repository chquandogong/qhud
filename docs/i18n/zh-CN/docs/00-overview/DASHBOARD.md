<!-- qhud:languages -->
<p align="center">
  <a href="../../../ko/docs/00-overview/DASHBOARD.md">한국어</a> · <a href="../../../../00-overview/DASHBOARD.md">English</a> · <strong>简体中文</strong>
</p>
<!-- /qhud:languages -->

<!-- qhud:anchor -->
<a id="dashboard--qhud"></a>

# 项目看板 — qhud

> 状态：v0.6.1 已发布 · 日期：2026-09-07 · 负责人：chquandogong
> 唯一事实来源是本 Git 仓库。本看板用于交接；换会话或智能体继续工作时首先阅读。

<!-- qhud:anchor -->
<a id="state"></a>

## 状态

| 项目 | 值 |
| --- | --- |
| 版本 | [v0.6.1](https://github.com/chquandogong/qhud/releases/tag/v0.6.1)，重写 SPEC/ARCHITECTURE 并补充带日期的 Ubuntu 证据；运行时行为与 [v0.6.0](https://github.com/chquandogong/qhud/releases/tag/v0.6.0) 相同，Linux x86_64 tarball + Windows x86_64 ZIP |
| 流水线依赖 | qmonster @ `6a21c44`；Linux 使用规范 Git 依赖，Windows 使用命令级补丁并恢复 lockfile |
| 平台 | Ubuntu 24.04 / GNOME 与原生 Windows x64 / WebView2；仍不支持 Windows 终端标签页观测 |
| 运行证据 | **2026-09-07 对公开 v0.6.0 Linux 产物进行 Ubuntu 验证**，校验和验证并安装：Ubuntu 24.04.3、GNOME Shell 46.0 Wayland、内核 7.0.0-28、eDP-1 2560×1600 + HDMI-1 3840×2160，scale 1、webkit2gtk 2.52.6、herdr 0.7.5 实时 8 窗格。`_NET_WM_STATE` = BELOW+STICKY+SKIP_PAGER+SKIP_TASKBAR，`_NET_WM_DESKTOP` = 0xFFFFFFFF，`WM_CLASS` = qhud/Qhud；间隔 3 s 两次 `xwd` 哈希不同，画面正在绘制；帧守卫已启动；三供应商均回答 ⟳。Windows v0.6.0 已验证原生显示、邮箱、重置、Codex 刷新和 app-server 回退 |
| 质量门槛 | Ubuntu fmt/clippy/**98/98** 测试/release 构建通过；Windows **98/98** 测试/release 构建通过，见 [CI 34117359064](https://github.com/chquandogong/qhud/actions/runs/34117359064)。2026-09-07 在 Ubuntu 对 v0.6.0 代码树本地重跑：fmt 正常、clippy `-D warnings` 正常、98/98，从固定 Git 依赖 6 m 21 s 完成 release 构建，无同级检出 |
| 输入验证 | Ubuntu **仅合成器路径**：Mutter RemoteDesktop 注入或人工，XTEST 不予采信，D-010 |
| 交叉验证 | Codex/GPT：AGREE-WITH-CHANGES，采纳 CV-1..4，见 CROSS_VALIDATION_LOG |
| 帧守卫现场统计 | 08-26 → 09-07，journal 覆盖：**28 次冻结、28 次重新映射恢复、0 次重新执行、0 次可见事故**（D-017）。v0.5.1/v0.5.2 数字来自附着终端实例，其 stderr 未进入 journal，无法重算 |
| 已知实机缺口 | 截至 2026-09-07 仍未解决：`~/claude-personal` 凭据于 2026-08-17 过期，每次 ⟳ 返回 401。只能由操作者登录清除；发布不会更新凭据。默认账户不受影响，局部失败保持局部 |

<!-- qhud:anchor -->
<a id="decision-index-full-entries-in-decision_log"></a>

## 决策索引（完整条目见 DECISION_LOG）

D-001 第二前端 · D-002 Tauri v2 · D-003 XWayland/EWMH 层 · D-004 qmonster 库 + NoopSink · D-005 演示即一致性夹具 · D-006 tarball 发布 · D-007 mux 后端工厂（herdr） · D-008 自行驱动几何 · D-009 pointerdown 选择 · D-010 DING 截获与验证协议 · D-011 正确范围显示 · D-012 缩放、临时置顶和禁止信号 · D-013 本地账户身份 · D-014 默认被动、网络按请求 · D-015 每账户 CLI 配置目录支持多账户 · D-016 委托获取路径，codex app-server、agy 环回 RPC · D-017 组件审计自身像素，帧守卫 · D-018 行身份为（账户、组织） · D-019 宽容传输数字及拒绝响应指明字段 · D-020 Windows 适配限定范围，用量保留账户和窗口。

<!-- qhud:anchor -->
<a id="work-board"></a>

## 工作看板

| 任务 | 状态 | 负责人 |
| --- | --- | --- |
| v0.1.0 最小切入：组件、桥接、演示、文档、CI、发布 | 2026-08-05 完成 | claude+chquandogong |
| herdr 后端实机（D-007）、输入架构（D-008/9/10） | 完成 | claude+chquandogong |
| 正确范围显示（D-011、v0.2.0） | 2026-08-06 完成 | claude+chquandogong |
| 缩放、临时置顶、单实例（D-012、v0.3.0）及浅色托盘图标 | 2026-08-06 完成 | claude+chquandogong |
| 可信配额：归属修复、身份、⟳、工作区（v0.4.0） | 2026-08-07 完成 | claude+chquandogong |
| 一眼查看所有账户：多账户、持久化、额外支出、agy RPC、codex app-server、全部刷新（v0.5.0、D-015/D-016） | 2026-08-10 完成 | claude+chquandogong |
| v0.5.0 标签与发布，CI 自 e6e31ed 后首次恢复绿色 | 2026-08-10 完成 | claude+chquandogong |
| “选择无效”历程：ptr/qsel 输入取证 → 配额行选择 → 仅 ⟳ 网络 → 像素帧守卫（v0.5.1、D-017） | 2026-08-14 完成 | claude+chquandogong |
| 行身份为（账户、组织）：一登录两组织，独立池（v0.5.2） | 2026-08-17 完成 | claude+chquandogong |
| `used_credits` 浮点导致 Claude ⟳ 失效两天：宽容数字及指明字段的解析错误（v0.5.3、D-019） | 2026-09-04 完成 | claude+chquandogong |
| 原生 Windows 账户组件、无 mux 本地回退、模型/重置可见性和归属保护（v0.6.0） | 已实现，本地测试 98/98 | codex+chquandogong |
| 可移植 Linux 依赖清单、Windows 脚本级补丁、Ubuntu + Windows CI/发布门槛 | 两平台 CI 均通过 | codex+chquandogong |
| v0.6.0 发布及下载验证 | [Release 34127575141 通过](https://github.com/chquandogong/qhud/actions/runs/34127575141)，两压缩包校验和匹配 | codex+chquandogong |
| 对公开 v0.6.0 构建重新验证 Ubuntu，补上“未在 Windows 主机重新验证”缺口 | 2026-09-07 完成 | claude+chquandogong |
| 为 v0.6.0/v0.6.1 从头重写 SPEC 和 ARCHITECTURE，依据现场证据更新 RISK_REGISTER 和 ASSUMPTIONS | 2026-09-07 完成 | claude+chquandogong |
| ⏳ TEST_PLAN 待检查项：概览、锁定、挂起、热插拔、全屏 | **待办，首次实机检查** | operator |
| 普通 tmux 服务器实机验证，回退路径 | 待办 | operator |
| `~/claude-personal` 个人组织登录，CLI 组织步骤选 PERSONAL；注册表已接入，OAuth 总自动选择团队会话 | 待办，由操作者按需执行 | operator |
| 点击卡片，聚焦终端对应窗格 | 待办 | — |
| GNOME Shell 扩展：概览干净固定与 DING 共存 | 待办 | — |
| qmonster 上游 `ObserveSnapshot` 导出，然后解除固定 | 待办 | — |
| `.deb` 包，CV 延后项 | 待办 | — |
| agy 多账户，OS 钥匙串逆向 | 待办 | — |

<!-- qhud:anchor -->
<a id="decision-queue-human"></a>

## 人工决策队列

_无待决策项。_ 参考机器 DING 保持关闭，`~/Desktop` 为空，操作者已批准；配套扩展存在前，重新启用会失去组件交互。

<!-- qhud:anchor -->
<a id="resume-point"></a>

## 继续工作的位置

v0.6.1 是文档与验证发布，运行时与 v0.6.0 相同；根据 v0.6.0 源码从头重写 `docs/03-spec/SPEC.md` 和 `docs/03-spec/ARCHITECTURE.md`，并依据带日期现场证据更新 RISK_REGISTER / ASSUMPTIONS。

**v0.6.0 留下的 Ubuntu 缺口已补上。** v0.6.0 从 Windows 主机发布，记录“未重新验证 Ubuntu 桌面集成”；2026-09-07 对公开 Linux 产物校验和验证、安装并在参考 Ubuntu 机器运行，确认桌面层状态、像素活性、herdr 8 窗格观测、三个供应商刷新。见上述运行证据及 TEST_PLAN 带日期条目。

本会话也确认：v0.5.3 传输漂移修复自 2026-09-04 持续有效，Claude 每次 ⟳ 均成功；v0.6.0 将 release 配置移到工作区根后，Linux 二进制从 25.13 MiB 降至 15.07 MiB。此前 `[profile.release]` 一直位于 Cargo 忽略的非根成员中，因此任何早期发布都未应用 `strip`/`lto`/`codegen-units = 1`。

下一批有意义的工作，按顺序：

1. **操作者验证**：需人在机器旁执行 TEST_PLAN ⏳ 条目：GNOME 概览、锁定/解锁、挂起/恢复、显示器热插拔、全屏；普通 tmux 后端检查；`~/claude-personal` 个人组织登录。注册表已接入，CLI 组织步骤选 PERSONAL，同时清除该行持续的 401。
2. **扩大 Windows 现场覆盖**：目前只有一台主机和一次冒烟检查。终端窗格归属按设计未实现，因此 Windows 的准确定位仍是“账户配额组件”，而非“会话 HUD”。
3. **待办**：卡片 → 窗格聚焦跳转仍是价值最高的小项；之后为 GNOME Shell 扩展、上游 `ObserveSnapshot` 导出及解除固定、`.deb` 打包、agy 多账户。
