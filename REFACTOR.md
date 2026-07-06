# 重构审查

审查时间：2026-07-04

审查范围：`src/`、`ui/app-window.slint`、`tests/`、构建与打包脚本。  
验证结果：`cargo test` 通过；`cargo clippy --all-targets --all-features -- -D warnings` 失败，当前阻塞项集中在 `src/canvas/export.rs`。

## 已完成或无需重构

- [x] Canvas 解析、基础 view-model 映射、CLI 导出、节点内容编辑已有基础测试覆盖。
- [x] UI 已具备空状态、文件打开、刷新、导出、最近文件、编辑保存、缩放/平移、性能模式等核心路径。
- [x] 负坐标节点已在 `CanvasViewModel::from_canvas` 中通过偏移和 padding 归一化。

## P0：先修复会阻塞质量门禁的问题

- [ ] 修复 `src/canvas/export.rs` 的 clippy 阻塞项，让 `cargo clippy --all-targets --all-features -- -D warnings` 通过：
  - `export_scale` 使用手写 clamp pattern。
  - `load_export_font` 存在可折叠的嵌套 `if let`。
  - `draw_text`、`draw_bitmap_text`、`draw_cubic`、`cubic_point` 参数过多。
  - `clean_markdown` 连续调用 `str::replace`。
- [ ] 给 PNG 导出增加最小单元/集成测试，而不是只通过 CLI “文件存在且非空”间接覆盖；至少覆盖导出尺寸、空画布、极大画布、长文本/中文文本。

## P1：稳定性与资源边界

- [ ] 为 Canvas 几何数据增加统一校验：`x`、`y`、`width`、`height` 必须是有限数值，节点宽高不能为负。当前 `model.rs` 直接反序列化为 `f32`，后续 view-model、Slint 布局和 PNG 导出都默认几何值可信。
- [ ] 明确 PNG 导出资源上限：`MAX_EXPORT_SIDE` 只限制单边缩放结果，仍应对像素总量、节点数、边数、输入文件大小或文本长度设置可解释的上限，并返回用户可读错误。
- [ ] 修正 `export_scale` 的 NaN/极端值策略：不要直接依赖浮点 `clamp` 行为；先拒绝非有限画布尺寸，再计算 scale。
- [ ] 改进 watcher debounce：`start_watcher` 当前每次相关文件事件都会新建线程并 sleep 250ms，连续保存可能产生多个并发 reload；应集中到 `watch` 模块，用单一 debounce/合并机制处理。
- [ ] 避免 watcher 的文件名匹配误命中：`paths_match` 在监听父目录时只比较 `file_name`，不同目录下同名 `.canvas` 可能被视为同一文件；应基于 canonical path 或明确的父目录事件语义判断。
- [ ] 保存节点内容时考虑 watcher 自触发：`update_node_content` 写回文件后 watcher 会再次 reload，容易造成重复状态更新；应在保存路径中复用同一 reload 流程或对自写事件做合并。

## P1：模块边界与职责拆分

- [ ] 拆分 `src/app.rs`（当前约 704 行）：保留 Slint 窗口装配和 callback 绑定，把最近文件、路径校验、watcher、剪贴板、字体选择、状态消息、UI model 映射分别移到小模块。
- [ ] 启用并充实 `src/watch/mod.rs`：该文件目前仍是占位模块，但真实文件监听逻辑在 `app.rs` 中；应迁移 `start_watcher`、事件过滤、debounce 和 reload 通知。
- [ ] 抽出共享路径校验模块：CLI 的 `validate_canvas_path(&Path)` 与 App 的 `validate_canvas_path(PathBuf)` 逻辑重复，且 canonicalize 行为不一致；应统一大小写、canonicalize、错误文案与测试。
- [ ] 拆分 `src/canvas/export.rs`（当前约 511 行）：按职责拆成导出入口、字体加载、文本排版、颜色解析、基础绘图/几何工具，减少单文件状态和长参数传递。
- [ ] 拆分 `ui/app-window.slint`（当前约 1651 行）：将按钮、菜单、Markdown 预览、画布视图、minimap、inspector、footer/header 拆为独立组件文件，降低 UI 修改风险。

## P2：性能与数据结构

- [ ] 在 `CanvasViewModel::from_canvas` 中为节点建立 `HashMap<&str, &CanvasNodeView>` 索引；当前每条边都通过 `nodes.iter().find(...)` 查找起止节点，边数多时退化为 O(E×N)。
- [ ] 优化 `apply_view_model_with_selection` 的选中节点查找：当前为了 index、label、text 对节点做多次线性查找，可一次遍历或建立临时结果。
- [ ] 降低 Slint 中重复 `for edge in root.edges` 的渲染成本：主画布边至少被多轮遍历用于阴影、路径、端点、ripple，性能模式之外大图仍可能卡顿；考虑合并层、按选中状态分组或在 Rust 侧预计算显示列表。
- [ ] 为 Markdown block 渲染增加更真实的高度计算或截断策略：`MarkdownPreview` 用固定高度估算 block，长段落、代码块和表格容易被过早 elide 或溢出。
- [ ] 缓存字体选择/加载结果：App 字体选择和导出字体加载都扫描系统字体；导出多次执行时应避免重复扫描。

## P2：类型建模与重复逻辑

- [ ] 用小型结构体表达导出绘图参数，例如 `Point`、`Rect`、`BezierCurve`、`TextLayout`，消除 `draw_text`、`draw_bitmap_text`、`draw_cubic`、`cubic_point` 的长参数列表。
- [ ] 集中颜色解析与规范化逻辑：`view_model.rs`、`app.rs`、`export.rs` 分别维护 hex 解析/默认值逻辑，容易出现 UI 与导出颜色不一致。
- [ ] 将 Obsidian Canvas 颜色枚举、默认颜色、可读文字色阈值集中定义，并为 3 位 hex、6 位 hex、数字色、非法色写表驱动测试。
- [ ] 明确 `CanvasNodeView` 的职责：它同时承载布局、展示文案、可编辑字段、颜色、Markdown blocks、几何字符串；可拆为 domain view-model 与 UI adapter，减少字段膨胀。
- [ ] 将 magic numbers 命名化，尤其是 padding、标题高度、字号、线宽、采样步数、debounce 时间、状态 settle 时间、minimap 尺寸、性能模式阈值。

## P3：功能细节与体验债务

- [ ] 改进 PNG 导出文本换行：`wrap_text` 只按 whitespace 分词，中文、长 URL、无空格长单词不会按宽度折行。
- [ ] 去掉导出文本的重复绘制：`draw_text` 使用字体绘制后仍无条件调用 `draw_bitmap_text`，会造成文字加粗/重影；应仅在缺字形或无字体时 fallback。
- [ ] recent 文件读写不要完全静默：`load_recent_file`、`save_recent_file` 当前吞掉所有错误，至少应在状态栏或日志中提供非阻塞提示。
- [ ] `update_node_content` 会重排整个 JSON 文件，可能改变用户原文件格式；若需要更低侵入保存，应评估保留格式或明确这是可接受行为。
- [ ] UI 中存在少量重复属性赋值，例如 `StatusPill` 的 `color`、group 顶条的 `opacity`；清理这些无意义重复以降低噪音。
- [ ] Inspector 中 “Export” 按钮长期 disabled；如果没有近期实现计划，应删除或改成明确的占位说明，避免误导。
- [ ] 为 UI 交互补充 smoke/截图测试或手动验收脚本：打开文件、保存节点、watch reload、缩放、导出、recent 文件是核心路径，但目前自动化只覆盖 CLI 和部分纯函数。

## 建议执行顺序

1. 先修 P0：让 clippy 重新成为可用质量门禁，并补上导出最小测试。
2. 再修 P1：几何校验、导出资源边界、watcher debounce/路径匹配，优先减少崩溃和重复 reload。
3. 然后做小步拆分：先迁移 `watch` 与路径校验，再拆 `export.rs`，最后拆 Slint 组件。
4. 最后处理 P2/P3：性能优化、类型建模、UI 噪音和体验细节。
