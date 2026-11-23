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

Linux v0.8.1 下载两个文件后：

```sh
sha256sum -c qhud-v0.8.1-linux-x86_64.tar.gz.sha256
tar -xzf qhud-v0.8.1-linux-x86_64.tar.gz
install -Dm755 qhud-v0.8.1-linux-x86_64/qhud "$HOME/.local/bin/qhud"
"$HOME/.local/bin/qhud"
```

Windows 下载后，将显示的摘要与 `.sha256` 文件比较：

```powershell
Get-FileHash -Algorithm SHA256 .\qhud-v0.8.1-windows-x86_64.zip
Get-Content .\qhud-v0.8.1-windows-x86_64.zip.sha256
```

当前 release workflow 发布的压缩包还带 GitHub 构建来源证明。安装 GitHub CLI，将文件名替换为实际下载文件，验证压缩包本身而非 `.sha256` 配套文件：

```sh
gh attestation verify ./qhud-vX.Y.Z-linux-x86_64.tar.gz --repo chquandogong/qhud
```

```powershell
gh attestation verify .\qhud-vX.Y.Z-windows-x86_64.zip --repo chquandogong/qhud
```

校验和用于发现相对配套文件的传输损坏或替换；证明用于确认 GitHub Actions 为本仓库构建了该压缩包。启用证明前创建的发布无法通过此验证。

v0.7.2 之后的发布还附带 CycloneDX 1.5 JSON 软件物料清单（SBOM）`qhud-vX.Y.Z-sbom.cdx.json`，并有独立的 `.sha256` 配套文件和构建来源证明。release workflow 基于打标签的源码及其锁定的 `Cargo.lock`、针对所有目标平台生成该清单，因此涵盖 Linux 与 Windows 构建的 Rust crate；前端是 qhud 自有的 HTML、CSS 和 JavaScript，没有第三方包。发布前会移除本地构建路径。请验证 SBOM 文件本身（Windows 上按上文用 `Get-FileHash` 与配套文件比对）：

```sh
sha256sum -c qhud-vX.Y.Z-sbom.cdx.json.sha256
gh attestation verify ./qhud-vX.Y.Z-sbom.cdx.json --repo chquandogong/qhud
```

Codex 和 Claude Code 使用各自的 CLI 登录。在 Windows 上，打开已登录的 Antigravity 原生应用，或保持 `agy` CLI 运行。启动 qhud 后点击供应商旁的 ⟳ 获取用量。便携包不注册快捷方式或自动启动。

<!-- qhud:anchor -->
<a id="read-the-widget"></a>

## 阅读组件

各供应商按**邮箱地址**分组，每个地址下先列**个人**，再列**团队**。每个工作区或组织分别显示自己的套餐、各时段用量和重置时间。等待连接的配置档案也列在相应的邮箱下。

- **5H / 7D** 表示窗口持续时长，不是距重置的剩余时间。
- **右侧时间**是距重置的剩余时间；悬停可看准确本地日期和时间。
- **模型名称**区分独立配额池，只有供应商提供时才显示。
- **快照时间**标识保存用量；重启 qhud 不会让旧读数变成新响应。
- **零窗格**表示没有可用的受支持终端来源；仍可显示账户用量。

缺少模型配额不等于用量为零。例如，只有当前 Codex 账户响应含有 Spark 5H/7D 限额时才显示它。刷新另一账户不能证明当前账户的限额。

**重置券 N张**标签显示供应商明确提供的次数。
Codex 使用官方 `rateLimitResetCredits.availableCount`；明确的 `0` 是有效值，
字段缺失则保留为不可用。这与额外用量金额及重置倒计时不同；qhud 只显示次数，
不会消耗重置券。若 Claude 用量响应没有提供手动重置次数，展开详情会说明信息
不可用。不会根据重置日期、用量百分比或不同类型的重置选项推算虚构的次数。
该次数同样适用所属账户行的快照时间。

Claude 旧版个人 `claude_max` 元数据与 `default_claude_max_20x` 组合，依据
[官方 Max 20x 更名](https://support.claude.com/en/articles/17728952-what-is-the-ultra-plan)
显示为 **Ultra**；`default_claude_max_5x` 显示为 **Max 5x**。对于
Team/Enterprise，主要标签按上述两种已知的用户等级显示成员的用量等级：
**Max 5x** 或 **Ultra**。单独的个人/团队范围及展开详情中的
**组织套餐：Team/Enterprise** 保留真实组织套餐。不会用组织等级替代未知的
成员等级；未识别用户等级时，标签保留 Team/Enterprise。显式套餐覆盖仍优先。
精确的 `claude_ultra` 或 `ultra` 元数据也显示为 Ultra；未知组织值不会被猜测。缺少的
5H 限额不会显示，包括新的 Ultra 响应不再提供该窗口的情况。

选择已识别套餐或席位标签旁的**支持的模型**，即可展开供应商说明。
这些内容是于 **2026-11-23** 核对的官方套餐参考信息，不是对该账户模型权限或
剩余配额的实时查询结果。请在供应商自己的模型选择器中，以当前登录的账户和
工作区为准确认实际可选模型：

- **Codex：**在 Codex CLI 中运行 `/model`。可用模型取决于账户、工作区权限和
  客户端版本；仅凭 Team/Business 套餐名称不能确定成员的席位类型。
  参见[官方 Codex 模型说明](https://developers.openai.com/codex/models)。
- **Claude Code：**在终端或 Claude Desktop 的 Code 标签页中运行 `/model`。
  Max、Ultra 套餐和 Team Premium 席位在套餐限额内包含 Fable 用量；
  Team Standard 席位使用 Fable 需要用量额度。显示的 **Max 5x** 成员用量等级
  本身不能证明该成员拥有 Premium 席位。参见
  [Claude Code 模型配置](https://code.claude.com/docs/en/model-config)及
  [Fable 使用资格与限额](https://support.claude.com/en/articles/15424964-claude-fable-models-on-your-plan)。
- **Antigravity：**使用其模型选择器。付费 Google AI Pro 与试用订阅的模型权限
  可能不同；部分模型要求付费订阅或符合特定条件的试用订阅。工作账户遵循组织的
  访问规则，包括合作伙伴模型所需的管理员批准。参见
  [Antigravity 套餐与模型条件](https://antigravity.google/pricing)。

受支持 Linux 环境的会话卡片另提供状态、上下文压力、模型、推理力度、分支、工作目录和冲突指示。本版本不原生观测 Windows Terminal、PowerShell 或 WezTerm 标签页。

<!-- qhud:anchor -->
<a id="system-usage-and-about"></a>

## 系统用量与关于

底部的小图表每 2 秒采样，保留最近约 60 秒内的最多 30 个样本。
选择 CPU、MEM、GPU、DISK 或 NET 可打开详情；再次选择同一指标可关闭，
选择其他指标则切换内容。CPU、内存和 GPU 使用固定的 0–100% 刻度；
磁盘与网络显示读取+写入或接收+发送吞吐量，刻度随近期流量调整。
将鼠标悬停在指标上可查看当前值；也可按 Tab 选择指标，再按 Enter 或 Space 操作。

| 指标 | 测量范围 |
| --- | --- |
| CPU | 所有逻辑处理器的总体 CPU 利用率。 |
| MEM | 已用物理内存 / 物理内存总量；详情显示字节容量。 |
| GPU | 在首次有效采样中选择最繁忙的受支持适配器，并在本次采样会话中保持选择。Windows 先合并同一引擎的进程实例，再显示最繁忙引擎的利用率。 |
| DISK | 物理磁盘的总读取/写入字节速率，避免重复计算分区和堆叠设备的 I/O。详情显示按卷去重的已挂载本地磁盘卷的已用/总容量，每 30 秒更新容量。 |
| NET | 具有可用 IP 地址的非回环接口总接收/发送速率。Linux 还会过滤运行状态为 down 的接口。VPN、虚拟和物理接口可能重复计算同一流量，因此这不是运营商带宽计量器。 |

GPU 测量取决于设备和驱动：Windows 使用 WDDM GPU Engine 计数器；Linux
支持 AMD 提供的 busy-percent sysfs 值、Intel i915/xe 空闲驻留计数器的补集，以及
通过已安装 NVML 库读取 NVIDIA 数据。仅当 Linux 数据源提供显存用量时显示该值；Windows 暂不显示显存。
不支持的 GPU 会隐藏。曾经支持的设备暂时读取失败时保留该列并显示缺口。
首次 CPU/速率采样、计数器重置和不可用读数显示 `--` 或缺口，不会填入虚构的零。
有效的空闲采样才显示为零。

窗口隐藏或最小化时暂停系统采样。重新显示后重新开始历史记录并预热速率计数器；
采样间隔过长时也会重置基线。历史只保存在内存中；系统采样不会扫描进程、启动
监控命令、读取账户凭据或发送遥测。`--demo` 使用明确标注为示例的合成系统数据。

点击左上角 **qhud** 打开 About，可查看运行中构建的版本、作者
**Chenghao Quan (chquandogong)** 和
[作者主页](https://chquandogong.github.io/CHENGHAO-QUAN/)。
通过 ×、Escape 或点击面板外部关闭。运行 `qhud --system-dump` 可输出仅含
本地系统信息的 JSON；它等待两次采样后退出，不会加载账户信息或打开组件窗口。

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
qhud --provider-health
qhud --refresh-health
qhud --dump
qhud --system-dump
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
  "plans": {
    "codex:YOUR_ACCOUNT_ID": "Pro 200"
  },
  "workspace_names": {
    "YOUR_OTHER_WORKSPACE_ID": "Work"
  },
  "workspace_plans": {
    "YOUR_OTHER_WORKSPACE_ID": "Pro 500"
  },
  "claude_config_dirs": ["~/claude-personal"],
  "codex_homes": ["~/.codex-personal"]
}
```

Codex 会自动显示本地 `auth.json` ID 令牌中的邮箱。这只是尽力读取的显示信息；邮箱缺失或格式错误时回退到账户/工作区 ID。新电脑上的普通邮箱显示不再需要单独的标签文件。匹配的 `labels` 条目仍优先于邮箱，两者均不改变用于区分用量的账户/工作区身份。

对于 Codex 套餐标签，qhud 对三个已审查的传输值采用当前上游状态名称：`prolite` 显示为 **Pro 100**，`pro` 显示为 **Pro 200**，`promax` 显示为 **Pro 500**。显式 `plans` 条目覆盖账户标签，`workspace_plans` 覆盖匹配的工作区标签。未知传输值只保留在展开详情和工具提示中，不会成为可见套餐标签。

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
<a id="windows-personal-and-team-profiles"></a>

### Windows 个人与团队配置档案

若要同时显示两个邮箱在 Codex 和 Claude 中的个人与团队范围，可在
`accounts.json` 的 `profiles` 中登记八个目标连接。登记只创建待连接显示项，
不会生成凭据、切换应用登录或编造账户 ID。将以下合成示例中的邮箱与私有目录
替换为自己的值，并将条目合并到现有配置：

```json
{
  "profiles": [
    { "key": "codex-first-personal", "provider": "codex", "label": "First personal", "email": "first@example.com", "scope": "personal", "directory": "~/qhud-accounts/codex-first" },
    { "key": "codex-first-team", "provider": "codex", "label": "First team", "email": "first@example.com", "scope": "team", "directory": "~/qhud-accounts/codex-first" },
    { "key": "codex-second-personal", "provider": "codex", "label": "Second personal", "email": "second@example.com", "scope": "personal", "directory": "~/qhud-accounts/codex-second" },
    { "key": "codex-second-team", "provider": "codex", "label": "Second team", "email": "second@example.com", "scope": "team", "directory": "~/qhud-accounts/codex-second" },
    { "key": "claude-first-personal", "provider": "claude", "label": "First personal", "email": "first@example.com", "scope": "personal", "directory": "~/qhud-accounts/claude-first-personal" },
    { "key": "claude-first-team", "provider": "claude", "label": "First team", "email": "first@example.com", "scope": "team", "directory": "~/qhud-accounts/claude-first-team" },
    { "key": "claude-second-personal", "provider": "claude", "label": "Second personal", "email": "second@example.com", "scope": "personal", "directory": "~/qhud-accounts/claude-second-personal" },
    { "key": "claude-second-team", "provider": "claude", "label": "Second team", "email": "second@example.com", "scope": "team", "directory": "~/qhud-accounts/claude-second-team" }
  ]
}
```

一次 Codex 登录可以枚举该邮箱有权访问的个人与团队工作区，因此它的两个
配置档案可以共享一个隔离的 Codex 主目录。Claude 则需要为选定的个人/团队
组织分别认证。Claude 登录时既要选择正确邮箱，也要选择目标组织。需要锁定
确切组织时，将已验证的 `expected_org_id` 加入配置档案，不要猜测。即使两个
邮箱共享同一 Codex 团队工作区，也会按邮箱加工作区 ID 分开显示。

在解压后的 Windows 包内含有 `qhud.exe` 的目录或源码仓库根目录打开 PowerShell。[连接脚本](../../../../scripts/Connect-Account.ps1)的 `-List` 参数可列出已注册的档案。使用 `-AllPending` 进行连接，或用 `-Profile codex-first-personal` 选择示例中的单个档案。

```powershell
.\scripts\Connect-Account.ps1 -List
.\scripts\Connect-Account.ps1 -AllPending
```

脚本处理待连接的配置档案，并按供应商/目录组合去重，因此不会对共享的 Codex
主目录重复登录。登录仍由用户在供应商自身流程中完成。脚本使用隔离的配置档案
目录，保留默认 `~/.codex` 和 `~/.claude` 登录，不覆盖它们。之后刷新 qhud。
未连接项以**需要连接 qhud**显示且不带数值；凭据过期、邮箱不符、组织不符也会明确
显示。出现匹配的真实账户/工作区行后会替代待连接项。添加配置档案不会授予账户
原本没有的个人或团队工作区访问权。

Claude 个人配置档案可以记录在 Claude 网页账户菜单中确认的
Free 套餐。在 `observed_plan` 中填写该档案的 `email`、值为 `"free"` 的
`plan` 和确认时间 `verified_at_ms`（Unix 毫秒时间戳）。个人行显示 **Free** 并注明
暂无用量数据。供应商返回该个人组织的用量时，优先显示实际用量行。

Codex 团队配置档案还可以保存从工作区管理员 Members 页面单独确认的席位
信息。即使尚需连接 qhud，也会显示为 **Standard · 网页确认** 或
**Premium · 网页确认**。这是带日期的手动确认记录，不是自动更新的 API 值，
也不能证明 qhud 已连接。工具提示和展开详情会显示来源及确认时间。只有邮箱
和工作区 ID 都匹配时才会附加到用量行；该信息不会替代供应商的 Business 等
实际套餐，qhud 也不会从套餐推断席位类型。

<!-- qhud:anchor -->
<a id="network-and-freshness"></a>

## 网络与新旧程度

<!-- qhud:anchor -->
<a id="windows-provider-connection-status"></a>

### Windows 供应商连接状态

即使用量尚不可用，Codex、Claude 和 Antigravity 也会显示。**可查询**表示已找到
本地连接条件；**查询成功**表示显式刷新成功。展开供应商状态可查看数据来源、最后
尝试、最后成功快照、可用的凭据过期时间及恢复操作。凭据过期指本地登录令牌，
不是订阅到期。刷新失败后保留旧读数及其时间；缺失用量绝不表示为零。

Codex 为当前账户使用已安装 CLI 的 `app-server`，并支持 Windows 的
`Programs/OpenAI/Codex/bin` 安装路径。Claude 使用 Claude Code 登录；Desktop
应用的登录不会更新已过期的 CLI 凭据。根据提示执行 `claude auth login` 后刷新
qhud。qhud 不自动操作登录，也不自行交换 refresh token。

Antigravity 支持现有 `agy` 监听器及正在运行的 Windows 原生应用的本地守护进程
注册。连接验证进程和环回端口，将用量绑定到应用返回的账户。此本地接口可能随
Antigravity 版本变化。若当前 CLI 身份不匹配保存结果，则标为**上次查询账户**，
不声称它是应用当前的登录账户。若本地服务不可用，请启动/打开 Antigravity 后刷新。

Antigravity 套餐标签使用所查询账户原生的 `userTier.name`，例如
**Google AI Pro**，并与该账户的用量快照一起保存。升级提示或配额形态不视为
当前套餐。新响应缺少套餐名称时会清除旧标签；显式账户 `plans` 覆盖仍优先。
若供应商未提供名称且没有覆盖值，则不显示标签。已保存的套餐名称与用量值具有
相同的快照时间和账户归属。

`qhud --provider-health` 输出不含邮箱、账户 ID、路径或凭据的有限本地连接诊断。
`qhud --refresh-health` 通过与组件相同的命令显式刷新三个供应商后输出诊断，因此
会发出供应商请求。两者均不打开额外窗口，常规后台观测保持被动。

Windows 源码构建可用 `scripts/Build-Windows.ps1 -DependenciesRoot <path>` 指定
单独的依赖目录，避免覆盖已有修改的 qmonster 检出。默认仍是相邻的 `qhud-deps`。

| 供应商 | 被动观测 | 显式刷新 |
| --- | --- | --- |
| Claude | 可用时的本地身份、状态行和用量缓存。 | 将现有 OAuth access token 发往供应商用量端点。 |
| Codex | 本地身份及受支持窗格状态行数据。 | 当前账户通过 `codex app-server`，回退及保存的凭据使用用量/账户端点。 |
| Antigravity | 可用时的本地账户、原生守护进程就绪状态及窗格信息。 | 经验证的 Windows 原生守护进程或正在运行的 `agy` 进程提供环回 RPC。 |

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
