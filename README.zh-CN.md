<!-- qhud:languages -->
<p align="center">
  <a href="README.md">한국어</a> · <a href="README.en.md">English</a> · <strong>简体中文</strong>
</p>
<!-- /qhud:languages -->

<p align="center">
  <img src="docs/assets/qhud-banner.svg" alt="qhud — 一眼掌握 AI 用量、重置时间与会话" width="100%">
</p>

<p align="center">
  <strong>Claude Code、Codex 和 Antigravity</strong> 的安静桌面助手。<br>
  在一个地方查看账户用量、重置时间和受支持的终端会话。
</p>

<p align="center">
  <a href="https://github.com/chquandogong/qhud/releases/latest"><img src="https://img.shields.io/github/v/release/chquandogong/qhud?style=flat-square&color=28a99e" alt="最新版本"></a>
  <a href="https://github.com/chquandogong/qhud/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/chquandogong/qhud/ci.yml?branch=main&style=flat-square&label=build" alt="Ubuntu 和 Windows 构建状态"></a>
  <img src="https://img.shields.io/badge/platforms-Linux%20%7C%20Windows-52677d?style=flat-square" alt="Linux 和 Windows">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-52677d?style=flat-square" alt="MIT 许可证"></a>
</p>

<p align="center">
  <a href="#install">下载</a> · <a href="docs/i18n/zh-CN/docs/GUIDE.md">用户指南</a> · <a href="docs/i18n/zh-CN/docs/README.md">文档</a> · <a href="docs/i18n/zh-CN/CONTRIBUTING.md">参与贡献</a>
</p>

---

<!-- qhud:anchor -->
<a id="one-view-for-your-ai-work"></a>

## 一个视图，掌握 AI 工作

qhud 将分散在各个命令行界面的账户配额和会话状态集中到小巧的桌面组件。它使用 [qmonster](https://github.com/chquandogong/qmonster) 的观测流水线，采用 Rust/Tauri 后端和轻量 webview，构建无需 Node.js 或 npm。

| 查看内容 | 获得的信息 |
| --- | --- |
| **用量与重置** | 账户时间窗口、可用时的模型独立配额池、重置倒计时和准确重置时刻。 |
| **账户信息** | 独立账户/工作区行、可配置显示名称，以及重启后仍带时间的快照。 |
| **会话活动** | 在受支持的 Linux 环境中：状态、上下文压力、模型、推理力度、分支、工作目录和冲突指示。 |
| **新旧程度** | 每 2 秒本地观测；只有显式刷新时才请求供应商用量。 |
| **系统用量与关于** | 低调显示 CPU、内存、受支持 GPU、磁盘和网络历史；选择指标查看详情。About 显示构建版本、作者和主页。 |

<!-- qhud:anchor -->
<a id="a-compact-view-with-room-for-detail"></a>

## 小巧概览，也容得下细节

<table>
  <tr>
    <td align="center"><img src="docs/assets/widget-compact.png" width="330" alt="Linux 演示：紧凑供应商配额区与会话卡片"><br><sub>紧凑的会话概览</sub></td>
    <td align="center"><img src="docs/assets/widget-expanded.png" width="330" alt="Linux 演示：展开会话与冲突详情"><br><sub>需要时展开详情</sub></td>
  </tr>
</table>

*截图使用 Linux 演示夹具和示例数值。当前账户布局及可用模型窗口取决于供应商、登录和平台。*

<!-- qhud:anchor -->
<a id="platform-support"></a>

## 平台支持

| 能力 | Linux x86_64 | Windows x64 |
| --- | --- | --- |
| 原生桌面组件 | GTK / WebKitGTK | WebView2，无需 WSL |
| 账户用量与重置时间 | 支持 | 支持 |
| 多个已登录配置目录 | 支持 | 支持 |
| 终端窗格观测 | tmux / herdr | 不支持原生 Windows 终端标签页 |
| 桌面集成 | 参考环境：Ubuntu 24.04、GNOME/Wayland 经 XWayland | 原生窗口与托盘，不保证 Linux 桌面层行为 |
| 分发 | 便携 `.tar.gz` | 便携 `.zip` |

其他 Linux 桌面尽力支持。不发布 macOS 包。没有受支持的多路复用器时，qhud 显示真实本地账户和零窗格。示例数据仅通过 `--demo` 提供。

<!-- qhud:anchor -->
<a id="install"></a>

## 安装

**当前版本：[v0.7.1](https://github.com/chquandogong/qhud/releases/tag/v0.7.1)。** 两平台构建/测试均为发布门槛，每份压缩包都有 SHA-256 文件。

| 下载 | 校验 |
| --- | --- |
| [Windows x64 ZIP](https://github.com/chquandogong/qhud/releases/download/v0.7.1/qhud-v0.7.1-windows-x86_64.zip) | [SHA-256](https://github.com/chquandogong/qhud/releases/download/v0.7.1/qhud-v0.7.1-windows-x86_64.zip.sha256) |
| [Linux x86_64 tarball](https://github.com/chquandogong/qhud/releases/download/v0.7.1/qhud-v0.7.1-linux-x86_64.tar.gz) | [SHA-256](https://github.com/chquandogong/qhud/releases/download/v0.7.1/qhud-v0.7.1-linux-x86_64.tar.gz.sha256) |

### Windows

解压 ZIP，运行版本目录内的 `qhud.exe`。需要 Microsoft Edge WebView2 Runtime。便携包不注册快捷方式或自动启动。前提、源码构建与配置见 [Windows 指南](docs/i18n/zh-CN/docs/05-ops/WINDOWS.md)。

### Linux

将压缩包及 SHA-256 文件下载到同一目录后：

```sh
sha256sum -c qhud-v0.7.1-linux-x86_64.tar.gz.sha256
tar -xzf qhud-v0.7.1-linux-x86_64.tar.gz
install -Dm755 qhud-v0.7.1-linux-x86_64/qhud "$HOME/.local/bin/qhud"
"$HOME/.local/bin/qhud"
```

Ubuntu 24.04 运行时依赖包括 `libwebkit2gtk-4.1-0`、`libgtk-3-0` 和用于托盘的 `libayatana-appindicator3-1`。[源码构建与 GNOME 自启动 →](docs/i18n/zh-CN/docs/05-ops/RUNBOOK.md)

<!-- qhud:anchor -->
<a id="everyday-use"></a>

## 日常使用

- **移动和缩放：** 拖动顶部或页脚，使用角落控点调整大小。
- **查看详情：** 选择账户行或会话卡片；悬停仪表查看准确重置时间。
- **刷新：** 顶部 ⟳ 刷新全部供应商，账户旁 ⟳ 刷新对应账户。
- **临时置顶：** 使用托盘 *Pin above windows*。
- **退出：** 在托盘选择 *Quit qhud*。

对**已经运行**的组件，可使用对应快捷命令：

```sh
qhud --refresh-all
qhud --peek
```

[命令、账户设置与故障排查 →](docs/i18n/zh-CN/docs/GUIDE.md)

<!-- qhud:anchor -->
<a id="understand-the-numbers"></a>

## 理解数字

百分比属于一个**账户和时间窗口**。Spark 5H/7D 等模型独立限额，只有供应商为该登录返回时才显示。缺失限额不会以虚构零值填充，一个账户保存的用量不会套用另一账户名称。

保存读数标明时间。剩余重置时间是倒计时，悬停仪表可看准确日期和时间。本地观测不请求供应商 API。显式刷新可能读取现有 access token 以调用供应商用量端点；qhud 自身不执行 OAuth refresh grant。Codex 可将回退获取委托给其自身 CLI，Antigravity 则使用运行中 CLI 的环回服务。

账户设置保留在仓库外。见[配置与本地数据指南](docs/i18n/zh-CN/docs/GUIDE.md#configuration-and-local-data)。

<!-- qhud:anchor -->
<a id="explore-the-documentation"></a>

## 探索文档

全部文档，包括设计决策和历史记录，均提供英语、韩语和简体中文。

| 从这里开始 | 深入了解 |
| --- | --- |
| [用户指南](docs/i18n/zh-CN/docs/GUIDE.md) | [架构](docs/i18n/zh-CN/docs/03-spec/ARCHITECTURE.md) |
| [Windows 设置](docs/i18n/zh-CN/docs/05-ops/WINDOWS.md) · [Linux 运维](docs/i18n/zh-CN/docs/05-ops/RUNBOOK.md) | [规格](docs/i18n/zh-CN/docs/03-spec/SPEC.md) · [决策日志](docs/i18n/zh-CN/docs/02-decisions/DECISION_LOG.md) |
| [参与贡献](docs/i18n/zh-CN/CONTRIBUTING.md) | [测试计划](docs/i18n/zh-CN/docs/04-quality/TEST_PLAN.md) · [风险登记](docs/i18n/zh-CN/docs/04-quality/RISK_REGISTER.md) |
| [更新日志](docs/i18n/zh-CN/CHANGELOG.md) | [完整文档索引](docs/i18n/zh-CN/docs/README.md) |

<!-- qhud:anchor -->
<a id="contribute"></a>

## 参与贡献

欢迎用 **English、한국어 或简体中文**提交缺陷报告、文档修正和翻译改进。可先阅读[贡献指南](docs/i18n/zh-CN/CONTRIBUTING.md)，或[创建 issue](https://github.com/chquandogong/qhud/issues/new/choose)。

qhud 是独立项目，与 Anthropic、OpenAI 或 Google 无关联。供应商名称用于标识兼容工具。

[MIT 许可证](LICENSE) · 基于 [qmonster](https://github.com/chquandogong/qmonster) 构建
