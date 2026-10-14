# RHenTai

A cross-platform desktop & mobile reader for E-Hentai and ExHentai, built with Rust, Tauri 2, and React.

基于 Rust、Tauri 2 和 React 构建的跨平台（桌面与移动端）E-Hentai / ExHentai 阅读器。

---

## English

### Features
- Online browsing and search across E-Hentai and ExHentai
- Gallery details, tags, comments, and high-resolution thumbnail grids
- Continuous vertical and paged reader with keyboard and touch navigation
- Batch download manager with pause, resume, and background queueing
- Local library supporting image folders and ZIP/CBZ archives
- History tracking, favorites management, and configurable proxy/network settings

### Development Commands
```bash
# Install frontend dependencies
pnpm install

# Run frontend typecheck and build
pnpm typecheck
pnpm build

# Generate TypeScript bindings from Rust IPC types
cargo test -p eh-client -p rhentai-core export_bindings

# Run Tauri app in development mode
pnpm tauri dev

# Build debug binary without installer bundle
pnpm tauri build --debug --no-bundle

# Run CLI
cargo run -p rhentai-cli -- --help

# Format and linting
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

---

## 中文

### 功能特性
- 支持 E-Hentai 和 ExHentai 在线画廊浏览与高级搜索
- 画廊详情、标签分组、评论以及雪碧图缩略图网格
- 纵向连续与分页双阅读模式，支持键盘与触控手势
- 批量下载管理器，支持暂停、断点续传及并发控制
- 本地书库，支持扫描图片文件夹与 ZIP/CBZ 压缩包
- 阅读历史记录、收藏夹管理以及代理配置

### 开发命令
```bash
# 安装前端依赖
pnpm install

# 前端类型检查与构建
pnpm typecheck
pnpm build

# 从 Rust 导出 TypeScript 类型绑定
cargo test -p eh-client -p rhentai-core export_bindings

# 启动 Tauri 开发环境
pnpm tauri dev

# 编译调试版程序（不打包安装包）
pnpm tauri build --debug --no-bundle

# 运行命令行工具
cargo run -p rhentai-cli -- --help

# 代码格式化与静态检查
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```
