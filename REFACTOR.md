# 已重构

# 待重构

## P1 稳定性与安全

- [ ] 为 Canvas 几何数据增加统一校验：`x`、`y`、`width`、`height` 需要是有限数值，节点宽高不应为负数，避免异常 `.canvas` 在 view-model 构建或 `clamp` 时触发 panic。
- [ ] 修正 PNG 导出尺寸限制：确保 `MAX_EXPORT_SIDE` 是严格上限，避免超大坐标或超大画布导致 `ImageBuffer` 申请过多内存。
- [ ] 为导出流程增加资源保护：对输出像素总量、节点数量、边数量或输入文件大小设定合理上限，并返回可读错误。

## P2 模块划分

- [ ] 拆分 `src/app.rs`：保留 Slint 窗口组装和 callback 绑定，将 watcher、recent 文件持久化、路径校验、剪贴板、字体选择、UI 数据映射拆到独立模块。
- [ ] 将 `src/watch/mod.rs` 从占位模块变成真实文件监听模块，承接 `start_watcher`、事件过滤和 debounce/reload 逻辑。
- [ ] 抽出共享路径校验模块，合并 CLI 和 App 中重复的 `.canvas` 文件校验逻辑，并统一是否 canonicalize、是否大小写不敏感。
- [ ] 梳理导出模块边界：将 `src/canvas/export.rs` 中的 PNG 渲染、字体加载、文本排版、基础绘图函数拆成更清晰的子模块或私有 helper 结构。

## P2 可读性与抽象

- [ ] 修复 `cargo clippy --all-targets --all-features -- -D warnings` 当前报告的问题：`manual_clamp`、`too_many_arguments`、`collapsible_str_replace`。
- [ ] 用小型结构体表达绘图参数，例如 `Point`、`BezierCurve`、`TextBox`，减少 `draw_text`、`draw_cubic`、`cubic_point` 的长参数列表。
- [ ] 在 `CanvasViewModel::from_canvas` 中为节点建立 `HashMap` 索引，避免每条边都线性查找起止节点。
- [ ] 将颜色解析/规范化逻辑集中到一个位置，避免 `view_model.rs`、`app.rs`、`export.rs` 分别维护相近的 hex 解析逻辑。
- [ ] 将 magic numbers 命名化，尤其是导出渲染中的 padding、标题高度、字号、线宽、采样步数、颜色常量等。

## P3 功能细节

- [ ] 改进 PNG 导出的文本换行：支持中文、长 URL、无空格长文本按字符或宽度折行。
- [ ] recent 文件保存、初始 watcher 启动失败时不要完全静默；可通过状态栏或日志记录非阻塞错误。
- [ ] 为 watcher 增加更稳健的 debounce/合并策略，避免连续保存时启动过多短生命周期线程。
- [ ] 为 PNG 导出增加更直接的单元测试或快照/尺寸测试，覆盖超大画布、负尺寸、中文文本和长文本场景。
