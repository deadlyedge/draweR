# draweR：

draweR 是一个致力于优化Windows桌面管理的app，它体积小巧，功能直观：类似第二个‘windows主菜单按钮’，将常用文件夹/常用app/常用资源等自定义的文件夹归于一个统一抽屉入口。支持high-DPI屏幕和自定义位置。


# Rust + windows-rs + WinUI 迁移准备文档

> 目的：记录当前 C#/WinForms 版本实际实现的功能、配置契约、已知缺口，以及迁移到 Rust / windows-rs / WinUI 时需要保留的行为和需要先做的技术验证。本文依据当前工作区中的源码检查；不代表 README 中所有描述都已经实现。

## 1. 迁移目标与边界

建议将迁移目标拆成三层，而不是把 WinForms 控件逐个机械翻译：

1. **Rust 应用与领域逻辑**：抽屉、设置、主题、文件列表、命令及状态管理；尽量不依赖 Windows UI 类型。
2. **Windows 平台适配层（windows-rs）**：Shell 图标/缩略图、打开文件、托盘、窗口句柄与消息、文件夹选择器、可选的 Windows 启动注册。
3. **WinUI 前端**：主窗口、抽屉窗口、设置窗口、主题资源和输入事件。

这里的 `windows-rs` 是 Windows API 的 Rust 绑定层，WinUI 3 则属于 Windows App SDK/XAML UI。两者不是 WinForms/SunnyUI 的直接替代品。建议先做一个小型技术验证：用目标 Rust 工具链完成一个 WinUI 窗口、拿到 HWND、实现无边框拖动、系统托盘回调和发布打包，再决定 UI 集成细节。若 WinUI 3 的 Rust/XAML 组合无法稳定满足构建与分发需求，应先评估纯 Win32 前端；业务/领域模块仍可复用。

## 2. 当前项目概况

- 当前应用：C# WinForms，目标框架为 `net9.0-windows`，`winDrawer.csproj` 配置 `win-x64`、单文件发布，并引用 SunnyUI 3.8.8 与 Tomlyn 0.11.0。
- UI：`MainWindow`、`DrawerWindow`、`SettingsForm`，使用 SunnyUI 控件；设计器布局在相应的 `*.Designer.cs` / `*.resx`。
- 配置：`Configuration/SettingsRepository.cs` 读写 TOML，当前默认路径是可执行文件目录下的 `Drawer-settings.toml`。
- 文件图标：`ShellIcon.cs` 通过 Shell32 `SHGetFileInfo` 获取图标，并调用 User32 `DestroyIcon` 释放原生图标句柄。
- 主题：`Theming/ThemePalette.cs` 将 HSLA 转换为整组颜色及窗体透明度。
- 日志：`DrawerLogger.cs` 追加写入可执行文件目录下的 `Drawer.log`，同时写到 Debug 输出。
- 当前仓库中未发现独立测试项目。迁移前应建立基线测试与行为清单。

## 3. 当前功能清单（按源码核实）

### 3.1 主窗口与抽屉选择

- 主窗口是无边框 WinForms 窗体，默认 UI 设计大小约为 413×400；按配置设置位置、透明度和主题。
- 拖动主窗口的 Logo 区域可以移动窗口；`window_locked = true` 时禁止拖动。拖动结束后将坐标写入设置。
- 当前实现没有检测屏幕边缘并自动吸附；“停靠到屏幕边缘”是 README 的表述，不应视为已经完成的行为。
- 主列表展示配置的抽屉目录；目录不存在时显示 `(missing)`。无抽屉时显示 `No drawers configured` 并禁用列表。
- 点击一个抽屉会在主窗口右侧打开它；再次点击同一个抽屉会关闭它。一次只管理一个当前抽屉窗口。抽屉窗口随主窗口的位置变化而移动。
- 打开缺失目录会显示警告；抽屉关闭后重新从 repository 读取快照并刷新主列表。
- `Add Drawer` 通过文件夹选择器添加目录，显示名默认为最后一级目录名；路径重复时拒绝添加（Windows 上不区分大小写）。目前没有抽屉编辑、重命名、删除、排序 UI。

### 3.2 抽屉内容窗口

- 枚举所选目录的一级子目录和一级文件（不递归）；目录、文件分别按名称排序，采用当前区域设置下的大小写不敏感比较。
- 目录先于文件显示。每个项目显示图标和名称，内容区域可滚动、自动换行；悬停会高亮卡片。
- 文件夹/普通文件图标由 Windows Shell 提供。图片文件（bmp/gif/ico/jpeg/jpg/jfif/png/tif/tiff/webp）尝试读取并按比例缩放为缩略图；失败时回退到 Shell 图标，再失败回退到 Windows Logo 图标。
- 点击卡片、图标或文本会通过系统默认关联打开文件或文件夹；失败时显示错误对话框。
- `Refresh` 重新枚举目录；`Close` 关闭抽屉。底部显示路径，但 `fullPathLabel_Click` 当前为空，点击路径没有打开 Explorer 的行为。
- 抽屉窗口采用无边框、非任务栏窗口；标题区域可以通过 Win32 `ReleaseCapture` / `SendMessage(WM_NCLBUTTONDOWN, HTCAPTION)` 拖动。右下角有自定义 resize grip，窗口最小宽度为 530；调整结束时尝试按路径保存宽高。
- 抽屉尺寸记录在该 drawer TOML 表的 `[drawers.size]` 下；只在窗口为 Normal 且尺寸变化时保存。缺少 `size` 时保留窗体初始尺寸；有 `size` 时宽高读取范围为 200–4000，缺失分量默认 800×600。
- 列表枚举、Shell 图标查询、图片解码与缩放都在当前 UI 调用路径中同步执行；大量文件或慢速网络目录可能阻塞界面。当前没有缩略图预加载/缓存。

### 3.3 设置与主题

- 设置窗口是无标题栏的 modeless 窗口，包含两个行为选项：`Start with windows`、`Lock position`；以及 Hue、Saturation、Lightness、Opacity 四个滑杆、预览区域、Save、Cancel、Default、Shut down 按钮。
- 设置窗口激活后，用户在窗口外按下鼠标键会关闭它；移动鼠标经过不触发关闭。
- 调整任一主题滑杆会即时预览到主窗口，以及当前打开的抽屉窗口。Save 保存工作快照；Cancel/其他非 Save 关闭会恢复打开设置前的主题。行为选项仅在 Save 时同步到工作快照。
- HSLA 范围：Hue 0–360，Saturation/Lightness/Alpha 0–100。调色板生成 accent、hover、pressed、border、主/内容/card 背景及文字色；文字根据背景相对亮度选择深灰或浅色。窗体透明度被限制到 0.05–1.0，即使 Alpha 为 0 也不会完全透明。
- 配置中有 `icon_size`，读取/归一化范围为 32–512，默认 96；缩略图尺寸再限制为 16–512。但当前设置 UI 没有调整 icon size 的控件。
- `Default` 按钮目前只记录日志，没有重置设置的实现。
- `Start with windows` 被显示并保存到 TOML，但当前代码没有使用它创建/移除启动项，因此目前只是一个持久化选项，并未真正控制 Windows 自启动。
- `Shut down app` 按钮会请求尽快退出进程：当前路径尝试 flush 设置，之后使用 `Environment.Exit` / 进程终止回退；迁移时应明确是否保留此强制退出语义。

### 3.4 系统托盘、启动和日志

- 系统托盘菜单包含 Show/Hide、Settings、Exit；双击托盘图标切换主窗口可见性。Show 会恢复普通窗口状态并激活窗口。
- 托盘图标优先读取 `assets/drawer.icon.4.ico`，失败时尝试克隆主窗口图标。
- `Program.cs` 注册 UI 线程与 AppDomain 未处理异常日志钩子。
- `Drawer.log` 每条记录包括时间、级别、调用成员和消息；异常写入文件，写日志失败时只回退到 Debug 输出。
- `SettingsRepository` 使用信号量驱动后台写入，并提供 `FlushAsync` / 同步 `Save`。但主窗口关闭流程没有明确 dispose repository；普通退出与异步写入的可靠性应在迁移中重新设计，不应照搬现有生命周期。

## 4. 配置文件兼容契约

当前 `Drawer-settings.toml` 的主要结构如下：

```toml
[startup]
start_with_windows = false
window_locked = false
icon_size = 96
window_color_hsla = [194, 100, 41, 60]
window_position = [100, 100]

[[drawers]]
name = "Documents"
path = "D:\\users\\Documents"
[drawers.size]
width = 800
height = 600
```

### 迁移时应保留的字段与语义

| TOML 字段 | 含义 | 当前行为/注意事项 |
|---|---|---|
| `startup.start_with_windows` | 自启动选项 | 读写持久化，但当前未实际注册 Windows 自启动。 |
| `startup.window_locked` | 锁定主窗口位置 | 当前会阻止拖动主窗口。 |
| `startup.icon_size` | 抽屉项目图标尺寸 | 默认 96，范围 32–512；UI 暂无编辑器。 |
| `startup.window_color_hsla` | `[h, s, l, a]` | 按分量夹取到 0–360 / 0–100；缺分量使用相应默认值。 |
| `startup.window_position` | `[x, y]` | 保存/恢复主窗口坐标；屏幕不可见时没有纠正逻辑。代码对存在但长度不足 2 的数组没有防护。 |
| `drawers[].name` | 抽屉显示名 | 空名会退化为路径最后一级名称。 |
| `drawers[].path` | 目录路径 | 展开环境变量；空路径条目跳过。 |
| `drawers[].size.width/height` | 抽屉窗口尺寸 | 按 drawer path 匹配并保存；默认范围 200–4000。 |

当前配置文件与日志都默认放在 exe 所在目录。这对安装到不可写目录、便携版、多用户场景有风险。迁移前应决定是否继续使用旧路径，或改用 `%APPDATA%` / `%LOCALAPPDATA%`；若迁移到新位置，首次启动必须查找并导入旧 TOML，避免用户的抽屉和主题设置丢失。

建议 Rust 侧保留一份 TOML golden fixture，并测试：旧字段读取、缺省值、超范围值夹取、环境变量展开、路径重复判断、可选尺寸、未知字段处理以及损坏配置的恢复/错误提示。若需要保留注释和未识别字段，可考虑 `toml_edit`；仅用反序列化再完整序列化通常不能保证原有排版/注释不变。

## 5. 当前已知缺口与迁移决策

### 已明确未实现或未接通

- 抽屉管理页（编辑名称、排序、移除）——README TODO。
- 点击抽屉路径打开 Explorer——TODO，当前 click handler 为空。
- 抽屉内容预加载及 `.cachefiles` 缓存——TODO。
- `Start with windows` 与 Windows 启动项未连通。
- 设置中的 `Default` 按钮无动作。
- 图标尺寸虽有配置字段但没有设置控件。
- 主窗口没有屏幕边缘吸附逻辑，也没有主窗口尺寸持久化字段。

### 需要在迁移前明确

1. **WinUI 技术栈可行性**：确认 Rust 调用 WinUI 3 / Windows App SDK 的 XAML、生命周期、HWND interop、打包与 CI；用 spike 验证，而不是先迁完业务才发现 UI 受限。
2. **窗口透明度**：WinForms `Form.Opacity` 与 WinUI/Composition 透明效果并非必然一致；确认期望是整个窗口透明，还是背景/亚克力透明，并定义可测试标准。
3. **坐标与 DPI**：明确 TOML 坐标继续按桌面像素解释，还是转换为 DIP；同时验证多屏幕、不同缩放比例、显示器断开后窗口回屏策略。
4. **托盘生命周期**：WinUI 窗口与传统 Shell 托盘通知需要 HWND/消息处理；验证关闭窗口是隐藏还是退出、托盘重启恢复、菜单点击与双击的行为。
5. **自动启动**：若保留此选项，决定使用 Startup 文件夹快捷方式还是 HKCU Run；安装、卸载、路径更新时也要同步。不要只迁移复选框而保留未生效状态。
6. **配置目录与写入策略**：决定保留 exe 目录还是迁到用户数据目录；保存采用防抖/原子替换，并在正常退出时等待 flush；避免更新/崩溃时写坏配置。
7. **程序包形式**：决定 MSIX、unpackaged 或便携发布，以及 Windows App SDK runtime、VC runtime 和资产的部署策略。

## 6. 建议的 Rust 模块边界

可以采用单一 Rust workspace/应用 crate 起步，先通过模块边界隔离平台依赖；避免早期拆分多个 crate 增加构建复杂度。

```text
src/
  main.rs
  app/                 # 应用状态、命令、事件协调、窗口生命周期
  domain/              # Settings、DrawerDefinition、Hsla、ThemePalette、ItemEntry
  services/            # 设置用例、目录枚举、抽屉尺寸更新、预览/保存流程
  ports/               # SettingsStore、Shell、Thumbnailer、StartupRegistration 等 trait
  infra/
    toml_settings.rs   # TOML 实现
    logging.rs         # 日志后端
  platform/windows/
    shell.rs           # 图标、缩略图、打开文件、文件夹选择
    tray.rs            # Shell 通知图标、菜单及消息分发
    windowing.rs       # HWND、无边框移动/调整、显示器工作区/DPI
    startup.rs         # 可选的自启动集成
  ui/winui/            # XAML 页面、窗口和 ViewModel/事件绑定
assets/
```

### 设计原则

- 领域类型避免依赖 `System.Drawing` 的概念；颜色可用 RGBA 值，坐标/尺寸需要明确单位，并在 WinUI 边界转换。
- 主题计算应是纯函数，输入 HSLA、输出 palette；用测试锁定当前默认主题及边界值。UI 控件只消费主题，不自行重复计算颜色。
- TOML 读写只在设置 adapter 中完成。UI 发送命令（AddDrawer、SaveSettings、RefreshDrawer、ResizeDrawer），不要直接修改磁盘模型。
- 文件枚举与图片处理放后台任务执行；结果通过 UI dispatcher 更新。目录刷新应有取消机制，避免旧任务覆盖新结果。
- Shell 图标、文件打开、托盘和窗口消息全部隔离在 Windows 平台模块中；不要让 `windows` crate 类型进入领域层。
- 保存通过 debounce/coalesce 减少频繁写入，写入临时文件后原子替换，并于关闭流程显式 await flush。
- 对拖动、resize grip 等特殊窗口行为优先评估 WinUI/AppWindow 能力；必须使用 HWND 消息时封装成一组安全、可测的 Windows adapter。

## 7. API/控件迁移映射

| 当前实现 | 迁移方向 |
|---|---|
| `Program.Main` / `Application.Run` | Rust 应用入口 + WinUI/Windows App SDK 应用生命周期；集中设置错误处理和关机流程。 |
| `MainWindow` / SunnyUI 控件 | WinUI `Window` + XAML 列表/按钮/资源；保留一份应用状态而非在控件事件中承担业务逻辑。 |
| `DrawerWindow` / `UIFlowLayoutPanel` | WinUI 窗口 + `ItemsRepeater`/合适的自适应布局；支持滚动、换行、空/缺失目录、重新加载。 |
| `SettingsForm` / 轨道滑块 | WinUI 设置窗口；用独立 working copy 实现即时预览、Save 提交、Cancel 回滚。 |
| `NotifyIcon` / `ContextMenuStrip` | Windows Shell 托盘 API（`Shell_NotifyIconW`）+ HWND 消息路由，或经 spike 验证的稳定包装。 |
| `SHGetFileInfo`、`Icon`、`System.Drawing` 图片处理 | windows-rs Shell/COM 与 Windows Imaging Component；评估 `IShellItemImageFactory`/系统缩略图能力，避免依赖 `System.Drawing` 语义。 |
| `ProcessStartInfo(UseShellExecute=true)` | Shell 默认关联打开文件/文件夹（如封装 `ShellExecuteExW`），保留权限/不存在等错误处理。 |
| `FolderBrowserDialog` | Windows 文件夹选择器（如 `IFileDialog` 或 Windows.Storage picker），处理 WinUI HWND 初始化。 |
| `ReleaseCapture` + `SendMessage` | 优先 WinUI/AppWindow 窗口操作；必要时以 windows-rs 封装窗口消息。 |
| `ThemePalette` | 纯 Rust HSLA 转换与 contrast 计算，WinUI 侧仅应用 brush、opacity、hover 状态。 |
| `Tomlyn` + `SettingsRepository` | Rust TOML crate + repository trait；保留旧 TOML schema 和并发/flush 语义，但改进原子写入与错误恢复。 |
| `DrawerLogger` | Rust tracing/log 门面，文件位置可配置；未处理 panic、后台任务错误、UI 线程错误统一汇入日志。 |

## 8. 分阶段迁移路线

### 阶段 0：锁定现状与行为基线

- 备份 `Drawer-settings.toml`、资产和用户实际 `Drawer.log`；记录现有窗口尺寸/主题/抽屉操作截图。
- 为当前版本建立手动回归清单，并将 TOML 文件纳入 Rust 兼容性测试样例。
- 将“已实现”与本文件第 5 节未实现项区分开，明确第一版迁移是否只做功能等价，还是包含 TODO 新功能。

### 阶段 1：WinUI/Rust 技术验证

- 最小应用能启动/退出、创建无边框窗口、透明/主题预览、窗口拖动和调整大小。
- 验证 Rust 到 WinUI 的 UI 线程调用、XAML 编译、窗口 HWND 获取和打包/发布。
- 单独验证托盘图标、菜单消息、显示/隐藏及退出；记录所需 crates、SDK 版本和 CI 环境。
- 未完成这些验证前，不将 WinUI 绑定方案视为已选定。

### 阶段 2：纯 Rust 领域与兼容层

- 实现设置模型、TOML repository、主题纯函数、路径比较和尺寸/坐标策略。
- 用旧 TOML 做 round-trip / golden tests；对 `window_position` 数组长度不足等输入增加健壮性处理。
- 实现配置备份、迁移导入、新旧路径冲突策略和原子保存。

### 阶段 3：Windows shell 服务

- 实现目录枚举、Shell 图标/缩略图、默认关联打开、文件夹选择、托盘、启动项（若决定支持）。
- 同步枚举改为后台工作；支持取消、错误分项显示与缺失目录处理。
- 为不支持的路径、网络目录、图标提取失败、文件被占用添加回退路径。

### 阶段 4：UI 功能等价

- 先完成主窗口与抽屉选择/切换，再完成抽屉网格、刷新、打开项目、resize 和记忆尺寸。
- 设置窗口实现 working copy、实时预览、Save/Cancel、锁定、颜色/透明度等价行为。
- 完成托盘菜单、外部点击关闭设置等交互；逐项核对视觉与键盘/鼠标操作。

### 阶段 5：加固与发布

- 验证多显示器、DPI 缩放、超出工作区位置恢复、长路径、中文路径、权限拒绝、空目录和大目录。
- 正常退出时取消任务、等待设置 flush、销毁托盘与窗口资源；检查不残留进程/托盘图标。
- 确认升级策略、配置导入、应用图标/字体资源、签名、安装和卸载行为。

## 9. 迁移验收清单

- [ ] 首次启动生成默认 TOML；旧 TOML 能读取，保存后抽屉/主题/尺寸没有丢失。
- [ ] 错误配置有可理解的日志/提示和安全恢复方式，不覆盖原文件而不留备份。
- [ ] 主窗口可隐藏、显示、拖动、锁定；抽屉紧贴主窗口并在移动时跟随。
- [ ] 点击同一抽屉可关闭；点击另一抽屉只保留一个活动抽屉。
- [ ] 缺失目录可识别；刷新结果正确；目录与文件按当前规则排序。
- [ ] 文件夹、普通文件及支持的图片类型显示合理图标/缩略图；点击使用系统默认应用打开。
- [ ] 抽屉尺寸在关闭和重新打开/重启后恢复；多显示器/DPI 下窗口不会不可见。
- [ ] 主题四个维度、文字对比度、透明度边界、即时预览及取消回滚行为通过测试。
- [ ] 托盘显示/隐藏、双击、设置、退出均可靠；退出后无残留图标/进程。
- [ ] 自启动若保留则实际生效，取消选项会清理启动项；若暂不实现则不将其伪装为有效开关。
- [ ] 大目录加载期间 UI 仍可响应；连续刷新时旧结果不会覆盖新结果。
- [ ] 正常关闭确保设置写入完成，日志覆盖 UI、Shell 与后台任务错误。

## 10. 源码参考

- `Program.cs`：入口与异常钩子。
- `MainWindow.cs` / `MainWindow.Designer.cs`：主窗口、抽屉切换、设置预览、托盘调用。
- `DrawerWindow.cs` / `DrawerWindow.Designer.cs`：目录枚举、卡片、图标、缩略图、打开项目、resize。
- `SettingsForm.cs` / `SettingsForm.Designer.cs`：设置控件、主题预览、保存/取消。
- `Configuration/SettingsRepository.cs`：TOML 数据、快照、后台写入。
- `Configuration/DrawerDefinition.cs`：抽屉模型。
- `Theming/ThemePalette.cs`：HSLA 到主题 palette 的转换。
- `TrayController.cs`、`ShellIcon.cs`、`DrawerLogger.cs`：平台与基础设施逻辑。
- `AppConstants.cs`、`Drawer-settings.toml`、`README.md`、`reference/adjustment.md`：默认值、配置、项目说明与原始需求记录。
