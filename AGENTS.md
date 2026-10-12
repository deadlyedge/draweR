# AGENTS.md — winDrawer Rust 项目开发规范

本文件可随项目迁移到新的工作目录。它约束在本项目中进行的 Rust 开发、代码审查和自动化修改；用户当次任务有更具体要求时，以该任务要求为准。

## 1. 项目目标与规范来源

- 项目目标：使用 Rust 构建 Windows 桌面抽屉应用，目标 UI 为 WinUI；Windows 系统能力通过 windows-rs 或经确认的封装访问。
- **windows-rs 与 WinUI 是不同层次**：windows-rs 提供 Windows API 绑定，WinUI 3 属于 Windows App SDK/XAML UI。二者组合的工具链、XAML 编译、HWND interop、运行时和打包方式必须通过技术验证后确定。不得自行假设某个第三方 Rust UI 封装是既定方案。
- 用户可见行为以 [`docs/PRODUCT_REQUIREMENTS.md`](docs/PRODUCT_REQUIREMENTS.md) 为准；配置格式以 [`docs/CONFIGURATION_FORMAT.md`](docs/CONFIGURATION_FORMAT.md) 为准；未决的平台/架构选择记录在 [`docs/TECHNICAL_DECISIONS.md`](docs/TECHNICAL_DECISIONS.md)。若文档与用户最新要求冲突，先按用户最新要求处理，并同步更新相关文档。
- 开始修改前检查项目布局与工作区状态。遵循仓库已经确定的 Cargo workspace/crate 组织；如果工程尚未创建，不要猜测其路径、target triple、MSRV、UI 绑定或分发形式，先确认相关技术决策。
- 需求文档中的“必需”项应保持；标记为“待决定/后续”的内容不得擅自当成 MVP 需求实现，也不得伪装成已实现功能。

### 当前 UI POC 基线（更新至 2026-10-12）

- 项目负责人已授权开始应用 scaffold，试验 API 基线为 `windows-rs` 仓库 commit `d5b6a1f68f4fdd0c79495954d162901ea90da4f4` 中的 `windows-reactor`；POC crate 在 `poc/windows-reactor-master/`，必须使用其完整 Git `rev` 和 `Cargo.lock`。不得改用浮动 `master` 或悄悄回退到 crates.io `windows-reactor = 0.100.0`（后者没有该 commit 的 Application/NotifyIcon/placement API）。
- **仅授权继续构建 scaffold，不代表 TD-001 正式选型已 ACCEPTED。** Reactor 仍为 preview；TD-001 状态及未完成项见 `docs/TECHNICAL_DECISIONS.md`。API/版本更改须更新锁文件、复跑 POC 验证并记录理由。
- 负责人报告其手动观察到该 POC 的通知图标和右键菜单工作正常；这是有归属的人工验证，不等同于自动化、Explorer 重启、所有菜单动作、双击激活或干净部署通过。不要重复表述为这些行为都已验证。
- 该 revision 没有独立的 hide-window 命令；关闭窗口会销毁窗口组件，应用声明/应用级状态可以独立存活并重新创建窗口。不要把 close/reopen 描述为“隐藏后原实例恢复”。如实现产品 Show/Hide，需单独设计、验证并说明应用状态、窗口实例和位置如何保留。
- `WindowPlacement` 是外窗物理屏幕像素坐标/尺寸（允许负坐标），client size 是 DIP；Reactor 的初始 placement 只在首次创建窗口时读取。DPI/多屏/显示器拔除后的恢复策略仍属待验证，不得仅凭 API 类型宣称符合产品需求。
- Reactor `TitleBar` 是 WinUI 内容标题栏，不等于移除 HWND native caption/frame。若使用 HWND style interop 移除 `WS_CAPTION | WS_THICKFRAME`，应预期系统标题栏按钮和 native resize 同时不可用；未另外实现并验证 drag/resize hit-testing 与关闭/恢复控件前，不得宣称满足无边框窗口交互需求。POC 的实测证据与限制见 `poc/windows-reactor-master/README.md`。
- POC 可执行文件通过 `windows_subsystem = "windows"` 使用 GUI subsystem；构建后检查 PE subsystem 为 GUI (2)，并在交互桌面启动确认无独立控制台。GUI subsystem 下不可依赖 `eprintln!` 作为可见错误反馈；正式应用应通过统一日志和用户可见的可恢复错误呈现诊断。

## 2. 架构与依赖边界

采用“领域与应用逻辑在内，平台与 UI 适配在外”的依赖方向。建议按职责组织，具体目录可依据已批准的工程布局调整：

- **domain**：设置、抽屉定义、项目条目、HSLA/主题 palette、坐标和尺寸等纯数据与算法。
- **application/services**：应用用例、状态转换、设置预览/提交/回滚、目录刷新、命令协调。
- **ports**：设置存储、Shell、缩略图、托盘、自启动等抽象边界；只在有隔离或测试价值时引入。
- **platform/windows**：windows-rs API、Shell/COM、HWND、消息、托盘、启动注册、文件夹选择等 Windows 实现。
- **ui**：WinUI 页面/窗口、ViewModel 或经批准方案中的等效 UI 状态绑定。
- **infra**：TOML、日志、文件写入等基础设施实现。

必须遵守：

1. 领域层不得依赖 WinUI/XAML、HWND、windows-rs 句柄或具体控件类型；平台类型只在 adapter 边界转换。
2. UI 事件负责调用应用命令并呈现状态，不直接解析/写入 TOML，也不重复实现主题与业务规则。
3. 主题转换应集中为纯函数；同一 palette 应供主窗口、设置预览和抽屉窗口使用。取消预览必须恢复打开设置前的主题。
4. 目录枚举、Shell 查询、缩略图生成等长任务不得阻塞 UI 线程。后台工作应支持取消或丢弃过期结果，并通过 UI dispatcher 更新界面。
5. 不为尚未需要的模块提前建立复杂抽象。新增 trait、crate、后台服务或状态层时，应说明它带来的测试、平台隔离或复用价值。

## 3. 行为与配置兼容

- 保持 `docs/PRODUCT_REQUIREMENTS.md` 中定义的必需交互和 `docs/CONFIGURATION_FORMAT.md` 中定义的配置兼容行为。
- 修改用户可见行为前，区分功能等价、错误修复和新增功能；不得把未实现/待决定能力描述为已有功能。
- 默认兼容既有 TOML 字段和含义。未经迁移逻辑、备份和兼容测试，不得重命名配置文件、改变字段格式或更改配置目录。
- 若更改配置存储位置，首次启动应查找并导入旧配置，避免静默覆盖或丢失用户数据；新旧配置冲突时使用文档规定的优先级，未规定时先确认。
- TOML 解析须验证字段、数组长度、数值范围、路径与尺寸；不完整配置不得导致 panic 或无声丢数据。对损坏配置应保留原文件并提供可诊断错误/恢复方式。
- 配置编辑应尽可能保留未识别字段；如需要保留注释和格式，应采用并测试能支持该需求的 TOML 编辑方式。
- 设置窗口必须使用独立 working copy：实时预览不等于持久化，Save 才提交，Cancel/非保存关闭必须回滚。
- 正常退出需取消后台任务、等待设置 flush 并释放托盘/原生资源；不得依赖进程退出自然完成异步写入。
- 打开文件或目录时应使用系统默认关联，不得拼接用户路径交给 `cmd.exe`、PowerShell 或其他 shell 执行。
- 自动启动、屏幕吸附、抽屉增删改排、路径点击打开 Explorer、缓存等仅在产品需求文档中批准后实现。

## 4. Windows API 与 `unsafe`

- 优先使用安全封装。必要的 Win32/COM interop 集中在 Windows 平台模块；每个 `unsafe` 块附近解释指针有效期、句柄所有权、线程要求和调用前置条件。
- HWND、图标、COM 对象及回调上下文应使用 RAII/Drop 或其他明确的生命周期管理；避免重复释放、资源泄漏及回调仍在执行时销毁状态。
- 检查 Win32 返回值、HRESULT、空句柄、缓冲区长度和编码转换。系统调用失败须返回带上下文的错误，不得对运行期错误直接 `unwrap()`/`expect()`。
- Windows API 边界优先使用 UTF-16，并通过 `Path`/`OsStr` 等平台类型处理路径；不得假设 Windows 路径总是 UTF-8。
- windows-rs 仅启用实际使用的 API features；依赖版本、feature 和 target 变更应说明理由。
- 所有 UI 窗口/控件更新必须在 UI 线程执行。HWND 消息与 UI 事件之间须明确线程、重入、关闭顺序和回调所有权。

## 5. 错误处理、并发与日志

- 对可预期错误使用 `Result` 并附带上下文。仅对已证明的不变量使用断言/`expect`，并说明理由。
- 不得吞掉文件 IO、TOML、Shell、COM、窗口创建或后台任务错误。给用户适当的可恢复反馈，并写入可诊断日志；日志不得暴露敏感信息。
- 不在锁内执行文件 IO、Shell/COM 调用、等待任务或同步派发 UI；避免阻塞 UI 线程或制造死锁。
- 设置保存应合并频繁变更，并采用临时文件 + 原子替换或等效安全策略；写入失败时保留旧配置并报告错误。
- 日志通过统一门面实现，覆盖 UI 错误、panic/未处理错误、后台任务和 Windows API 失败；不要在各 UI 页面散布不同格式的文件追加代码。

## 6. 测试与质量门禁

Rust 工程创建后，按实际 workspace 布局执行检查。若命令因 Windows SDK、目标平台或 UI 工具链缺失无法运行，应明确报告环境限制，不得宣称验证通过。

建议的基础检查（根据实际工程调整）：

```text
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

要求：

- HSLA 转换、范围夹取、文字对比色、路径比较、尺寸/坐标校正及配置解析尽可能进行平台无关单元测试。
- 使用脱敏配置 fixture 验证兼容性；不得把用户真实路径、机器名或个人数据加入测试、快照或日志。
- 对设置保存/回滚、抽屉切换、刷新取消、缺失目录和窗口尺寸持久化添加逻辑测试或可重复集成测试。
- 托盘、Shell 图标、默认应用启动、窗口拖动、DPI、多显示器、文件夹选择等 Windows 行为应安排 Windows 环境验证；纯 Linux 测试不能证明这些功能通过。
- UI / HWND / Shell 行为的证据应明确标注为单元测试、UI Automation、人工观察或源码/API 检查；一种证据不能替代另一种。例如托盘图标人工可见不证明菜单各项、双击、Explorer 重启恢复或干净机器部署；UI Automation 未枚举到托盘也不必然证明图标不存在。
- 对响应式图标网格，使用真实窗口调整尺寸并通过 UI Automation 或等效方式计数/量测条目与列数；记录窗口宽度、列数、DPI/显示器及运行环境。布局数据是 host-specific 观察；代码/布局变化后应重跑，不把先前临时 spike 的结果当作当前实现验证。使用本地图像样例只能验证布局，不能宣称 Shell 图标提取、文件枚举或 Explorer 等价性已通过。
- 测试 keyed window 的打开和关闭时，区分关闭组件、应用进程存活、重新创建窗口和真正隐藏/恢复原窗口实例及状态；不得把 close/reopen 记为 Hide/Show。
- 修改窗口生命周期、设置格式、unsafe/COM、并发写入或打包时，增加针对性测试或验证说明。
- 文档-only 变更无需构建；代码或依赖变更应报告实际运行的命令及结果。

## 7. Rust 工程与代码风格

- 遵循 `rustfmt`。模块/函数使用 `snake_case`，类型/trait 使用 `UpperCamelCase`，常量使用 `SCREAMING_SNAKE_CASE`。
- 优先采用清晰的所有权设计；不要为消除编译器提示而无意义克隆、过度使用 `Arc<Mutex<_>>`、全局可变状态或 `unsafe`。
- 依赖保持精简。新增 crate 前说明其用途，并确认与 Windows、目标工具链和发布方式兼容；避免引入重复职责的库。
- 桌面应用的 `Cargo.lock` 应纳入版本控制。依赖、feature、target 或发布配置变更应说明影响。
- 未有性能问题或基准证据时避免过度优化；优先关注大目录、缩略图、UI 响应和重复 Shell 查询。
- 用 `cfg(windows)` 或清晰的模块边界隔离 Windows-only 代码，使纯领域逻辑可在可用环境测试；不得宣称整个 UI 跨平台。

## 8. 文档与改动范围

- 修改产品行为、TOML schema、配置/日志路径、坐标/DPI 语义、托盘/启动行为、打包步骤时，同步更新对应需求、配置或技术决策文档。
- 技术选型通过 spike 验证后，记录选择、版本、已知限制、替代方案和发布日期/升级影响；未决事项保持显式标记，不要静默定案。
- 改动聚焦当前任务；不要顺手重格式化整个仓库或提交生成文件、个人 IDE 状态和用户数据。
- 开始前检查工作区状态并区分已有改动；不得覆盖或撤销用户未提交修改。
- 完成时说明改动文件、关键决策、实际测试命令及结果，并列出仍需 Windows 环境确认的事项。
