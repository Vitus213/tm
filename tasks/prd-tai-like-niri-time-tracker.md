# PRD: Tai-like Niri Time Tracker

## 1. Introduction / Overview

这是一个面向 **Niri + Wayland** 环境的桌面应用时间记录产品，目标是在 Linux 上实现一个**功能与信息架构严格参考 Tai** 的本地优先时间追踪软件。

它解决的问题是：当前 Linux/Niri 环境缺少一个像 Tai 那样，能够长期后台运行、记录应用使用时长、后续扩展到网站浏览统计、并提供完整桌面统计界面的产品。

当前仓库已经完成了**基础采集内核**：Niri 焦点窗口轮询、会话切分、SQLite 本地持久化、daemon 运行时、shutdown flush。这份 PRD 既记录**当前进度**，也定义**完整产品目标**，供后续分阶段实现使用。

## 2. Goals

- 提供一个在 **Niri** 下长期稳定运行的本地时间记录产品。
- 在首个完整产品版本中，对齐 Tai 的核心页面结构：总览、图表、数据、详情、分类管理、设置。
- 支持应用使用时长统计，并为网站统计扩展预留稳定架构。
- 所有数据默认保存在本地 SQLite 中，不依赖云端。
- 提供可扩展的规则系统：过滤、白名单、分类、关联组。
- 保持后台 daemon、桌面 GUI、浏览器扩展之间的边界清晰。

## 3. User Stories

### US-001: 后台记录当前聚焦应用
**Description:** 作为用户，我希望系统能够持续记录我当前正在使用的应用，以便我知道时间都花在了哪些软件上。

**Acceptance Criteria:**
- [ ] 在 Niri 会话中，后台 daemon 能轮询当前聚焦窗口。
- [ ] 焦点变化时会生成新的应用事件。
- [ ] 应用会话在 SQLite 中持久化。
- [ ] shutdown 时会 flush 最后一个活动 session。
- [ ] `cargo test --workspace` 通过。

### US-002: 查看桌面端总览页面
**Description:** 作为用户，我希望打开主窗口后先看到当天/当前周期的总览信息，以便快速了解我的时间分布。

**Acceptance Criteria:**
- [ ] 主窗口包含“总览”页面入口。
- [ ] 总览页至少显示总时长、Top 应用、最近活动片段。
- [ ] 总览页数据来自本地 SQLite，而不是硬编码。
- [ ] 页面能在没有数据时显示空状态。
- [ ] `cargo test` / `cargo clippy` 通过。
- [ ] **Verify in browser using dev-browser skill**

### US-003: 查看数据列表页面
**Description:** 作为用户，我希望按列表查看记录下来的应用会话，以便检查具体时间明细。

**Acceptance Criteria:**
- [ ] 主窗口包含“数据”页面入口。
- [ ] 数据页展示 session 列表，至少包含应用名、标题、开始时间、结束时间、时长。
- [ ] 支持按日期范围切换查看。
- [ ] 支持无数据空状态。
- [ ] `cargo test` / `cargo clippy` 通过。
- [ ] **Verify in browser using dev-browser skill**

### US-004: 查看图表分析页面
**Description:** 作为用户，我希望通过图表查看时间分布趋势，以便更直观理解使用模式。

**Acceptance Criteria:**
- [ ] 主窗口包含“图表”页面入口。
- [ ] 图表页至少有一类时间分布图（按应用或按时间段）。
- [ ] 图表数据与总览/数据页使用同一统计来源。
- [ ] 在数据量少时仍能稳定展示。
- [ ] `cargo test` / `cargo clippy` 通过。
- [ ] **Verify in browser using dev-browser skill**

### US-005: 查看应用与网站详情
**Description:** 作为用户，我希望点击某个应用或网站后进入详情页，以便查看它的细粒度统计数据。

**Acceptance Criteria:**
- [ ] 提供应用详情页。
- [ ] 提供网站详情页占位或完整实现入口。
- [ ] 详情页至少展示统计汇总与时间列表。
- [ ] 页面参数与列表/图表点击能正确联动。
- [ ] `cargo test` / `cargo clippy` 通过。
- [ ] **Verify in browser using dev-browser skill**

### US-006: 管理分类规则
**Description:** 作为用户，我希望定义分类规则，把应用或网站自动归入分类，以便从“应用级”上升到“类别级”分析。

**Acceptance Criteria:**
- [ ] 提供分类管理页面。
- [ ] 可新增、编辑、删除分类。
- [ ] 分类规则至少支持 app_id / 路径 / 域名中的一种匹配方式。
- [ ] 规则会影响后续统计聚合。
- [ ] `cargo test` / `cargo clippy` 通过。
- [ ] **Verify in browser using dev-browser skill**

### US-007: 配置后台行为
**Description:** 作为用户，我希望在设置页调整记录行为，以便让软件符合我的使用习惯。

**Acceptance Criteria:**
- [ ] 提供设置页。
- [ ] 至少支持：空闲时暂停、网站统计开关、启动行为。
- [ ] 设置项持久化到本地。
- [ ] 修改后重启仍能生效。
- [ ] `cargo test` / `cargo clippy` 通过。
- [ ] **Verify in browser using dev-browser skill**

### US-008: 网站浏览统计
**Description:** 作为用户，我希望系统也能记录网站浏览时间，以便获得和 Tai 类似的完整使用画像。

**Acceptance Criteria:**
- [ ] Chromium 扩展可发送网站活动消息到本地程序。
- [ ] Firefox 扩展可发送网站活动消息到本地程序。
- [ ] 主程序能把网站事件落入统一统计模型。
- [ ] 网站详情页可展示网站时长数据。
- [ ] `cargo test` / `cargo clippy` 通过。

### US-009: 导出统计数据
**Description:** 作为用户，我希望导出统计结果，以便在其他工具里复用或归档。

**Acceptance Criteria:**
- [ ] 支持 CSV 导出。
- [ ] 支持 XLSX 导出。
- [ ] 导出内容与本地统计结果一致。
- [ ] 导出失败时有明确错误提示。
- [ ] `cargo test` / `cargo clippy` 通过。

## 4. Functional Requirements

- FR-1: 系统必须在 Niri 环境下识别当前聚焦窗口，并转换为应用活动事件。
- FR-2: 当聚焦窗口变化时，系统必须关闭上一条活动 session 并开始新的 session。
- FR-3: 当运行时失去焦点窗口或收到关闭信号时，系统必须收口当前 session。
- FR-4: 系统必须将活动 session 持久化到本地 SQLite。
- FR-5: SQLite 存储必须支持文件型数据库，并在进程重启后保留数据。
- FR-6: 桌面主窗口必须提供 Tai 风格的主导航结构，至少覆盖总览、图表、数据、详情、分类管理、设置。
- FR-7: 总览页必须展示从本地数据库聚合出来的统计摘要。
- FR-8: 数据页必须展示 session 明细列表。
- FR-9: 图表页必须展示基于 session 聚合的图形统计结果。
- FR-10: 应用详情页必须展示单个应用的聚合与明细。
- FR-11: 网站详情页必须展示单个网站的聚合与明细。
- FR-12: 分类管理页必须支持分类规则维护。
- FR-13: 设置页必须支持后台行为配置并持久化。
- FR-14: 系统必须支持本地 IPC 契约，供未来 GUI、扩展和后台服务通信。
- FR-15: Chromium 与 Firefox 扩展必须能够通过本地协议上报网站活动事件。
- FR-16: 系统必须支持网站活动与应用活动的统一统计模型。
- FR-17: 系统必须支持白名单、过滤规则、自动分类和关联组能力。
- FR-18: 系统必须支持 CSV 和 XLSX 导出。
- FR-19: 所有核心能力必须在本地运行，不依赖云端后端。

## 5. Non-Goals (Out of Scope)

- 不做云同步。
- 不做多设备账户体系。
- 不做移动端客户端。
- 不做团队协作或在线报表共享。
- 不追求逐像素复刻 Tai 的 WPF 渲染效果；重点是信息架构、交互方式和功能对齐。

## 6. Design Considerations

- UI 风格应严格参考 Tai：左侧主导航 + 右侧内容区。
- 总览页优先强调“今天/本周期的使用情况”。
- 图表页与数据页要共用同一套查询/聚合来源，避免口径不一致。
- 分类页和设置页要作为一级入口，而不是埋在次级对话框中。
- 视觉上应优先统一主题、间距、卡片层级和导航节奏。

## 7. Technical Considerations

### 当前已完成的基础架构
- Rust workspace 已建立：`Cargo.toml`
- 当前 crates：
  - `crates/core`
  - `crates/storage`
  - `crates/tracker`
  - `crates/ipc`
  - `crates/daemon`

### 当前已实现能力
- `crates/core`
  - 活动事件模型
  - session 累积器
  - 时间顺序约束与 session 不变量
- `crates/storage`
  - SQLite schema
  - typed repository error
  - in-memory 与 file-backed repository
- `crates/tracker`
  - Niri focused window 查询
  - tracker error 类型
  - 焦点快照归一化
  - polling helper
- `crates/ipc`
  - 显式 tagged JSON IPC message 格式
- `crates/daemon`
  - daemon runtime loop
  - tracker polling 接入
  - file-backed SQLite 持久化
  - ctrl-c shutdown flush

### 推荐完整产品技术栈
- 语言：Rust 2024
- 异步运行时：Tokio
- 时间处理：chrono
- 序列化：serde / serde_json
- 本地数据库：SQLite + sqlx
- Niri 集成：niri-ipc
- 桌面 GUI：建议 Rust 原生 GUI（当前推荐优先 `egui/eframe` 路线）
- 图表：egui_plot 或同级 Rust 图表方案
- 浏览器扩展：Chromium WebExtension + Firefox WebExtension
- 主程序与扩展通信：native messaging / 本地 IPC
- 托盘/状态图标：后续根据 Wayland/Niri 兼容性选择 SNI 路线

### 整体产品架构
1. **tracker 层**
   - 从 Niri 获取当前 focused window
   - 从浏览器扩展接收网站事件
2. **core 层**
   - 负责活动事件模型与 session 生命周期
3. **storage 层**
   - 负责 SQLite 持久化和查询
4. **daemon 层**
   - 负责运行时调度、轮询、flush、后台生命周期
5. **ipc 层**
   - 负责 GUI / daemon / 扩展之间的消息契约
6. **future ui 层**
   - 负责所有 Tai 风格桌面页面与本地数据显示

## 8. Success Metrics

- daemon 能稳定运行并把 session 写入本地 SQLite。
- 用户能在桌面 GUI 里查看当天统计、图表和明细。
- 用户能维护分类与规则，并看到统计结果随之变化。
- 用户能启用浏览器扩展后查看网站时长统计。
- 全 workspace 保持 `cargo test`、`cargo fmt --check`、`cargo clippy -D warnings` 为绿。

## 9. Open Questions

- 桌面 GUI 最终选 `egui/eframe` 还是其他 Rust 原生 GUI 框架？
- 托盘在 Niri/Wayland 下的最终兼容方案是否要引入额外依赖？
- 网站统计主程序与浏览器扩展的通信最终是否统一走 native messaging，还是 daemon socket + host bridge 双层结构？
- 分类/规则系统是否在第一版 GUI 就全部开放，还是先只做只读展示？
- 图表页第一版优先哪些图形：时间分布、趋势、分类占比？
