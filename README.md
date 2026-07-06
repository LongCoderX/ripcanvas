# RipCanvas

RipCanvas 是一个快速、轻量的 Obsidian Canvas `.canvas` 桌面查看器，使用 Rust 和 Slint 构建，命令行入口为 `rocv`。

它适合快速检查由 agent、脚本或其他自动化工具生成的 Canvas 文件。当前阶段专注于查看体验，不支持编辑、保存或回写源文件。

## 核心功能

- 打开并渲染 Obsidian Canvas `.canvas` 文件。
- 支持 `text`、`file`、`link`、`group` 节点和曲线边。
- 支持缩放、平移、适配视图、1:1 重置和小地图。
- 支持节点选择、关联高亮和 Inspector 元数据查看。
- 支持复制节点信息、最近文件、手动刷新和文件变更自动刷新。
- 支持从 CLI 或 UI 导出 PNG 图片。

## 构建方式

要求 Rust stable 和 Cargo。

```powershell
cargo run --bin rocv
cargo run --bin rocv -- tests\fixtures\basic.canvas
cargo test
```

## 打包方式

### Windows

生成 Windows 便携 zip：

```powershell
.\scripts\package-windows.ps1 -ZipOnly
```

生成指定架构的 Windows 11 便携包：

```powershell
.\scripts\package-windows.ps1 -ZipOnly -Arch x64
.\scripts\package-windows.ps1 -ZipOnly -Arch x86
.\scripts\package-windows.ps1 -ZipOnly -Arch arm
```

生成的包会输出到 `dist/` 目录。

### macOS

生成 macOS 便携 zip：

```bash
./scripts/package-macos.sh
```

生成指定架构的 macOS 便携包：

```bash
./scripts/package-macos.sh --arch x64
./scripts/package-macos.sh --arch arm64
./scripts/package-macos.sh --arch universal
```

生成的包会输出到 `dist/` 目录。

## 许可证

RipCanvas 使用 MIT License。详见 [LICENSE](LICENSE)。
