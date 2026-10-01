<!-- qhud:languages -->
<p align="center">
  <a href="../ko/CONTRIBUTING.md">한국어</a> · <a href="../../../CONTRIBUTING.md">English</a> · <strong>简体中文</strong>
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

`main` 的日常 stderr 事件记录省略可识别值。已发布的 v0.7.1 二进制包早于此
安全修订。`--dump`、`--codex-usage` 等显式 JSON 命令在两个版本中仍输出
操作者请求的完整载荷；分享前检查并脱敏。

<!-- qhud:anchor -->
<a id="build-and-check"></a>

## 构建与检查

仓库声明 Rust 1.88 或更新版本；CI 使用 stable Rust。应用构建采用普通 HTML/CSS/JavaScript，无需 Node.js 或 npm。Node.js 22 仅用于独立的前端系统指标回归测试。qmonster 固定在 `src-tauri/Cargo.toml` 所列 revision。

在任一开发平台运行：

```sh
node --test tests/system-metrics.test.cjs
```

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

默认 `README.md` 为韩语。英语入口为 `README.en.md`，简体中文入口为 `README.zh-CN.md`；为兼容已有链接，保持 `README.ko.md` 与 `README.md` 一致。其他英文源文档仍位于仓库根和 `docs/`。完整韩语和简体中文镜像位于 `docs/i18n/ko/`、`docs/i18n/zh-CN/`，保留源文档相对路径。

<!-- qhud:anchor -->
<a id="which-documents-are-mirrored"></a>

### 需要镜像的文档

- **三种语言完整镜像。** `README.en.md` 及其韩语版 `README.md`、中文版 `README.zh-CN.md`；`CHANGELOG.md`、`CODE_OF_CONDUCT.md`、`CONTRIBUTING.md`、`SECURITY.md`、`SUPPORT.md`；以及 `docs/` 下的所有 Markdown 文件，包括带日期的研究、决策、回顾和发布说明。每个镜像都是完整译文，而不是摘要。
- **单一规范语言，无镜像。** `AGENTS.md` 和 `.github/copilot-instructions.md` 是面向编码代理的英文指令。PR 模板和 issue 表单在同一文件中同时包含三种语言。`LICENSE` 保持英文原文。源代码注释、提交信息和工作流文件使用英文。
- **规范记录加摘要。** 目前没有文档采用此级别。只有带日期的内部记录（office hours、研究笔记、可行性报告、备选方案、交叉验证日志和回顾）是候选。将文档移入此级别需要一条决策日志记录，并修改 `scripts/check-repository.mjs` 中的登记表；在此之前它们仍是完整镜像。

仓库检查器保存这份登记表，并拒绝未分类的新根目录 Markdown 页面。

<!-- qhud:anchor -->
<a id="keep-facts-mechanically-comparable"></a>

### 保持事实可机械比较

修改文档时，在同一个 pull request 中同步两种译文。翻译文字，不翻译事实。`node scripts/check-repository.mjs` 对每个镜像将以下内容与英文源文档比较，任何差异都会使检查失败：

- 代码块之外的需求、决策、风险和假设 ID（`FR-`、`NFR-`、`D-`、`CV-`、`R1`、`A1` 形式）；
- `YYYY-MM-DD` 日期和发布版本集合；
- 去除缩进后逐行比较的围栏代码块；
- 表格行数和标题数。

命令语法、API 标识符和行内代码保持不变，保留每一个表格行。历史记录描述当时情况，不要静默把旧观察改成当前事实。MIT 许可证原文保持不变。

纯文档修改运行 `node scripts/check-repository.mjs`，检查各语言文件集合、翻译事实、版本、
发布说明链接和本地 Markdown 链接。下载版本应对应最新公开发布；新二进制标签
发布之前，`main` 的后续行为须明确标为未发布。

从各文档实际目录检查相对链接与段落锚点。使用现有或明确标为演示的截图，不把示例用量当作实机账户读数。文档索引列出[英语](../../../docs/README.md)、[韩语](../ko/docs/README.md)及[简体中文](docs/README.md)全部页面。

<!-- qhud:anchor -->
<a id="pull-request-translation-signal"></a>

### Pull request 翻译信号

在 pull request 中，`docs · release metadata` 检查还会以 `--base origin/<基准分支>` 再次运行检查器。若 pull request 修改了英文源文档，却没有同时修改它的两个镜像，检查失败。推送前请运行同样的比较：

```sh
node scripts/check-repository.mjs --base origin/main
```

若仅英文的修改确实正确，例如译文中并不存在的拼写、失效链接或格式修正，请在该 pull request 的任一提交信息中加入 trailer 予以确认，写明路径和原因：

```text
Translation-Exempt: docs/GUIDE.md fix an English misspelling
```

检查随后通过，并以 notice 报告该确认。不要将 trailer 用于含义变化、增删事实或版本更新；这些都需要两种译文。仅修改译文时不需要 trailer。

<!-- qhud:anchor -->
<a id="reviewer-checklist"></a>

### 审阅者检查清单

批准文档修改前，逐一检查每个语言版本：

1. **含义。** 镜像与英文源文档的主张、条件、限制和注意事项一致，没有增添、遗漏或弱化。带日期的观察在所有语言中都保留日期。
2. **检查器看不到的事实。** 正文中的数字和单位、产品、命令和设置名称以及引用的界面标签与源文档一致。
3. **链接。** 相对链接和锚点从译文文件的实际目录解析。翻译后的标题通过 `<!-- qhud:anchor -->` 和 `<a id>` 保留英文锚点，发布说明的语言链接保持为固定到标签的绝对 URL。
4. **隐私。** 任何语言版本都没有新增邮箱地址、账户或组织 ID、token、cookie、会话内容、本地路径或未脱敏诊断，截图在所有语言中均为演示或已脱敏。
5. **确认。** 每个 `Translation-Exempt` trailer 指向正确路径，并说明一项确实只涉及英文的修改。

CI 的执行方式和首次试行的成本记录在[仓库运维指南](docs/05-ops/REPOSITORY.md#translation-maintenance)中。

<!-- qhud:anchor -->
<a id="submit-a-pull-request"></a>

## 提交 pull request

描述改了什么、为什么，列出相关验证，说明尚存的平台或视觉验证缺口。修改解析器、scope、持久化或身份行为时增加回归测试。纯文档变更需要链接、翻译和渲染检查，无需重建应用。

从聚焦的分支向 `main` 提交 PR，不要以推送发布标签替代评审。受保护分支要求
Linux 与 Windows 构建/测试、仓库完整性与依赖审查，以及 Rust、
JavaScript/TypeScript、Actions CodeQL 分析。解决评审对话；检查过期时更新
分支。个人维护的必需批准数为 0；团队仓库应要求独立评审者。设置及
private/public 变体见[仓库运维指南](docs/05-ops/REPOSITORY.md)。

构建通过不能证明桌面行为。窗口/层级变化须遵循[测试计划](docs/04-quality/TEST_PLAN.md)，区分自动检查与实际合成器/像素验证。不要仅根据合成夹具声称做过真实供应商测试。

<!-- qhud:anchor -->
<a id="release-maintenance"></a>

## 发布维护

带版本压缩包通过现有发布工作流发布，两个平台作业通过后才执行。保持包版本、Tauri 配置、lockfile、更新日志和发布说明一致。只有文档和设计更新时不需要新二进制发布。完整流程见[运维手册](docs/05-ops/RUNBOOK.md#release-procedure)。
