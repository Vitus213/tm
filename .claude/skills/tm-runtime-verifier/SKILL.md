---
name: tm-runtime-verifier
description: 验证 tm 项目的真实运行链路。凡是用户要求测试、验收、回归检查 tm-ui、tm-daemon、Overview 页面截图是否正确、时长字符串是否正确、或 live daemon/socket 路径是否真的工作时，都应使用这个 skill，而不是只做编译或只看程序能不能启动。当前环境下如果缺少页面切换自动化，Data 和 Charts 必须明确报告为 BLOCKED，而不能假装验证通过。
---

# tm 运行时验证器

这个 skill 用来执行 tm 项目的真实运行时验收，而不是做零散的临时检查。目标是证明：用户看到的 UI 和 daemon / IPC / sqlite 的真实数据一致，而不只是二进制能编译。

## 当前 v1 稳定支持范围

当前第一版已经稳定脚本化的能力只有两类：

1. `cargo test -p tm-ui`
2. `cargo build -p tm-ui -p tm-daemon`
3. 合成数据驱动的 **Overview** 验证
4. live daemon/socket 启动与 ping 验证

## 当前 v1 条件性 / 阻塞范围

下面这些能力仍然取决于环境是否提供非交互式页面切换能力：

- Data 页面截图验证
- Charts 页面截图验证
- live 真实焦点变化的稳定自动观察

如果做不到，必须返回 `BLOCKED`，并写明原因。不要把“程序启动了”当成通过。

## 开始前必须检查

开始之前先确认：

- 当前仓库根目录就是 tm 项目
- `niri` 可用
- `cargo` 可用
- `nix develop` 可用于启动 UI
- `Read` 工具可以读取截图文件

如果前置条件不满足，不要猜测，不要硬跑，直接报告阻塞。

## 执行流程

### 1. 基线验证

先运行：

```bash
cargo test -p tm-ui
cargo build -p tm-ui -p tm-daemon
```

没有新鲜输出，不要声称成功。

### 2. 合成数据 Overview 验证

使用：

```bash
python .claude/skills/tm-runtime-verifier/scripts/run_seeded_overview_check.py --report <path>
```

这个脚本当前已经能：
- 创建隔离的 XDG 数据目录
- 往 sqlite 写入确定性的样本数据，其中包含：
  - 一个不到 1 分钟、必须显示为 `0 min` 的样本
  - 一个超过 1 小时、必须显示为 `Hh Mm` 的样本
- 启动 `tm-daemon`
- 通过 `niri msg action spawn-sh` + `nix develop` 启动 `tm-ui`
- 查询 Overview IPC 数据
- 捕获 Overview 页面截图
- 写出 JSON 报告，包含截图路径、IPC payload、环境信息、期望字符串

然后必须读取截图并核对：
- 页面文字标签是否存在
- `Tracked:` 后面的时长字符串是否和 IPC 汇总一致
- Top apps 文本是否和样本 subject / duration 一致
- Recent activity 文本是否和样本 subject / duration 一致

### 3. Live daemon/socket 验证

使用：

```bash
python .claude/skills/tm-runtime-verifier/scripts/run_live_sampling_check.py --report <path>
```

当前这个脚本已经能：
- 在真实桌面会话里运行 `tm-daemon`，但使用隔离数据库
- 确认 socket 出现
- 确认 ping 正常
- 写出 JSON 报告

当前它还**不能稳定自动证明真实焦点变化已经被观察到**，所以这一步如果只做到 daemon + socket + ping，应返回 `BLOCKED`，并明确说明尚未脚本化真实 focus sampling 观察。

### 4. Data / Charts 验证

如果用户要求验证 Data 或 Charts：
- 先检查环境里是否存在可用的页面切换自动化工具
- 如果没有，就返回 `BLOCKED`
- 不要临场发明不稳定替代方案

## 输出格式

最终回答必须以这个结构结尾：

```markdown
# tm 运行时验证
- Status: PASS | FAIL | BLOCKED
- Scope: <实际执行了哪些检查>
- Tests: <test/build 结果>
- Overview: <通过 / 失败摘要>
- Data: <通过 / BLOCKED 摘要>
- Charts: <通过 / BLOCKED 摘要>
- Live sampling: <通过 / BLOCKED 摘要>
- Artifacts:
  - <json 报告路径>
  - <截图路径>
- Notes:
  - <关键差异或阻塞原因>
```

## 常见错误

- 把“UI 启动了”当成成功
- 用主观感觉看截图，而不是和 IPC payload 对照
- 忘了放 `0 min` 的 seeded case
- 忘了放 `Hh Mm` 的 seeded case
- 复用了旧的 `tm` 窗口，而不是这次新启动的窗口
- 在没有输入自动化时还假装 Data/Charts 已经验证通过

## 产物要求

每个脚本都应该写出机器可读的 JSON 报告。后续检查必须能基于截图、IPC payload 和期望值复核，而不是依赖临时记忆。
