# RipCanvas

RipCanvas 是一个用于查看 Obsidian Canvas `.canvas` 文件的轻量桌面应用。

它的命令行入口是 `rocv`。这个项目主要服务于 agent、脚本或其他自动化工具生成 `.canvas` 文件后的快速检查场景：用户可以保持一个独立查看器打开，随时查看画布结构、节点关系和元数据。

当前阶段 RipCanvas 以查看体验为主，暂时不做编辑、保存和回写源文件。

## 功能

- 从命令行或应用内文件菜单打开 `.canvas` 文件。
- 渲染 Obsidian Canvas 节点、分组、颜色和曲线边。
- 支持缩放、平移、适配视图和重置视图。
- 支持鼠标滚轮缩放和拖拽平移。
- 支持小地图查看全局画布位置。
- 支持选择节点并在 Inspector 中查看 id、类型、标题、标签、来源、颜色和几何信息。
- 支持复制节点信息，方便粘贴到 prompt、脚本或 issue 中。
- 支持重新打开最近使用的 Canvas 文件。
- 支持监听当前文件变化，并在外部修改后自动刷新。
- 刷新解析失败时保留最后一次成功显示的画布，并在状态栏显示错误。

完整进度记录见 [PROGRESS.md](PROGRESS.md)。

## 使用

无参数启动：

```powershell
rocv
```

打开指定 Canvas 文件：

```powershell
rocv path\to\architecture.canvas
```

在应用内可以通过文件菜单打开文件、打开最近文件、刷新当前文件，也可以使用工具栏控制缩放、适配视图、小地图、边效果和节点标题显示。

## 从源码构建

要求：

- Rust stable 和 Cargo
- Windows、macOS 或 Linux，具体取决于 Slint 支持的桌面平台

构建并运行：

```powershell
cargo run --bin rocv
cargo run --bin rocv -- tests\fixtures\basic.canvas
```

运行检查：

```powershell
cargo fmt --all -- --check
cargo check --all-targets
cargo test
```

## Windows 打包

生成当前常用目标的便携 zip：

```powershell
.\scripts\package-windows.ps1 -ZipOnly
```

生成不同架构的 Windows 11 便携包：

```powershell
.\scripts\package-windows.ps1 -ZipOnly -Arch x64
.\scripts\package-windows.ps1 -ZipOnly -Arch x86
.\scripts\package-windows.ps1 -ZipOnly -Arch arm
```

使用 `cargo-packager` 生成安装包：

```powershell
cargo install cargo-packager --locked
.\scripts\package-windows.ps1
```

生成的包会输出到 `dist/` 目录。

## 许可证

RipCanvas 使用 MIT License。详见 [LICENSE](LICENSE)。

## English Summary

RipCanvas is a lightweight desktop viewer for Obsidian Canvas `.canvas` files.
It is designed for workflows where agents, scripts, or other tools generate
canvas documents and users need fast visual feedback in a separate viewer.
