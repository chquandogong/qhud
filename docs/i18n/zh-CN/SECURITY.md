<!-- qhud:languages -->
<p align="center">
  <a href="../ko/SECURITY.md">한국어</a> · <a href="../../../SECURITY.md">English</a> · <strong>简体中文</strong>
</p>
<!-- /qhud:languages -->

<a id="security-policy"></a>

# 安全政策

<a id="supported-versions"></a>

## 支持的版本

安全修复在 `main` 上开发，并发布到最新的正式版本。条件允许时，请在最新版本或当前 `main` 分支上复现疑似漏洞。除非维护者另行说明，更早的版本不受支持。

<a id="report-a-vulnerability-privately"></a>

## 私下报告漏洞

请勿通过公开 issue、discussion 或 pull request 报告疑似漏洞。请使用 GitHub 的[私密漏洞报告表单](https://github.com/chquandogong/qhud/security/advisories/new)。如果该表单不可用，请勿公开细节，并等待私密报告渠道恢复。

只需提供调查所需的信息：

- 受影响的版本或 commit、操作系统及安装方式；
- 影响及触发条件；
- 使用合成数据的最小复现步骤或概念验证；
- 已测试的缓解措施或修复方案。

切勿提交有效凭据、`auth.json`、`.credentials.json`、服务商配置目录、未经脱敏的诊断包或他人的数据。发送报告前，请撤销任何已经泄露的凭据。

维护者会尽力在三个工作日内确认收到报告，并在七个工作日内给出初步评估。这些是尽力而为的目标，并非服务等级保证。请在公开披露前留出修复和协调发布安全公告的时间。

<a id="scope-and-safe-research"></a>

## 范围与安全研究

安全敏感区域包括本地凭据发现、账户和快照归属、服务商请求、WebView 渲染内容、文件系统路径，以及构建和发布流水线。上游 AI 服务商的漏洞应报告给该服务商；只有当 qhud 引入或扩大风险时，才同时在此报告。

请使用您自己的账户和设备、合成或已撤销的凭据，并只进行证明影响所需的最小测试。请勿访问他人数据、干扰服务商、在设备上维持持久访问或使用社会工程。本项目目前不提供漏洞赏金。

普通缺陷和支持请求请遵循 [SUPPORT.md](SUPPORT.md)。
