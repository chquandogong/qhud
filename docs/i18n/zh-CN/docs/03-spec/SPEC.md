<!-- qhud:languages -->
<p align="center">
  <a href="../../../ko/docs/03-spec/SPEC.md">한국어</a> · <a href="../../../../03-spec/SPEC.md">English</a> · <strong>简体中文</strong>
</p>
<!-- /qhud:languages -->

<!-- qhud:anchor -->
<a id="spec--qhud"></a>

# 规格 — qhud

> 状态：持续维护（根据 v0.6.0 源码从头重写） · 日期：2026-09-07 · 负责人：chquandogong
> qhud 必须做什么。ARCHITECTURE 说明如何构建，DECISION_LOG 说明原因。
> FR-1 … FR-24 保留其他文档已引用的编号；FR-25 … FR-34 补上 v0.5.1 → v0.6.0 已发布但此前未列入规格表的需求。

<!-- qhud:anchor -->
<a id="product-statement"></a>

## 产品定位

无须切换到任何地方，就能回答一个问题：**每个 AI CLI 账户还有多少余量，何时重置？** 小组件显示的其他一切都为此服务。有终端多路复用器时，逐窗格视图回答第二个更具体的问题：每个正在运行的会话此刻在做什么。

这一先后顺序很重要，也发生过变化。v0.1 是兼带配额的窗格监视器；从 v0.5.0 起，操作者明确的目的是一眼看到全部账户和工作区，从而无需再打开供应商网页；v0.6.0 的主打平台甚至完全不观测窗格。

<!-- qhud:anchor -->
<a id="provider-vocabulary"></a>

## 供应商术语

部分供应商有三种名称，以代码为准。其他文档均应遵循此映射表。

| 载荷值 | 分组标题 | 正文名称 | 说明 |
| --- | --- | --- | --- |
| `claude` | CLAUDE | Claude Code | |
| `codex` | CODEX | Codex | 一次登录可拥有多个工作区 |
| `agy` | AGY | Antigravity | |
| `gemini` | GEMINI | Gemini | 既是同级供应商字符串，**也是** agy 读数中的池名称；`gemini-*` 配额桶使用 agy 主仪表，其他池成为限定 scope 的标签 |

<!-- qhud:anchor -->
<a id="functional-requirements"></a>

## 功能需求

<!-- qhud:anchor -->
<a id="the-widget-as-an-object-on-the-desktop"></a>

### 作为桌面对象的小组件

| ID | 需求 | 状态 |
| --- | --- | --- |
| FR-1 | 桌面层无边框、透明、圆角窗口：位于所有窗口下方，在所有工作区固定，不出现在任务栏和分页器 | 已完成，Linux；2026-09-07 通过 `_NET_WM_STATE` 验证 |
| FR-2 | 通过顶部和页脚栏拖动，可跨显示器移动 | 已完成，自行驱动，D-008 |
| FR-3 | 通过 ◢ 控点调整大小，内容自动重排 | 已完成，自行驱动，D-008 |
| FR-4 | 位置和大小在重启后保留 | 已完成，窗口状态插件，约每 30 s 保存检查点 |
| FR-10 | 托盘：显示/隐藏、置于窗口上方、重置位置、退出；没有托盘仍可运行 | 已完成，尽力支持 |
| FR-11 | Ctrl+滚轮缩放 70–160%，持久化，仅指针交互 | 已完成，D-012 |
| FR-12 | 通过托盘复选项和 `qhud --peek` 切换层级；重复启动被吸收 | 已完成，D-012 |
| FR-26 | 小组件检测自己的画面冻结并自动恢复：采样自身像素、隐藏再显示，然后重新执行，无需操作者操作 | 已完成，v0.5.1、D-017；仅 Linux |
| FR-29 | 原生 Windows x64 小组件通过 WebView2 渲染相同账户用量，使用 Windows 路径及隐藏辅助进程 | 已完成，v0.6.0、D-020；明确不包含窗格观测 |

<!-- qhud:anchor -->
<a id="what-it-renders"></a>

### 渲染内容

| ID | 需求 | 状态 |
| --- | --- | --- |
| FR-5 | 每 2 s 轮询 qmonster 流水线，渲染带状态标签和上下文仪表的窗格卡片，以及按供应商分组的账户级配额区 | 已完成，D-011 |
| FR-6 | 点击卡片展开配置标签及冲突横幅，再点击收起；选择状态持久化 | 已完成 |
| FR-7 | 仪表严重程度分级：低于 60 正常，60–74 关注，75–84 警告，85 及以上严重 | 已完成 |
| FR-8 | 重置倒计时和空闲时长标签在轮询间隔内本地更新 | 已完成 |
| FR-14 | 按供应商分组：供应商为分组标题，身份行显示账户和套餐，每个窗口一行仪表 | 已完成，v0.4.0 |
| FR-25 | 配额行可选择并原地展开详情；选择行绝不执行网络操作 | 已完成，v0.5.1 |
| FR-31 | 行超过窗口时用量区可滚动；长模型名称换行而非截断；仪表和展开行均可查看准确重置日期及时间 | 已完成，v0.6.0 |
| FR-9 | 无多路复用器时显示真实本地账户行、带时间的配额和零窗格；示例数据须 `--demo` 且显示 DEMO 标签；每 10 s 重新探测实时来源 | 已完成，v0.6.0 替代演示回退，D-005 |
| FR-30 | 无 mux 启动不虚构用量：缺失读数保持缺失，无保存读数的账户仍显示，以便访问其刷新控件 | 已完成，v0.6.0 |

<!-- qhud:anchor -->
<a id="whose-numbers-these-are"></a>

### 数字属于谁

| ID | 需求 | 状态 |
| --- | --- | --- |
| FR-13 | 每条配额行注明所属账户：邮箱或 id、组织、套餐、团队席位的两层等级，均仅从本地文件读取 | 已完成，D-013 |
| FR-15 | 曾连接但无实时凭据的账户显示为带日期的占位行，折叠在一行后，可忽略；实时凭据绝不隐藏 | 已完成，D-013 |
| FR-22 | 通过注册表配置目录同时显示多个 Claude 账户，每账户一行，各有自己的快照和刷新；局部获取失败保持局部 | 已完成，D-015 |
| FR-27 | 行身份为（账户、组织）：同一登录的团队席位和个人组织显示为两行，配额池独立；刷新结果按配置目录和账户 id 匹配行 | 已完成，v0.5.2、D-018；真实第二组织行仍未现场验证 |
| FR-34 | 保存快照仅用于当前登录账户；先检查归属再比较时间，避免上一登录较新读数掩盖当前登录较旧读数 | 已完成，v0.6.0 |

自 D-015 沿用的已知限制：无法归属窗格的账户，因此窗格提供的仪表始终属于默认账户行。

<!-- qhud:anchor -->
<a id="getting-the-numbers"></a>

### 获取数字

| ID | 需求 | 状态 |
| --- | --- | --- |
| FR-16 | Claude 模型独立用量由显式控件或 `--refresh-claude` 刷新，不由计时器触发，不执行 OAuth refresh grant | 已完成，D-014 |
| FR-17 | Codex 工作区配额须显式请求；描述另一工作区的响应直接丢弃，不错误贴标签 | 已完成，v0.4.0 |
| FR-19 | Claude 用量点数支出在账户行显示，金额为最小货币单位，严重程度取自供应商，无此功能的套餐隐藏 | 已完成，v0.5.0 |
| FR-20 | 显式刷新结果重启后保留，显示真实来源和时间；实时窗格数据始终优先 | 已完成，v0.5.0 |
| FR-21 | 一个控件并发刷新所有供应商，并反映它们状态的并集；保留各供应商控件；`--refresh-all` 可从快捷方式转发 | 已完成，v0.5.0 |
| FR-23 | agy 显式刷新通过 CLI 自身环回 RPC 完成，无令牌、仅本机，从操作系统发现端口；保存最后读数 | 已完成，v0.5.0 |
| FR-24 | 当前 Codex 登录的直接获取失败时，由短生命周期 `codex app-server` 子进程回答，令牌轮换由 CLI 管理，qhud 自身在此路径不使用令牌 | 已完成，v0.5.0、D-016 |
| FR-33 | 同时运行的供应商刷新保留彼此保存结果 | 已完成，v0.6.0 |

<!-- qhud:anchor -->
<a id="being-trustworthy-about-it"></a>

### 保持可信

| ID | 需求 | 状态 |
| --- | --- | --- |
| FR-18 | webview 外可观察错误输出：小组件报告渲染结构、实际文本和标签、每个真实指针事件及前端异常；每条获取路径都有对应命令行入口 | 已完成，v0.4.0，v0.5.1 扩展 |
| FR-28 | 按语义读取整数语义传输数字；一个可选字段不得使整份响应失败；拒绝响应时指出漂移字段 | 已完成，v0.5.3、D-019 |
| FR-32 | 仅模型读数重启后仍保留每个窗口独立的百分比和重置时刻；供应商当前响应缺少的模型绝不合成为 0% | 已完成，v0.6.0 |

<!-- qhud:anchor -->
<a id="non-functional-requirements"></a>

## 非功能需求

**仅观测。** 对 qmonster 自身目录的任何位置零写入，零通知。状态和告警由 TUI 管理（D-004）。

**默认被动；网络仅按请求执行**（D-013、D-014、D-016）。2 s 循环读取本地文件和多路复用器，不请求供应商 API，并选择身份字段，不使用令牌进行观测。其他工作仅为每约 30 s 的窗口几何检查点，以及 Linux 上每约 28 s 的像素采样。所有更进一步的操作均来自操作者显式手势或对应命令行入口，绝不执行 OAuth refresh grant。在获取路径中，qhud 为 Claude 和 Codex 读取 access token；agy 无需凭据，Codex 回退将认证委托给 CLI。

**选择不是获取。** 自 v0.5.1 起，选择、展开及点击行均不执行网络操作。只有刷新控件和转发标志会执行。

**体积。** v0.6.0 Linux 发布二进制为 15.07 MiB，预算为 25 MB（D-014）。修正两项记录：此前所有数字都测自本地构建；v0.5.3 实际为 25.13 MiB，已静默超出预算，因为 release 配置放在非根工作区清单而被 Cargo 忽略。应测量发布产物，而非本地构建。尚未记录 Windows 数字。

**前端。** 静态 HTML、CSS 和 JavaScript。无打包器、无 node_modules；两个平台构建都不需要 Node 或 npm。

**规模范围。** 一台工作站。窗格视图为 1–12 个窗格设计，再多应改变交互模型，不应勉强扩展。账户视图允许配额区超过窗口：v0.6.0 增加滚动，因为三个供应商的八个账户放不下；零窗格现在是正常受支持状态，而非错误。

<!-- qhud:anchor -->
<a id="the-payload-contract"></a>

## 数据载荷契约

一个 Tauri 事件 `qhud://report`，每 2 s 发出。事实来源为 `src-tauri/src/view.rs`；schema 版本为 1，每次扩展都只增加字段。

```jsonc
{
  "schema": 1,
  "source": "live",        // "live" | "local" | "demo"
  "backend": "herdr",      // "herdr" | "tmux" | null
  "generated_at_ms": 1788790282397,
  "poll_secs": 2,

  "quotas": [{             // one row per (provider, account, organization)
    "provider": "claude",  // claude | codex | agy | gemini
    "h5": { "pct": 49, "source": "providerofficial",
            "reset_unix": 1788438599, "of_tokens": null },
    "d7": { "pct": 7, "source": "providercache",
            "reset_unix": 1788976799, "of_tokens": null },
    "from_label": "claude:1:main",   // pane whose reading won; "" if synthesized
    "session": "dogu-3d-studio",
    "account": {
      "display": "chquan@dogu.xyz", // precomputed label → email → id
      "label": null, "email": "…", "account_id": "…",
      "org": "…", "org_type": "claude_team", "org_id": "…",
      "tiers": [{ "kind": "org", "tier": "max_5x" }],
      "plan": "team", "config_dir": null   // null ⇒ the default account
    },
    "origin": "pane",              // pane | cache | fetched
    "cache_fetched_at_ms": 1788790243688,
    "scoped": [{ "kind": "weekly_scoped", "scope": "Fable",
                 "pct": 12, "reset_unix": 1788976799 }],
    "extra": { "enabled": true, "used_minor": 4997, "currency": "USD",
               "exponent": 2, "limit_minor": null, "percent": 0,
               "severity": "normal", "limit_reached": false }
  }],

  "panes": [{
    "pane_id": "wC:p1", "label": "claude:1:main", "session": "…",
    "provider": "claude", "status": "active", "status_label": "active",
    "elapsed_secs": null, "cli_version": null, "update_hint": null,
    "model": "…", "effort": "max", "branch": null, "cwd": "~/qhud",
    "mem": "165 KB", "cost_usd": 0.0, "flags": [],
    "gauges": { "ctx": { … }, "h5": null, "d7": { … } },
    "conflicts": [{ "reason": "…", "severity": "warning",
                    "paths": ["…"], "peers": ["codex:1:review"] }]
  }],

  "summary": { "panes": 8, "conflicts": 0, "max_5h_pct": 6 },

  "account_placeholders": [{ "provider": "claude", "key": "personal-free",
                             "label": "…", "hint": "…", "plan": "free" }],
  "workspace_names":  { "<account_id>": "business" },
  "workspace_plans":  { "<account_id>": "ChatGPT Business" },
  "codex_workspaces": [{ "account_id": "…", "name": null,
                         "plan_type": "prolite", "credits_balance": "0",
                         "active": true,
                         "windows": [{ "label": "weekly", "used_percent": 41,
                                       "reset_unix": 1789147159,
                                       "scope": null }] }],
  "codex_fetched_at_ms": 1788790245717
}
```

通常为空的字段省略而非发送 null：`account`、`origin`、`cache_fetched_at_ms`、`scoped`、`extra`、`account_placeholders`、`workspace_names`、`workspace_plans`、`codex_workspaces`、`codex_fetched_at_ms`。`qhud --dump` 输出完全相同的载荷。

**契约规则。**

- **事实在其真实所属范围渲染**（D-011）。配额属于账户，每账户行只显示一次；上下文和状态属于窗格，显示在卡片上。逐窗格 `gauges.h5`/`d7` 仍保留在载荷中供汇总使用，但卡片不渲染它们。
- **单位只在此边界转换一次。** 压力输出为整数百分比，重置时刻为 Unix 秒；前端负责倒计时文本。webview 不接触 qmonster 类型。
- **窗口标签使用传输值**（`5h`、`daily`、`weekly`、`30d`、`yearly`）；界面各处统一使用持续时间术语：5H、1D、7D、30D，因为“weekly”和“7D”是同一个七日滚动窗口，一个事实不应有两个名称。
- **`scoped[].kind`** 是供应商对非主窗口的称呼：Claude 的 `session`、`weekly_all`、`weekly_scoped`，以及任何供应商额外池的 `pool_<label>`；`scope` 携带模型或池名称。未知 kind 退化为提示中的一行，不消失。
- **`origin` 是诚实字段。** `pane` 为实时读数，`cache` 为 CLI 自有磁盘副本，`fetched` 为 qhud 上次显式刷新。非 `pane` 均必须显示时间；缺少 origin 表示该行不作任何声明。
- **金额采用最小货币单位**，加币种和指数，避免经浮点金额往返转换。

<!-- qhud:anchor -->
<a id="verification"></a>

## 验证

两个平台均有自动化门槛，发布等待两者完成：格式检查、警告视为错误的 lint、完整测试套件、release 构建。v0.6.0 套件共 98 项测试。

交互声明有专门协议（D-010）：仅合成 X11 输入不予采信，因为它绕过合成器的表面选择，而真实输入正是在那里被截走。声明必须通过合成器路径注入或人工操作，并由小组件自身事件记录确认。

渲染声明另有协议（D-017）：事件记录证明逻辑，不证明绘制。像素**可以**验证：间隔几秒对窗口哈希两次，或读取帧守卫自身日志。没有错误不能证明任何东西已被绘制。

手工检查清单、带日期证据及仍待检查的项目见 TEST_PLAN。

<!-- qhud:anchor -->
<a id="out-of-scope"></a>

## 范围之外

- **控制操作。** qhud 不向窗格发送按键，不重启 CLI，不更改供应商设置。
- **通知。** 告警属于 TUI 和供应商，第二个通知器会重复触发（D-004）。
- **设置界面。** 显示名称和注册账户通过一个操作者自有文件手动编辑。
- **macOS。**
- **原生 Windows 终端窗格观测。** Windows 小组件显示账户用量；未实现 Windows Terminal 或 PowerShell 窗格发现及归属（D-020）。Linux `/proc` 衍生进程指标在那里可能为空。
- **安装程序和包格式。** 发布集合为 Linux tarball 和 Windows ZIP：没有 npm、deb、AppImage、MSI；Windows 不自动安装、不注册自启动或快捷方式。
- **逐仪表阈值配置。** 严重程度分级继承设计稿，不可配置。
- **虚构数据。** 不为供应商未返回的池合成配额，不将缺失读数显示为零。
