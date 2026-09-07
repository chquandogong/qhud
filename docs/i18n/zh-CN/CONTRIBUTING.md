<!-- qhud:languages -->
<p align="center">
  <a href="../../../CONTRIBUTING.md">English</a> · <a href="../ko/CONTRIBUTING.md">한국어</a> · <strong>简体中文</strong>
</p>
<!-- /qhud:languages -->

<!-- qhud:anchor -->
<a id="contributing-to-qhud"></a>

# 为 qhud 贡献

感谢你帮助 qhud 变得更清晰、更可靠。欢迎用 **English、한국어 和简体中文**提交 issue、pull request 和文档改进。

<!-- qhud:anchor -->
<a id="before-you-start"></a>

## 开始之前

- 阅读[用户指南](docs/GUIDE.md)，了解受支持行为和已知限制。
- 提交新报告前搜索[现有 issue](https://github.com/chquandogong/qhud/issues)，尽可能提供简小、可复现的示例。
- 在投入大规模实现前，先通过 issue 讨论较大的行为或架构变化。小修复和翻译可直接提交 PR。
- 每项变更保持聚焦，说明问题、最终行为以及如何验证。

<!-- qhud:anchor -->
<a id="report-a-bug"></a>

## 报告缺陷

使用[缺陷报告表单](https://github.com/chquandogong/qhud/issues/new?template=bug_report.yml)。包括 qhud 版本、操作系统/桌面、安装方式、供应商、复现步骤、预期和实际结果。区分新获取供应商数据与保存快照。视觉问题应说明桌面是否解锁，以及托盘、刷新和窗口控件是否响应。

诊断可能包含邮箱、账户 ID、本地路径、会话内容及凭据。分享前检查并脱敏。绝不附上 `auth.json`、`.credentials.json` 或完整供应商配置目录。若涉及凭据泄露，请描述问题，不要在公开 issue 粘贴凭据。

<!-- qhud:anchor -->
<a id="build-and-check"></a>

## 构建与检查

仓库声明 Rust 1.88 或更新版本；CI 使用 stable Rust。前端为普通 HTML/CSS/JavaScript，无需 npm。qmonster 固定在 `src-tauri/Cargo.toml` 所列 revision。

### Linux

安装[运维手册](docs/05-ops/RUNBOOK.md)中的开发依赖，然后运行：

```sh
cargo fmt --all --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
cargo build --locked --release
```

### Windows

使用带 C++ 工作负载和 Windows SDK 的 Developer PowerShell for Visual Studio：

```powershell
.\scripts\Build-Windows.ps1 -Test
git diff --exit-code -- Cargo.toml Cargo.lock
```

脚本准备固定 qmonster 检出，仅为自身 Cargo 命令应用 Windows 补丁，随后恢复规范 lockfile。不要在该检出并发运行其他 Cargo 命令。工具发现与显式路径见 [Windows 指南](docs/05-ops/WINDOWS.md)。

<!-- qhud:anchor -->
<a id="protect-the-data-contract"></a>

## 保护数据契约

- 每项配额始终保留所属账户、组织、模型 scope、时长和重置时刻；缺失数据不能成为虚构零值。
- 保留快照时间和归属。解析成功不能证明保存值属于当前登录。
- 供应商请求保持在显式刷新操作之后。qhud 不管理供应商登录，也不执行自己的 OAuth refresh grant。
- Windows 特有适配保持限定范围；普通 Linux 克隆必须能继续构建，不依赖开发者同级依赖目录。
- Linux 的 Tauri/WebKitGTK 进程不要安装 Unix 信号处理器，见 [D-012](docs/02-decisions/DECISION_LOG.md)。

[规格](docs/03-spec/SPEC.md)、[架构](docs/03-spec/ARCHITECTURE.md)和[决策日志](docs/02-decisions/DECISION_LOG.md)解释这些约束。

<!-- qhud:anchor -->
<a id="documentation-and-translations"></a>

## 文档与翻译

英文源文档位于仓库根和 `docs/`。`README.ko.md`、`README.zh-CN.md` 为本地化入口。完整韩语和简体中文镜像位于 `docs/i18n/ko/`、`docs/i18n/zh-CN/`，保留源文档相对路径。

修改文档时同步对应翻译。保留命令语法、API 标识符、日期、需求/决策 ID 及完整表格行。历史记录描述当时情况，不要静默把旧观察改成当前事实。MIT 许可证原文保持不变。

从各文档实际目录检查相对链接与段落锚点。使用现有或明确标为演示的截图，不把示例用量当作实机账户读数。文档索引列出[英语](../../../docs/README.md)、[韩语](../ko/docs/README.md)及[简体中文](docs/README.md)全部页面。

<!-- qhud:anchor -->
<a id="submit-a-pull-request"></a>

## 提交 pull request

描述改了什么、为什么，列出相关验证，说明尚存的平台或视觉验证缺口。修改解析器、scope、持久化或身份行为时增加回归测试。纯文档变更需要链接、翻译和渲染检查，无需重建应用。

构建通过不能证明桌面行为。窗口/层级变化须遵循[测试计划](docs/04-quality/TEST_PLAN.md)，区分自动检查与实际合成器/像素验证。不要仅根据合成夹具声称做过真实供应商测试。

<!-- qhud:anchor -->
<a id="release-maintenance"></a>

## 发布维护

带版本压缩包通过现有发布工作流发布，两个平台作业通过后才执行。保持包版本、Tauri 配置、lockfile、更新日志和发布说明一致。只有文档和设计更新时不需要新二进制发布。完整流程见[运维手册](docs/05-ops/RUNBOOK.md#release-procedure)。
