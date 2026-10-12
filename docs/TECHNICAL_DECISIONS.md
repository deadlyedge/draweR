# winDrawer 技术决策与验证记录

本文跟踪 Rust/Windows 实现中会影响架构、兼容性、用户数据或发布方式的决定。**PENDING 表示尚未批准，不得由实现者静默定案。** 选型完成后，将状态改为 `ACCEPTED`，记录日期、负责人、依据、替代方案和后果。

## 1. 决策状态

| ID | 主题 | 状态 | 优先级 | 决策门槛 |
|---|---|---|---|---|
| TD-001 | Rust 与 WinUI 3 的集成方式 | PENDING（已授权 scaffold） | 高（正式定案仍阻塞） | 已允许以固定 commit `d5b6a1f68f4fdd0c79495954d162901ea90da4f4` 的 Reactor API 开始隔离 scaffold；正式采用前仍须完成本节剩余验证。禁止使用浮动 master。 |
| TD-002 | WinUI 失败时的 UI 备选路线 | PENDING | 高 | 明确是否允许纯 Win32 或其他 UI 技术；未经批准不得自行更换目标。 |
| TD-003 | Windows SDK、Rust target 与最低系统版本 | PENDING | 高 | 选定 windows-rs/Windows App SDK 版本并验证目标机器和 CI。 |
| TD-004 | 安装/分发形式 | PENDING | 高 | 验证运行时依赖、升级、卸载、资产和启动项生命周期。 |
| TD-005 | 设置与日志目录 | PENDING | 高 | 明确便携版/安装版、多用户、权限和配置导入行为。 |
| TD-006 | 窗口坐标、DPI 与多显示器策略 | PENDING | 高 | 明确持久化单位、显示器断开、屏幕外恢复和窗口相邻规则。 |
| TD-007 | 自启动实现及是否属于 MVP | PENDING | 中 | 产品确认后选择用户级启动方式并设计卸载清理。 |
| TD-008 | 无边框窗体透明度/材质效果 | PENDING | 中 | 在目标 UI 技术上确认 Alpha 的视觉与可读性语义。 |
| TD-009 | 图标/缩略图提取与缓存 | PENDING | 中 | 定义优先级、失败回退、性能目标、缓存失效和清理。 |
| TD-010 | Rust 异步运行时与 UI 工作调度 | PENDING | 中 | 技术 spike 确认 UI dispatcher、后台任务取消和退出顺序。 |

## 2. TD-001：Rust 与 WinUI 3 集成

### 必须验证

- Rust 工程可创建 WinUI 3 / Windows App SDK 窗口并运行原生控件；记录应用是否需要直接维护 XAML，以及资产/绑定如何生成。
- 构建和绑定生成步骤可在干净 Windows 环境和 CI 中重复执行，不依赖个人 IDE 状态。
- 能安全获取窗口 HWND，并实现无标题栏拖动、尺寸调整和窗口关闭/隐藏。
- WinUI dispatcher 与后台 Rust 任务之间可以安全传递结果，并且关闭窗口时不发生 use-after-free / 悬空回调。
- 能显示并接收传统 Windows Shell 托盘图标回调。
- 产物可在目标 Windows 版本上以计划的分发形式运行。

### 候选方案

1. **windows-reactor（优先调查）**：位于 `microsoft/windows-rs` 仓库的 Rust/WinUI 组件框架；master 样例覆盖托盘图标、多窗口、窗口位置和虚拟列表。文档标为 preview，不等同于已批准生产依赖；并且这些样例 API 不一定存在于 crates.io 固定发布版。验证步骤见 [`WINDOWS_REACTOR_POC.md`](WINDOWS_REACTOR_POC.md)。
2. **Winio WinUI backend**：Rust 异步 GUI runtime 的 WinUI backend，可作为 Reactor POC 未通过时的 Rust/WinUI 备选；需单独评估其 API、运行时与窗口/托盘互操作。
3. **直接使用 windows-rs 与 Windows App SDK/WinUI API**：底层控制最多，但 XAML、应用生命周期和部署 interop 风险较高；仅在有具体缺口且经 spike 验证后采用。
4. **经批准的替代 UI/host**：仅在候选方案无法满足需求并经用户批准后考虑；不得静默更换目标。

### 已有验证证据（更新至 2026-10-12）

1. crates.io `windows-reactor = 0.100.0` 隔离 POC：通过 fmt/check/clippy/test，并启动主窗；组件窗、后台目录刷新和设置草稿可运行。该发布版没有 `Application`、`ApplicationView`、通知图标菜单、`App::run_application` 或 placement API，因此它不验证 app-level 托盘/placement 能力。详见 [`poc/windows-reactor/README.md`](../poc/windows-reactor/README.md)。
2. Commit-pinned POC：固定 `windows-rs` revision `d5b6a1f68f4fdd0c79495954d162901ea90da4f4`。`Cargo.lock` 锁定同一 SHA；在 Windows x64/MSVC 上通过 fmt/check/clippy/test/build，并实测创建主窗和 placement 窗、读取物理像素 placement、关闭 placement 后主窗仍在，以及用 POC 按钮关闭主窗后进程仍存活。
3. **负责人手动观察**：负责人报告该 POC 的通知图标显示及右键菜单工作正常。该观察补充了我的自动化无法确认托盘 UI 的限制；但没有单独证明菜单各项行为、Hide、双击、Explorer 重启恢复或干净机器部署。
4. **无边框/图标网格 POC（2026-10-12）**：固定 revision 现包含独立 keyed `chrome-grid` 测试窗，可从主窗和托盘菜单打开；使用同 SHA 的 `windows-sys` 试验依赖。最终构建上 UI Automation 观察到 48 个本地图像图块，Win32 resize 探针调整窗口宽度 1666→1346 px 时列数 6→5，恢复宽度后回到 6。`WS_CAPTION | WS_THICKFRAME` 去除后系统 caption buttons 消失、恢复后回来。此证明需 Win32 style interop，不是 Reactor TitleBar 的单一配置；去框后原生 resize 和系统按钮均不可用，不能据此宣称拖动/resize 产品要求已满足。Shell 图标获取和真实文件浏览未验证。详见该 POC README。
5. **控制台窗口**：入口设 `windows_subsystem = "windows"`；最终 POC 构建的 PE subsystem 为 GUI (2)，启动时未出现独立控制台窗口。

**授权范围**：负责人已授权开始应用 scaffold，可以把上述完整 SHA 作为当前试验基线。此授权不是对 preview API 的无条件/永久采用，也不是 TD-001 完成。`d5b6…` 版本没有独立 Hide API；当前 POC 的关闭动作销毁窗口组件，但应用资源/进程由仍声明的托盘保持存活。产品 FR-TRAY-01/02 要求的 Hide（保留可恢复窗口状态）仍须在正式交互设计或后续 spike 中明确，不能拿 close/reopen 冒充 Hide。

剩余决策门槛：Explorer `TaskbarCreated` 恢复和错误路径；托盘各项及双击的逐项确认；Hide 与 close 的产品语义；DPI、负坐标多显示器、显示器断开后的 placement 策略；整窗透明度/材质；无开发工具干净机器上的 Windows App Runtime 部署；正式 crate/版本与升级策略。当前详细记录见 [`poc/windows-reactor-master/README.md`](../poc/windows-reactor-master/README.md)、[`poc/windows-reactor/README.md`](../poc/windows-reactor/README.md) 和 [`WINDOWS_REACTOR_POC.md`](WINDOWS_REACTOR_POC.md)。

### Spike 通过标准

- 新机器/干净 checkout 按文档完成构建，不依赖个人 IDE 状态或本机生成文件。
- UI 窗口、托盘、设置对话框可启动、交互和退出；重复启动退出不会留下进程或托盘图标。
- 写明 Rust/toolchain、crate、Windows SDK、Windows App SDK、target、运行时与打包工具版本。
- 记录不支持的功能、unsafe/COM 使用点、已知限制和升级路径。

## 3. TD-002：UI 备选方案

WinUI 是产品方向，不意味着任何 Rust UI 绑定都已经选定。若 WinUI spike 失败，先记录失败点与修复成本，再由项目负责人选择：调整 Rust/XAML 集成、采用纯 Win32，或批准其他方案。**不得未沟通就改成 WebView、Electron、Tauri 或其他 UI 技术。** 领域模型、配置格式和平台 Shell adapter 应保持与该选择解耦。

## 4. TD-003：工具链和系统支持

接受方案前需记录：

- Rust stable channel 与 MSRV；
- windows-rs crate/version 及启用的 API features；
- Windows App SDK / Windows SDK 版本；
- 支持的最低 Windows 版本和架构（如 x64/arm64）；
- Windows 原生构建所需的 Visual Studio Build Tools、SDK、XAML 编译工具；
- CI 使用的 runner 与构建/测试命令。

版本选择以可重现构建和目标机器兼容为先；不依赖“开发机碰巧已安装”的 SDK。

## 5. TD-004：打包与发布

待选择的分发方式包括 MSIX、unpackaged 安装包或便携发布。决策时检查：

- Windows App SDK runtime、VC runtime 和其他系统依赖的部署方式；
- 安装到不可写目录时设置仍可读写；
- 升级保留用户配置与缓存；卸载清理应用文件但不意外删除用户数据；
- 应用图标、字体、许可证、签名、版本号和崩溃日志的位置；
- 用户级自启动项、托盘和窗口启动行为与安装/卸载的关系。

发布包必须从干净环境验证，而非只在开发机运行。

## 6. TD-005：配置与日志目录

选择其中一种明确策略，并覆盖安装版与便携版：

- 用户级 `%APPDATA%`/`%LOCALAPPDATA%` 下的应用目录；或
- 可选的便携模式（配置与可执行文件同目录，需检测写权限并提示）。

决策需包含：配置文件优先级、旧文件首次导入、备份命名与保留周期、多个用户隔离、日志轮转/大小上限、只读目录行为。任何路径迁移都不得静默覆盖已有设置。TOML 字段本身遵循 [`CONFIGURATION_FORMAT.md`](CONFIGURATION_FORMAT.md)。

## 7. TD-006：坐标、DPI 与多显示器

需要统一定义：

- WinUI layout 使用的 DIP 与 Win32 屏幕坐标之间如何转换；
- 配置中窗口位置采用虚拟桌面坐标、显示器相对坐标还是其他持久化表达；
- DPI 变化时窗口尺寸/位置如何保持稳定；
- 保存时显示器断开或位置超出工作区时如何恢复；
- 主窗口与抽屉相邻时考虑右侧空间不足的策略（保持相邻、移到左侧、限制位置等）。

验收至少覆盖单屏、不同缩放率双屏、负坐标显示器、主屏变化和断开外接显示器后重启。恢复策略应保持窗口完整可见，并避免每次运行意外重置用户位置。

## 8. TD-007：Windows 自启动

产品需求文档将其列为待确认能力。若批准实现，选定用户级方案（例如 Startup 文件夹快捷方式或相应受支持的应用启动机制），定义：

- 启用/禁用时如何创建/移除该应用自己的条目；
- 路径变化、更新、卸载和权限错误时如何同步；
- 是否启动到托盘/隐藏状态；
- 用户如何获知配置失败。

不能只保存一个布尔值却不影响 Windows 登录行为。

## 9. TD-008：透明度与主题

当前目标包含 HSLA 主题和窗口透明度。需在目标 UI 实现上确认：

- Alpha 是整个窗口透明度还是仅背景透明度；
- 文本、图标、阴影、鼠标悬停区域的实际效果；
- 最低透明度是否为 5%，Alpha=0 时如何处理；
- 多窗口预览与 Cancel 回滚是否一致；
- 深浅背景上的文本对比度如何测量。

主题颜色生成规则放在领域层并用单元测试固定；UI 层不重复计算颜色。

## 10. TD-009：图标、缩略图与性能

基础目标是快速展示文件/文件夹条目，并对常见图片提供预览。选定提取策略时比较：Shell 关联图标、系统缩略图缓存、图片解码器的性能、质量和资源释放。决策必须涵盖：

- 大目录、网络目录和慢速文件系统下的响应目标；
- 请求取消、并发上限和 UI 更新批次；
- 图片类型范围、缩放与缓存键；
- 文件修改、路径移动、主题/尺寸变化后的失效规则；
- 缓存上限、清理和用户数据目录；
- 错误/权限拒绝时的图标回退。

首版如无实测需求，可只做内存级缓存；磁盘 `.cachefiles` 属后续功能，需用户批准。

## 11. TD-010：后台任务与生命周期

可选择 async runtime 或标准线程/任务机制，但应避免仅为少量 IO 引入不必要复杂度。当前应用模型以 `PlatformCommand` / `PlatformResult` 将阻塞平台操作与 UI 状态更新分开；这是线程无关的边界，不代表已经选定 executor 或 async runtime。无论选型，必须定义：

- UI 线程如何发起工作并接收结果；
- 刷新任务取消与过期结果判定；
- 正常退出、异常退出和设置 flush 的顺序；
- 托盘回调与 HWND 销毁的竞态处理；
- 后台错误如何显示/记录，避免 unobserved task/panic。

## 12. 决策记录模板

接受或修改任一决策时追加记录：

```markdown
### TD-XXX：标题
- 状态：ACCEPTED / REJECTED / SUPERSEDED
- 日期：YYYY-MM-DD
- 决策人：
- 背景与约束：
- 选择：
- 验证证据：命令、目标环境、结果或 spike 链接
- 考虑过的替代方案：
- 后果/风险：
- 重新评估触发条件：
```
