# windows-reactor WinUI 集成 POC 指南

## 1. 目标与结论边界

目的不是在这一步迁移完整应用，而是在 Windows 开发机上验证 `windows-reactor` 是否能承载 draweR 的 UI 与生命周期需求。POC 通过后才决定是否把它定为正式 UI 栈。

### 已完成的验证与重要发现（2026-10-11）

独立 POC 已固定 `windows-reactor = 0.100.0`，能创建主窗口、抽屉窗和设置窗；后台目录枚举、刷新取消/generation 过滤及设置草稿测试均已通过。完整结果记录在 [`poc/windows-reactor/README.md`](../poc/windows-reactor/README.md)。

关键发现：**仓库 `master` 的示例 API 与 crates.io 发布的 `0.100.0` 不是同一 API 面。** `0.100.0` 不提供 `Application`、`ApplicationView`、通知图标菜单、`App::run_application` 或文档中显示的窗口 placement API。以下所有依赖这些 API 的步骤必须先确认选用并固定了包含这些 API 的源码 revision；不得将它们当成 `0.100.0` 已有能力。

当前最值得继续调查的候选是 Microsoft `windows-rs` 仓库中的 `windows-reactor`：它用 Rust 组件、类型化消息和原生 WinUI 控件描述 UI；仓库 `master` 的应用级 API 覆盖多窗口和系统托盘。示例目录中包含 `notifyicon`、`secondary-window`、`window-placement`、`virtual`、`explorer` 等，与本项目需求高度相关，但应先确认每项 API 对应的具体版本/revision：

- [windows-reactor 文档](https://github.com/microsoft/windows-rs/blob/master/docs/crates/windows-reactor.md)
- [Reactor 示例目录](https://github.com/microsoft/windows-rs/tree/master/crates/samples/reactor)
- [notifyicon 示例](https://github.com/microsoft/windows-rs/tree/master/crates/samples/reactor/notifyicon)
- [secondary-window 示例](https://github.com/microsoft/windows-rs/tree/master/crates/samples/reactor/secondary-window)
- [window-placement 示例](https://github.com/microsoft/windows-rs/tree/master/crates/samples/reactor/window-placement)
- [Crate 版本信息](https://crates.io/crates/windows-reactor)

需要保留的风险判断：Reactor 文档目前将其标为 **preview**，并说明公开 API 仍可能变化；Windows App Runtime 部署也必须在目标环境验证。因此，本文件给出的是验证方案，不是已经批准的最终技术选型。

## 2. POC 范围

### 必须验证

1. Rust 工程可构建并创建可见的原生 WinUI 3 窗口。
2. 主窗口、抽屉窗口和设置窗口可同时/按需管理；窗口关闭与隐藏语义清楚。
3. 系统托盘图标、Show/Hide、Settings、Exit 与双击激活可用。
4. 主窗口与抽屉窗口能拖动、调整尺寸、恢复位置；DPI/多显示器坐标符合预期。
5. 主题预览/保存/取消可在状态模型上工作；验证透明度是否能达到产品预期。
6. 目录读取、Shell 打开等阻塞工作能在后台完成，再通过 Reactor 消息更新 UI。
7. 构建和运行步骤可从干净 checkout 按文档重现；确定 Windows App Runtime 与分发要求。

### 暂不做

- 不迁移全部产品功能、TOML repository、缩略图缓存或自启动注册。
- 不更改现有领域模型和配置 schema；复用目前 `drawer_model` 的公开类型，或在 POC crate 做薄适配。
- 不因 POC 方便而移除 `UiPort` / `WindowsPort` 的职责边界。
- 不在 POC 阶段实现自定义 XAML 控件或复杂动画；先验证框架本身及关键系统互操作。

## 3. 建议隔离方式

建议在独立分支或独立 crate 中验证，避免替换当前最小模型入口。若采用子 crate，保持主项目 `cargo test` 不会隐式构建 POC；README 记录它需要的 Windows SDK/Windows App Runtime。

锁定试验版本和 Cargo.lock。示例起点：

```toml
[dependencies]
windows-reactor = "=0.100.0"
```

Crate 元数据报告此版本的 Rust 版本要求为 1.95。先在目标 Windows 机检查 `rustc --version`；若本机工具链低于要求，应选择兼容版本或升级工具链，并把实际版本记入结果。不要直接依赖未固定的 `master`。

先采用 framework-dependent 运行方式，减少部署变量；需要验证 self-contained 部署时，再按 Reactor 文档增加 `windows-reactor-setup` build dependency 并调用 `as_self_contained()`。两种部署方式都须在未安装开发工具的干净环境验证。

## 4. 已发布 API 与仓库 master API 必须分开

### A. crates.io `windows-reactor = 0.100.0`（已验证）

当前 POC 对应此版本：使用 `App::run_component`，通过 `ComponentContext::open_window` 创建抽屉/设置窗口，并以 `spawn_background` 做目录枚举。测试和运行结果见 [`poc/windows-reactor/README.md`](../poc/windows-reactor/README.md)。

此发布版本没有 `Application`、`ApplicationView`、托盘/通知图标菜单、`App::run_application` 或文档 master 示例中的窗口 placement API。因此它能验证组件 UI 和窗口内容，但**不能单独验证 app-level tray 生命周期、托盘守护主进程或发布版窗口 placement**。不要把 master 的代码示例直接贴入这个 crate。

### B. 固定 commit 的 `windows-rs` Reactor API（当前 scaffold 基线）

独立 POC 已固定 `windows-rs` commit `d5b6a1f68f4fdd0c79495954d162901ea90da4f4`（2026-10-10，`windows-reactor` native event dispatch commit），使用 `Application` / `ApplicationView`、类型化应用消息、keyed window、`NotifyIcon` 和 placement 通知。负责人已授权以此作为应用 scaffold 的试验基线，但 TD-001 仍未正式接受：Reactor 是 preview，Hide 产品语义和部署、DPI 等项目尚未验证。

POC 位于 [`poc/windows-reactor-master/`](../poc/windows-reactor-master/)，其 manifest 与 lockfile 固定相同的完整 SHA，且不属于根 workspace。项目负责人手动报告通知图标和右键菜单工作正常；此前自动 UI Automation 未能枚举通知区域项，所以将该结果记录为负责人现场观察，不扩展成 Explorer 恢复或每项菜单操作均通过。README 记录了我实际验证的启动、关闭和 placement 行为以及未覆盖项。

**Lifecycle 注意**：此 commit 的 `Application` 状态与普通窗口组件分离；只要 ApplicationView 仍声明托盘资源，关闭主窗后进程可继续运行。`show_window(key)` 会重开/恢复声明的窗口，但窗口被关闭时该组件实例会销毁。该版本没有独立 hide API，因此 close/reopen 不等价于产品所需的 Hide/Show（保留窗口状态/实例）。正式 scaffold 必须明确这项差异，不能把关闭按钮直接当作产品隐藏行为。

### UI 与后台操作的共通原则

- 主窗口展示抽屉名称、缺失目录状态和添加入口；抽屉窗口先只支持一个活动目录；设置窗口复用纯 Rust `SettingsDraft`。
- 目录枚举和 Shell 打开操作放到后台；刷新带 request/generation id，窗口关闭、抽屉切换或新刷新发起后，过期结果不得覆盖当前视图。
- 保留 `WindowsPort` 边界；UI 层负责提交平台命令，并把完成结果转成组件消息。
- 不在 UI 线程同步执行 `std::fs::read_dir`、图片解码或长时间 Shell 操作。
- 先使用 Reactor 原生控件和 Rust builder API；除非 POC 证明必要，不先引入自定义 XAML 或自行构建整套控件。

## 5. POC 验证步骤

### A. 已发布 `0.100.0` POC（已完成）

用户已在 `poc/windows-reactor` 完成此轨道的最小验证：独立锁定 crate、后台枚举、刷新取消和过期结果过滤、设置 draft，以及窗口启动 smoke test。具体工具版本和通过项见 [`poc/windows-reactor/README.md`](../poc/windows-reactor/README.md)。该结果**不覆盖**下方托盘、ApplicationView 或 placement API，因为它们不存在于固定发布版中。

### B. 固定 commit POC 已完成项与 scaffold 约束

1. 已检查 `windows-rs` commit `d5b6a1f68f4fdd0c79495954d162901ea90da4f4`，并在独立 crate 中固定 `windows-reactor` Git revision 与 `Cargo.lock`；没有依赖浮动 branch。
2. 已确认这份源码公开 `Application` / `ApplicationView` / `NotifyIcon` 和 `WindowPlacement` API，并据此实现 main、placement 两个 keyed window 与 tray resource。独立 POC 构建、测试及当前运行记录见 [`poc/windows-reactor-master/README.md`](../poc/windows-reactor-master/README.md)。
3. 尚未运行 upstream 未修改样例，也未验证所有真实托盘动作、Explorer restart、clean checkout/CI 和干净机器运行时部署；负责人对通知图标/右键菜单正常的观察单独标记为手动证据。
4. 已允许开始正式应用 scaffold，但生产功能实现前必须把 Reactor 组件限制在 UI adapter 层，保留纯 Rust model/controller 和平台 port；不得把 close/reopen 当作产品 Hide。对未解决的 Hide、位置持久化/DPI 和部署风险，应先建立明确的产品/技术决策与验证计划。

### C. 托盘和应用级生命周期（固定 commit）

此节不属于 crates.io `0.100.0` POC；固定 commit 提供这些 API。当前部分生命周期已验证，逐项证据/限制见 [`poc/windows-reactor-master/README.md`](../poc/windows-reactor-master/README.md)。

1. **负责人观察通过**：通知图标可见且右键菜单正常。仍需逐项验证 Show、Settings、Exit、双击激活及实际重开各窗口。
2. POC 按钮关闭主窗后，**进程仍在运行**；Reactor/Application 声明托盘资源与用户窗口分离。真实托盘重新激活未由自动化验证。
3. Exit 的图标移除、正常 flush、进程退出及无残留仍待验证。
4. Explorer 重启后的 `TaskbarCreated` 自动恢复及错误处理仍待验证。
5. 本 commit 没有独立 Hide API；窗口 close 会结束窗口组件实例。产品 FR-TRAY-01/02 要求的 Hide/Show 语义需另行验证/设计。

### D. 窗口及定位

主/抽屉/设置内容在 `0.100.0` 组件 POC 中 smoke-test；commit-pinned POC 另验证 main/placement keyed 窗口及 placement 通知。多显示器/DPI 结论仍未验证。

1. POC 中 main 和 placement 两个 keyed 窗口可按需打开；关闭 placement 后主窗保持运行。关闭主窗后的进程存活也已验证；实际托盘触发重新打开、隐藏语义和抽屉切换仍未验证。
2. placement 回调在当前主机报告 `(1020, 448)`、`1306 × 791` physical pixels；尚未实测负坐标、多显示器、不同 DPI、断开屏幕后恢复及保存后重新打开是否符合产品策略。
3. 验证边框样式、标题栏、拖动、resize grip 和主窗口/抽屉窗口相邻行为。
4. 重点测当前设计依赖的窗口透明度：Alpha 参数是否能产生预期的整窗效果，文字和图标是否仍可读。分别记录普通窗口透明、半透明背景/材质两种结果，不能将它们当作等价功能。

### E. draweR 关键交互

当前 `0.100.0` POC 已验证一层目录枚举、取消/过期刷新和设置草稿的 Save/Cancel；它没有验证设置状态跨窗口共享或跨进程持久化。其余产品交互按以下清单验证：

- 主窗口抽屉列表支持点选/再次点选关闭；切换抽屉时只保留一个活动抽屉。
- 目录只显示一级文件/子目录；目录排在文件前，名称排序稳定；空目录/缺失目录/权限错误状态可见。
- Refresh 反映新内容；连续快速刷新时，旧结果不能覆盖新结果。
- 图片/文件条目使用合理图标；点击项目走系统默认关联打开。
- Settings 主题实时预览；Cancel 完整回滚；Save 后重开仍保留。
- 长路径、大目录和图片解码时 UI 保持响应。

### F. 部署

- framework-dependent：记录目标机器所需 Windows App Runtime 的安装方式和版本。
- self-contained：如果产品需要便携分发，再验证 `windows-reactor-setup` 的构建与部署结果。按 Reactor 文档在 POC crate 中添加 build dependency，并在 `build.rs` 调用 setup helper：

  ```toml
  [build-dependencies]
  windows-reactor-setup = "=0.100.0"
  ```

  ```rust
  fn main() {
      windows_reactor_setup::as_self_contained();
  }
  ```
- 至少用一台未安装 Visual Studio/Rust 开发工具的 Windows 环境运行 POC。
- 记录启动耗时、包体、所需运行库、签名/安装限制和升级策略。

## 6. 测试与决策门槛

POC 至少保留以下证据：

```text
rustc --version
cargo --version
cargo fmt --all -- --check
cargo check --all-targets
cargo test
cargo run
cargo tree -i windows-reactor
cargo tree -d
```

需要另附手动验证结果：托盘生命周期、DPI/多显示器、无边框和透明度、运行时安装方式。若在没有交互桌面的 CI 上运行，明确标注 UI Automation/托盘测试未覆盖，不得把 headless 单元测试当作视觉集成通过。

**允许开始 scaffold**：负责人批准按 commit `d5b6a1f68f4fdd0c79495954d162901ea90da4f4` 开始；该许可仅供隔离 POC 及应用骨架，不改变 TD-001 的 PENDING 状态。

**正式选型通过条件**：固定 revision 确实公开所需 app-level 与 tray API；干净构建稳定；三类窗口与托盘生命周期可靠；Hide 产品语义有可行实现；目录刷新不阻塞 UI；边框/尺寸/位置和主题效果达到需求；Windows App Runtime 部署成本可接受；没有依赖内部/未公开 API 的高风险 interop。仅凭负责人观察托盘菜单和主窗关闭后的进程存活，仍不足以满足完整通过条件。

**不通过或暂缓采用条件**：托盘和隐藏窗口生命周期需要大量私有 HWND hook；整窗透明度无法达到最低产品要求；每次普通更新都需要大规模不兼容改动；或者生产分发必须依赖不受控的本机开发环境。

如果失败，不应继续把业务逻辑写进 Reactor 专属组件；保留 `drawer_model` 与 ports，再比较 Winio WinUI backend、Win32 backend 或经用户批准的 WinUI host + Rust core。

## 7. 当前工程适配建议

- 将 `AppModel`、`SettingsDraft`、`DrawerDefinition` 和主题算法留在纯 Rust 模块；不要让 Reactor 的组件类型泄漏到 `model.rs`。
- `AppController` 负责业务状态和 `PlatformCommand` 生成；组件通过类型化消息调用它，并将后台完成结果派回 UI 线程。
- **若所选 API revision 提供 `ApplicationView`**，由它负责“应用始终存在”的窗口/托盘生命周期；主窗口组件只负责主窗口内容，避免把托盘生命周期绑定到可见窗口。`0.100.0` 轨道当前没有此 API，因此不得在该轨道假设此生命周期已实现。
- 核实当前直接依赖的 `windows-sys 0.59` 与 Reactor 传递依赖之间是否出现多版本；除非需要兼容旧接口，不要同时维护多套重复 Windows API wrapper。
- POC 通过后再调整 `TD-001` 状态、固定正式依赖版本、更新 Cargo.lock、完善 `TECHNICAL_DECISIONS.md` 并开始迁移正式 UI。

## 8. 已有验证结果

`0.100.0` 发布版的本机结果见 [`poc/windows-reactor/README.md`](../poc/windows-reactor/README.md)；固定 commit 的构建及生命周期/placement 结果见 [`poc/windows-reactor-master/README.md`](../poc/windows-reactor-master/README.md)。现阶段允许进入隔离 scaffold 开发，但托盘实际交互仅有负责人手动观察，Explorer 恢复、Hide 语义、DPI/多屏、透明度和干净机器部署仍未全部通过；TD-001 继续为 PENDING，不得写成生产栈已最终批准。

## 9. 结果记录模板

```markdown
### Windows Reactor POC 结果
- 日期：
- Git commit：
- Windows 版本/架构：
- Rust/Cargo/MSVC/Windows App Runtime 版本：
- windows-reactor 版本与 Cargo.lock：
- framework-dependent / self-contained：
- 运行命令和结果：
- 托盘/多窗口/DPI/透明度检查：
- 依赖版本冲突或 unsafe/HWND interop：
- 未通过项：
- 建议：采用 / 延后 / 放弃
- 依据与后续动作：
```
