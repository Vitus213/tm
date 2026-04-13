# Tai-like Niri Time Tracker Product Spec

## 1. Overview

这是面向 **Niri + Wayland** 的本地优先应用时间记录产品总 spec。目标是在 Linux 上实现一个在**界面结构、功能覆盖、数据组织和使用体验上严格参考 Tai** 的桌面软件。

当前产品已经完成了**基础后台采集与持久化内核**，包括：Niri 聚焦窗口采集、应用事件归一化、session 切分、SQLite 文件存储、daemon runtime、shutdown flush。接下来仍需补齐完整产品层：桌面 GUI、分类与规则、网站统计、导出和托盘体验。

本 spec 既记录**当前进度**，也定义**最终产品目标状态**，作为后续多个子项目设计与实施的主参考。

## 2. Product Goals

1. 在 Niri 上提供稳定可长期运行的后台时间记录产品。
2. 在桌面端提供与 Tai 对齐的信息架构：总览、图表、数据、详情、分类管理、设置。
3. 支持应用使用时长统计，并扩展到网站统计。
4. 所有核心数据默认保存在本地 SQLite，不依赖远程服务。
5. 保持模块边界清晰，便于后续迭代完整产品。

## 3. Current Implementation Status

### 3.1 已完成模块

#### Workspace 与代码结构
- 仓库已升级为 Rust workspace：`Cargo.toml:0`
- 当前成员：
  - `crates/core`
  - `crates/storage`
  - `crates/tracker`
  - `crates/ipc`
  - `crates/daemon`

#### Core 领域层
- `crates/core/src/activity.rs`
  - `ActivityKind`
  - `ActivityEvent`
- `crates/core/src/session.rs`
  - `ClosedSession`
  - `SessionAccumulator`
  - 时间顺序校验与不变量约束
- `crates/core/src/idle.rs`
  - idle transition 基础模型

#### Storage 持久化层
- `crates/storage/src/schema.rs`
  - SQLite schema
- `crates/storage/src/repository.rs`
  - `SqliteRepository::in_memory()`
  - `SqliteRepository::open_at()`
  - typed errors
  - schema 约束与读取时校验
- 当前默认持久化文件路径已用于 daemon runtime

#### Tracker 采集层
- `crates/tracker/src/niri.rs`
  - `FocusedWindowSnapshot`
  - `focused_window_once()`
  - `map_snapshot_to_event()`
  - `should_emit_focus_event()`
  - `TrackerError`
- 已处理：
  - `app_id` 缺失时的稳定 fallback subject
  - pid 异常值
  - unexpected reply

#### IPC 契约层
- `crates/ipc/src/messages.rs`
  - `DaemonCommand`
  - `DaemonEvent`
- 采用显式 tagged JSON 格式，而不是 serde 默认枚举编码

#### Daemon 运行时层
- `crates/daemon/src/session_service.rs`
  - `SessionService`
  - `IngestOutcome`
  - `FlushOutcome`
  - `SessionRepository` trait seam
- `crates/daemon/src/app.rs`
  - daemon runtime loop
  - file-backed SQLite 打开
  - tracker polling
  - previous_focus dedupe
  - no-focus 边界 flush
  - ctrl-c shutdown flush
  - HOME 缺失时非 panic 错误路径

### 3.2 已验证状态

最近验证结果已通过：
- `cargo test --workspace`
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo run -p tm-daemon` 可启动
- 数据库文件存在：`~/.local/share/tm/tm.db`

### 3.3 尚未实现模块

#### 桌面 GUI
- 主窗口
- 左侧导航
- 总览页
- 图表页
- 数据页
- 应用详情页
- 网站详情页
- 分类管理页
- 设置页

#### 产品功能层
- 白名单模式
- 过滤规则
- 自动分类
- 关联组
- 导出 CSV / XLSX

#### 网站统计层
- Chromium 扩展
- Firefox 扩展
- native messaging / 本地桥接
- 网站 session 模型完整接入

#### 桌面体验层
- 托盘/状态图标交互
- 后台常驻入口
- 完整安装与分发

## 4. Overall Product Architecture

### 4.1 架构分层

#### Layer 1: Event Sources
负责产生原始活动信号。

- Niri focused window
- Chromium extension events
- Firefox extension events
- idle/away detector（后续补齐）

#### Layer 2: Normalization
负责把不同来源转换成统一活动事件。

- app focus -> `ActivityEvent::App`
- website visit -> `ActivityEvent::Website`
- future idle event -> session boundary event

#### Layer 3: Sessionization
负责 session 生命周期管理。

- 打开 session
- 关闭 session
- 时间顺序校验
- shutdown flush
- no-focus 边界切分

#### Layer 4: Persistence
负责本地数据持久化与读取。

- SQLite schema
- session repository
- typed error surface
- file-backed database path handling

#### Layer 5: Query / Aggregation
负责给 GUI 和导出提供统计口径。

- 总览数据聚合
- 图表数据聚合
- 明细数据查询
- 分类/详情数据查询

#### Layer 6: Presentation
负责桌面 GUI 与未来导出/浏览器接入。

- desktop GUI
- tray entry
- settings UI
- export pipeline

### 4.2 当前推荐模块边界

#### `crates/core`
职责：纯领域模型与 session 规则。

#### `crates/storage`
职责：SQLite schema、读写、错误类型。

#### `crates/tracker`
职责：从 Niri 或未来其他采集源读取运行时活动状态，并转换为领域事件。

#### `crates/ipc`
职责：进程间消息契约。

#### `crates/daemon`
职责：运行时调度、后台循环、tracker 接线、session 落库、shutdown flush。

#### `future crates/ui`
职责：Tai 风格 GUI 与统计展示。

#### `future extensions/chromium` / `future extensions/firefox`
职责：浏览器活动采集。

## 5. Detailed Tech Stack

### Core Language / Runtime
- Rust 2024
- Tokio
- chrono
- serde / serde_json
- thiserror
- anyhow（目前主要用于 runtime glue）

### Storage
- SQLite
- sqlx

### Window Manager Integration
- `niri-ipc`

### Async / Concurrency Model
- Tokio async runtime
- `spawn_blocking` 用于阻塞型 Niri IPC 调用与 async runtime 解耦

### IPC / Local Protocol
- JSON message contracts
- tagged enum serialization
- 后续可扩展到 Unix domain socket / native messaging host

### Planned GUI Stack
当前推荐优先：
- `eframe` / `egui`
- `egui_plot`

原因：
- Rust 原生栈，和当前 backend 更一致
- 实现速度与跨平台成本平衡较好
- 足够承载第一版 Tai 风格统计 UI

### Planned Browser Extension Stack
- Chromium WebExtension
- Firefox WebExtension
- Native Messaging / 本地桥接进程

### Planned Export Stack
- CSV exporter
- XLSX exporter

## 6. Product Modules and Responsibilities

### 6.1 Background Tracking Module
职责：后台常驻、轮询 Niri、写入会话。

输入：Niri focused window snapshot
输出：SQLite persisted sessions

### 6.2 Desktop GUI Module
职责：展示所有统计页面与设置页。

输入：本地 SQLite 查询结果 / daemon IPC
输出：用户可操作的桌面界面

### 6.3 Rule Engine Module
职责：过滤、白名单、分类、关联组。

输入：activity events / settings / rules
输出：修正后的统计归类结果

### 6.4 Website Tracking Module
职责：接收浏览器事件并转换成网站 activity sessions。

输入：browser tab/url events
输出：website sessions

### 6.5 Export Module
职责：把统计结果导出为 CSV/XLSX。

输入：统计查询结果
输出：导出文件

## 7. Full Product Screen Map

### 一级导航
- 总览
- 图表
- 数据
- 应用详情
- 网站详情
- 分类管理
- 设置

### 页面职责

#### 总览
- 今日总时长
- Top 应用
- 最近活动
- 分类汇总卡片

#### 图表
- 时间分布图
- 应用占比图
- 周期趋势图

#### 数据
- session 明细表
- 时间筛选
- 应用/网站筛选

#### 应用详情
- 单应用统计
- 使用趋势
- session 明细

#### 网站详情
- 单网站统计
- 时间趋势
- 访问明细

#### 分类管理
- 分类列表
- 分类规则编辑
- 规则匹配范围

#### 设置
- 空闲暂停
- 网站统计开关
- 启动行为
- 路径/数据库信息

## 8. Product Delivery Roadmap

### Phase 1 — 基础后台内核
状态：**已完成**

包含：
- Niri app tracking
- sessionization
- SQLite persistence
- daemon runtime

### Phase 2 — 桌面 GUI 主体
状态：**未开始**

目标：
- 主窗口
- Tai 风格导航
- 总览 / 图表 / 数据 / 设置页
- 从本地 DB 读取并展示

### Phase 3 — 详情与分类管理
状态：**未开始**

目标：
- 应用详情页
- 网站详情页
- 分类管理页
- 基础规则系统接入

### Phase 4 — 网站统计
状态：**未开始**

目标：
- Chromium 扩展
- Firefox 扩展
- 本地桥接
- 网站 session 入库与展示

### Phase 5 — 产品完善
状态：**未开始**

目标：
- 导出
- 托盘交互
- 安装/分发
- 性能与修复工具

## 9. Technical Constraints

- 首期平台只支持 **Niri**。
- 所有核心数据必须本地保存。
- 现阶段不引入远程后端。
- 现阶段不引入账户/同步系统。
- UI 与 daemon 要解耦，不让 GUI 直接写 SQLite。
- 浏览器统计必须通过本地桥接进入统一数据模型。

## 10. Risks

- Niri 的 IPC 能力和实时性在复杂场景下可能仍需更多实机验证。
- Wayland 下托盘/状态图标兼容性可能比 Windows Tai 更复杂。
- 浏览器扩展与本地进程通信在 Chromium / Firefox 上会有差异。
- 如果 GUI 框架选型不合适，会明显影响 Tai 风格页面推进效率。

## 11. Recommended Next Spec

下一份子项目 spec 应优先覆盖：

**桌面 GUI 主体（Tai 风格主程序）**

建议范围：
- 主窗口壳层
- 左侧导航
- 总览页
- 图表页
- 数据页
- 设置页
- 基础查询层接入

这样可以在现有 daemon + SQLite 基础上，尽快得到第一个真正“可见可用”的桌面产品版本。
