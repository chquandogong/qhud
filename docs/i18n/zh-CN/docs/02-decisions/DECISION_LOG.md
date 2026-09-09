<!-- qhud:languages -->
<p align="center">
  <a href="../../../ko/docs/02-decisions/DECISION_LOG.md">한국어</a> · <a href="../../../../02-decisions/DECISION_LOG.md">English</a> · <strong>简体中文</strong>
</p>
<!-- /qhud:languages -->

<!-- qhud:anchor -->
<a id="decision_log"></a>

# 决策日志

> 状态：持续维护 · 日期：2026-09-07 · 负责人：chquandogong

格式：背景 → 方案 → 决策 → 理由 → 剩余风险。

---

<!-- qhud:anchor -->
<a id="d-001--build-a-second-frontend-not-a-tui-feature"></a>

## D-001 · 构建第二前端，而非 TUI 功能

- **背景**：常驻可见性的缺口（OFFICE_HOURS）也可以通过保持终端始终可见来弥补。
- **方案**：(a) 专用常显终端运行 TUI；(b) 新建独立小组件应用；(c) 将 qmonster 分叉为 GUI。
- **决策**：(b)，独立 `qhud`，将 qmonster 链接为库。
- **理由**：(a) 占用终端和 alt-tab 位置，无法匹配设计稿；(c) 分裂维护负担。(b) 复用 100% 数据流水线，不修改 qmonster。
- **剩余风险**：两个前端共享隐式契约，见 D-004。

<!-- qhud:anchor -->
<a id="d-002--tauri-v2-over-electron--gnome-extension--native-gtk"></a>

## D-002 · 选择 Tauri v2，而非 Electron / GNOME 扩展 / 原生 GTK

- **背景**：约 15 MB 的常驻小组件需要与纯 HTML/CSS 设计稿精确保持视觉一致。
- **方案**：比较表见 `ALTERNATIVES.md`（A–E）。
- **决策**：Tauri v2，无边框透明窗口，静态前端，`withGlobalTauri`，无打包器。
- **理由**：HTML 设计稿几乎可原样迁移；Rust 后端直接链接 qmonster crate，无需 IPC 层；约 15 MB 二进制和较低 RSS 比 Electron 更适合常驻组件；决定前已实机验证 keep-below。
- **剩余风险**：WebKitGTK 的透明和 dmabuf 渲染问题。缓解：运维手册中的环境变量回退；Electron 仍是已记录备选方案。

<!-- qhud:anchor -->
<a id="d-003--force-gdk_backendx11-xwayland-on-gnome"></a>

## D-003 · 在 GNOME 强制 `GDK_BACKEND=x11`（XWayland）

- **背景**：Mutter 无 layer-shell；Wayland 顶层窗口不能全局定位或保持底层，tauri#14913 操作不生效。
- **方案**：(a) XWayland + EWMH below/sticky；(b) GNOME Shell 扩展持有该层；(c) 原生 Wayland，接受普通窗口；(d) wlr-layer-shell，仅非 GNOME。
- **决策**：(a)，以 `QHUD_NO_X11_FORCE=1` 为退出开关，并尊重已设置的 `GDK_BACKEND`。
- **理由**：当前原生 GNOME 只有 (a) 同时提供 below、sticky 和全局定位；目标机器已验证两次。
- **剩余风险**：以后启用分数缩放可能导致 XWayland 模糊（A3）；组件出现在 GNOME 概览中，是已接受问题，只能通过 (b) 修复，保留为路线图选项。

<!-- qhud:anchor -->
<a id="d-004--link-qmonster-at-a-pinned-rev-observe-only-noopsink"></a>

## D-004 · 固定 revision 链接 qmonster，仅观测（`NoopSink`）

- **背景**：TUI 在 `~/.qmonster` 写入 sqlite、审计和归档；第二写入者会发生竞争。上游库 API 没有稳定性承诺。
- **方案**：(a) 子进程 `qmonster --once` 加解析；(b) 共享 sqlite 读取；(c) 直接链接库并使用不写入接收器；(d) 先请求上游提供 JSON 导出契约。
- **决策**：目前采用 (c)，`Context::new(..., Box::new(NoopSink))` + `SilentNotify`，通过 `rev = "aa2bd39…"` 固定依赖；稳定后升级为 (d)。
- **理由**：(a) 当前无机器可读输出；(b) 在读取时与私有 schema 耦合；(c) 零竞争复用解析器。从 `~/.qmonster/config/qmonster.toml` **只读**共享配置。
- **剩余风险**：revision 升级可能破坏编译，这正是固定版本的目的：明确报错，不静默失败。路线图：两个前端共享上游 `--emit-json` / 带版本导出。

<!-- qhud:anchor -->
<a id="d-005--demo-payload-mirrors-the-mockup-exactly"></a>

## D-005 · 演示载荷精确对应设计稿

- **背景**：无 tmux 服务器意味着无内容可渲染；视觉一致性也需要夹具。
- **决策**：`demo.rs` 逐窗格复现设计稿，UI 标注 `DEMO`，同时用于一致性检查。

<!-- qhud:anchor -->
<a id="d-006--release--plain-binary-tarball-via-tag-driven-ci"></a>

## D-006 · 发布采用标签驱动 CI 的普通二进制 tarball

- **背景**：qmonster 提供 npm 和二进制；qhud v0.1 需要最小但可信的发布方式。
- **决策**：推送标签时 GitHub Actions 构建，上传 `qhud-vX.Y.Z-linux-x86_64.tar.gz`、sha256 和构建来源证明。暂不提供 npm/deb/AppImage。
- **剩余风险**：用户须安装 webkit2gtk runtime，已在 RUNBOOK 说明。

<!-- qhud:anchor -->
<a id="d-007--mux-backend-follows-qmonsters-factory-widget-auto-probes-herdr--tmux"></a>

## D-007 · Mux 后端遵循 qmonster 工厂；widget-auto 依次探测 herdr → tmux

- **背景**：v0.1.0 硬编码 tmux `PollingSource`，因此 herdr 环境（本机主要 mux）永远无法进入实时状态。qmonster 的 `[mux] backend = "auto"` 通过 `HERDR_ENV`/`HERDR_SOCKET_PATH` 解析，但这些变量只存在于 herdr 窗格**内部**，桌面组件通常不在其中。
- **方案**：(a) 要求用户显式设置 `backend = "herdr"`；(b) qhud 重新实现后端检测；(c) 复用 `app::tmux_source::build_tmux_source`，配置为 `auto` 且未继承 herdr 环境时，先探测 herdr 再探测 tmux。
- **决策**：(c)。显式 `tmux`/`herdr` 配置原样传递；页脚后端标签来自**解析完成**的来源，不来自配置。
- **理由**：(a) 破坏共享配置承诺：两个前端中同一文件含义应相同；(b) 重复上游逻辑并会漂移。探测最坏只多一次失败 CLI 调用。
- **证据**：2026-08-05 对运行中 herdr 0.7.5 服务器和真实智能体工作区验证了实时转换，日志为 `qhud: live via herdr (4 panes: …)`；保留 tmux 回退。
- **剩余风险**：herdr 和 tmux 同时运行时，widget-auto 优先 herdr，已说明；可显式设置 `[mux] backend` 覆盖。

<!-- qhud:anchor -->
<a id="d-008--all-window-geometry-interaction-is-self-driven-no-compositor-interactive-ops"></a>

## D-008 · 所有窗口几何交互自行驱动，不使用合成器交互操作

- **背景**：用户报告运行中组件的选择、移动、缩放全部失效。2026-08-05 在机器上通过 setTitle 信标和合成输入诊断。
- **发现**（每项均已验证，单独即可导致失败）：
  1. wry 拖动区域 / `startDragging` / `startResizeDragging`，即合成器交互移动和缩放，对 GNOME 上底层 XWayland 窗口不可靠：不生效或只部分生效。
  2. tao 不可见无边框缩放内边距吞掉窗口边缘约 10px 内的全部指针输入：实测 ≤8px 被吞、≥12px 可收到；缩放控点完全处于其中。
  3. 窗口自身移动时 WebKitGTK `event.screenX/Y` 陈旧，造成固定滞后的位移差。
  4. tao `outerPosition()`/`outerSize()` 为该无装饰窗口报告虚构约 37px 边框，y 少 37、高度多 37。
- **决策**：完全自行驱动几何。指针事件只启停基于 Tauri 全局 `cursorPosition()` 的 rAF 循环；直接应用 setPosition/setSize；抓取尺寸来自 DOM（`clientX/Y`、`innerWidth/Height`），绝不用 tao 外部尺寸。扩大控点命中区，使其深入 tao 内边距以内。权限精简为 cursor-position、set-position、set-size。
- **证据**：合成拖动在两台显示器均精确到像素：移动 Δ=(70,55)/(70,55)，缩放 Δ=(52,36)/(52,36)，缩小 Δ=(-45,-36)；两边 DOM 点击派发均由信标确认。
- **同次调查发现并额外修复**：`generate_context!` 不会把 `../ui` 注册到 Cargo 变更跟踪，造成旧资源构建，现由 build.rs 发出 rerun-if-changed；窗口状态只在正常退出保存，现由轮询每 30 s 建立检查点。
- **剩余风险**：拖动时 rAF 每帧一次 IPC 往返，成本可忽略；Tauri 中 `document.title` 不是 WM_NAME，今后调试须用 `setTitle`。

<!-- qhud:anchor -->
<a id="d-009--tile-selection-binds-to-pointerdown-not-click"></a>

## D-009 · 卡片选择绑定 pointerdown，不绑定 click

- **背景**：D-008 后移动和缩放可用，但选择仍失败，用户报告于 2026-08-06。
- **发现**：该 WebKitGTK/X11 webview 可靠派发 `pointerdown`（信标证明），却**从不根据 down/up 对合成 `click`**，所以选择处理器从未执行。信标构建验证：绑定 pointerdown 的选择可切换并渲染（`sel:wC:p1:R → sel:none:- → sel:wD:p1:R`）。
- **排除嫌疑**：曾怀疑 localStorage，因为其后端目录不存在；实际写入正常，目录缺失只是 click 处理器从未执行。不过持久化仍用 try/catch 包裹，并排在 `render()` 之后，防止存储阻挡 UI。
- **决策**：组件所有交互绑定指针事件（`pointerdown`/`pointerup`），绝不依赖合成 `click`。
- **剩余风险**：用户想拖动时也会触发 pointerdown 选择；目前卡片不是拖动面，仅顶部和页脚可拖，因此无冲突。若以后允许拖动卡片，再评估。

<!-- qhud:anchor -->
<a id="d-010--ubuntu-ding-intercepts-real-pointer-input-verification-must-use-the-compositor-path"></a>

## D-010 · Ubuntu DING 截获真实指针输入；验证必须走合成器路径

- **背景**：D-009 后选择通过合成 XTEST 测试，但真实鼠标仍失败，报告于 2026-08-06。
- **承认方法失误**：xdotool/XTEST 在 XWayland 内部注入事件，**绕过 Mutter 表面选择**，而真实输入在该层路由。之前所有“已验证”都共有此盲点。
- **新工具**：通过 `org.gnome.Mutter.RemoteDesktop` 注入合成器路径，配合 ScreenCast 流取得绝对坐标，事件像硬件一样进入 Mutter（`scratchpad rd_abs_click.py`，保持一个持久 D-Bus 连接，因为会话随创建者连接结束）。
- **发现，A/B 已证明**：Ubuntu **Desktop Icons NG（DING）**扩展窗口吞掉组件上方全部真实指针输入。关闭 DING，合成器路径点击可切换选择；开启 DING，相同点击消失。组件区域上 XWayland 指针视图冻结也独立证实选中了 Wayland 表面，而非 qhud。
- **处理**：参考机器关闭 DING（`gnome-extensions disable ding@rastersoft.com`），因为 `~/Desktop` 为空，本就未渲染图标。操作者可重新开启，但会失去组件交互，见 RUNBOOK。图标用户的共存方案是配套 GNOME Shell 扩展，已列待办。
- **代码变化**：永久 stderr 事件记录（`ui_event` 命令、`qhud ui: sel:…` 行），无需污染 WM_NAME 即可从日志验证真实输入。
- **最终证据**：v0.1.4、合成器路径、DING 关闭时，stderr 显示 `sel:none:-` → `sel:wC:p3:R` 的切换往返。

<!-- qhud:anchor -->
<a id="d-011--scope-correct-display-quota-is-an-account-fact-shown-once-per-provider"></a>

## D-011 · 按正确范围显示：配额属于账户，每供应商显示一次

- **背景**：操作者于 2026-08-06 指出两个 `claude:1:main` 卡片显示**不同** 5H/7D。指正成立：5h/7d 窗口属于**账户**，v0.1 却把每个窗格的旁路快照当作窗格级配额。空闲会话持有旧快照，同一账户数字分歧，误导用户，重复标签也无法区分。最初设计稿本身就有此语义错误，忠实移植保留了它。
- **决策**：在事实真实所属范围显示。
  1. **供应商配额区**：顶部栏下每供应商一行，5H/7D 仪表来自最新快照。窗口内用量只增长，因此**该供应商各窗格中最大百分比就是最新读数**，每个快照为下界。提示注明来源窗格。
  2. **卡片只显示窗格事实**：状态标签和 CTX，以及展开配置/冲突。移除逐窗格配额行。
  3. **工作区标签**（`@workspace`）区分跨工作区同名标签。
- **载荷**：schema v1 仅新增 `quotas[]`（provider、h5、d7、from_label、session）、`panes[].session`。
- **已知限制**：假设机器上每供应商一个账户；多账户需要供应商接口提供身份，当时未暴露。汇总有单元测试 `provider_quotas_takes_max_snapshot_per_window`。
- **证据**：新布局重新验证实机合成器路径选择（`sel:wC:p1:R`）；重新生成 README 截图。

<!-- qhud:anchor -->
<a id="d-012--font-zoom-via-ctrlwheel-layer-peek-via-single-instance-argv-signals-are-forbidden"></a>

## D-012 · Ctrl+滚轮调整字体；单实例 argv 切换层级，禁止信号

- **背景**：操作者要求可调整字号，并能看到按设计位于最底层的组件。
- **字号**：组件上 Ctrl+滚轮驱动 webview 页面缩放，70–160%、步长 10%，保存到 localStorage；仅指针交互，保持无键盘焦点契约。
- **临时置顶**：托盘复选项“Pin above windows”加第二进程 `qhud --peek`，经 tauri-plugin-single-instance 转发到运行实例。GNOME 自定义快捷键可绑定 `~/.local/bin/qhud --peek`。置顶时页脚显示 `pinned ·`，切回重新设置 below+sticky。
- **教训，三次段错误验证**：**Tauri/WebKitGTK 进程绝不能安装 Unix 信号处理器。** 最初方案使用 SIGUSR1 + signal-hook，第一个信号到来时，处理线程尚未记录日志，进程就因 SIGSEGV 死亡。WebKitGTK 的 JavaScriptCore 保留 SIGUSR1 用于线程挂起，挂钩会破坏 VM 线程控制。Wayland 上的 XWayland 客户端也不能注册应用全局热键，因此采用单实例 argv 转发：无崩溃，并吸收误双开，完成单实例待办。
- **证据**：`--peek` 往返验证：BELOW → ABOVE（`layer:pinned`）→ BELOW+STICKY（`layer:below`），重复启动被吸收，保持 1 个进程，无崩溃。

<!-- qhud:anchor -->
<a id="d-013--quota-rows-carry-account-identity-identity-reads-stay-local"></a>

## D-013 · 配额行携带账户身份；身份读取保持本地

- **背景**：操作者在一台机器持有多个供应商登录：agy 两个 Google 账户、通过交换 `auth.json` 的两个 Codex 凭据、Claude 团队席位，并问“这是谁的配额？”D-011 的已知限制称多账户“需要供应商接口身份，目前未暴露”。该说法**错误**：每个 CLI 都在凭据旁以明文持久化登录身份。
- **决策**：仅从本地文件读取身份：`~/.claude.json:oauthAccount`、`~/.codex/auth.json:tokens.account_id`、`~/.gemini/google_accounts.json:active`。无网络、不使用令牌，也不打开凭据字段本身。缺失或格式错误降级为“无标签”，不使整轮失败。
- **两层而非一层**：Claude 团队席位既有组织池，也有成员自身席位，各自有独立限速等级（`organizationRateLimitTier` / `userRateLimitTier`）。因此 `tiers` 是列表，合并会隐藏配额池。
- **显示名称**：可选操作者清单 `~/.config/qhud/accounts.json`，键为 `<provider>:<account_id-or-email>`。刻意放在仓库**外**，因为 qhud 是公开仓库，这些键是个人标识。无映射时回退到邮箱，再到账户 id。
- **载荷**：schema v1 新增 `quotas[].account`，未知时省略。
- **已知限制**：仅标记**当前**账户。同时显示每个账户剩余配额需要逐账户获取；停放凭据需要执行 refresh grant，已延后，并要求操作者明确批准，因为 Codex refresh token 单次使用并轮换，写回失败会破坏 `codex login`。Codex 不提供明文邮箱，因此清单键为 UUID。
- **证据**：`cargo test` 17 项通过；实机 `--dump` 显示 `claude → dogu/team <chquan@dogu.xyz> DOGU (claude_team)` 及两层等级，还有 `codex → 3f13fa37…`。

<!-- qhud:anchor -->
<a id="d-014--binary-budget-20-mb--25-mb-network-is-opt-in-not-ambient"></a>

## D-014 · 二进制预算 20 MB → 25 MB；网络为主动选择

- **背景**：Codex 工作区用量需要 HTTP 客户端。`reqwest` + `rustls` 将 release 二进制从 17.9 MB 增至 22.8 MB，超过 `SPEC.md` 的 20 MB 非功能预算。
- **决策**：预算提高到 **25 MB**。20 MB 是 v0.1 规格初始估计，不是测量约束，没有打包、下载或内存限制依赖它；`strip` + `lto` 当时已开启，因此 2.8 MB 是功能真实成本而非松散空间。否决 `native-tls`（以 OpenSSL 链接依赖换体积，不利 tarball）及删除功能（操作者明确要求）。
- **更重要的另一半**：“无网络”改为“**默认被动，网络仅按请求执行**”。2 s 轮询仍不打开 socket、不接触凭据；唯一出站调用由点击触发，绝不执行 OAuth refresh grant。这使组件稳态与完全没有网络代码时一样安全。
- **证据**：测得 22.8 MB；同次修改更新 `SPEC.md` 非功能部分。

<!-- qhud:anchor -->
<a id="d-015--multi-account--per-account-cli-config-dirs-not-qhud-owned-logins"></a>

## D-015 · 多账户采用各账户 CLI 配置目录，不由 qhud 持有登录

- **背景**：操作者于 2026-08-10 明确目的：一眼查看各供应商多个账户的用量和重置时间，显式刷新，从而不再打开供应商网页。v0.4.0 的“每供应商一个实时登录”只是读取默认凭据路径的事实，不是机器限制：Claude Code 每个 `CLAUDE_CONFIG_DIR` 各保存完整身份、缓存和凭据，Codex 每个 `CODEX_HOME` 也是如此。
- **决策**：注册表 `~/.config/qhud/accounts.json` 新增 `claude_config_dirs` 和 `codex_homes`。每个 Claude 目录生成独立配额行：身份来自该目录 `.claude.json`，数字取其缓存与 qhud 上次 ⟳ 较新者。一次 ⟳ 遍历全部账户，各写自己的存储键。Codex 额外主目录加入现有逐凭据扫描。登录和令牌轮换仍归 CLI；qhud 从不执行 refresh grant。
- **否决**：qhud 持有各账户 OAuth 登录（设备流程）。虽能完全独立于 CLI，却将凭据保管、轮换失败造成“登录损坏”及策略风险移入 HUD。只有目录方案实际不足时再评估。
- **已知限制**：无法归属窗格账户，因此窗格仪表落在默认账户行。agy 多账户需要 OS 钥匙串逆向，未尝试。
- **证据**：带第二配置目录的实机 `--dump` 显示两个 Claude 行，各有等级、来源和时间；无凭据目录导致 `fetch_all` 记录局部错误，默认账户仍正常获取。

<!-- qhud:anchor -->
<a id="d-016--delegated-fetch-paths-the-providers-own-process-may-do-the-talking"></a>

## D-016 · 委托获取：可由供应商自身进程回答

- **背景**：原始路径无法解决两种故障：Codex access token 过期返回 401，qhud 又不能执行单次轮换 refresh grant；agy 没有可用保存凭据调用的 HTTP 用量端点，实时令牌在 OS 钥匙串。
- **决策**：在 D-014“网络仅按请求执行”上新增第三类路径，询问供应商**自身**进程：当前登录回退使用 `codex -s read-only -a untrusted app-server`，stdio 上 JSON-RPC，**v0.6.0 修订：批准选项改为受支持的 `-a on-request`，沙箱仍为 `read-only`**，调用 `account/rateLimits/read`；agy 使用环回 Connect RPC `RetrieveUserQuotaSummary`，无令牌、本机、通过 /proc 发现端口。两者的凭据保管和轮换完全归 CLI，qhud 不读取令牌。
- **顺序**：Codex 先原始 HTTP，速度快，仅当前登录失败时才 app-server；agy 无原始路径，环回为主。两者都仅点击触发，2 s 轮询不变。
- **证据**：`--codex-appserver` 返回当前工作区、套餐、点数和窗口；`--agy-usage` 实机发现端口并解析全部四个池。

<!-- qhud:anchor -->
<a id="d-017--the-widget-audits-its-own-pixels-frame-guard"></a>

## D-017 · 组件审计自身像素：帧守卫

- **背景**：连续三天收到“选择无效”报告，但日志显示每次点击都正常。屏幕冻结在数小时前的帧，JS、输入和 IPC 仍运行：显示器睡眠时 Mutter 停止为底层窗口调度帧，GTK 帧时钟不再恢复。所有机制级修复都实机失败：关闭 DMABUF 当天复发；关闭合成加 JS rAF 看门狗两小时内复发，且看门狗失明，因为软件模式下 rAF 仍触发，与屏幕脱钩。外部 1 px 缩放被此窗口忽略（D-008），抖动恢复无效；取消映射再映射已实机证明可恢复绘制。
- **决策**：不再押注机制，直接测量症状。Rust 守卫约每 28 s 对组件页脚像素条哈希，那里时钟每秒重绘。两个相同样本 ⇒ 冻结 ⇒ 隐藏再显示并重新设置层级；下个样本仍静态 ⇒ 以 `--respawned` 重新执行，子进程等待旧进程退出，避免被单实例守卫吸收。启动时一行“frame guard armed”证明采样器有效；每次检测与恢复均记录。
- **否决**：仅环境变量缓解，虽保留为低成本保障——`WEBKIT_DISABLE_DMABUF_RENDERER`、`WEBKIT_DISABLE_COMPOSITING_MODE`，退出开关 `QHUD_KEEP_*`——但单独使用已证明不足；页内 rAF 看门狗天然失明；缩放抖动对此窗口无效。
- **推论，已写入 RUNBOOK**：事件记录证明逻辑，不证明绘制。像素可验证：两次 `xwd | md5sum` 或守卫自身日志。D-010“无错误不证明已绘制”如今有执行机制。
- **证据**：恢复阶梯有单元测试；部署出现“armed”行；2026-08-13→14 实际三次冻结、三次不足一分钟的重新映射恢复、零重新执行、零操作者可见事故。
- **现场统计，替代上述数字**：journal 覆盖 2026-08-26 → 09-07，**28 次冻结、28 次第一级恢复、0 次重新执行、0 次操作者可见事故**。冻结常在显示器睡眠后相隔数分钟成簇发生，守卫运行频率远超最初几天预期。两项结论：重启级仍从未在现场触发；早期数字来自附着终端实例，stderr 未进入 journal，因此无法重算。自 v0.6.0 起仅 Linux（D-020）；Windows 编译排除守卫，因为没有对应故障模式。

<!-- qhud:anchor -->
<a id="d-018--a-quota-rows-identity-is-account-organization"></a>

## D-018 · 配额行身份为（账户、组织）

> 随 v0.5.2 发布（2026-08-14/17，`dab68af`）；2026-09-04 补记，文档审计发现决策已写入 CHANGELOG、DASHBOARD、RUNBOOK，却没进入本日志。

- **背景**：接入操作者所谓“第二账户”后发现并无第二账户。同一个 claude.ai 登录，同邮箱、同 `accountUuid`，属于两个组织：团队席位和个人免费组织，各有配额池。每个配置目录的 CLI 登录限定到一个组织，浏览器 OAuth 后在 CLI 组织步骤选择，不由浏览器选择。D-015 只按账户 id 去重，第二组织被当作“默认账户重复”丢弃。
- **决策**：身份为一对值。`AccountLabel` 除账户 id 外还带 `org_id`（`organizationUuid`）并公开 `config_dir`；按（账户、组织）去重；前端相同键控配额行，**按配置目录**匹配每行的 ⟳ 结果。只按账户 id 匹配会把一个组织数字给同登录两行，正是这对值要防止的故障。
- **否决**：只按配置目录键控，因为两个目录持有同一（账户、组织）确实应是一行，重新登录会产生这种情况；按邮箱键控，因为邮箱不是配额范围。
- **已知限制**：真实第二组织行的现场验证仍等待 CLI 组织步骤中选择 PERSONAL 的登录；OAuth 总自动继续使用浏览器活动团队会话。
- **证据**：额外目录重新登录团队不会增加行，同账户同组织正确去重；额外目录自己的 `.claude.json` 无网络提供身份。

<!-- qhud:anchor -->
<a id="d-019--wire-numbers-are-read-for-their-meaning-a-rejected-body-must-name-its-field"></a>

## D-019 · 按语义读取传输数字；拒绝响应必须指出字段

- **背景**：2026-09-01，`/api/oauth/usage` 开始把 `extra_usage.used_credits` 从 `4997` 序列化为 `4997.0`。serde 的 `i64` 拒绝浮点数，一个可选回退字段使包括 5h/7d 的整个响应失败，Claude 每次 ⟳ 连续失效两天。错误只有“usage response did not parse”，无法区分死令牌或空响应。同周供应商客户端也为此端点加入空响应和无字段响应的带内处理，因此数字格式不能当作稳定契约。
- **决策**：两条规则。(1) 整数语义金额字段（`used_credits`、`decimal_places`、`amount_minor`、`exponent`）使用宽容读取器：整数值浮点数**就是**该整数；带小数浮点数因单位歧义丢弃，遵循禁止猜测单位原则，$50 绝不能显示成 $0.50；两种情况都不能使外层响应失败。可选字段必须从头到尾保持可选，任何单字段都不能让操作者失去窗口数据。(2) 解析拒绝携带 serde 字段、类型和位置信息。该信息设计上不含身份：未知字段无类型跳过，所有类型化字段都是数字、布尔或窗口/套餐字符串，因此仍遵循 D-013 禁止记录响应体。
- **否决**：逐字段 `#[serde(untagged)]` 枚举会静默兜底并吞掉字符串；整份解析为 `serde_json::Value` 后手工遍历会失去曾发现 Codex 漂移的类型契约；将带小数浮点数四舍五入为最小单位属于 v0.5.0 起禁止的单位猜测；把响应体放入错误会泄露身份。
- **证据**：三项测试：09-03 实机响应原文、整数与小数浮点对照、断言错误指出不匹配且不回显响应体。修复后实机组件 ⟳ 记录 `claude usage ok [default] (5h 49%, 7d 7%, 3 scoped)`。

<!-- qhud:anchor -->
<a id="d-020--windows-adaptation-stays-scoped-usage-keeps-its-account-and-window"></a>

## D-020 · Windows 适配保持限定范围；用量保留账户及窗口

- **背景**：增加 Windows 组件不能让普通 Linux 克隆依赖开发者同级目录。仅模型配额响应也必须在序列化和显示后保留，不虚构缺失限额。
- **决策**：Linux 保留固定 Git 依赖、GTK 帧守卫和 X11/tmux/herdr 路径。Windows 构建脚本通过命令级 Cargo 配置应用仓库内依赖补丁，之后恢复规范 lockfile 原始字节。CI 测试并构建两个平台，发布等待两者。共享配额修复保留账户身份、模型 scope 和每个重置时刻，容忍可为 null 的限额映射，不合成服务器未返回的模型。
- **共同行为变化**：无 mux 启动显示真实本地账户和零窗格；演示需显式启用。遵循主目录/配置环境覆盖；没有匹配账户身份的快照须刷新。
- **边界**：未实现原生 Windows 终端窗格归属。Windows 支持不替代 Linux 桌面集成。
