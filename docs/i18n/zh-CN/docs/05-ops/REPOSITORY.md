<!-- qhud:languages -->
<p align="center">
  <a href="../../../ko/docs/05-ops/REPOSITORY.md">한국어</a> · <a href="../../../../05-ops/REPOSITORY.md">English</a> · <strong>简体中文</strong>
</p>
<!-- /qhud:languages -->

# 仓库运营

本文区分随提交追踪的 qhud 工作流与无需提交即可改变的 GitHub 设置。仓库、`main` 和发布标签规则集已于 **2026-09-15** 从 GitHub 读取。安全功能、Actions 设置、`release` 环境和零警报数量最后于 **2026-09-14** 核对；设置发生变化后应再次核对。目前 qhud 是默认分支为 `main`、由**一名维护者管理的公开仓库**。下表的私有仓库列是未来改变可见性或团队人数时可选的运营方式，并非当前仓库设置。

## 按实际协作场景选择规则

| 项目 | 单人私有 | 团队私有 | 公开 qhud（当前） |
| --- | --- | --- | --- |
| 可见性与反馈渠道 | 仅让所有者看到源码和运营 issue。使用内部 issue 清单和私密安全报告渠道。 | 只授予所需的最小仓库权限；使用内部 issue 和私密安全报告渠道。 | 源码和普通 issue 公开。接受英语、韩语或中文的具体 issue 与 PR；疑似漏洞进入私密报告渠道。 |
| 修改 `main` | 保留 PR、七项状态检查、解决审核对话、线性历史和 squash 合并。必需批准数为零时，单独维护者可在检查通过后合并。 | 保留 PR 和检查；要求至少一次独立批准，考虑代码所有者审核和对最后一次 push 的批准，并指定独立发布审批人。 | 生效规则要求 PR、七项严格检查、解决审核对话、线性历史与 squash 合并。只有一名维护者期间，批准数为零，但检查仍不可跳过。 |
| 发布 | 限制标签创建与发布权限为所有者；分享前检查说明和产物。 | 标签及发布权限只交给指定维护者，并要求独立发布批准。 | 保护 `v*` 标签，只通过质量门禁和 `release` 环境发布。发布前检查 release 草稿，发布后检查四项资产、校验和与来源证明。 |
| 安全与警报 | 确认仓库套餐可用的 GitHub 安全功能；私下处理警报，不把真实密钥复制进测试。 | 指定警报分类与修复负责人，对安全 PR 进行独立审核。 | 保持私密漏洞报告、依赖与代码扫描、密钥扫描和 push 保护；不要在公开 issue 中披露新警报的利用细节。 |

将公开仓库设置用于私有仓库前，先确认功能可用性、费用和目标读者。增加第二名维护者时，先安排审核者和发布审批人的职责，并用测试 PR 证明新规则能够满足，再调整批准要求。

## 当前分支和标签保护

生效的 [Protect main 规则集](https://github.com/chquandogong/qhud/rules/23268129)（ID `23268129`）适用于默认分支。它禁止删除和非快进更新，要求线性历史和 PR，且 PR 只能使用 **squash** 合并。它还要求解决审核对话，并对无法归属作者的变更追加批准。**必需批准审核数为零**；代码所有者审核及最后一次 push 的批准不是强制要求。**没有绕过者**。必需状态检查采用严格策略：合并前须将 PR 分支更新到最新 `main`。[Protect release tags 规则集](https://github.com/chquandogong/qhud/rules/23267522)（ID `23267522`）适用于 `refs/tags/v*`，禁止删除、更新和非快进修改，也没有绕过者。这些是 GitHub 服务器设置；仅靠仓库文件无法强制执行。

GitHub 目前仅允许 squash 合并，提供自动合并及分支更新，并在合并后删除 PR 的源分支。[CODEOWNERS](../../../../../.github/CODEOWNERS) 为全部变更以及 `.github/`、`scripts/`、`src-tauri/` 指定 `@chquandogong`。该文件明确所有者，但当前规则集未将代码所有者批准设为强制条件。

七项必需状态检查的准确名称如下：

1. `fmt · clippy · test · build`
2. `Windows MSVC · test · build`
3. `docs · release metadata`
4. `dependency review`
5. `Analyze (rust)`
6. `Analyze (javascript-typescript)`
7. `Analyze (actions)`

前三项由 [CI](../../../../../.github/workflows/ci.yml) 提供：Ubuntu 24.04 运行 Node 系统指标测试、Rust 格式检查、将警告当作错误的 Clippy、Rust 测试和 release 构建；Windows 2022 用限定范围的依赖补丁测试和构建，再检查 `Cargo.toml`/`Cargo.lock` 没有变化；完整性作业运行[仓库检查器](../../../../../scripts/check-repository.mjs)，核对版本、发布元数据、三种语言文档的文件对应关系及本地链接。CI 在推送到 `main` 和 `codex/**`、PR，以及作为发布的可复用品质门禁时运行。[依赖审核](../../../../../.github/workflows/dependency-review.yml) 在 PR 新增中等或更高严重程度的依赖时失败。三个 Analyze 项来自 GitHub CodeQL 默认设置，而非仓库中追踪的 CodeQL 工作流；最后确认的设置在 PR 和每周扫描 Rust、JavaScript/TypeScript、Actions。改变必需检查名称或扫描语言前，应同时查看 CodeQL 配置和实际状态检查。

## Actions、依赖维护与安全

最后确认的 Actions 策略允许选定的 GitHub 官方 action，以及 `dtolnay/rust-toolchain` 和 `Swatinem/rust-cache`；第三方 action 固定到完整提交 SHA。默认工作流 token 为只读。工作流文件也分别声明狭窄权限：CI 和依赖审核只读 contents；发布打包为来源证明增加 `id-token: write` 与 `attestations: write`；仅发布作业具有 `contents: write`。新增 action 或修改发布作业时，同时复核仓库整体 Actions 策略与每个作业的权限。

[Dependabot 配置](../../../../../.github/dependabot.yml) 在每周一 09:00（Asia/Seoul）请求 Cargo 更新，在每月 09:00 请求 GitHub Actions 更新；每个生态系统最多保留五个开放 PR，minor/patch 更新分组处理。最后确认的 GitHub 设置启用了 Dependabot 安全更新、私密漏洞报告、带 push 保护的密钥扫描和 CodeQL 默认设置。这些服务器端开关独立于 `dependabot.yml`。**2026-09-14** 核对的本仓库安全警报队列为空（**0 条**）。这是带日期的观察结果，不保证目前仍无警报。对新的代码、依赖和密钥扫描警报，确认受影响版本及实际可达性；优先处理暴露的凭据；在 PR 中记录已脱敏的修复和验证。协调披露前不要公开漏洞复现方法；遵循 [SECURITY.md](../../../../../SECURITY.md)。

## PR、issue 与发布

创建 PR 前先搜索已有 issue，并使变更范围明确。[PR 模板](../../../../../.github/PULL_REQUEST_TEMPLATE.md)要求填写相关 issue 或无需 issue 的原因、变更后的行为、实际验证、未经验证的行为、隐私与安全检查，以及同步三种语言的文档。解决审核对话并为严格检查更新分支，七项状态全部通过后才能 squash 合并。自动合并同样不能绕过门禁。普通问题、功能建议和支持问题使用收集版本及环境的三个 [issue 表单](../../../../../.github/ISSUE_TEMPLATE/)；空白 issue 被禁用，选择页面链接文档和私密安全报告。Issues 已启用，Wiki、Projects、Discussions 已关闭。发布前对凭据、账户数据、本地路径和截图脱敏。根据可复现性、支持版本、平台，以及问题属于 qhud 还是上游服务商，对公开 issue 分类；渠道选择见 [SUPPORT.md](../../../../../SUPPORT.md)。

[发布工作流](../../../../../.github/workflows/release.yml)由 `v*` 标签触发。预检只接受稳定的 `vX.Y.Z` 标签，并要求 Cargo 与 Tauri 版本匹配、对应版本的发布说明非空、CHANGELOG 有条目、标签提交已包含在 `origin/main`。它调用完整 CI 品质门禁，打包 Linux x86_64 和 Windows x86_64，生成 SHA-256 文件和构建来源证明，再验证恰好四项资产及其校验和。发布经过 `release` 环境，先创建草稿。已公开的 release 不会被替换；若现有草稿资产的字节不同，也不会覆盖。资产验证后才公开草稿。最后确认的 `release` 环境要求维护者作为审批人，并允许这个单人仓库自行批准。**发布不可变设置于 2026-09-14 启用，适用于之后的新 release；既有 `v0.7.1` 不会追溯改变，其 API `immutable` 值为 `false`。** 标签保护已经禁止修改 `v*` 引用。操作顺序见[运行手册](RUNBOOK.md#release-procedure)。

## GitHub 或工作流变化后的复核

1. 从 GitHub 读取仓库元数据、两套生效规则、合并设置、Actions 权限、`release` 环境、CodeQL 默认设置、Dependabot 与安全开关，以及当前警报队列，并记录核对日期。不要从 YAML 文件推断服务器设置。
2. 将新测试 PR 产生的状态检查名称、CodeQL 语言与必需列表比较。复核严格分支更新、审核对话解决、合并方式、CODEOWNERS，以及单独维护者能否实际满足任何新增批准规则。
3. 若追踪文件改变，运行 `node scripts/check-repository.mjs` 和相关 CI 检查。若发布流程改变，还需核对标签祖先关系、版本、三种语言的说明、草稿审批、校验和、来源证明及支持平台上的下载资产。
4. 根据观察到的设置、证据日期，以及预期策略与生效策略间的差异，更新本文和英语、韩语镜像。将警报数量视为带日期的观察结果，宣称警报已解决前再次核对。
