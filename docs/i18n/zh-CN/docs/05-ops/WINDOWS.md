<!-- qhud:languages -->
<p align="center">
  <a href="../../../../05-ops/WINDOWS.md">English</a> · <a href="../../../ko/docs/05-ops/WINDOWS.md">한국어</a> · <strong>简体中文</strong>
</p>
<!-- /qhud:languages -->

<!-- qhud:anchor -->
<a id="windows-native-build-and-run"></a>

# Windows 原生构建与运行

qhud 从 v0.6.0 起提供 Windows x64 WebView2 小组件，无需 WSL 即可显示账户用量。前端使用仓库的 `ui/` 文件，不需要 Node.js 或 npm。

<!-- qhud:anchor -->
<a id="install-a-release"></a>

## 安装发布版

从 [GitHub Releases](https://github.com/chquandogong/qhud/releases) 下载 `qhud-v0.6.1-windows-x86_64.zip` 及其校验和文件。解压后运行版本目录内的 `qhud.exe`。必须安装 Microsoft Edge WebView2 Runtime。

ZIP 为便携包，不自动注册安装程序、登录自启动或快捷方式。更新时先通过托盘 **Quit qhud** 退出，再替换可执行文件。设置与供应商登录文件保存在独立位置。

<!-- qhud:anchor -->
<a id="build-from-source"></a>

## 从源码构建

- Git for Windows，`git.exe` 必须在 PATH 中。
- Rust 1.88 或更高版本及 `x86_64-pc-windows-msvc` 工具链。
- Visual Studio Build Tools，包含 **Desktop development with C++** 和 Windows SDK。
- Microsoft Edge WebView2 Runtime。

在 Developer PowerShell for Visual Studio 中进入仓库并运行：

```powershell
.\scripts\Build-Windows.ps1 -Test
```

不在开发者 shell 中时，可指定 C++ 工具安装位置：

```powershell
.\scripts\Build-Windows.ps1 -Test -VisualStudioPath 'C:\BuildTools'
```

产物为 `target\x86_64-pc-windows-msvc\release\qhud.exe`。`-Run` 在构建后启动；`-DebugBuild` 选择 debug 配置；`-PrepareOnly` 只准备工具和依赖，不编译。`-Test` 先在同一构建设置下执行测试。`-PrepareOnly` 不得与 `-Run` 或 `-Test` 同用。

脚本优先使用 PATH 中的工具和现有 Visual Studio 开发环境，也会检查仓库旁的 `tools/rust/cargo/bin`、`tools/cargo/bin` 和 `tools/vs-buildtools`。它不会安装工具或修改全局 PATH。

Windows 构建在同级 `qhud-deps/qmonster` 目录准备固定 revision，并应用 `patches/qmonster-windows.patch`，不会覆盖其他已修改检出目录。补丁只通过对应 Cargo 命令的临时配置生效；临时修改的 lockfile 无论成功或失败都会恢复原始字节。不要在同一检出目录并发运行其他 Cargo 命令。普通 Linux `cargo build --release --locked` 使用固定 Git 依赖，无需此准备步骤。

<!-- qhud:anchor -->
<a id="run-and-interact"></a>

## 运行与操作

```powershell
Start-Process -FilePath '.\target\x86_64-pc-windows-msvc\release\qhud.exe'
& '.\target\x86_64-pc-windows-msvc\release\qhud.exe' --peek
& '.\target\x86_64-pc-windows-msvc\release\qhud.exe' --refresh-all
```

拖动顶部或页脚移动窗口，通过角落调整大小。在托盘选择 **Quit qhud** 退出。已经运行时，后续 `--peek` 和 `--refresh-all` 请求会转发到该实例。用量通过 ⟳ 或显式刷新命令获取；本地观测间隔不是网络刷新间隔。

<!-- qhud:anchor -->
<a id="account-and-model-usage"></a>

## 账户与模型用量

新 Windows 安装将设置保存在 `%APPDATA%\qhud`。`accounts.json` 保存显示名称和注册账户路径；`fetched-usage.json` 保存带获取时间的用量。Git 仓库和发布包不包含个人设置及获取结果。

`QHUD_CONFIG_DIR` 覆盖配置位置。尊重 `XDG_CONFIG_HOME` 或已存在的 `%USERPROFILE%\.config\qhud`。默认从 `%USERPROFILE%\.codex` 和 `%USERPROFILE%\.claude` 读取供应商登录信息，可通过 `CODEX_HOME` 和 `CLAUDE_CONFIG_DIR` 覆盖。若 Codex 本地只提供工作区 ID，可用 `accounts.json` 的 `labels` 显示邮箱。结构见[运维手册](RUNBOOK.md)。

服务器返回时显示 GPT-5.3-Codex-Spark 5H/7D 等模型限额。长名称自动换行，条目较多时用量区可滚动。右侧时间是距重置的剩余时间；悬停仪表可看准确重置日期和时间。不会把缺失模型限额假定为 0%，也不会用其他账户的读数替代。

<!-- qhud:anchor -->
<a id="support-and-verification"></a>

## 支持与验证

保留 Linux GTK 帧恢复、X11 窗口处理及 tmux/herdr 观测。未实现 Windows Terminal、PowerShell 或 WezTerm 标签页的原生观测。没有可观测的多路复用器时，qhud 显示真实本地账户和 `panes = 0`；示例数据必须使用 `--demo`。Windows 上可能没有 Linux `/proc` 进程指标。

CI 在 Ubuntu 24.04 和 Windows 上执行测试及 release 构建，两者成功后才发布。单元测试覆盖 Spark 5H/7D 百分比和重置保留、仅模型快照恢复及并发保存结果。自动构建和测试不能证明所有桌面或供应商账户上的实际行为。
