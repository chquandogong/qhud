<!-- qhud:languages -->
<p align="center">
  <a href="../../../ko/docs/04-quality/TEST_PLAN.md">한국어</a> · <a href="../../../../04-quality/TEST_PLAN.md">English</a> · <strong>简体中文</strong>
</p>
<!-- /qhud:languages -->

<!-- qhud:anchor -->
<a id="test_plan"></a>

# 测试计划

> 状态：v0.6.0 自动化门槛通过，桌面验证缺口已记录 · 日期：2026-09-07 · 负责人：chquandogong

<!-- qhud:anchor -->
<a id="automated-gates"></a>

## 自动化门槛

CI 在推送到 `main`、`codex/**` 及 pull request 时运行。Ubuntu 24.04 使用规范固定 Git 依赖，执行：

```bash
cargo fmt --all --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
cargo build --locked --release
```

Windows MSVC 通过构建脚本使用限定范围的依赖补丁：

```powershell
./scripts/Build-Windows.ps1 -Test -VisualStudioPath '<Visual Studio installation>'
git diff --exit-code -- Cargo.toml Cargo.lock
```

`-Test` 在同一依赖上下文先运行 QHUD 测试，再构建。Windows 命令结束后脚本恢复规范 lockfile；Linux 构建无需同级检出或 Windows 补丁。发布等待 Ubuntu 和 Windows 测试、构建全部成功。

2026-09-07：[CI 34117359064](https://github.com/chquandogong/qhud/actions/runs/34117359064) 在 `1514e09` 上通过：Ubuntu fmt/clippy、**98/98 项测试**和 release 构建；Windows **98/98 项测试**、release 构建及规范依赖文件检查。本地 Windows 脚本也通过 98/98 项测试和 release 构建。

2026-09-07，重新验证公开 v0.6.0 Linux 产物，补上 v0.6.0 所记录的缺口。下载发布 tarball，SHA-256 与配套文件一致，将同一二进制安装至 `~/.local/bin/qhud` 并与产物比较哈希，在参考机器重新运行。环境：Ubuntu 24.04.3、GNOME Shell 46.0 Wayland、内核 7.0.0-28、eDP-1 2560×1600 + HDMI-1 3840×2160，均 scale 1、webkit2gtk 2.52.6、herdr 0.7.5、rustc 1.94.1。同一代码树本地也通过 fmt、clippy `-D warnings`、98/98 测试，并在 6 m 21 s 内从固定 Git 依赖完成 release 构建，无同级检出目录。

<!-- qhud:anchor -->
<a id="unit-coverage"></a>

## 单元覆盖

- `view.rs`：百分比限制和取整、字节可读化、`~` 折叠、标签截断。完整 `PaneReport` 构造为上游私有；映射由下方实机清单覆盖。
- 账户切换后的保存配额归属、无 mux 时本地账户行、重启后仅 scope 和仅额外支出快照。
- Codex 模型池经两个供应商解析器及账户行合并，保留模型名、5H/7D 时长、用量和重置。夹具不证明某个真实账户当前拥有这些池。

<!-- qhud:anchor -->
<a id="manual-verification-checklist--ubuntu-desktop-layer"></a>

## 手工验证清单 — Ubuntu 桌面层

每次窗口层级变更后在目标机器运行：

| 检查 | 命令 / 操作 | 通过标准及证据日期 |
| --- | --- | --- |
| 底层 + 固定 + 跳过 | `xprop -id $(xdotool search --class qhud \| tail -1) _NET_WM_STATE _NET_WM_DESKTOP` | 四项状态齐全；desktop `4294967295`；**2026-09-07 在公开 v0.6.0 产物通过** |
| 保持在窗口下 | 将任意应用拖过组件 | 组件绝不上浮 |
| 工作区固定 | 切换工作区 | 所有工作区可见 |
| 跨显示器移动 | 拖顶部栏到另一显示器 | 重启后位置保留 |
| 调整大小 | 拖动 ◢ 控点 | 内容重排，重启后保留 |
| 本地回退 | 停止活动 tmux/herdr 服务器 | 保留真实账户和带时间配额；无示例窗格或 `DEMO` 标签 |
| 默认真实数据 | `qhud --dump` 和 `qhud --demo --dump` | 无标志时 `source` 为 `live`/`local`，显示真实窗格；有标志时 `source` 为 `demo`，显示三个设计稿窗格；**2026-09-07 通过** |
| 显式演示 | 关闭现有实例后启动 `qhud --demo` | `DEMO` 标签；卡片对应设计稿 |
| 实时恢复 | 启动 tmux 和 AI CLI | ≤12 s 内获得实时数据，10 s 重探测 + 2 s 轮询；2026-08-05 |
| 实时观测 | 读取组件 stderr 启动行 | `live via herdr` 及窗格列表；**2026-09-07 通过**，8 窗格，配额区 3 分组 / 7 行 |
| 各供应商刷新 | `qhud --refresh-all`，再读组件 stderr | 三供应商均无错误回答；**2026-09-07 通过**，claude 5h 3% / 7d 67% / 3 scoped，agy 2 池，codex 1 工作区 |
| 像素持续绘制 | 间隔数秒两次 `xwd -id <window> \| md5sum` | 哈希**不同**，页脚时钟每秒重绘；**2026-09-07 通过**。相同哈希表示帧冻结（D-017） |
| 帧守卫已启动 | 在组件 stderr 搜索 `frame guard armed` | 每次进程启动恰好一行；**2026-09-07 通过**。缺失意味着采样失败，冻结将无法检测 |
| 视觉一致性 | 与 `docs/assets/widget-*.png` 对照 | 配色、卡片、仪表、标签与设计稿一致 |
| GNOME 概览 | 打开 Activities / 工作区手势 | ⏳ 组件可能作为窗口出现，已接受 R2；之后必须回到底层 |
| 锁定 / 解锁 | 锁屏，再解锁 | ⏳ 仍为 below + sticky，用 `xprop` 重查 |
| 挂起 / 恢复 | 挂起，再恢复 | ⏳ 同锁定/解锁 |
| 全屏应用 | 在组件所在显示器全屏一个窗口 | ⏳ 组件绝不透出 |
| 显示器热插拔 | 拔出并重插外接显示器 | ⏳ 可通过托盘 → Reset position 恢复 |

⏳ 项来自 Codex 交叉验证（CV 日志），等待首次实机验证。其中锁定/解锁、挂起/恢复正是触发 D-017 冻结的显示器睡眠条件，该问题由现场发现，而非此清单。在实际执行前，唯一路径证据是帧守卫现场统计：journal 覆盖 2026-08-26 → 09-07，**28 次检测、28 次第一级恢复、0 次重新执行、0 次操作者可见事故**。

2026-08-05（D-008）：自行驱动几何实现后，在两台显示器上重新验证点击派发、拖动移动和控点缩放，均精确到像素；合成输入证据见 DECISION_LOG D-008。

<!-- qhud:anchor -->
<a id="manual-verification-checklist--windows-and-account-usage"></a>

## 手工验证清单 — Windows 和账户用量

- 启动原生 WebView2 应用，移动和调整大小，验证托盘、已有实例的 `--peek` 以及显式刷新控件。
- 无 tmux/herdr 后端时，确认零窗格及真实账户行。重启后刷新结果仍带日期，不替换为模拟用量。
- 缩小窗口或显示多个账户/模型行。所有配额和重置均可滚动访问，窗格和页脚控件仍保留。
- 确认模型名称及重置倒计时可读。服务器缺少的模型限额保持缺失，不填零、不使用其他账户用量。
- 验证后台 CLI 探测不弹控制台窗口。原生 Windows 终端标签页监控不在当前支持范围。

安装后已验证本地 Windows v0.6.0 显示、账户邮箱、模型/重置行、显式 Codex 刷新以及实际 app-server 回退。安装可执行文件 SHA-256：`3D688DB2F90E6D4780C0F016CB0CED1572189AC83FC1658F73A6688D1477F9FF`。它标识本地 MSVC 构建，不是独立的 GitHub 发布产物。

最终标签 `v0.6.0`（`cfdd850`）通过 [main CI](https://github.com/chquandogong/qhud/actions/runs/34127575130) 和 [Release](https://github.com/chquandogong/qhud/actions/runs/34127575141)。两份公开压缩包均已下载并匹配 SHA-256 文件。公开 Windows 可执行文件已安装，SHA-256 为 `1233E4E7D58B8E6A0C855D3B528C7810FFD99DA476509F9C19509A3B5F094917`。安装后的公开进程能响应并发出账户行。因 Windows 会话进入锁屏，最终合成器截图仍待确认；此前本地构建的视觉检查仍是直接像素证据。

<!-- qhud:anchor -->
<a id="ubuntu-input-verification-protocol-mandatory-since-d-010"></a>

## Ubuntu 输入验证协议（自 D-010 起强制）

**交互声明不能只依赖 XTEST（xdotool）**：它在 XWayland 内部注入，绕过 Mutter 表面选择，而真实输入正是在那里被截走（D-010）。任何“交互正常”声明都必须通过**合成器路径注入**：Mutter RemoteDesktop 绝对指针点击，见 D-010 中 `rd_abs_click.py` 技术，或人工操作，并以 `qhud ui:` stderr 事件记录确认。

2026-08-06（D-010、v0.1.4）：DING 关闭时验证合成器路径选择往返（`sel:none:-` → `sel:wC:p3:R`）；DING 开启时相同点击消失，A/B 验证。

<!-- qhud:anchor -->
<a id="non-regression-invariants"></a>

## 防回归不变量

- qhud 不创建或修改 `~/.qmonster` 下的文件（R5）：运行 10 分钟后 `find ~/.qmonster -newer /tmp/mark` 只能列出归于 TUI 的文件。
- 点击组件绝不取得键盘焦点，遵循仅指针交互契约。
