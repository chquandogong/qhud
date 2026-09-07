<!-- qhud:languages -->
<p align="center">
  <a href="../../../README.md">English</a> · <a href="../../ko/docs/README.md">한국어</a> · <strong>简体中文</strong>
</p>
<!-- /qhud:languages -->

<!-- qhud:anchor -->
<a id="documentation"></a>

# 文档


qhud 全部文档均提供三种语言，涵盖首次安装、架构、决策和发布历史。

<!-- qhud:anchor -->
<a id="start-using-qhud"></a>

## 开始使用 qhud

| 文档 | 内容 |
| --- | --- |
| [项目介绍](../../../../README.zh-CN.md) | 功能、截图、平台支持和下载。 |
| [用户指南](GUIDE.md) | 用量/重置语义、命令、多账户、配置及故障排查。 |
| [Windows 设置](05-ops/WINDOWS.md) | WebView2 要求、便携安装和原生 MSVC 构建。 |
| [运维手册](05-ops/RUNBOOK.md) | Linux 设置、GNOME 集成、诊断和发布流程。 |
| [参与贡献](../CONTRIBUTING.md) | 缺陷报告、开发检查、翻译和 pull request。 |
| [更新日志](../CHANGELOG.md) | 各版本变化和带日期的验证说明。 |

<!-- qhud:anchor -->
<a id="understand-the-implementation"></a>

## 理解实现

| 文档 | 内容 |
| --- | --- |
| [项目看板](00-overview/DASHBOARD.md) | 当前版本、验证证据、工作看板及继续工作位置。 |
| [项目简介](00-overview/PROJECT_BRIEF.md) | 目的、支持范围、需求和非目标。 |
| [规格](03-spec/SPEC.md) | 需求、CLI 标志、环境变量和载荷契约。 |
| [架构](03-spec/ARCHITECTURE.md) | 数据流、模块、身份、供应商获取路径及平台边界。 |
| [决策日志](02-decisions/DECISION_LOG.md) | 设计决策及其证据。 |
| [测试计划](04-quality/TEST_PLAN.md) | 自动检查与带日期的桌面验证。 |
| [风险登记](04-quality/RISK_REGISTER.md) | 已知风险、缓解和观察到的故障。 |
| [假设](01-discovery/ASSUMPTIONS.md) | 当前前提及验证状态。 |

<!-- qhud:anchor -->
<a id="explore-the-project-history"></a>

## 探索项目历史

这些文档保留带日期的调研和决策。早期提案或旧验证缺口是历史陈述，不一定代表当前产品行为。了解现状请先阅读规格和看板。

| 文档 | 记录 |
| --- | --- |
| [Office hours](01-discovery/OFFICE_HOURS.md) | 最初问题界定与讨论。 |
| [调研笔记](01-discovery/RESEARCH_NOTES.md) | 早期技术参考和观察。 |
| [可行性报告](01-discovery/FEASIBILITY_REPORT.md) | 初始实现选项及可行性。 |
| [备选方案](02-decisions/ALTERNATIVES.md) | 曾考虑的方法及取舍。 |
| [交叉验证日志](02-decisions/CROSS_VALIDATION_LOG.md) | 外部评审及采纳的修改。 |
| [复盘](05-ops/RETRO.md) | 开发过程中记录的经验。 |
| [v0.6.0 发布说明](05-ops/releases/v0.6.0.md) | 原生 Windows 支持及配额/重置修复。 |
| [v0.6.1 发布说明](05-ops/releases/v0.6.1.md) | 文档重写与 Ubuntu 验证。 |

<!-- qhud:anchor -->
<a id="languages-and-source-fidelity"></a>

## 语言与源文档一致性

英文页面保留原路径，已有链接继续有效。韩文和中文镜像在各语言目录内保留这些路径。每页链接到对应语言页面；翻译标题保留供源文档链接使用的锚点。

翻译覆盖全文，不只是概览。保留命令块、API/schema 标识符、需求/决策 ID、日期及验证值。[MIT 许可证](../../../../LICENSE)保持英文原文。

需要修正时，请[创建 issue](https://github.com/chquandogong/qhud/issues/new/choose)或遵循[贡献指南](../CONTRIBUTING.md)。欢迎三种语言。
