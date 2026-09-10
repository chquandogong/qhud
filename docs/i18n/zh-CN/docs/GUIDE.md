<!-- qhud:languages -->
<p align="center">
  <a href="../../ko/docs/GUIDE.md">한국어</a> · <a href="../../../GUIDE.md">English</a> · <strong>简体中文</strong>
</p>
<!-- /qhud:languages -->

<!-- qhud:anchor -->
<a id="user-guide"></a>

# 用户指南

本指南介绍安装 qhud、连接已有 CLI 账户，以及如何阅读用量，避免把保存快照误认为当前响应。

<!-- qhud:anchor -->
<a id="install-and-start"></a>

## 安装与启动

从[发布页面](https://github.com/chquandogong/qhud/releases/latest)选择 Windows ZIP 或 Linux tarball。每份压缩包包含带版本的目录，并配有独立 SHA-256 文件。

| 平台 | 启动 | 要求 |
| --- | --- | --- |
| Windows x64 | 解压 ZIP，运行解压目录内的 `qhud.exe`。 | Microsoft Edge WebView2 Runtime。见 [Windows 设置](05-ops/WINDOWS.md)。 |
| Linux x86_64 | 解压 tarball，安装版本目录内的二进制。 | GTK/WebKitGTK；参考系统为 Ubuntu 24.04。见 [Linux 运维](05-ops/RUNBOOK.md)。 |

Linux v0.6.2 下载两个文件后：

```sh
sha256sum -c qhud-v0.6.2-linux-x86_64.tar.gz.sha256
tar -xzf qhud-v0.6.2-linux-x86_64.tar.gz
install -Dm755 qhud-v0.6.2-linux-x86_64/qhud "$HOME/.local/bin/qhud"
"$HOME/.local/bin/qhud"
```

Windows 下载后，将显示的摘要与 `.sha256` 文件比较：

```powershell
Get-FileHash -Algorithm SHA256 .\qhud-v0.6.2-windows-x86_64.zip
Get-Content .\qhud-v0.6.2-windows-x86_64.zip.sha256
```

使用各供应商自身 CLI 登录。qhud 读取现有账户信息，不替代供应商登录流程。启动 qhud 后按 ⟳ 获取用量。便携包不注册快捷方式或自动启动。

<!-- qhud:anchor -->
<a id="read-the-widget"></a>

## 阅读组件

主要层级为**供应商 → 账户 → 时间窗口**。每个百分比属于对应账户和窗口。当前 Codex 工作区合入账户行；其他已认证工作区可显示为额外行。

- **5H / 7D** 表示窗口持续时长，不是距重置的剩余时间。
- **右侧时间**是距重置的剩余时间；悬停可看准确本地日期和时间。
- **模型名称**区分独立配额池，只有供应商提供时才显示。
- **快照时间**标识保存用量；重启 qhud 不会让旧读数变成新响应。
- **零窗格**表示没有可用的受支持终端来源；仍可显示账户用量。

缺少模型配额不等于用量为零。例如，只有当前 Codex 账户响应含有 Spark 5H/7D 限额时才显示它。刷新另一账户不能证明当前账户的限额。

受支持 Linux 环境的会话卡片另提供状态、上下文压力、模型、推理力度、分支、工作目录和冲突指示。本版本不原生观测 Windows Terminal、PowerShell 或 WezTerm 标签页。

<!-- qhud:anchor -->
<a id="controls-and-commands"></a>

## 控件与命令

拖动顶部/页脚移动，角落控点调整大小，Ctrl+滚轮缩放。选择账户或卡片展开详情。托盘提供 *Pin above windows* 和 *Quit qhud*。

以下命令面向**已经运行**的实例：

| 命令 | 操作 |
| --- | --- |
| `qhud --peek` | 切换置顶与桌面层模式。 |
| `qhud --refresh-all` | 请求刷新全部受支持供应商。 |
| `qhud --refresh-claude` | 请求刷新 Claude 用量。 |
| `qhud --fetch-codex` | 请求刷新 Codex 工作区用量。 |

Windows 使用可执行文件路径，例如 `& '.\qhud.exe' --refresh-all`。首次启动时转发标志不会执行其操作，须先启动组件再使用。

以下诊断独立运行：

```sh
qhud --dump
qhud --claude-usage
qhud --codex-usage
qhud --agy-usage
qhud --codex-appserver
```

`--dump` 输出一次本地观测载荷。三个 `--*-usage` 命令获取、输出 JSON 并保存结果。`--codex-appserver` 输出 CLI 回退响应，不保存。关闭已有实例后用 `qhud --demo` 查看示例数据。诊断可能包含账户身份、本地路径和会话详情，提交 issue 前应脱敏。

<!-- qhud:anchor -->
<a id="configuration-and-local-data"></a>

## 配置与本地数据

qhud 将配置保存在源码仓库外：

| 项目 | 位置或行为 |
| --- | --- |
| 默认 Linux 配置 | `~/.config/qhud/` |
| 新 Windows 安装 | `%APPDATA%\qhud\` |
| 显示设置与注册账户 | 配置目录内 `accounts.json` |
| 最近显式获取用量 | 配置目录内 `fetched-usage.json` |
| 默认 Codex 主目录 | `~/.codex`，或 `CODEX_HOME` |
| 默认 Claude 配置目录 | `~/.claude`，或 `CLAUDE_CONFIG_DIR` |

配置目录优先级：原样 `QHUD_CONFIG_DIR`；`XDG_CONFIG_HOME/qhud`；Windows 上已存在的 `%USERPROFILE%\.config\qhud`，然后 `%APPDATA%\qhud`；最后 `~/.config/qhud`。Windows 主目录优先 `USERPROFILE`，Linux 优先 `HOME`。

`accounts.json` 为普通 JSON，不支持注释或尾随逗号。以下示例使用账户 ID 占位符，应替换为 CLI 或 qhud 诊断报告的身份。合并到已有文件，不要丢弃其他条目。

```json
{
  "labels": {
    "codex:YOUR_ACCOUNT_ID": "work@example.com"
  },
  "workspace_names": {
    "YOUR_ACCOUNT_ID": "Work"
  },
  "workspace_plans": {
    "YOUR_ACCOUNT_ID": "My workspace plan"
  },
  "claude_config_dirs": ["~/claude-personal"],
  "codex_homes": ["~/.codex-personal"]
}
```

Codex 会自动显示本地 `auth.json` ID 令牌中的邮箱。这只是尽力读取的显示信息；邮箱缺失或格式错误时回退到账户/工作区 ID。新电脑上的普通邮箱显示不再需要单独的标签文件。匹配的 `labels` 条目仍优先于邮箱，两者均不改变用于区分用量的账户/工作区身份。套餐标签应由操作者提供，不要根据传输枚举猜测。

<!-- qhud:anchor -->
<a id="more-than-one-account"></a>

## 多个账户

让每个额外账户在自己的供应商目录保持登录，然后用 `claude_config_dirs` 或 `codex_homes` 注册目录。

```sh
CLAUDE_CONFIG_DIR=~/claude-personal claude
CODEX_HOME=~/.codex-personal codex login
```

Windows 中为 CLI 会话设置环境变量：

```powershell
$previousCodexHome = $env:CODEX_HOME
try {
  $env:CODEX_HOME = "$env:USERPROFILE\.codex-personal"
  codex login
} finally {
  $env:CODEX_HOME = $previousCodexHome
}
```

该示例登录后恢复 shell 原来的 `CODEX_HOME`。注册目录字符串会展开开头的 `~/` 或单独 `~`；JSON 文件中的 `%ENV%`、`$ENV`、`~\` 不展开。也可用绝对路径。

Claude 行区分**账户和组织**。两个目录登录同一账户及组织，不会产生独立配额池。Antigravity 多账户支持未实现。`known` 账户列表由操作者维护，qhud 不自动学习。忽略占位行不会隐藏实时账户。

<!-- qhud:anchor -->
<a id="network-and-freshness"></a>

## 网络与新旧程度

| 供应商 | 被动观测 | 显式刷新 |
| --- | --- | --- |
| Claude | 可用时的本地身份、状态行和用量缓存。 | 将现有 OAuth access token 发往供应商用量端点。 |
| Codex | 本地身份及受支持窗格状态行数据。 | 按保存凭据访问用量/账户端点，当前登录通过 `codex app-server` 回退。 |
| Antigravity | 可用时的本地账户和窗格信息。 | 正在运行的 agy 进程提供环回 RPC。 |

本地观测每 2 秒执行，不请求供应商 API。它可能从同时含凭据的文件中读取身份字段。显式刷新可能读取现有 access token；qhud 自身不执行 OAuth refresh grant。供应商 CLI 管理认证及任何委托的令牌轮换。

获取结果保存并标注时间。账户匹配防止上一登录快照显示在新登录名下；并发刷新保留彼此保存结果。qhud 观测 qmonster 流水线，不写入 qmonster 数据库。

<!-- qhud:anchor -->
<a id="troubleshooting"></a>

## 故障排查

| 症状 | 检查 |
| --- | --- |
| 账户缺失 | 确认供应商 CLI 已登录，qhud 读取预期配置/主目录。 |
| 邮箱被 ID 替代 | 确认 qhud 为 v0.6.2 或更新版本，并读取预期的 Codex 主目录。如果本地 ID 令牌没有可用邮箱，可添加匹配的 `labels` 条目覆盖显示。 |
| Spark 或其他模型缺失 | 刷新并检查当前账户响应；不会合成缺失模型。 |
| 旧读数或 401 错误 | 阅读行的时间/错误，在受影响目录通过供应商 CLI 重新登录。 |
| 没有终端窗格 | Linux 检查 tmux/herdr；原生 Windows 终端标签页不在本版本支持范围。 |
| 组件不可见 | 检查托盘，对运行实例使用 `--peek`，确认桌面已解锁。 |
| 行超过窗口 | 滚动配额区或增大窗口。 |
| Linux 显示器睡眠后像素冻结 | 查看[运维手册](05-ops/RUNBOOK.md#troubleshooting)中的帧守卫诊断。 |

报告缺陷前记录 qhud 版本、操作系统、供应商、安装方式、复现步骤及脱敏错误。自动检查与实际桌面验证的区别见[参与贡献](../CONTRIBUTING.md)和[测试计划](04-quality/TEST_PLAN.md)。
