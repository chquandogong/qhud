<!-- qhud:languages -->
<p align="center">
  <a href="../../../../05-ops/RUNBOOK.md">English</a> · <a href="../../../ko/docs/05-ops/RUNBOOK.md">한국어</a> · <strong>简体中文</strong>
</p>
<!-- /qhud:languages -->

<!-- qhud:anchor -->
<a id="runbook"></a>

# 运维手册

> 状态：持续维护 · 日期：2026-09-07 · 负责人：chquandogong

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
- **怀疑某个数字？** `~/.local/bin/qhud --dump` 输出组件实际渲染的完整载荷，一次观测，格式化 JSON。
- **配额行丢失费用或重置倒计时？** 旁路文件归属会按设计在三个位置静默拒绝。用 `QMONSTER_SIDEFILE_DIAG=1 qhud --dump 2>&1 >/dev/null` 明确是哪一个：no-cwd-match、60 s 同 cwd 歧义守卫，或后代 CLI 不匹配。
- **组件确实在渲染吗？** stderr 报告生成内容：`strip: N sections, M rows, K gauges`、每行实际文本 `labels(...)`、每次真实点击的 `ptr:`/`sel:`/`qsel:`、任何前端异常的 `js-error …`。但事件记录证明的是**逻辑**，不是像素；无错误**不能**证明已绘制。曾有帧呈现冻结，使点击和获取不可见地执行数天，才学到此教训。像素**可以**验证：间隔几秒两次 `xwd -id <qhud window> | md5sum` 必须不同，因为页脚时钟每秒重绘；xwd 也可解码为图片。`framestall:` 记录表示内置看门狗发现冻结帧时钟并正在恢复，抖动 → 重新执行。
- **无需点击的获取入口**：底层组件收不到合成指针输入，见 D-010。`qhud --refresh-all`、`qhud --refresh-claude`、`qhud --fetch-codex` 经单实例通道转发到运行组件，可绑定快捷键。`qhud --claude-usage`、`qhud --codex-usage`、`qhud --agy-usage` 独立运行相同获取，输出 JSON，并像点击一样写入获取存储。`qhud
--codex-appserver` 按需运行令牌过期回退。`QHUD_EXTRA_DIAG=1 qhud --claude-usage` 输出实时响应中不含身份的 `extra_usage`/`spend` 子对象，用于格式漂移诊断。
- **⟳ 显示“usage response did not parse: …”？** 端点结构漂移；自 v0.5.3 起，后面是 serde 字段和类型信息，阅读即可定位字段。先例：2026-09-01、v0.5.3，`extra_usage.used_credits` 从 `4997` 变为 `4997.0`，一个可选字段让整份响应失败两天。整数语义金额字段现接受整数值浮点数；**新**整数类型字段也必须通过同一宽容读取器（`usage_cache.rs` 中 `lenient_i64`/`lenient_u8`，D-019），否则成为下次两天中断。若指出窗口或百分比，漂移发生在实际渲染字段，需要修改代码，而不是宽容处理。
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
| Claude 配额行停在旧 ⟳，时间说明持续增长，顶部 ⟳ 提示错误 | 读取 stderr：`usage response did not parse: <serde message>` 表示端点结构变化，见诊断和 D-019；`Claude token rejected (401)` 表示该配置目录需重新运行 `claude`。过期额外账户不能隐藏默认账户数字，因此只有一行陈旧时，只是该目录有问题 |
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

1. 发布前更新 README、CHANGELOG、平台说明及 `docs/05-ops/releases/v<version>.md`。保持包、Tauri 配置和 Cargo lock 包版本一致。
2. 推送 `codex/` 准备分支，要求 Ubuntu 和 Windows CI 均通过。Linux 使用规范固定 Git 依赖；Windows 运行 `scripts/Build-Windows.ps1 -Test` 并验证清单/lock 已恢复。该 Windows 检出中不要同时运行 Cargo 命令。
3. 将已测试 revision 合入 `main`，不覆盖无关提交。在发布 revision 创建带注释 `v<version>` 标签并推送。
4. Release 工作流测试、构建两平台，打包现有 Linux x86_64 tarball 和 Windows x86_64 ZIP，附 SHA-256 校验和及证明。发布作业仅在两构建成功后运行，使用仓库内发布说明。
5. 确认工作流成功，各平台下载和校验和均已附上。CI 构建通过不能代替实机桌面检查；记录任何未验证的桌面集成。
