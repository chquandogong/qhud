<!-- qhud:languages -->
<p align="center">
  <a href="../../../ko/docs/05-ops/RUNBOOK.md">한국어</a> · <a href="../../../../05-ops/RUNBOOK.md">English</a> · <strong>简体中文</strong>
</p>
<!-- /qhud:languages -->

<!-- qhud:anchor -->
<a id="runbook"></a>

# 运维手册

> 状态：持续维护 · 日期：2026-09-15 · 负责人：chquandogong

<!-- qhud:anchor -->
<a id="install"></a>

## 安装

**Windows x64：** 使用 ZIP 发布包，或 [WINDOWS](WINDOWS.md) 中的仓库构建脚本。以下 Linux 说明的适用范围不变。

**发布 tarball**（Linux x86_64）：

```bash
gh release download --repo chquandogong/qhud --pattern '*linux-x86_64.tar.gz'
tar -xzf qhud-v*-linux-x86_64.tar.gz && cd qhud-v*/
./qhud &
```

运行时依赖（Ubuntu 24.04 包名）：`libwebkit2gtk-4.1-0`、`libgtk-3-0`、`libayatana-appindicator3-1`，最后一个为可选托盘依赖。

**从源码构建：**

```bash
sudo apt-get install -y libwebkit2gtk-4.1-dev libgtk-3-dev \
  libayatana-appindicator3-dev librsvg2-dev pkg-config
git clone https://github.com/chquandogong/qhud && cd qhud
cargo build --release --locked # binary at target/release/qhud
```

<!-- qhud:anchor -->
<a id="run--quit"></a>

## 运行 / 退出

- 启动：`./qhud &`，组件出现在桌面层。无多路复用器时显示真实本地账户快照和零窗格；`--demo` 显式选择示例数据。
- 移动：拖动顶部栏或页脚；缩放：拖动 ◢ 控点。
- 退出：托盘图标 → _Quit qhud_，或 `pkill qhud`；按设计没有标题栏。
- **字号：** 按住 Ctrl，在组件上滚动鼠标滚轮，范围 70–160%，会记住设置。
- **临时置顶：** 托盘 → _Pin above windows_，再次选择送回底层；或运行 `~/.local/bin/qhud --peek`。键盘快捷方式：GNOME Settings → Keyboard → Custom Shortcuts，命令 `/home/USER/.local/bin/qhud --peek`，例如 Super+Q。不要向 qhud 发送 Unix 信号，WebKitGTK 保留这些信号（D-012）。
- `qhud` 已运行时再次启动，会被现有实例吸收，单实例守卫生效。
- **怀疑某个数字？** `~/.local/bin/qhud --dump` 输出组件实际渲染的完整载荷，一次观测，格式化 JSON。它可能包含账户、工作区、会话、命令和路径标识符；共享前请脱敏。
- **配额行丢失费用或重置倒计时？** 旁路文件归属会按设计在三个位置静默拒绝。用 `QMONSTER_SIDEFILE_DIAG=1 qhud --dump 2>&1 >/dev/null` 明确是哪一个：no-cwd-match、60 s 同 cwd 歧义守卫，或后代 CLI 不匹配。
- **组件确实在渲染吗？** stderr 只记录脱敏后的渲染与交互阶段：配额区/标签完成、指针与选择传递、前端错误是否发生；账户标签、工作区与窗格 ID、路径、用量值及异常原文均不会记录。但事件记录证明的是**逻辑**，不是像素；无错误**不能**证明已绘制。曾有帧呈现冻结，使点击和获取不可见地执行数天，才学到此教训。像素**可以**验证：间隔几秒两次 `xwd -id <qhud window> | md5sum` 必须不同，因为页脚时钟每秒重绘；xwd 也可解码为图片。`framestall:` 记录表示内置看门狗发现冻结帧时钟并正在恢复，抖动 → 重新执行。
- **无需点击的获取入口**：底层组件收不到合成指针输入，见 D-010。`qhud --refresh-all`、`qhud --refresh-claude`、`qhud --fetch-codex` 经单实例通道转发到运行组件，可绑定快捷键。`qhud --claude-usage`、`qhud --codex-usage`、`qhud --agy-usage` 独立运行相同获取，输出 JSON，并像点击一样写入获取存储。`qhud --codex-appserver` 按需运行令牌过期回退。`QHUD_EXTRA_DIAG=1 qhud --claude-usage >/dev/null` 的诊断记录只报告实时响应中是否存在 `extra_usage` 与 `spend`，用作格式漂移线索。
- **⟳ 显示“usage response did not parse: …”？** 解析器仅返回粗略类别（`schema mismatch`、JSON 语法错误、不完整 JSON 或 I/O 错误）和行、列位置；不会显示字段名或供应商值。仅凭位置无法确定是窗口、百分比还是金额字段发生变化。要获取额外用量的线索，可运行 `QHUD_EXTRA_DIAG=1 qhud --claude-usage >/dev/null`：stderr 中的诊断记录只报告 `extra_usage` 和 `spend` 是否存在。供应商响应和本地 Claude 缓存须保持私密。将本地持有的响应与 `src-tauri/src/usage_cache.rs` 中预期的类型比较，在提出解析器修复前用脱敏的合成最小 JSON 复现。2026-09-01、v0.5.3，`extra_usage.used_credits` 从整数变为值为整数的浮点数，是已确认的一例（D-019）；`lenient_i64` / `lenient_u8` 处理整数语义的金额字段。若确认显示的窗口或百分比发生变化，须另行审查数据契约。不要发布原始响应、账户标识符或含令牌的文件。
- **账户和套餐**位于 `~/.config/qhud/accounts.json`，刻意在此公开仓库外。`labels` / `plans` / `workspace_names` / `workspace_plans` 设置显示文字；`known[]` 列出曾连接账户；`forgotten` 隐藏占位行，但不隐藏实时账户。显示名称由操作者提供，不得根据传输 `plan_type`“修正”：`prolite` 显示为 ChatGPT Pro 5x，`team` 为 ChatGPT Business。
- **每供应商多个账户**（D-015）：各额外账户在自己的目录保持登录，然后注册目录：

  ```bash
  CLAUDE_CONFIG_DIR=~/claude-personal claude   # sign in once, keep it
  CODEX_HOME=~/.codex-dogu codex login          # same idea for codex
  ```

  将合法 JSON 保存到 `accounts.json`，不支持注释或尾随逗号：

  ```json
  {
    "claude_config_dirs": ["~/claude-personal"],
    "codex_homes": ["~/.codex-dogu"]
  }
  ```

  每个 Claude 目录渲染自己的行，包含身份、快照和 ⟳；Codex 额外主目录加入逐凭据扫描。行身份是**（账户、组织）**：一个 claude.ai 登录可同时拥有团队席位和个人组织。在浏览器 OAuth 之后的 CLI 组织选择步骤选择所需组织；浏览器会自动延续活动会话，真正选择发生在组织步骤。与默认（账户、组织）匹配的目录跳过，不重复。

- **qhud 自己的 ⟳ 结果**保存在 `~/.config/qhud/fetched-usage.json`，同样遵守仓库外隐私规则，先临时写入再重命名。删除始终安全，下次 ⟳ 会重建。

<!-- qhud:anchor -->
<a id="autostart--app-launcher-gnome"></a>

## 自启动与应用启动器（GNOME）

先将二进制安装到稳定路径；自启动指向 `target/release/` 会在下次 `cargo clean` 后失效：

```bash
install -Dm755 target/release/qhud ~/.local/bin/qhud
install -Dm644 src-tauri/icons/128x128.png \
  ~/.local/share/icons/hicolor/128x128/apps/qhud.png

mkdir -p ~/.config/autostart ~/.local/share/applications
cat > ~/.config/autostart/qhud.desktop <<EOF
[Desktop Entry]
Type=Application
Name=qhud
Comment=Ambient desktop HUD for AI CLI sessions
Exec=$HOME/.local/bin/qhud
Icon=qhud
Terminal=false
Categories=System;Monitor;
StartupNotify=false
StartupWMClass=qhud
X-GNOME-Autostart-enabled=true
X-GNOME-Autostart-Delay=3
EOF
cp ~/.config/autostart/qhud.desktop ~/.local/share/applications/qhud.desktop
```

`applications` 副本也会让 qhud 出现在 GNOME 应用网格中。3 s 自启动延迟让桌面和托盘 AppIndicator 扩展先稳定。重复启动由运行实例吸收，单实例守卫，v0.3.0。

**重新构建新版本后**，更新已安装副本：`install -m755 target/release/qhud ~/.local/bin/qhud && pkill -x qhud && ~/.local/bin/qhud &`

<!-- qhud:anchor -->
<a id="troubleshooting"></a>

## 故障排查

| 症状 | 修复 |
| --- | --- |
| 组件空白、不透明或**像素冻结**：点击有效、事件记录出现，但画面从不变化，常见于整夜 DPMS 后 | 自 v0.5.1 起 qhud 自行关闭两个脆弱 WebKitGTK 路径，`WEBKIT_DISABLE_DMABUF_RENDERER=1` + `WEBKIT_DISABLE_COMPOSITING_MODE=1`；线程化合成器帧时钟在显示器睡眠时死亡，启动 libEGL DRI3 错误是征兆。同时自动恢复：Rust 像素守卫每约 28 s 哈希页脚，两个静态样本记录 `frame freeze detected`，取消映射再映射，已证明能解冻；仍静态则重新执行。若以 `QHUD_KEEP_DMABUF=1` / `QHUD_KEEP_COMPOSITING=1` 覆盖，取消这些变量。冻结检查：间隔几秒两次 `xwd -id $(xdotool search --class qhud \| tail -1) \| md5sum`，哈希相同即冻结，页脚时钟每秒重绘 |
| Claude 配额行停在旧 ⟳，时间说明持续增长，顶部 ⟳ 提示错误 | `usage response did not parse: <脱敏类别和位置>` 表示端点结构变化，见诊断和 D-019；`Claude token rejected (401)` 表示该配置目录需重新运行 `claude`。过期额外账户不能隐藏默认账户数字，因此只有一行陈旧时，只是该目录有问题 |
| 组件跑到窗口上方 | 确认 XWayland：对窗口 `xprop WM_CLASS` 应有结果；若设置 `QHUD_NO_X11_FORCE=1`，层级由你的合成器负责 |
| 拔掉显示器后出现在错误屏幕 | 几何恢复指向已断开显示器，删除 `~/.config/xyz.dogu.qhud/` 下窗口状态文件并重启 |
| 没有托盘图标 | 缺 AppIndicator 扩展；组件仍运行，通过 `pkill qhud` 退出 |
| HiDPI 模糊 | GNOME 46 的分数缩放 + XWayland 使 X11 客户端模糊；采用整数缩放或升级 GNOME 47+ 的 `xwayland-native-scaling` |
| tmux 正在运行却未检测到窗格 | qhud 每 10 s 探测；检查 TUI 同一配置 `~/.qmonster/config/qmonster.toml` 的 `[mux]/[tmux]` 目标。若显式选择了示例数据，重启时去掉 `--demo` |
| 拖动/缩放忽略窗口最外约 10px | 此边缘属于 tao 内置边缘处理器（D-008）；移动抓顶部/页脚内部，缩放抓 ◢ 图标 |
| 组件可见但忽略全部真实鼠标输入，合成/xdotool 有效 | Ubuntu Desktop Icons NG 扩展吞掉桌面层真实指针输入（D-010）：`gnome-extensions disable ding@rastersoft.com`。图标用户的配套扩展共存方案在待办。查看 `qhud ui:` stderr 记录，确认点击是否到达 |

<!-- qhud:anchor -->
<a id="update-the-qmonster-pipeline"></a>

## 更新 qmonster 流水线

升级 `src-tauri/Cargo.toml` 的 `rev`，运行 `cargo build`，修复编译器在 `view.rs`/`poll.rs` 指出的问题，重跑 TEST_PLAN 手工清单。

<!-- qhud:anchor -->
<a id="release-procedure"></a>

## 发布流程

1. 更新 README、CHANGELOG、平台说明，以及英文、韩文和中文 `docs/05-ops/releases/v<version>.md` 发布说明。保持 Cargo 包、Tauri 配置和 Cargo lock 中的包版本一致。
2. 推送 `codex/` 准备分支并创建 PR。按严格检查策略更新分支以包含 `main`，解决评审对话，并等待全部七项必需状态检查（Ubuntu、Windows、文档/元数据、依赖审查及三项 CodeQL Analyze）通过。当前 `main` 规则要求 PR、线性历史和 squash merge；单人维护时所需的批准评审数为零。Windows CI 运行 `scripts/Build-Windows.ps1 -Test` 并核实清单/lock 已恢复；不要在该检出中同时运行 Cargo。
3. 将通过检查的 PR 以 squash merge 合入 `main`。在已包含于 `origin/main` 的提交上创建并推送带注释的稳定版本 `vX.Y.Z` 标签；受保护的 `v*` 标签不可修改或删除。发布预检还要求 Cargo/Tauri 版本一致、发布说明和 CHANGELOG 条目齐备。
4. 标签触发的 Release 工作流运行完整 CI 质量关卡，然后把通过质量关卡的 Linux x86_64 与 Windows x86_64 二进制文件打包为 tarball 和 ZIP，并生成 SHA-256 文件与构建来源证明。它核验恰好四项资产及两份校验和，随后通过 `release` 环境审核来控制发布。截至 2026-09-14 最近一次核查，该关卡要求维护者审核并允许自行批准。批准后，发布作业**先创建或继续处理草稿**，逐字节比对已有草稿资产，上传缺少的资产，仅发布完整草稿；拒绝替换已发布的版本。
5. 确认工作流成功，检查发布说明和四项可下载资产，验证校验和及构建来源证明，并记录尚未在受支持平台实机上验证的桌面集成。
