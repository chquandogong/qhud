<!-- qhud:languages -->
<p align="center">
  <a href="../../../ko/docs/00-overview/DASHBOARD.md">한국어</a> · <a href="../../../../00-overview/DASHBOARD.md">English</a> · <strong>简体中文</strong>
</p>
<!-- /qhud:languages -->

<!-- qhud:anchor -->
<a id="dashboard--qhud"></a>

# 项目看板 — qhud

> 状态：v0.7.1 已发布 · 更新：2026-09-14 · 负责人：chquandogong
> 公开项目快照。Git 历史、带标签发布及 workflow 运行是权威记录；下方带日期观察保留其原始验证范围。

<!-- qhud:anchor -->
<a id="state"></a>

## 状态

| 项目 | 值 |
| --- | --- |
| 当前版本 | [v0.7.1](https://github.com/chquandogong/qhud/releases/tag/v0.7.1)：增加 Linux Intel GPU 测量，补全 [v0.7.0](https://github.com/chquandogong/qhud/releases/tag/v0.7.0) 引入的条件 GPU 支持。提供 Linux x86_64 tarball 和 Windows x86_64 ZIP。 |
| 流水线依赖 | qmonster @ `6a21c44`；Linux 使用规范 Git 依赖，Windows 使用命令级补丁并恢复 lockfile。 |
| 支持平台 | Ubuntu 24.04 / GNOME 与原生 Windows x64 / WebView2。原生 Windows 终端标签页观测仍在支持范围之外。 |
| 当前运行证据 | **Ubuntu 2026-09-11，v0.7.1：**测量 Intel Arc（Meteor Lake、`i915`、gt0+gt1）。`--system-dump` 报告 24.8%，同一区间独立 rc6 空闲驻留计算也是 24.8%；运行中小组件显示 CPU、内存、GPU、磁盘和网络历史。v0.7.1 只修改 Linux GPU 采样器，因此该补丁未重新执行 Windows 实机验证。 |
| 较早跨平台证据 | **公开 v0.6.0 Linux 产物，Ubuntu 2026-09-07：**校验和匹配；GNOME/Wayland 层状态、像素活性、八个 herdr 窗格及三个供应商刷新均通过。**Windows v0.6.0：**原生显示、账户邮箱、重置时间、Codex 刷新及 app-server 回退通过。准确环境和哈希见 TEST_PLAN。 |
| 质量证据 | **v0.7.1 Ubuntu 本地，2026-09-11：**格式无误、clippy `-D warnings` 无误、127/127 项 Rust 测试、6/6 项 Node 测试、release 构建 2 m 08 s。较早 v0.6.0 Ubuntu 与 Windows CI 通过 98/98 项 Rust 测试及 release 构建：[CI 34117359064](https://github.com/chquandogong/qhud/actions/runs/34117359064)。 |
| 输入验证原则 | Linux 交互声明需要合成器路径注入或人工操作；仅 XTEST 不予采信（D-010）。 |
| 帧守卫现场证据 | Journal 区间 2026-08-26 → 09-07：28 次冻结、28 次重新映射恢复、0 次重新执行、0 次操作者可见事故（D-017）。更早的终端附着实例 stderr 未进入 journal，因此无法重算其数字。 |
| 待扩展现场证据 | 系统指标的直接 v0.7.1 证据来自一台 Intel/i915 Linux 主机。AMD、NVIDIA、WDDM、普通 tmux 服务器、真实第二组织行及若干桌面生命周期检查仍需更广现场证据。无硬件证据时，以自动化覆盖为门槛。 |

<!-- qhud:anchor -->
<a id="decision-index"></a>

## 决策索引

D-001 第二前端 · D-002 Tauri v2 · D-003 XWayland/EWMH 层 · D-004 qmonster 库 + NoopSink · D-005 演示即一致性夹具 · D-006 tarball 发布 · D-007 mux 后端工厂（herdr） · D-008 自行驱动几何 · D-009 pointerdown 选择 · D-010 输入验证协议 · D-011 正确范围显示 · D-012 缩放 + 临时置顶 + 禁止信号 · D-013 本地账户身份 · D-014 默认被动、网络按请求 · D-015 每账户 CLI 配置目录支持多账户 · D-016 委托获取路径 · D-017 帧守卫 · D-018 行身份为（账户、组织） · D-019 宽容传输数字及字段级错误 · D-020 限定范围的 Windows 适配。

完整条目及证据见 [DECISION_LOG](../02-decisions/DECISION_LOG.md)。

<!-- qhud:anchor -->
<a id="delivery-record"></a>

## 交付记录

| 交付 | 状态 |
| --- | --- |
| v0.1.0–v0.3.0：最小组件、herdr/tmux 观测、输入架构、缩放、临时置顶、单实例 | 2026-08-05/06 发布 |
| v0.4.0–v0.5.3：正确范围配额、身份、多账户、持久化、委托获取、帧守卫、组织身份及传输漂移修复 | 2026-08-07 → 09-04 发布 |
| v0.6.0–v0.6.2：原生 Windows 账户组件、无 mux 真实账户视图、归属保护、便携打包、核心文档重写及 Codex 邮箱自动显示 | 2026-09-07/08 发布 |
| v0.7.0：CPU、内存、条件 GPU、磁盘、网络的有限系统历史及无障碍 About | 2026-09-10 发布 |
| v0.7.1：Intel i915/xe 空闲驻留 GPU 采样器及同区间现场比较 | 2026-09-11 发布并验证 |

<!-- qhud:anchor -->
<a id="open-verification-and-backlog"></a>

## 待验证与待办

| 项目 | 状态 |
| --- | --- |
| TEST_PLAN 中的 GNOME 概览、锁定/解锁、挂起/恢复、显示器热插拔、全屏 | 等待实机检查 |
| 普通 tmux 实时观测回退 | 等待现场检查 |
| 真实多组织登录的第二组织行 | 等待现场检查；凭据和机器路径保存在仓库外 |
| Windows、AMD、NVIDIA 系统指标 | 等待额外硬件证据 |
| 卡片 → 终端窗格聚焦跳转 | 待办 |
| 用于概览干净固定及桌面图标共存的 GNOME Shell 扩展 | 待办 |
| 上游 `ObserveSnapshot`、`.deb` 打包、agy 多账户 | 待办 |

<!-- qhud:anchor -->
<a id="evidence-notes"></a>

## 证据说明

v0.7.0 在不改变账户和窗格归属规则的情况下加入系统栏及 About。采样留在现有 2 s 轮询线程，内存最多保留 30 个点，窗口隐藏或最小化时暂停；缺失值显示为断点，不虚构零。v0.7.1 增加 Linux Intel 测量。上述 24.8% 比较只验证一台 i915 主机，不表示覆盖所有适配器。

2026-09-07 公开产物验证补上了由 Windows 主机发布的 v0.6.0 所留 Ubuntu 桌面缺口。它仍是层状态、画面更新、herdr 观测及供应商刷新的有效历史证据；v0.7.1 观察是当前系统指标证据。准确步骤及未完成项目见 [TEST_PLAN](../04-quality/TEST_PLAN.md)，当前需求和实现边界见 [SPEC](../03-spec/SPEC.md) 与 [ARCHITECTURE](../03-spec/ARCHITECTURE.md)。
