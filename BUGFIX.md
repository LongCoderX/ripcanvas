# 已修复

# 待修复

## UI/UX 待优化

### 底部状态栏

- 将底部状态栏改为结构化信息区，而不是只显示一段状态文本。
- 左侧显示文件监听和加载状态，例如 `Loaded`、`Watching`、`Reloading`、`Reload failed`。
- 中间显示当前 Canvas 文件路径，过长时省略。
- 右侧显示画布统计信息，例如节点数和边数。
- 最右侧显示最近一次操作结果，例如 `Copied node information`、`Exported PNG`。
- 为状态增加轻量颜色区分：正常、成功、警告、错误。
- 错误状态应保持可见，直到下一次成功操作覆盖。

### 视图工具栏

- 将 `Fit View` 和 `Reset 1:1` 合并为一个视图模式按钮。
- 默认按钮执行适配视图，当前已接近 Fit 状态时再次点击切换为 `1:1`。
- tooltip 根据当前状态显示 `Fit View` 或 `Reset 1:1`。
- 展开的浮动工具栏中优先展示常用操作：适配/1:1、缩放、导出。
- 次要显示开关，例如标题、边渐变、波纹和小地图，可以继续放在展开区。

### Inspector 分区

- 将 Inspector 从单纯元数据列表改为分区面板。
- 顶部显示选中节点摘要：标题、类型、颜色 swatch。
- `Content` 区显示 text 或 label 的主要内容。
- `Source` 区显示 file、url 或原始文本来源，并提供复制入口。
- `Geometry` 区使用紧凑 2x2 网格显示 x、y、w、h。
- `Actions` 区集中放置 Copy，以及未来 Edit、Export selected 等操作。
- 为未来在 Inspector 中编辑 text 和 label 预留可编辑控件位置。

### Canvas 视觉层级

- 降低普通 edge 的视觉强度，使用更细线条和更低透明度。
- 强化选中节点相关 edge，使其更粗、更深，必要时增加轻微发光。
- 被选中节点使用最强视觉层级：深色边框、增强阴影或覆盖层。
- 关联节点使用次一级高亮，例如浅色描边或半透明外框。
- group 节点应弱化成容器视觉，避免和普通内容节点同等突出。
- 为节点类型增加轻量 badge，例如 `text`、`file`、`link`、`group`。
- 选中后的视觉优先级应保持为：选中节点 > 关联 edge > 关联节点 > 普通节点 > 背景 edge > 小地图。

### 空状态

- 空状态直接提供 `Open Canvas` 主按钮。
- 如果存在最近文件，提供 `Open Recent` 次按钮；没有最近文件时禁用。
- 空状态应复用现有 `request-open-file` 和 `request-open-recent` 回调。
- 文案保持简短，不做教程式说明。

### 状态反馈

- 定义明确状态类型：Idle、Loaded、Watching、Reloading、Reloaded、Exported、Error。
- 成功状态可以短暂高亮后回到普通状态。
- 刷新失败时明确提示会保留最后一次成功视图。
- 文件监听失败时需要在状态栏中显示，而不是静默降级。
- 长错误详情可以后续放入 tooltip 或详情面板，状态栏只显示短句。
